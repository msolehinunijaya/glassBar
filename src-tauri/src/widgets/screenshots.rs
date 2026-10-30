//! Screenshots: full screen, a region, a window, or a whole scrolling page. Each one is
//! copied to the clipboard, saved as a PNG in %APPDATA%\GlassBar\screenshots, listed in
//! the history, and deleted automatically after the number of days chosen (default 7).
use std::sync::atomic::{AtomicBool, Ordering};
use super::stitch::{cell_hashes, fixed_edges, fixed_sides, scrollbar_cols, row_hashes, scroll_offset, unchanged};
use crate::db::{now_ms, Db};
use crate::platform::{screen, WinRect};
use base64::Engine as _;
use rusqlite::params;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

fn dir() -> PathBuf { crate::data_dir().join("screenshots") }

fn rgba(mut bgra: Vec<u8>) -> Vec<u8> { for p in bgra.chunks_mut(4) { p.swap(0, 2); } bgra }

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShotItem { pub id: i64, pub at: i64, pub w: u32, pub h: u32, pub kind: String, pub thumb: String }

/// Save + copy + remember. Returns the history id.
pub fn save(app: &AppHandle, bgra: Vec<u8>, w: u32, h: u32, kind: &str) -> Result<i64, String> {
    let px = rgba(bgra);
    let img = image::RgbaImage::from_raw(w, h, px.clone()).ok_or("bad image")?;
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let at = now_ms();
    let path = dir().join(format!("{}-{}.png", chrono::Local::now().format("Screenshot %Y-%m-%d %H%M%S"), at % 1000));
    img.save_with_format(&path, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    // Small preview for the history list. A tall scrolling shot previews its top part
    // (shrunk whole it would be a few pixels wide).
    let top = if h > w * 6 / 5 { image::imageops::crop_imm(&img, 0, 0, w, w * 5 / 8).to_image() } else { img.clone() };
    let (tw, th) = top.dimensions();
    let scale = (240.0 / tw as f32).min(150.0 / th as f32).min(1.0);
    let thumb = image::imageops::thumbnail(&top, ((tw as f32 * scale) as u32).max(1), ((th as f32 * scale) as u32).max(1));
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    let thumb = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(buf.into_inner()));
    let id = {
        let db = app.state::<Db>();
        let c = db.lock();
        c.execute("INSERT INTO screenshots(at, path, w, h, kind, thumb) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![at, path.to_string_lossy(), w, h, kind, thumb]).map_err(|e| e.to_string())?;
        c.last_insert_rowid()
    };
    app.clipboard().write_image(&tauri::image::Image::new_owned(px, w, h)).map_err(|e| e.to_string())?;
    // Feedback card by the dock (preview + "Copied"), instead of a Windows notification.
    let _ = app.emit("capture:done", serde_json::json!({ "kind": "shot", "id": id, "thumb": thumb, "w": w, "h": h, "shotKind": kind }));
    prune(app);
    let _ = app.emit("shots:changed", ());
    Ok(id)
}

/// How many screenshots are kept (files and history); older ones are deleted.
pub const KEEP: i64 = 5;

/// Keep only the latest `KEEP` screenshots (files and history rows).
pub fn prune(app: &AppHandle) {
    let db = app.state::<Db>();
    let c = db.lock();
    let old: Vec<String> = c.prepare("SELECT path FROM screenshots WHERE id NOT IN (SELECT id FROM screenshots ORDER BY at DESC, id DESC LIMIT ?1)").and_then(|mut s| {
        s.query_map([KEEP], |r| r.get::<_, String>(0)).map(|rows| rows.filter_map(Result::ok).collect())
    }).unwrap_or_default();
    for p in &old { let _ = std::fs::remove_file(p); }
    let _ = c.execute("DELETE FROM screenshots WHERE id NOT IN (SELECT id FROM screenshots ORDER BY at DESC, id DESC LIMIT ?1)", [KEEP]);
    // No sweep of files without a row: if the database ever comes up empty, that would
    // delete every screenshot (it did once). A stray file costs only disk space.
}

fn path_of(app: &AppHandle, id: i64) -> Option<String> {
    let db = app.state::<Db>();
    let c = db.lock();
    c.query_row("SELECT path FROM screenshots WHERE id = ?1", [id], |r| r.get(0)).ok()
}

#[tauri::command]
pub fn shots_list(app: AppHandle) -> Vec<ShotItem> {
    let db = app.state::<Db>();
    let c = db.lock();
    let mut st = c.prepare("SELECT id, at, w, h, kind, thumb FROM screenshots ORDER BY id DESC LIMIT 100").unwrap();
    st.query_map([], |r| Ok(ShotItem { id: r.get(0)?, at: r.get(1)?, w: r.get(2)?, h: r.get(3)?, kind: r.get(4)?, thumb: r.get(5)? }))
        .unwrap().filter_map(Result::ok).collect()
}

#[tauri::command]
pub async fn shots_copy(app: AppHandle, id: i64) -> Result<(), String> {
    crate::widgets::blocking(move || {
        let p = path_of(&app, id).ok_or("not found")?;
        let img = image::open(&p).map_err(|_| "the file is gone".to_string())?.to_rgba8();
        let (w, h) = img.dimensions();
        app.clipboard().write_image(&tauri::image::Image::new_owned(img.into_raw(), w, h)).map_err(|e| e.to_string())
    }).await?
}

/// Discard one (or all, with None).
#[tauri::command]
pub fn shots_delete(app: AppHandle, id: Option<i64>) {
    {
        let db = app.state::<Db>();
        let c = db.lock();
        let paths: Vec<String> = match id {
            Some(id) => c.query_row("SELECT path FROM screenshots WHERE id = ?1", [id], |r| r.get(0)).ok().into_iter().collect(),
            None => c.prepare("SELECT path FROM screenshots").and_then(|mut s| s.query_map([], |r| r.get(0)).map(|r| r.filter_map(Result::ok).collect())).unwrap_or_default(),
        };
        for p in &paths { let _ = std::fs::remove_file(p); }
        let _ = match id { Some(id) => c.execute("DELETE FROM screenshots WHERE id = ?1", [id]), None => c.execute("DELETE FROM screenshots", []) };
    }
    let _ = app.emit("shots:changed", ());
}

/// Open in the default image viewer, or (None) open the screenshots folder.
#[tauri::command]
pub fn shots_open(app: AppHandle, id: Option<i64>) -> Result<(), String> {
    let target = match id { Some(id) => path_of(&app, id).ok_or("not found")?, None => { let _ = std::fs::create_dir_all(dir()); dir().to_string_lossy().into_owned() } };
    crate::platform::shell().launch(&target, "")
}

/// Whole-screen screenshot of the monitor under the mouse (no selection step).
#[tauri::command]
pub async fn shot_fullscreen(app: AppHandle) -> Result<i64, String> {
    // More than one display: freeze them all and let the user pick one (or all of them).
    if app.available_monitors().map(|m| m.len()).unwrap_or(1) > 1 {
        super::ocr::start_purpose(&app, "display").await?;
        return Ok(0);
    }
    crate::widgets::blocking(move || {
        let s = screen().capture_at_cursor().ok_or("couldn't capture the screen")?;
        save(&app, s.bgra, s.w as u32, s.h as u32, "full")
    }).await?
}

/// Set by the dock's Stop button while a scrolling capture runs.
static STOP: AtomicBool = AtomicBool::new(false);

/// Stop a scrolling capture now; what was captured so far is saved.
#[tauri::command]
pub fn shot_scroll_stop() { STOP.store(true, Ordering::Relaxed); }

/// Scroll a window top to bottom, capturing and joining as it goes. Stops at the end of
/// the page (nothing moves), after 150 steps, or at 30 000 px tall. The wheel step adapts:
/// it grows while frames overlap a lot (fast) and shrinks when the overlap gets thin, so
/// joins stay reliable in apps that scroll a little or a lot per notch.
pub fn scroll_capture(win: &WinRect) -> Result<(Vec<u8>, u32, u32), String> {
    STOP.store(false, Ordering::Relaxed);
    let s = screen();
    let at = (win.x + win.w / 2, win.y + win.h / 2);
    s.scroll_window(win.id, at, 0, true); // bring it forward (before measuring what covers it)
    std::thread::sleep(Duration::from_millis(300));
    let (cx, cy, cw, ch) = s.unobscured(win.id, (win.x, win.y, win.w, win.h));
    if ch < 120 { return Err("the window is mostly covered".into()); }
    let grab = || s.capture_rect(cx, cy, cw, ch).ok_or_else(|| "couldn't capture the window".to_string());
    let (w, n) = (cw as usize, ch as usize);
    // Browsers scroll smoothly: wait until the picture stops moving (two frames alike,
    // at most ~1.5 s) so a frame is never taken mid-animation.
    let settled = || -> Result<Vec<u8>, String> {
        std::thread::sleep(Duration::from_millis(200));
        let mut last = grab()?.bgra;
        for _ in 0..13 {
            std::thread::sleep(Duration::from_millis(100));
            let f = grab()?.bgra;
            let same = unchanged(&row_hashes(&last, w as u32, n as u32), &row_hashes(&f, w as u32, n as u32));
            last = f;
            if same { break; }
        }
        Ok(last)
    };
    let row = w * 4;
    let mut prev = grab()?.bgra;
    let prev_h = row_hashes(&prev, w as u32, n as u32);
    // Window parts that don't scroll, found from the first pair of frames: toolbars at the
    // top, status bars at the bottom, side panels / vertical tabs at the left and right.
    let mut layout: Option<(usize, usize, usize, usize)> = None;
    let mut prev_c: Vec<Vec<u64>> = Vec::new();
    let mut body: Vec<u8> = prev.clone();
    let mut notches = 3;
    // Rows one wheel notch scrolls this page, from the steps that lined up (browsers: 100 px).
    let mut per_notch: Option<f64> = None;
    let mut bar = 0; // scrollbar columns at the right, cut off at the end
    // The last rows of the view are left out of every join (and added once, from the last
    // frame): browsers round the page's bottom corners, which would mark every seam.
    const SEAM: usize = 12;
    // The page's columns, without still side panels and the scrollbar (its thumb moves).
    let cols = |left: usize, right: usize| -> (usize, usize) {
        let x0 = if left == 0 && w > 120 { 8 } else { left };
        let x1 = if w > 120 { (w - right).min(w - 36) } else { w - right };
        if x1 > x0 + 16 { (x0, x1) } else { (0, w) }
    };
    for _ in 0..150 {
        if STOP.load(Ordering::Relaxed) { break; } // the dock's Stop button: keep what we have
        s.scroll_window(win.id, at, notches, false);
        let next = settled()?;
        let next_h = row_hashes(&next, w as u32, n as u32);
        let (top, bottom, left, right) = match layout {
            Some(l) => l,
            None => {
                let (top, bottom) = fixed_edges(&prev_h, &next_h);
                if top + bottom >= n { break; } // nothing moved at all
                let (left, right) = fixed_sides(&prev, &next, w, top..n - bottom);
                if n - top - bottom < 3 * SEAM { break; } // too short to scroll-capture
                body.truncate((n - bottom - SEAM) * row);
                let (x0, x1) = cols(left, right);
                prev_c = cell_hashes(&prev, w as u32, n as u32, x0, x1);
                *layout.insert((top, bottom, left, right))
            }
        };
        let (x0, x1) = cols(left, right);
        let next_c = cell_hashes(&next, w as u32, n as u32, x0, x1);
        let (pv, nx) = (&prev_c[top..n - bottom], &next_c[top..n - bottom]);
        // Browsers scroll 100 px per notch times the display scaling (100–200 %).
        let expect: Vec<usize> = match per_notch {
            Some(p) => vec![(p * notches as f64).round() as usize],
            None => [100, 125, 150, 175, 200].iter().map(|p| p * notches as usize).collect(),
        };
        let found = scroll_offset(pv, nx, &expect);
        if cfg!(test) {
            eprintln!("step notches={notches} edges=({top},{bottom}) sides=({left},{right}) d={found:?} expect={expect:?} body={}", body.len() / row);
        }
        let Some(d) = found else {
            if unchanged(pv, nx) || notches == 1 { break; } // end of the page (or truly lost)
            // Moved too far to line up: go back and retry with the smallest step. `prev`
            // stays the reference, so the join is still made against the last good frame.
            s.scroll_window(win.id, at, -notches, false);
            notches = 1;
            settled()?;
            continue;
        };
        per_notch = Some(d as f64 / notches as f64);
        let view = n - top - bottom;
        if bar == 0 { bar = scrollbar_cols(&prev, &next, w, top..n - bottom - d, d, w - right, 40.min(w / 8)); }
        if d * 3 < view { notches = (notches * 2).min(15); } else if d * 10 > view * 7 { notches = (notches / 2).max(1); }
        body.extend_from_slice(&next[(n - bottom - SEAM - d) * row..(n - bottom - SEAM) * row]);
        if body.len() / row > 30_000 { break; }
        prev = next;
        prev_c = next_c;
    }
    if let Some((_, bottom, _, _)) = layout { body.extend_from_slice(&prev[(n - bottom - SEAM) * row..]); }
    let h = body.len() / row;
    let Some((_, bottom, left, right)) = layout else { return Ok((body, w as u32, h as u32)) };
    // Still side panels (a site's menu, the browser's tab strip) stay as on the first screen;
    // further down they'd repeat in slices, so they continue in their own background colour.
    fill_sides(&mut body, w, n - bottom - SEAM, h - bottom, left, right);
    // The scrollbar (its thumb moves) is left out.
    if bar == 0 { return Ok((body, w as u32, h as u32)); }
    let x_bar = w - right - bar;
    let mut out = Vec::with_capacity((w - bar) * h * 4);
    for r in 0..h {
        out.extend_from_slice(&body[r * row..(r * w + x_bar) * 4]);
        out.extend_from_slice(&body[(r * w + w - right) * 4..(r + 1) * row]);
    }
    Ok((out, (w - bar) as u32, h as u32))
}

/// Paint columns `..left` and `w - right..` of rows `from..to` with the average colour each
/// side has in row `from - 1` (the last row the first screen gave them).
pub fn fill_sides(px: &mut [u8], w: usize, from: usize, to: usize, left: usize, right: usize) {
    if from == 0 || to <= from { return; }
    let avg = |x0: usize, x1: usize| -> [u8; 4] {
        let r = &px[((from - 1) * w + x0) * 4..((from - 1) * w + x1) * 4];
        let n = (x1 - x0).max(1) as u32;
        let mut s = [0u32; 4];
        for p in r.chunks_exact(4) { for i in 0..4 { s[i] += p[i] as u32; } }
        s.map(|v| (v / n) as u8)
    };
    let sides: Vec<(usize, usize, [u8; 4])> = [(0, left), (w - right, w)].into_iter().filter(|(a, b)| b > a).map(|(a, b)| (a, b, avg(a, b))).collect();
    for (x0, x1, c) in sides {
        for r in from..to { for x in x0..x1 { px[(r * w + x) * 4..(r * w + x) * 4 + 4].copy_from_slice(&c); } }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn still_sides_continue_in_their_colour_below_the_first_screen() {
        // 6 px wide, 4 rows: left 2 px is a menu (row 1 ends it in grey 10/20), right 1 px a border.
        let w = 6;
        let mut px = vec![0u8; w * 4 * 4];
        for x in 0..2 { px[(w + x) * 4..(w + x) * 4 + 4].copy_from_slice(&[if x == 0 { 10 } else { 30 }, 20, 20, 255]); }
        px[(w + 5) * 4..(w + 5) * 4 + 4].copy_from_slice(&[9, 9, 9, 255]);
        px[(3 * w + 3) * 4] = 77; // page content stays
        super::fill_sides(&mut px, w, 2, 4, 2, 1);
        for r in 2..4 {
            assert_eq!(&px[(r * w) * 4..(r * w) * 4 + 4], &[20, 20, 20, 255], "left: average of the last menu row");
            assert_eq!(&px[(r * w + 5) * 4..(r * w + 5) * 4 + 4], &[9, 9, 9, 255]);
        }
        assert_eq!(px[(3 * w + 3) * 4], 77);
        assert_eq!(px[(w) * 4], 10, "the first screen is untouched");
    }

    /// Live run on a real window: `GB_WIN=Chrome GB_OUT=out.png cargo test scroll_capture_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn scroll_capture_live() {
        let want = std::env::var("GB_WIN").unwrap_or_else(|_| "Chrome".into());
        let s = crate::platform::screen();
        let wins = s.window_rects((-10_000, -10_000, 20_000, 20_000));
        let win = wins.into_iter().find(|w| w.title.contains(&want)).expect("no such window");
        eprintln!("window {:?} {}x{} at {},{}", win.title, win.w, win.h, win.x, win.y);
        for _ in 0..40 { // start at the top (up to 40 000 px up)
            s.scroll_window(win.id, (win.x + win.w / 2, win.y + win.h / 2), -10, true);
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let (px, w, h) = super::scroll_capture(&win).unwrap();
        eprintln!("result {w}x{h}");
        if let Ok(out) = std::env::var("GB_OUT") { image::RgbaImage::from_raw(w, h, super::rgba(px)).unwrap().save(out).unwrap(); }
    }
}
