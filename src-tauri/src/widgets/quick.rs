use super::blocking;
use crate::platform::{display, radios, DisplayMode, RadioInfo, RadioKind, Res};

#[tauri::command]
pub async fn radios_list() -> Res<Vec<RadioInfo>> { blocking(move || radios().list()).await? }
#[tauri::command]
pub async fn radio_set(kind: RadioKind, on: bool) -> Res<()> { blocking(move || radios().set(Some(kind), on)).await? }
/// Windows has no public airplane-mode API: "on" turns every radio off.
#[tauri::command]
pub async fn airplane_set(on: bool) -> Res<()> { blocking(move || radios().set(None, !on)).await? }
#[tauri::command]
pub async fn display_mode() -> Option<DisplayMode> { blocking(move || display().mode()).await.ok().flatten() }
#[tauri::command]
pub async fn display_set_mode(mode: DisplayMode) -> Res<()> { blocking(move || display().set_mode(mode)).await? }
#[tauri::command]
pub async fn brightness_get() -> Option<u32> { blocking(move || display().brightness()).await.ok().flatten() }
#[tauri::command]
pub async fn brightness_set(value: u32) -> Res<()> { blocking(move || display().set_brightness(value)).await? }
