#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bar;
mod config;
mod platform;
mod widgets;

use config::{Config, TaskbarMode};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

pub struct AppState {
    pub config: Mutex<Config>,
    pub path: PathBuf,
    pub popup: Mutex<bar::PopupState>,
    pub edit: AtomicBool,
}

/// Single write path for config: validate, persist, apply, broadcast.
pub fn commit(app: &AppHandle, cfg: Config) -> Result<(), String> {
    let cfg = cfg.normalized();
    let st = app.state::<AppState>();
    config::save(&st.path, &cfg).map_err(|e| e.to_string())?;
    *st.config.lock().unwrap() = cfg.clone();
    bar::apply(app);
    bar::refresh_tray(app);
    let al = app.autolaunch();
    if al.is_enabled().unwrap_or(false) != cfg.startup.launch_at_login {
        let _ = if cfg.startup.launch_at_login { al.enable() } else { al.disable() };
    }
    let _ = app.emit("config:changed", &cfg);
    Ok(())
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Config { state.config.lock().unwrap().clone() }

#[tauri::command]
fn save_config(app: AppHandle, config: Config) -> Result<(), String> { commit(&app, config) }

#[tauri::command]
fn export_config(state: State<AppState>) -> String { config::export(&state.config.lock().unwrap()) }

#[tauri::command]
fn import_config(app: AppHandle, text: String) -> Result<(), String> { commit(&app, config::import(&text)?) }

#[tauri::command]
fn apply_bar(app: AppHandle) { bar::apply(&app) }

#[tauri::command]
fn restore_taskbar(app: AppHandle) -> Result<(), String> {
    let mut c = app.state::<AppState>().config.lock().unwrap().clone();
    c.profile_mut().bar.native_taskbar = TaskbarMode::Show;
    commit(&app, c)
}

fn main() {
    // A crash must never leave the Windows taskbar hidden.
    std::panic::set_hook(Box::new(|info| {
        platform::shell().set_taskbar(TaskbarMode::Show);
        eprintln!("GlassBar panic: {info}");
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| { let _ = bar::open_settings(app); }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let path = app.path().config_dir()?.join("GlassBar").join("config.json");
            let cfg = config::load(&path);
            app.manage(AppState {
                config: Mutex::new(cfg),
                path,
                popup: Mutex::new(Default::default()),
                edit: AtomicBool::new(false),
            });
            let h = app.handle();
            bar::build_tray(h)?;
            bar::apply(h);
            if let Some(w) = app.get_webview_window("bar") { w.show()?; }
            widgets::start_watchers(h);
            Ok(())
        })
        .on_window_event(|w, e| match (w.label(), e) {
            ("popup", WindowEvent::Focused(false)) => bar::popup_hidden(w.app_handle()),
            ("bar", WindowEvent::ThemeChanged(_) | WindowEvent::ScaleFactorChanged { .. }) => bar::apply(w.app_handle()),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, export_config, import_config, apply_bar, restore_taskbar,
            bar::popup_open, bar::popup_hide, bar::open_settings_cmd, bar::set_edit_mode,
            widgets::apps::apps_list, widgets::apps::app_icon, widgets::apps::app_activate,
            widgets::apps::app_close, widgets::apps::app_launch,
            widgets::sound::audio_state, widgets::sound::audio_set_volume, widgets::sound::audio_set_mute,
            widgets::sound::audio_set_default, widgets::sound::audio_sessions,
            widgets::sound::audio_set_session_volume, widgets::sound::audio_set_session_mute,
            widgets::media::media_info, widgets::media::media_control, widgets::media::media_seek,
            widgets::quick::radios_list, widgets::quick::radio_set, widgets::quick::airplane_set,
            widgets::quick::display_mode, widgets::quick::display_set_mode,
            widgets::quick::brightness_get, widgets::quick::brightness_set,
        ])
        .build(tauri::generate_context!())
        .expect("error while building GlassBar")
        .run(|app, e| if let RunEvent::Exit = e { bar::cleanup(app) });
}
