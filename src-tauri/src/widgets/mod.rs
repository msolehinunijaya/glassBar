//! Backend command modules, one per widget. Each only calls `crate::platform` traits.
pub mod apps;
pub mod media;
pub mod quick;
pub mod sound;

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
