//! Backend command modules, one per widget. Each only calls `crate::platform` traits.
pub mod ai;
pub mod apps;
pub mod clipboard;
pub mod currency;
pub mod focus;
pub mod ipu;
pub mod jumplist;
pub mod location;
pub mod media;
pub mod notes;
pub mod ocr;
pub mod ocr_text;
pub mod quick;
pub mod screenshots;
pub mod screentime;
pub mod solat;
pub mod sound;
pub mod stitch;
pub mod sysmon;
pub mod todo;
pub mod weather;

use tauri::{AppHandle, Emitter};

/// Live-update watchers: OS change -> Tauri event the widgets subscribe to.
pub fn start_watchers(app: &AppHandle) {
    let p = crate::platform::shell();
    let a = app.clone();
    p.watch_windows(Box::new(move || { let _ = a.emit("apps:changed", crate::platform::shell().list_windows()); }));
    let a = app.clone();
    crate::platform::audio().watch(Box::new(move |s| { let _ = a.emit("audio:changed", s); }));
    let a = app.clone();
    crate::platform::media().watch(Box::new(move |m| { let _ = a.emit("media:changed", m); }));
}

/// Run a blocking OS call (COM, WinRT `.join()`, DDC/CI, display changes) on the
/// blocking pool, so slow calls can't starve the async workers other commands use.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())
}
