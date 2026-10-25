//! OCR Snap: hotkey -> freeze the screen under the mouse -> drag a box -> text on the
//! clipboard. Engine: Windows OCR (default, offline) or Claude vision (opt-in).
use super::ocr_text::{format, Mode};
use crate::db::{now_ms, Db};
use crate::platform::{screen, Shot};
use base64::Engine as _;
use rusqlite::params;
use serde::Serialize;
use serde_json::json;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub const DEFAULT_HOTKEY: &str = "Super+Shift+T";
const FALLBACK_HOTKEY: &str = "Control+Alt+T";
static SHOT: Mutex<Option<Shot>> = Mutex::new(None);
/// What the open overlay is for: "ocr", or a screenshot of a "region", "window" or "scroll"ing window.
static PURPOSE: Mutex<String> = Mutex::new(String::new());
/// Windows visible in the frozen shot (absolute coords), for picking one.
static WINDOWS: Mutex<Vec<crate::platform::WinRect>> = Mutex::new(Vec::new());
static HOTKEY: Mutex<(String, Option<String>)> = Mutex::new((String::new(), None)); // (registered, error)

struct Opts { mode: Mode, engine: String, auto_copy: bool, toast: bool }
fn opts(app: &AppHandle) -> Opts {
    let st = app.state::<crate::AppState>();
    let cfg = st.config.lock().unwrap();
    let s = cfg.widget_settings("ocr");
    Opts {
        mode: serde_json::from_value(s["mode"].clone()).unwrap_or_default(),
        engine: s["engine"].as_str().unwrap_or("windows").to_string(),
        auto_copy: s["autoCopy"].as_bool().unwrap_or(true),
        toast: s["toast"].as_bool().unwrap_or(true) && !cfg.dnd,
    }
}

// ---- hotkey ----

/// (Re)register the snap hotkey from settings; call at startup and after config saves.
pub fn register_hotkey(app: &AppHandle) {
    let want = {
        let st = app.state::<crate::AppState>();
        let cfg = st.config.lock().unwrap();
        cfg.widget_settings("ocr")["hotkey"].as_str().filter(|s| !s.is_empty()).unwrap_or(DEFAULT_HOTKEY).to_string()
    };
    let mut cur = HOTKEY.lock().unwrap();
    if (cur.0 == want || (want == DEFAULT_HOTKEY && cur.0 == FALLBACK_HOTKEY)) && cur.1.is_none() { return; }
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let try_reg = |s: &str| s.parse::<Shortcut>().map_err(|e| format!("\"{s}\" isn't a valid shortcut ({e})"))
        .and_then(|sc| gs.register(sc).map_err(|_| format!("{s} is already used by Windows or another app")));
    cur.1 = try_reg(&want).err();
    cur.0 = want.clone();
    // The default is often taken (PowerToys Text Extractor uses Win+Shift+T): fall back once.
    if cur.1.is_some() && want == DEFAULT_HOTKEY && try_reg(FALLBACK_HOTKEY).is_ok() {
        cur.0 = FALLBACK_HOTKEY.into();
        cur.1 = None;
    }
}

