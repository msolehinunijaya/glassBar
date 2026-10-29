//! Clipboard history: every copied text or image (except what apps mark as private, e.g.
//! password managers), newest first. Copying an item puts it back on the clipboard.
//! Keeps the last 200 items; images follow the screenshot retention (default 7 days).
use crate::db::{now_ms, Db};
use base64::Engine as _;
use rusqlite::params;
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Copies kept (pinned ones come on top and don't count).
const KEEP: i64 = 20;
fn dir() -> PathBuf { crate::data_dir().join("clipboard") }

fn enabled(app: &AppHandle) -> bool {
    let st = app.state::<crate::AppState>();
    let c = st.config.lock().unwrap();
    c.uses("ocr") && c.widget_settings("ocr")["clipHistory"] != false
}

fn hash_of(bytes: &[u8]) -> String {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut s);
    format!("{:016x}", s.finish())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem { pub id: i64, pub at: i64, pub kind: String, pub text: Option<String>, pub thumb: Option<String>, pub w: Option<u32>, pub h: Option<u32>, pub pinned: bool }

/// Called on every clipboard change (from Windows' clipboard listener).
pub fn on_change(app: &AppHandle) {
    if !enabled(app) || crate::platform::shell().clipboard_is_private() { return; }
    let a = app.clone();
    // The copying app may still hold the clipboard for a moment.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(120));
        let _ = record(&a);
    });
}

