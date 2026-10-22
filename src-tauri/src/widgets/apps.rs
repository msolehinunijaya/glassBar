use crate::platform::{shell, AppWindow, Res};

#[tauri::command]
pub async fn apps_list() -> Vec<AppWindow> { shell().list_windows() }
#[tauri::command]
pub async fn app_icon(exe: String) -> Option<String> { shell().exe_icon(&exe) }
#[tauri::command]
pub async fn app_activate(id: isize) { shell().activate(id) }
#[tauri::command]
pub async fn app_close(id: isize) { shell().close(id) }
#[tauri::command]
pub async fn app_launch(path: String, args: String) -> Res<()> { shell().launch(&path, &args) }
