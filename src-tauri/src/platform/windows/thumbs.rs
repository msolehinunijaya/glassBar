//! Live window thumbnails (the same DWM mechanism the Windows taskbar uses).
//! DWM draws a live copy of each source window into a rectangle of our popup.
//! Every registration is undone in `clear()`, which runs whenever the popup hides.
use super::super::Rect;
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::UI::WindowsAndMessaging::IsIconic;

static THUMBS: Mutex<Vec<isize>> = Mutex::new(Vec::new());

/// Fit `src` (w, h) inside `area`, centred, keeping its aspect ratio.
pub fn fit(src: (i32, i32), area: Rect) -> Rect {
    if src.0 <= 0 || src.1 <= 0 { return area; }
    let s = (area.w as f64 / src.0 as f64).min(area.h as f64 / src.1 as f64);
    let (w, h) = ((src.0 as f64 * s).round() as i32, (src.1 as f64 * s).round() as i32);
    Rect { x: area.x + (area.w - w) / 2, y: area.y + (area.h - h) / 2, w, h }
}

pub fn clear() {
    for h in THUMBS.lock().unwrap().drain(..) {
        unsafe { let _ = DwmUnregisterThumbnail(h); }
    }
}

/// Show thumbnails of `items` (source window, area in `dest` client pixels).
/// Returns the windows that got a live thumbnail; the rest need a fallback (icon + title).
pub fn show(dest: isize, items: &[(isize, Rect)]) -> Vec<isize> {
    clear();
    let mut ok = Vec::new();
    let mut held = THUMBS.lock().unwrap();
    for &(src, area) in items {
        let src_hwnd = HWND(src as *mut _);
        // A minimised window has nothing to show: use the fallback.
        if unsafe { IsIconic(src_hwnd) }.as_bool() { continue; }
        let Ok(t) = (unsafe { DwmRegisterThumbnail(HWND(dest as *mut _), src_hwnd) }) else { continue };
        let size = unsafe { DwmQueryThumbnailSourceSize(t) }.map(|s| (s.cx, s.cy)).unwrap_or((area.w, area.h));
        let r = fit(size, area);
        let props = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE | DWM_TNP_OPACITY | DWM_TNP_SOURCECLIENTAREAONLY,
            rcDestination: RECT { left: r.x, top: r.y, right: r.x + r.w, bottom: r.y + r.h },
            opacity: 255,
            fVisible: true.into(),
            fSourceClientAreaOnly: false.into(),
            ..Default::default()
        };
        if unsafe { DwmUpdateThumbnailProperties(t, &props) }.is_ok() {
            held.push(t);
            ok.push(src);
        } else {
            unsafe { let _ = DwmUnregisterThumbnail(t); }
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_aspect_and_centres() {
        let area = Rect { x: 10, y: 10, w: 200, h: 120 };
        assert_eq!(fit((1920, 1080), area), Rect { x: 10, y: 13, w: 200, h: 113 });
        assert_eq!(fit((600, 800), area), Rect { x: 65, y: 10, w: 90, h: 120 });
        assert_eq!(fit((0, 0), area), area);
    }
}
