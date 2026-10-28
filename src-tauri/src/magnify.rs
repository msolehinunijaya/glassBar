//! Dock magnification: the enlarged App Launcher icons are drawn in their own small window
//! over the launcher. The dock's native glass covers its whole window, so icons growing past
//! the dock can't be drawn in the dock itself. This window is transparent, never focusable
//! and lets every click through to the dock below.
use crate::platform::{shell, Glass};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const LABEL: &str = "magnify";

/// One enlarged icon, in the magnifier's own CSS px.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MagIcon { pub src: String, pub x: f64, pub y: f64, pub size: f64 }

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window(LABEL) { return Some(w); }
    let w = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("GlassBar magnifier")
        .decorations(false).transparent(true).shadow(false)
        .always_on_top(true).skip_taskbar(true).resizable(false)
        .focused(false).focusable(false).visible(false)
        .build().ok()?;
    let _ = w.set_ignore_cursor_events(true);
    shell().style_window(&w, &Glass { native: false, dark: true, material: String::new(), radius: 0 });
    shell().no_animation(&w);
    Some(w)
}

// Async on purpose: building a window inside a synchronous command (which runs on the main
// thread) deadlocks on Windows.

/// Create the magnifier ahead of the first hover (its page takes a moment to load).
#[tauri::command]
pub async fn magnify_prepare(app: AppHandle) { let _ = window(&app); }

/// Show the magnifier over `x, y, w, h`, given in the calling dock's CSS px.
#[tauri::command]
pub async fn magnify_show(app: AppHandle, window: WebviewWindow, x: f64, y: f64, w: f64, h: f64) {
    let (Some(m), Ok(pos), Ok(s)) = (self::window(&app), window.outer_position(), window.scale_factor()) else { return };
    let _ = m.set_position(PhysicalPosition::new(pos.x + (x * s).round() as i32, pos.y + (y * s).round() as i32));
    let _ = m.set_size(PhysicalSize::new((w * s).round().max(1.0) as u32, (h * s).round().max(1.0) as u32));
    let _ = m.show();
    shell().raise_top(&m);
}

#[tauri::command]
pub fn magnify_frame(app: AppHandle, icons: Vec<MagIcon>) { let _ = app.emit_to(LABEL, "magnify:frame", icons); }

#[tauri::command]
pub fn magnify_hide(app: AppHandle) {
    let _ = app.emit_to(LABEL, "magnify:frame", Vec::<MagIcon>::new());
    if let Some(m) = app.get_webview_window(LABEL) { let _ = m.hide(); }
}
