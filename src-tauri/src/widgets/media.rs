use crate::platform::{media, MediaAction, MediaInfo, Res};

#[tauri::command]
pub async fn media_info() -> Res<Option<MediaInfo>> { media().info() }
#[tauri::command]
pub async fn media_control(action: MediaAction) -> Res<()> { media().control(action) }
#[tauri::command]
pub async fn media_seek(position_ms: i64) -> Res<()> { media().seek(position_ms) }