/// Called by the global-shortcut plugin on key press.
pub fn on_hotkey(app: &AppHandle) {
    let a = app.clone();
    // Never build/show windows on the event-loop thread (deadlocks on Windows).
    tauri::async_runtime::spawn(async move { let _ = start(&a, "ocr").await; });
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus { hotkey: String, error: Option<String> }
#[tauri::command]
pub fn ocr_hotkey_status() -> HotkeyStatus {
    let h = HOTKEY.lock().unwrap();
    HotkeyStatus { hotkey: h.0.clone(), error: h.1.clone() }
}

// ---- snap ----

async fn start(app: &AppHandle, purpose: &str) -> Result<(), String> {
    let (shot, wins) = crate::widgets::blocking(|| {
        let s = screen().capture_at_cursor();
        let w = s.as_ref().map(|s| screen().window_rects((s.x, s.y, s.w, s.h))).unwrap_or_default();
        (s, w)
    }).await?;
    let shot = shot.ok_or("couldn't capture the screen")?;
    let (x, y, w, h) = (shot.x, shot.y, shot.w, shot.h);
    *SHOT.lock().unwrap() = Some(shot);
    *PURPOSE.lock().unwrap() = purpose.to_string();
    *WINDOWS.lock().unwrap() = wins;
    let win = match app.get_webview_window("snap") {
        Some(w) => w,
        None => WebviewWindowBuilder::new(app, "snap", WebviewUrl::App("index.html".into()))
            .title("GlassBar Snap").decorations(false).skip_taskbar(true).always_on_top(true)
            .resizable(false).shadow(false).visible(false).focused(true)
            .build().map_err(|e| e.to_string())?,
    };
    win.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    win.set_size(PhysicalSize::new(w as u32, h as u32)).map_err(|e| e.to_string())?;
    // The overlay shows itself (ocr_snap_show) once the screenshot is drawn: shown now, it
    // would be a black window until the image has loaded.
    let _ = app.emit_to("snap", "snap:ready", ());
    Ok(())
}

#[tauri::command]
pub fn ocr_snap_show(app: AppHandle) {
    if let Some(w) = app.get_webview_window("snap") { let _ = w.show(); let _ = w.set_focus(); }
}

/// purpose: "ocr" (default) | "region" | "window" | "scroll".
#[tauri::command]
pub async fn ocr_snap_start(app: AppHandle, purpose: Option<String>) -> Result<(), String> {
    let p = purpose.unwrap_or_else(|| "ocr".into());
    if !matches!(p.as_str(), "ocr" | "region" | "window" | "scroll") { return Err("unknown capture".into()); }
    start(&app, &p).await
}

#[derive(Serialize)]
pub struct SnapView { img: String, purpose: String, windows: Vec<crate::platform::WinRect> }

/// The frozen screenshot (PNG data URL) for the overlay, what it's for, and the windows in
/// it (coordinates relative to the screenshot).
#[tauri::command]
pub async fn ocr_shot() -> Option<SnapView> {
    crate::widgets::blocking(|| {
        let g = SHOT.lock().unwrap();
        let s = g.as_ref()?;
        let img = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png(&s.bgra, s.w as u32, s.h as u32)?));
        let windows = WINDOWS.lock().unwrap().iter().map(|w| crate::platform::WinRect { x: w.x - s.x, y: w.y - s.y, ..w.clone() }).collect();
        Some(SnapView { img, purpose: PURPOSE.lock().unwrap().clone(), windows })
    }).await.ok().flatten()
}

fn png(bgra: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let mut rgba = bgra.to_vec();
    for p in rgba.chunks_mut(4) { p.swap(0, 2); }
    // Fast compression: this PNG only feeds the overlay, and the wait shows as a delay.
    use image::ImageEncoder;
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new_with_quality(&mut out, image::codecs::png::CompressionType::Fast, image::codecs::png::FilterType::Sub)
        .write_image(&rgba, w, h, image::ExtendedColorType::Rgba8).ok()?;
    Some(out)
}

/// Crop (x, y, w, h) out of the shot, clamped to its bounds.
pub fn crop(bgra: &[u8], sw: u32, sh: u32, x: u32, y: u32, w: u32, h: u32) -> Option<(Vec<u8>, u32, u32)> {
    let (x, y) = (x.min(sw), y.min(sh));
    let (w, h) = (w.min(sw - x), h.min(sh - y));
    if w < 4 || h < 4 { return None; }
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for row in y..y + h {
        let s = ((row * sw + x) * 4) as usize;
        out.extend_from_slice(&bgra[s..s + (w * 4) as usize]);
    }
    Some((out, w, h))
}

fn hide(app: &AppHandle) { if let Some(w) = app.get_webview_window("snap") { let _ = w.hide(); } }

#[tauri::command]
pub fn ocr_snap_cancel(app: AppHandle) { hide(&app); *SHOT.lock().unwrap() = None; }

