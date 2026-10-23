//! Screenshots: full screen, a region, a window, or a whole scrolling page. Each one is
//! copied to the clipboard, saved as a PNG in %APPDATA%\GlassBar\screenshots, listed in
//! the history, and deleted automatically after the number of days chosen (default 7).
use super::stitch::{fixed_edges, row_hashes, scroll_offset};
use crate::db::{now_ms, Db};
use crate::platform::{screen, WinRect};
use base64::Engine as _;
use rusqlite::params;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

fn dir() -> PathBuf { crate::data_dir().join("screenshots") }

fn keep_days(app: &AppHandle) -> i64 {
    let st = app.state::<crate::AppState>();
    let c = st.config.lock().unwrap();
    c.widget_settings("ocr")["keepDays"].as_i64().filter(|d| *d > 0).unwrap_or(7)
}

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
        let c = db.0.lock().unwrap();
        c.execute("INSERT INTO screenshots(at, path, w, h, kind, thumb) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![at, path.to_string_lossy(), w, h, kind, thumb]).map_err(|e| e.to_string())?;
        c.last_insert_rowid()
    };
    app.clipboard().write_image(&tauri::image::Image::new_owned(px, w, h)).map_err(|e| e.to_string())?;
    let toast = {
        let st = app.state::<crate::AppState>();
        let c = st.config.lock().unwrap();
        c.widget_settings("ocr")["toast"] != false && !c.dnd
    };
    if toast { let _ = app.notification().builder().title("Screenshot copied").body(format!("{w} × {h} · saved to history")).show(); }
    prune(app);
    let _ = app.emit("shots:changed", ());
    Ok(id)
}

/// Delete screenshots older than the retention period (files and history rows).
pub fn prune(app: &AppHandle) {
    let cutoff = now_ms() - keep_days(app) * 86_400_000;
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    let old: Vec<String> = c.prepare("SELECT path FROM screenshots WHERE at < ?1").and_then(|mut s| {
        s.query_map([cutoff], |r| r.get::<_, String>(0)).map(|rows| rows.filter_map(Result::ok).collect())
    }).unwrap_or_default();
    for p in &old { let _ = std::fs::remove_file(p); }
    let _ = c.execute("DELETE FROM screenshots WHERE at < ?1", [cutoff]);
}

fn path_of(app: &AppHandle, id: i64) -> Option<String> {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    c.query_row("SELECT path FROM screenshots WHERE id = ?1", [id], |r| r.get(0)).ok()
}

#[tauri::command]
pub fn shots_list(app: AppHandle) -> Vec<ShotItem> {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
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
        let c = db.0.lock().unwrap();
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
    crate::widgets::blocking(move || {
        let s = screen().capture_at_cursor().ok_or("couldn't capture the screen")?;
        save(&app, s.bgra, s.w as u32, s.h as u32, "full")
    }).await?
}

/// Scroll a window top to bottom, capturing and joining as it goes. Stops at the end of
/// the page (nothing moves), after 150 steps, or at 30 000 px tall. The wheel step adapts:
/// it grows while frames overlap a lot (fast) and shrinks when the overlap gets thin, so
/// joins stay reliable in apps that scroll a little or a lot per notch.
pub fn scroll_capture(win: &WinRect) -> Result<(Vec<u8>, u32, u32), String> {
    let s = screen();
    let at = (win.x + win.w / 2, win.y + win.h / 2);
    let grab = || s.capture_rect(win.x, win.y, win.w, win.h).ok_or_else(|| "couldn't capture the window".to_string());
    s.scroll_window(win.id, at, 0, true); // bring it forward
    std::thread::sleep(Duration::from_millis(300));
    let (w, n) = (win.w as usize, win.h as usize);
    let row = w * 4;
    let mut prev = grab()?.bgra;
    let mut prev_h = row_hashes(&prev, w as u32, n as u32);
    let mut edges: Option<(usize, usize)> = None;
    let mut body: Vec<u8> = prev.clone();
    let mut footer: Vec<u8> = Vec::new();
    let mut notches = 3;
    for _ in 0..150 {
        s.scroll_window(win.id, at, notches, false);
        std::thread::sleep(Duration::from_millis(300));
        let next = grab()?.bgra;
        let next_h = row_hashes(&next, w as u32, n as u32);
        let (top, bottom) = *edges.get_or_insert_with(|| {
            let e = fixed_edges(&prev_h, &next_h);
            // First pair: keep the first frame's footer aside, to add once at the very end.
            if e.0 < n { footer = prev[(n - e.1) * row..].to_vec(); body.truncate((n - e.1) * row); }
            e
        });
        if top + bottom >= n { break; }
        let Some(d) = scroll_offset(&prev_h[top..n - bottom], &next_h[top..n - bottom]) else { break };
        let view = n - top - bottom;
        if d * 3 < view { notches = (notches * 2).min(15); } else if d * 10 > view * 7 { notches = (notches / 2).max(1); }
        body.extend_from_slice(&next[(n - bottom - d) * row..(n - bottom) * row]);
        if body.len() / row > 30_000 { break; }
        prev = next;
        prev_h = next_h;
    }
    body.extend_from_slice(&footer);
    let h = (body.len() / row) as u32;
    Ok((body, w as u32, h))
}
