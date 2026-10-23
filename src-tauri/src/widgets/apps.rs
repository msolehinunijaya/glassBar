use super::blocking;
use crate::platform::{shell, AppWindow, Res};

#[tauri::command]
pub async fn apps_list() -> Vec<AppWindow> { blocking(move || shell().list_windows()).await.unwrap_or_default() }
#[tauri::command]
pub async fn app_icon(exe: String, window: Option<isize>) -> Option<String> {
    blocking(move || shell().exe_icon(&exe).or_else(|| window.and_then(|w| shell().window_icon(w)))).await.ok().flatten()
}
const VK_LWIN: u16 = 0x5B;

/// Start button: the same as pressing the Windows key.
#[tauri::command]
pub fn start_menu() { shell().press_keys(&[VK_LWIN]) }

/// Search button: Win+S.
#[tauri::command]
pub fn windows_search() { shell().press_keys(&[VK_LWIN, b'S' as u16]) }

#[tauri::command]
pub async fn app_activate(id: isize) { let _ = blocking(move || shell().activate(id)).await; }
#[tauri::command]
pub async fn app_close(id: isize) { let _ = blocking(move || shell().close(id)).await; }
#[tauri::command]
pub async fn app_launch(path: String, args: String) -> Res<()> { blocking(move || shell().launch(&path, &args)).await? }
