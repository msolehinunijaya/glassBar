//! Microphone (Quick Settings): mute the default recording device, its level and device,
//! and which apps use the microphone or camera right now (the dock's "on air" light).
use super::blocking;
use crate::platform::{audio, MicState, UsingApp};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug, Default)]
pub struct InUse { pub mic: Vec<UsingApp>, pub cam: Vec<UsingApp> }

#[tauri::command]
pub async fn mic_state() -> Result<MicState, String> { blocking(|| audio().mic_state()).await? }
#[tauri::command]
pub async fn mic_set_mute(muted: bool) -> Result<(), String> { blocking(move || audio().mic_set_mute(muted)).await? }
#[tauri::command]
pub async fn mic_set_volume(volume: f32) -> Result<(), String> { blocking(move || audio().mic_set_volume(volume)).await? }
/// Make a recording device the default (all roles, like Windows' Sound settings).
#[tauri::command]
pub async fn mic_set_default(id: String) -> Result<(), String> { blocking(move || audio().set_default(&id)).await? }
#[tauri::command]
pub async fn privacy_in_use() -> Result<InUse, String> {
    blocking(|| InUse { mic: audio().using("microphone"), cam: audio().using("webcam") }).await
}

/// Watchers for the life of the app (each asleep in Windows until something changes):
/// `mic:changed` on mute/level/device, `privacy:changed` when an app starts or stops using
/// the microphone or camera.
pub fn start(app: &AppHandle) {
    let a = app.clone();
    audio().watch_mic(Box::new(move || { let _ = a.emit("mic:changed", ()); }));
    let a = app.clone();
    audio().watch_using(Box::new(move || { let _ = a.emit("privacy:changed", ()); }));
}
