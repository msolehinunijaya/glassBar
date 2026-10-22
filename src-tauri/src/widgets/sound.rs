use crate::platform::{audio, AudioSession, AudioState, Res};

#[tauri::command]
pub async fn audio_state() -> Res<AudioState> { audio().state() }
#[tauri::command]
pub async fn audio_set_volume(volume: f32) -> Res<()> { audio().set_volume(volume) }
#[tauri::command]
pub async fn audio_set_mute(muted: bool) -> Res<()> { audio().set_mute(muted) }
#[tauri::command]
pub async fn audio_set_default(id: String) -> Res<()> { audio().set_default(&id) }
#[tauri::command]
pub async fn audio_sessions() -> Res<Vec<AudioSession>> { audio().sessions() }
#[tauri::command]
pub async fn audio_set_session_volume(pid: u32, volume: f32) -> Res<()> { audio().set_session_volume(pid, volume) }
#[tauri::command]
pub async fn audio_set_session_mute(pid: u32, muted: bool) -> Res<()> { audio().set_session_mute(pid, muted) }
