//! Screen capture (GDI) of the monitor under the cursor, and Windows' built-in OCR
//! (Windows.Media.Ocr, using the user's installed languages).
use super::super::{Res, Shot, Word};
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
        let (w, h) = (r.right - r.left, r.bottom - r.top);
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
