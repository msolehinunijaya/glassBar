//! Screen capture (GDI) of the monitor under the cursor, and Windows' built-in OCR
//! (Windows.Media.Ocr, using the user's installed languages).
use super::super::{Res, Shot, WinRect, Word};
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

pub fn capture_at_cursor() -> Option<Shot> {
    unsafe {
        let mut pt = POINT::default();
        GetCursorPos(&mut pt).ok()?;
        let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(mon, &mut mi).as_bool() { return None; }
        let r = mi.rcMonitor;
        capture_rect(r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
}

/// Live pixels of a desktop rectangle (physical pixels).
pub fn capture_rect(x: i32, y: i32, w: i32, h: i32) -> Option<Shot> {
    if w <= 0 || h <= 0 { return None; }
    let r = windows::Win32::Foundation::RECT { left: x, top: y, right: x + w, bottom: y + h };
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let ok = BitBlt(mem, 0, 0, w, h, Some(screen), r.left, r.top, SRCCOPY | CAPTUREBLT).is_ok();
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w, biHeight: -h, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; (w * h * 4) as usize];
        SelectObject(mem, old);
        let n = if ok { GetDIBits(mem, bmp, 0, h as u32, Some(bgra.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS) } else { 0 };
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        if n <= 0 { return None; }
        for p in bgra.chunks_mut(4) { p[3] = 255; }
        Some(Shot { x: r.left, y: r.top, w, h, bgra })
    }
}

/// OCR a BGRA image. Lines in reading order, words with left edge and width (pixels).
pub fn ocr(bgra: &[u8], w: u32, h: u32) -> Res<Vec<Vec<Word>>> {
    let e = |x: windows::core::Error| x.message().to_string();
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|_| "Windows OCR isn't available: add a language with OCR support in Settings › Time & language".to_string())?;
    let max = OcrEngine::MaxImageDimension().map_err(e)?;
    if w > max || h > max { return Err(format!("selection too large for Windows OCR (max {max}px)")); }
    let dw = DataWriter::new().map_err(e)?;
    dw.WriteBytes(bgra).map_err(e)?;
    let buf = dw.DetachBuffer().map_err(e)?;
    let bmp = SoftwareBitmap::CreateCopyFromBuffer(&buf, BitmapPixelFormat::Bgra8, w as i32, h as i32).map_err(e)?;
    let result = engine.RecognizeAsync(&bmp).map_err(e)?.join().map_err(e)?;
    let mut lines = Vec::new();
    for line in result.Lines().map_err(e)? {
        let mut words = Vec::new();
        for wd in line.Words().map_err(e)? {
            let r = wd.BoundingRect().map_err(e)?;
            words.push(Word { text: wd.Text().map_err(e)?.to_string(), x: r.X, w: r.Width });
        }
        lines.push(words);
    }
    Ok(lines)
}

/// Visible app windows overlapping `area`, front-most first, with their on-screen bounds
/// (without the invisible resize borders). For "screenshot a window".
pub fn window_rects(area: (i32, i32, i32, i32)) -> Vec<WinRect> {
    use windows::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsIconic};
    unsafe extern "system" fn each(w: HWND, out: LPARAM) -> windows::core::BOOL {
        unsafe { (*(out.0 as *mut Vec<HWND>)).push(w) };
        true.into()
    }
    let mut all: Vec<HWND> = Vec::new();
    unsafe { let _ = EnumWindows(Some(each), LPARAM(&mut all as *mut _ as isize)); }
    let me = std::process::id();
    let (ax, ay, aw, ah) = area;
    all.into_iter()
        .filter(|&w| super::windows_list::is_taskbar_window(w) && super::windows_list::pid_of(w) != me && !unsafe { IsIconic(w).as_bool() })
        .filter_map(|w| {
            let mut r = RECT::default();
            unsafe { DwmGetWindowAttribute(w, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut _ as *mut _, std::mem::size_of::<RECT>() as u32).ok()? };
            let (x0, y0, x1, y1) = (r.left.max(ax), r.top.max(ay), r.right.min(ax + aw), r.bottom.min(ay + ah));
            (x1 - x0 > 40 && y1 - y0 > 40).then(|| WinRect { id: w.0 as isize, title: super::windows_list::title_of(w), x: x0, y: y0, w: x1 - x0, h: y1 - y0 })
        })
        .collect()
}

/// Bring a window forward and roll the mouse wheel over it (`notches` > 0 scrolls down).
/// The mouse pointer is put back where it was.
pub fn scroll_window(id: isize, at: (i32, i32), notches: i32, focus: bool) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::{SetCursorPos, SetForegroundWindow};
    unsafe {
        if focus { let _ = SetForegroundWindow(HWND(id as *mut _)); }
        let mut old = POINT::default();
        let _ = GetCursorPos(&mut old);
        let _ = SetCursorPos(at.0, at.1);
        let input = INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT {
            mouseData: (-120 * notches) as u32, dwFlags: MOUSEEVENTF_WHEEL, ..Default::default() } } };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        std::thread::sleep(std::time::Duration::from_millis(30));
        let _ = SetCursorPos(old.x, old.y);
    }
}