fn record(app: &AppHandle) -> Result<(), String> {
    let cb = app.clipboard();
    let (kind, text, path, thumb, w, h, hash) = if let Some(t) = cb.read_text().ok().map(|t| t.trim_end_matches('\0').to_string()).filter(|t| !t.trim().is_empty()) {
        let t: String = t.chars().take(100_000).collect();
        let hash = hash_of(t.as_bytes());
        ("text", Some(t), None, None, None, None, hash)
    } else if let Ok(img) = cb.read_image() {
        let (w, h) = (img.width(), img.height());
        let rgba = img.rgba().to_vec();
        let hash = hash_of(&rgba);
        let im = image::RgbaImage::from_raw(w, h, rgba).ok_or("bad image")?;
        std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
        let p = dir().join(format!("clip-{}.png", now_ms()));
        im.save_with_format(&p, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        let scale = (240.0 / w as f32).min(120.0 / h as f32).min(1.0);
        let t = image::imageops::thumbnail(&im, ((w as f32 * scale) as u32).max(1), ((h as f32 * scale) as u32).max(1));
        let mut buf = std::io::Cursor::new(Vec::new());
        t.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        let thumb = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(buf.into_inner()));
        ("image", None, Some(p.to_string_lossy().into_owned()), Some(thumb), Some(w), Some(h), hash)
    } else {
        return Ok(()); // files, formats we don't keep
    };
    {
        let db = app.state::<Db>();
        let c = db.lock();
        // Same content again (e.g. re-copied from the history): just move it to the top.
        let dup: Option<(i64, Option<String>)> = c.query_row("SELECT id, path FROM clipboard WHERE hash = ?1", [&hash], |r| Ok((r.get(0)?, r.get(1)?))).ok();
        if let Some((id, _)) = dup {
            if let Some(p) = &path { let _ = std::fs::remove_file(p); }
            let _ = c.execute("UPDATE clipboard SET at = ?1 WHERE id = ?2", params![now_ms(), id]);
        } else {
            c.execute("INSERT INTO clipboard(at, kind, text, path, thumb, w, h, hash) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![now_ms(), kind, text, path, thumb, w, h, hash]).map_err(|e| e.to_string())?;
        }
    }
    prune(app);
    let _ = app.emit("clip:changed", ());
    Ok(())
}

/// Keep the newest `KEEP` unpinned copies; drop images older than the retention period.
/// Pinned items always stay.
pub fn prune(app: &AppHandle) {
    let days = {
        let st = app.state::<crate::AppState>();
        let c = st.config.lock().unwrap();
        c.widget_settings("ocr")["keepDays"].as_i64().filter(|d| *d > 0).unwrap_or(7)
    };
    let cutoff = now_ms() - days * 86_400_000;
    let db = app.state::<Db>();
    let gone = prune_db(&db.lock(), cutoff, KEEP);
    for p in &gone { let _ = std::fs::remove_file(p); }
    // No sweep of image files without a row: if the database ever comes up empty, that would
    // delete every copied image (it did once). A stray file costs only disk space.
}

/// The database part of `prune`: deletes the rows, returns the image files to delete.
fn prune_db(c: &rusqlite::Connection, cutoff: i64, keep: i64) -> Vec<String> {
    let gone: Vec<String> = c.prepare("SELECT path FROM clipboard WHERE pinned = 0 AND path IS NOT NULL AND (at < ?1 OR id NOT IN (SELECT id FROM clipboard WHERE pinned = 0 ORDER BY at DESC LIMIT ?2))")
        .and_then(|mut s| s.query_map(params![cutoff, keep], |r| r.get::<_, String>(0)).map(|r| r.filter_map(Result::ok).collect()))
        .unwrap_or_default();
    let _ = c.execute("DELETE FROM clipboard WHERE pinned = 0 AND ((path IS NOT NULL AND at < ?1) OR id NOT IN (SELECT id FROM clipboard WHERE pinned = 0 ORDER BY at DESC LIMIT ?2))", params![cutoff, keep]);
    gone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_20_newest_plus_every_pinned_copy() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrate(&c).unwrap();
        for i in 0..30i64 {
            c.execute("INSERT INTO clipboard(at, kind, text, hash, pinned) VALUES(?1, 'text', ?2, ?2, ?3)",
                params![1000 + i, format!("copy {i}"), (i == 2) as i64]).unwrap();
        }
        prune_db(&c, 0, KEEP);
        let left: Vec<String> = c.prepare("SELECT text FROM clipboard ORDER BY pinned DESC, at DESC").unwrap()
            .query_map([], |r| r.get(0)).unwrap().filter_map(Result::ok).collect();
        assert_eq!(left.len(), 21, "20 newest + the pinned one");
        assert_eq!(left[0], "copy 2", "the old pinned copy stays, on top");
        assert_eq!(left[1], "copy 29");
        assert_eq!(left[20], "copy 10");
    }

    #[test]
    fn history_survives_a_restart() {
        let path = std::env::temp_dir().join(format!("glassbar-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let db = crate::db::open(&path).unwrap();
            let c = db.lock();
            c.execute("INSERT INTO clipboard(at, kind, text, hash) VALUES(1, 'text', 'hello', 'h')", []).unwrap();
            c.execute("INSERT INTO ocr_history(at, text, mode, engine) VALUES(1, 'read', 'text', 'windows')", []).unwrap();
        } // closed, like quitting GlassBar
        let db = crate::db::open(&path).unwrap();
        let c = db.lock();
        let n: i64 = c.query_row("SELECT (SELECT COUNT(*) FROM clipboard) + (SELECT COUNT(*) FROM ocr_history)", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
        drop(c); drop(db);
        for ext in ["", "-wal", "-shm"] { let _ = std::fs::remove_file(format!("{}{ext}", path.display())); }
    }
}

#[tauri::command]
pub fn clip_list(app: AppHandle, query: Option<String>) -> Vec<ClipItem> {
    let db = app.state::<Db>();
    let c = db.lock();
    let q = format!("%{}%", query.unwrap_or_default());
    // Pinned first, then newest first.
    let mut st = c.prepare("SELECT id, at, kind, text, thumb, w, h, pinned FROM clipboard WHERE kind = 'image' OR text LIKE ?1 ORDER BY pinned DESC, at DESC LIMIT 200").unwrap();
    st.query_map([q], |r| Ok(ClipItem { id: r.get(0)?, at: r.get(1)?, kind: r.get(2)?, text: r.get(3)?, thumb: r.get(4)?, w: r.get(5)?, h: r.get(6)?, pinned: r.get::<_, i64>(7)? != 0 }))
        .unwrap().filter_map(Result::ok).collect()
}

/// Put an item back on the clipboard (it moves to the top of the history).
#[tauri::command]
pub async fn clip_copy(app: AppHandle, id: i64) -> Result<(), String> {
    crate::widgets::blocking(move || {
        let (text, path): (Option<String>, Option<String>) = {
            let db = app.state::<Db>();
            let c = db.lock();
            c.query_row("SELECT text, path FROM clipboard WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|_| "not found".to_string())?
        };
        match (text, path) {
            (Some(t), _) => app.clipboard().write_text(t).map_err(|e| e.to_string()),
            (_, Some(p)) => {
                let img = image::open(&p).map_err(|_| "the image is gone".to_string())?.to_rgba8();
                let (w, h) = img.dimensions();
                app.clipboard().write_image(&tauri::image::Image::new_owned(img.into_raw(), w, h)).map_err(|e| e.to_string())
            }
            _ => Err("empty item".into()),
        }
    }).await?
}

/// Pin or unpin an item: pinned items stay at the top and are never removed automatically
/// or by "Clear".
#[tauri::command]
pub fn clip_pin(app: AppHandle, id: i64, on: bool) {
    {
        let db = app.state::<Db>();
        let _ = db.lock().execute("UPDATE clipboard SET pinned = ?1 WHERE id = ?2", params![on as i64, id]);
    }
    let _ = app.emit("clip:changed", ());
}

/// Delete one item, or (no id) everything that isn't pinned.
#[tauri::command]
pub fn clip_delete(app: AppHandle, id: Option<i64>) {
    {
        let db = app.state::<Db>();
        let c = db.lock();
        let paths: Vec<String> = match id {
            Some(id) => c.query_row("SELECT path FROM clipboard WHERE id = ?1 AND path IS NOT NULL", [id], |r| r.get(0)).ok().into_iter().collect(),
            None => c.prepare("SELECT path FROM clipboard WHERE path IS NOT NULL AND pinned = 0").and_then(|mut s| s.query_map([], |r| r.get(0)).map(|r| r.filter_map(Result::ok).collect())).unwrap_or_default(),
        };
        for p in &paths { let _ = std::fs::remove_file(p); }
        let _ = match id { Some(id) => c.execute("DELETE FROM clipboard WHERE id = ?1", [id]), None => c.execute("DELETE FROM clipboard WHERE pinned = 0", []) };
    }
    let _ = app.emit("clip:changed", ());
}
