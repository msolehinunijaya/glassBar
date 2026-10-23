#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod scheduler;
mod alerts;
mod bar;
mod config;
mod db;
mod platform;
mod widgets;

use config::{Config, TaskbarMode};
use platform::SystemEvent;
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
    /// The user is dragging the bar window (ignore DPI re-layout until they let go).
    pub dragging: AtomicBool,
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("GlassBar")
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
    // A crash must never leave the Windows taskbar hidden or space reserved.
    // Release builds have no console, so the message goes to panic.log.
    std::panic::set_hook(Box::new(|info| {
        platform::shell().appbar_remove();
        platform::shell().set_taskbar(TaskbarMode::Show);
        let msg = format!("{info}
{}
", std::backtrace::Backtrace::force_capture());
        let _ = std::fs::write(data_dir().join("panic.log"), &msg);
        eprintln!("GlassBar panic: {msg}");
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| { let _ = bar::open_settings(app); }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let path = data_dir().join("config.json");
            let database = db::open(&data_dir().join("glassbar.db")).map_err(|e| format!("database: {e}"))?;
            app.manage(database);
            let cfg = config::load(&path);
            app.manage(AppState {
                config: Mutex::new(cfg),
                path,
                popup: Mutex::new(Default::default()),
                edit: AtomicBool::new(false),
                dragging: AtomicBool::new(false),
            });
            let h = app.handle();
            let a = h.clone();
            platform::shell().init(&data_dir(), Box::new(move |e| match e {
                SystemEvent::DisplayChanged | SystemEvent::ShellRestarted => bar::apply(&a),
                // Step back from always-on-top while a game or video is fullscreen.
                SystemEvent::Fullscreen(on) => {
                    if let Some(w) = a.get_webview_window("bar") { let _ = w.set_always_on_top(!on); }
                }
            }));
            bar::build_tray(h)?;
            bar::apply(h);
            if let Some(w) = app.get_webview_window("bar") { w.show()?; }
            widgets::start_watchers(h);
            scheduler::start(h.clone());
            Ok(())
        })
        .on_window_event(|w, e| match (w.label(), e) {
            ("popup", WindowEvent::Focused(false)) => bar::popup_hidden(w.app_handle()),
            ("bar", WindowEvent::ScaleFactorChanged { .. })
                if w.app_handle().state::<AppState>().dragging.load(std::sync::atomic::Ordering::Relaxed) => {}
            ("bar", WindowEvent::ThemeChanged(_) | WindowEvent::ScaleFactorChanged { .. }) => bar::apply(w.app_handle()),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, export_config, import_config, apply_bar, restore_taskbar,
            bar::popup_open, bar::popup_hide, bar::open_settings_cmd, bar::set_edit_mode,
            bar::bar_preview, bar::bar_drag, bar::mouse_down,
            widgets::todo::todo_list, widgets::todo::todo_add, widgets::todo::todo_update,
            widgets::todo::todo_delete, widgets::todo::todo_clear_done,
            widgets::notes::notes_list, widgets::notes::note_create, widgets::notes::note_update,
            widgets::notes::note_delete, widgets::notes::note_restore, widgets::notes::note_add_image,
            widgets::notes::note_remove_image, widgets::notes::note_image, widgets::notes::note_drag_image,
            widgets::focus::pomo_get, widgets::focus::pomo_start, widgets::focus::pomo_pause,
            widgets::focus::pomo_skip, widgets::focus::pomo_reset,
            widgets::focus::timers_list, widgets::focus::timer_add, widgets::focus::timer_pause,
            widgets::focus::timer_resume, widgets::focus::timer_delete,
            widgets::focus::sw_get, widgets::focus::sw_toggle, widgets::focus::sw_lap, widgets::focus::sw_reset,
            alerts::alerts_list, alerts::alerts_mark_read, alerts::alerts_dismiss, alerts::alerts_clear,
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