fn recognise(app: &AppHandle, bgra: Vec<u8>, w: u32, h: u32, o: &Opts) -> Result<String, String> {
    if o.engine == "ai" {
        let png = png(&bgra, w, h).ok_or("couldn't encode the selection")?;
        let how = match o.mode {
            Mode::Text => "Keep the line breaks.",
            Mode::Code => "It is source code: keep indentation exactly. No markdown fences.",
            Mode::Terminal => "It is a terminal: keep commands and output, drop shell prompts. No markdown fences.",
        };
        let content = json!([
            { "type": "image", "source": { "type": "base64", "media_type": "image/png",
              "data": base64::engine::general_purpose::STANDARD.encode(png) } },
            { "type": "text", "text": format!("Transcribe all text in this image exactly. Output only the text. {how}") },
        ]);
        return super::ai::ask(app, content, "low", 8000).map(|t| t.trim().to_string());
    }
    // Windows OCR reads small text better when it's bigger.
    let (bgra, w, h) = if w < 1200 && h < 600 {
        let img = image::RgbaImage::from_raw(w, h, bgra).ok_or("bad image")?;
        let big = image::imageops::resize(&img, w * 2, h * 2, image::imageops::FilterType::Triangle);
        (big.into_raw(), w * 2, h * 2)
    } else { (bgra, w, h) };
    Ok(format(&screen().ocr(&bgra, w, h)?, o.mode))
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OcrItem { pub id: i64, pub at: i64, pub text: String, pub mode: String, pub engine: String }

/// Selection in screenshot pixels (and the picked window, for window/scroll shots).
/// OCR returns the recognised text; screenshots return "saved".
#[tauri::command]
pub async fn ocr_snap_finish(app: AppHandle, x: u32, y: u32, w: u32, h: u32, window: Option<isize>) -> Result<String, String> {
    hide(&app);
    let purpose = std::mem::take(&mut *PURPOSE.lock().unwrap());
    if purpose != "ocr" && !purpose.is_empty() {
        let a = app.clone();
        let r = crate::widgets::blocking(move || -> Result<String, String> {
            let shot = SHOT.lock().unwrap().take().ok_or("no screenshot")?;
            if purpose == "scroll" {
                let win = WINDOWS.lock().unwrap().iter().find(|w| Some(w.id) == window).cloned().ok_or("pick a window to scroll")?;
                std::thread::sleep(std::time::Duration::from_millis(150)); // let the overlay vanish
                let (px, pw, ph) = super::screenshots::scroll_capture(&win)?;
                super::screenshots::save(&a, px, pw, ph, "scroll")?;
            } else {
                let (px, cw, ch) = crop(&shot.bgra, shot.w as u32, shot.h as u32, x, y, w, h).ok_or("selection too small")?;
                super::screenshots::save(&a, px, cw, ch, if purpose == "window" { "window" } else { "region" })?;
            }
            Ok("saved".into())
        }).await?;
        if let Err(e) = &r { let _ = app.emit("capture:done", json!({ "kind": "error", "text": e })); }
        return r;
    }
    let a = app.clone();
    let r = crate::widgets::blocking(move || -> Result<String, String> {
        let shot = SHOT.lock().unwrap().take().ok_or("no screenshot")?;
        let (bgra, cw, ch) = crop(&shot.bgra, shot.w as u32, shot.h as u32, x, y, w, h).ok_or("selection too small")?;
        let o = opts(&a);
        let text = recognise(&a, bgra, cw, ch, &o)?;
        if text.trim().is_empty() { return Err("No text found in the selection".into()); }
        {
            let db = a.state::<Db>();
            let c = db.0.lock().unwrap();
            let mode = serde_json::to_value(o.mode).unwrap().as_str().unwrap_or("text").to_string();
            let _ = c.execute("INSERT INTO ocr_history(at, text, mode, engine) VALUES(?1, ?2, ?3, ?4)", params![now_ms(), text, mode, o.engine]);
            let _ = c.execute("DELETE FROM ocr_history WHERE id NOT IN (SELECT id FROM ocr_history ORDER BY id DESC LIMIT 200)", []);
        }
        if o.auto_copy { a.clipboard().write_text(text.clone()).map_err(|e| e.to_string())?; }
        let shown: String = text.chars().take(400).collect();
        let _ = a.emit("capture:done", json!({ "kind": "ocr", "text": shown, "copied": o.auto_copy, "lines": text.lines().count() }));
        Ok(text)
    }).await?;
    if let Err(e) = &r { let _ = app.emit("capture:done", json!({ "kind": "error", "text": e })); }
    let _ = app.emit("ocr:changed", ());
    r
}

// ---- history ----

#[tauri::command]
pub fn ocr_history(db: tauri::State<Db>) -> Vec<OcrItem> {
    let c = db.0.lock().unwrap();
    let mut st = c.prepare("SELECT id, at, text, mode, engine FROM ocr_history ORDER BY id DESC LIMIT 200").unwrap();
    st.query_map([], |r| Ok(OcrItem { id: r.get(0)?, at: r.get(1)?, text: r.get(2)?, mode: r.get(3)?, engine: r.get(4)? }))
        .unwrap().filter_map(Result::ok).collect()
}
#[tauri::command]
pub fn ocr_history_delete(app: AppHandle, id: Option<i64>) {
    {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let _ = match id { Some(id) => c.execute("DELETE FROM ocr_history WHERE id = ?1", [id]), None => c.execute("DELETE FROM ocr_history", []) };
    }
    let _ = app.emit("ocr:changed", ());
}
#[tauri::command]
pub fn ocr_copy(app: AppHandle, text: String) -> Result<(), String> { app.clipboard().write_text(text).map_err(|e| e.to_string()) }

#[cfg(test)]
mod tests {
    #[test]
    fn crop_takes_rows_and_clamps() {
        // 4x2 image, pixel value = index
        let img: Vec<u8> = (0..8u8).flat_map(|i| [i, i, i, 255]).collect();
        assert!(super::crop(&img, 4, 2, 0, 0, 2, 2).is_none(), "under 4px is rejected");
        let big: Vec<u8> = (0..100u8).flat_map(|i| [i, 0, 0, 255]).collect(); // 10x10
        let (c, w, h) = super::crop(&big, 10, 10, 6, 5, 20, 20).unwrap();
        assert_eq!((w, h), (4, 5));
        assert_eq!(c[0], 56); // row 5, col 6
        assert_eq!(c[4 * 4], 66); // next row
    }
}
