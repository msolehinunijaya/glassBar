use super::blocking;
use crate::platform::{audio, AudioSession, AudioState, Res};

#[tauri::command]
pub async fn audio_state() -> Res<AudioState> { blocking(move || audio().state()).await? }
#[tauri::command]
pub async fn audio_set_volume(volume: f32) -> Res<()> { blocking(move || audio().set_volume(volume)).await? }
#[tauri::command]
pub async fn audio_set_mute(muted: bool) -> Res<()> { blocking(move || audio().set_mute(muted)).await? }
#[tauri::command]
pub async fn audio_set_default(id: String) -> Res<()> { blocking(move || audio().set_default(&id)).await? }
#[tauri::command]
pub async fn audio_sessions() -> Res<Vec<AudioSession>> { blocking(move || audio().sessions()).await? }
#[tauri::command]
pub async fn audio_set_session_volume(pid: u32, volume: f32) -> Res<()> { blocking(move || audio().set_session_volume(pid, volume)).await? }
#[tauri::command]
pub async fn audio_set_session_mute(pid: u32, muted: bool) -> Res<()> { blocking(move || audio().set_session_mute(pid, muted)).await? }
