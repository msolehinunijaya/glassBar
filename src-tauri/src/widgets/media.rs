use super::blocking;
use crate::platform::{media, MediaAction, MediaInfo, Res};

#[tauri::command]
pub async fn media_info() -> Res<Option<MediaInfo>> { blocking(move || media().info()).await? }
#[tauri::command]
pub async fn media_control(action: MediaAction) -> Res<()> { blocking(move || media().control(action)).await? }
#[tauri::command]
pub async fn media_seek(position_ms: i64) -> Res<()> { blocking(move || media().seek(position_ms)).await? }
#[tauri::command]
pub async fn media_pause_playing() -> Res<Vec<String>> { blocking(move || media().pause_playing()).await? }
#[tauri::command]
pub async fn media_resume(ids: Vec<String>) -> Res<()> { blocking(move || media().resume(&ids)).await? }
