//! How bright the wallpaper is behind the dock, so the page can raise the glass tint
//! where white (or dark) text would no longer read. With reserved space, windows stop at
//! the dock, so the wallpaper is what's usually behind it.
//! ponytail: assumes the wallpaper fills the screen (no fit/tile modes, one wallpaper for
//! all monitors); sample the real desktop only if that proves too rough.
use crate::config::Edge;
use crate::AppState;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, Manager};

/// Relative luminance (0 black .. 1 white) of the darkest and brightest part of the strip.
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Behind { pub min: f32, pub max: f32 }

static LAST: Mutex<Option<Behind>> = Mutex::new(None);

#[tauri::command]
pub fn wallpaper_behind() -> Option<Behind> { *LAST.lock().unwrap() }

/// Check every 2 minutes; decode only when the wallpaper or the dock's edge changed.
pub fn watch(app: &AppHandle) {
    let app = app.clone();
    std::thread::Builder::new().name("backdrop".into()).spawn(move || {
        let mut seen: Option<(PathBuf, Option<SystemTime>, Edge, Option<u32>)> = None;
        loop {
            let edge = app.state::<AppState>().config.lock().unwrap().profile().bar.edge;
            let path = wallpaper_path();
            let solid = solid_color();
            let key = path.clone().map(|p| { let t = std::fs::metadata(&p).and_then(|m| m.modified()).ok(); (p, t, edge, solid) })
                .unwrap_or((PathBuf::new(), None, edge, solid));
            if seen.as_ref() != Some(&key) {
                let b = path.and_then(|p| image::open(p).ok()).map(|img| strip(&img.thumbnail(256, 256).to_rgb8(), edge))
                    .or_else(|| solid.map(|c| { let l = luminance([(c >> 16) as u8, (c >> 8) as u8, c as u8]); Behind { min: l, max: l } }));
                if *LAST.lock().unwrap() != b {
                    *LAST.lock().unwrap() = b;
                    let _ = app.emit("appearance:behind", b);
                }
                seen = Some(key);
            }
            std::thread::sleep(Duration::from_secs(120));
        }
    }).ok();
}

fn luminance(c: [u8; 3]) -> f32 {
    let lin = |v: u8| { let v = v as f32 / 255.0; if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) } };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// The strip along the dock's edge (8 % deep), cut into 16 blocks along it.
fn strip(img: &image::RgbImage, edge: Edge) -> Behind {
    let (w, h) = img.dimensions();
    let (w, h) = (w.max(1), h.max(1));
    let depth = |n: u32| (n * 8 / 100).max(1);
    let (x0, y0, x1, y1) = match edge {
        Edge::Top => (0, 0, w, depth(h)),
        Edge::Bottom => (0, h - depth(h), w, h),
        Edge::Left => (0, 0, depth(w), h),
        Edge::Right => (w - depth(w), 0, w, h),
    };
    let along_x = matches!(edge, Edge::Top | Edge::Bottom);
    let len = if along_x { x1 - x0 } else { y1 - y0 };
    let blocks = 16.min(len);
    let mut b = Behind { min: 1.0, max: 0.0 };
    for i in 0..blocks {
        let (a, z) = (i * len / blocks, (i + 1) * len / blocks);
        let (mut sum, mut n) = (0.0, 0);
        for y in y0..y1 { for x in x0..x1 {
            let t = if along_x { x - x0 } else { y - y0 };
            if t >= a && t < z { sum += luminance(img.get_pixel(x, y).0); n += 1; }
        } }
        if n > 0 { let l = sum / n as f32; b.min = b.min.min(l); b.max = b.max.max(l); }
    }
    b
}

fn wallpaper_path() -> Option<PathBuf> {
    use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS};
    let mut buf = [0u16; 520];
    unsafe { SystemParametersInfoW(SPI_GETDESKWALLPAPER, buf.len() as u32, Some(buf.as_mut_ptr() as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)).ok()? };
    let s = String::from_utf16_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())]);
    // Slideshows and Spotlight report a path that may not exist: Windows' own copy then.
    let p = PathBuf::from(s);
    if p.is_file() { return Some(p); }
    let t = PathBuf::from(std::env::var_os("APPDATA")?).join(r"Microsoft\Windows\Themes\TranscodedWallpaper");
    t.is_file().then_some(t)
}

/// Desktop background colour (0xRRGGBB) when there's no picture.
fn solid_color() -> Option<u32> {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};
    let mut buf = [0u16; 32];
    let mut size = std::mem::size_of_val(&buf) as u32;
    unsafe { RegGetValueW(HKEY_CURRENT_USER, windows::core::w!(r"Control Panel\Colors"), windows::core::w!("Background"),
        RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr() as *mut _), Some(&mut size)).ok().ok()? };
    let s = String::from_utf16_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(0)]);
    let v: Vec<u32> = s.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    (v.len() == 3).then(|| (v[0] << 16) | (v[1] << 8) | v[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strip_reads_only_the_dock_edge() {
        // Top 8 % white, the rest black.
        let img = image::RgbImage::from_fn(100, 100, |_, y| if y < 8 { image::Rgb([255, 255, 255]) } else { image::Rgb([0, 0, 0]) });
        assert_eq!(strip(&img, Edge::Top), Behind { min: 1.0, max: 1.0 });
        assert_eq!(strip(&img, Edge::Bottom), Behind { min: 0.0, max: 0.0 });
        let left_half_white = image::RgbImage::from_fn(100, 100, |x, _| if x < 50 { image::Rgb([255, 255, 255]) } else { image::Rgb([0, 0, 0]) });
        assert_eq!(strip(&left_half_white, Edge::Bottom), Behind { min: 0.0, max: 1.0 });
    }
}
