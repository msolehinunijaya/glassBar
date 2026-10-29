//! Live window thumbnails (the same DWM mechanism the Windows taskbar uses).
//! DWM draws a live copy of each source window into a rectangle of our popup.
//! Kept per destination window (the popup's previews, a dock's minimized-window spots), so
//! one never clears the other. The popup's are undone in `clear()` whenever it hides.
use super::super::Rect;
use std::collections::HashMap;
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::*;

static THUMBS: Mutex<Option<HashMap<isize, Vec<isize>>>> = Mutex::new(None);

/// Fit `src` (w, h) inside `area`, centred, keeping its aspect ratio.
pub fn fit(src: (i32, i32), area: Rect) -> Rect {
    if src.0 <= 0 || src.1 <= 0 { return area; }
    let s = (area.w as f64 / src.0 as f64).min(area.h as f64 / src.1 as f64);
    let (w, h) = ((src.0 as f64 * s).round() as i32, (src.1 as f64 * s).round() as i32);
    Rect { x: area.x + (area.w - w) / 2, y: area.y + (area.h - h) / 2, w, h }
}

pub fn clear(dest: isize) {
    let gone = THUMBS.lock().unwrap().as_mut().and_then(|m| m.remove(&dest)).unwrap_or_default();
    for h in gone { unsafe { let _ = DwmUnregisterThumbnail(h); } }
}

/// Show thumbnails of `items` (source window, area in `dest` client pixels).
/// Returns the windows that got a live thumbnail; the rest need a fallback (icon + title).
pub fn show(dest: isize, items: &[(isize, Rect)]) -> Vec<isize> {
    clear(dest);
    let mut ok = Vec::new();
    let mut map = THUMBS.lock().unwrap();
    let held = map.get_or_insert_with(HashMap::new).entry(dest).or_default();
    for &(src, area) in items {
        let src_hwnd = HWND(src as *mut _);
        // Minimised windows too: DWM keeps their last frame and draws it, like the taskbar.
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
