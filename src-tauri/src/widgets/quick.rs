use crate::platform::{display, radios, DisplayMode, RadioInfo, RadioKind, Res};

#[tauri::command]
pub async fn radios_list() -> Res<Vec<RadioInfo>> { radios().list() }
#[tauri::command]
pub async fn radio_set(kind: RadioKind, on: bool) -> Res<()> { radios().set(Some(kind), on) }
/// Windows has no public airplane-mode API: "on" turns every radio off.
#[tauri::command]
pub async fn airplane_set(on: bool) -> Res<()> { radios().set(None, !on) }
#[tauri::command]
pub async fn display_mode() -> Option<DisplayMode> { display().mode() }
#[tauri::command]
pub async fn display_set_mode(mode: DisplayMode) -> Res<()> { display().set_mode(mode) }
#[tauri::command]
pub async fn brightness_get() -> Option<u32> { display().brightness() }
#[tauri::command]
pub async fn brightness_set(value: u32) -> Res<()> { display().set_brightness(value) }
