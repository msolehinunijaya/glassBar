#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod scheduler;
mod updates;
mod alerts;
mod autohide;
mod backdrop;
mod bar;
mod config;
mod db;
mod net;
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
    /// Auto-hide: is the bar currently slid in?
    pub revealed: AtomicBool,
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("GlassBar")
}

/// Single write path for config: validate, persist, apply, broadcast.
pub fn commit(app: &AppHandle, cfg: Config) -> Result<(), String> {
    let cfg = cfg.normalized();
    let st = app.state::<AppState>();
    let moved = st.config.lock().unwrap().location != cfg.location;
    config::save(&st.path, &cfg).map_err(|e| e.to_string())?;
    *st.config.lock().unwrap() = cfg.clone();
    bar::apply(app);
    bar::refresh_tray(app);
    widgets::ocr::register_hotkey(app);
    widgets::quick::sync_tray(app);
    sync_autostart(app, &cfg);
    let _ = app.emit("config:changed", &cfg);
    if moved { widgets::location::poke(); }
    Ok(())
}

/// Make Windows' "run at sign-in" entry match the setting (also on start, so a fresh
/// install that never saved its config still starts with Windows).
fn sync_autostart(app: &AppHandle, cfg: &Config) {
    platform::shell().set_starts_with_windows(cfg.startup.launch_at_login);
    // Development runs never touch it: they'd point Windows at the slow debug build, which
    // also needs the dev server (a blank, late dock after a restart).
    if cfg!(debug_assertions) { return; }
    let al = app.autolaunch();
    // Re-enable even when already on: the entry then names this program, not an old copy.
    let _ = if cfg.startup.launch_at_login { al.enable() } else if al.is_enabled().unwrap_or(false) { al.disable() } else { Ok(()) };
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

    // Load state before any window exists: the webviews call get_config as soon as
    // they load, and that can happen before `setup` runs.
    let path = data_dir().join("config.json");
    let state = AppState {
        config: Mutex::new(config::load(&path)),
        path,
        popup: Mutex::new(Default::default()),
        edit: AtomicBool::new(false),
        dragging: AtomicBool::new(false),
        revealed: AtomicBool::new(false),
    };
    // Never recreated over an existing file; a failure here is logged by the panic hook.
    let database = db::open(&data_dir().join("glassbar.db")).expect("could not open glassbar.db");

    tauri::Builder::default()
        .manage(state)
        .manage(database)
        .plugin(tauri_plugin_single_instance::init(|app, _, _| { let _ = bar::open_settings(app); }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _sc, ev| if ev.state == tauri_plugin_global_shortcut::ShortcutState::Pressed { widgets::ocr::on_hotkey(app) })
            .build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let h = app.handle();
            let a = h.clone();
            platform::shell().init(&data_dir(), Box::new(move |e| match e {
                SystemEvent::DisplayChanged | SystemEvent::ShellRestarted => bar::apply(&a),
                SystemEvent::PowerChanged => { let _ = a.emit("battery:changed", ()); widgets::sound::power_event(&a); }
                SystemEvent::Resumed => widgets::location::poke(),
                SystemEvent::Device(on) => widgets::sound::device_event(&a, on),
                SystemEvent::Clipboard => widgets::clipboard::on_change(&a),
                SystemEvent::WorkArea => bar::replace(&a),
                // Step back from always-on-top while a game or video is fullscreen.
                // Windows also sends this when the desktop itself is clicked (Explorer): not a
                // real fullscreen app, so the dock must not drop behind other windows then.
                SystemEvent::Fullscreen(on) => {
                    let desktop = on && platform::shell().foreground_exe()
                        .is_some_and(|e| e.to_lowercase().ends_with("\\explorer.exe"));
                    if let Some(w) = a.get_webview_window("bar") { let _ = w.set_always_on_top(!on || desktop); }
                }
            }));
            sync_autostart(h, &h.state::<AppState>().config.lock().unwrap().clone());
            bar::start_apply_worker(h);
            widgets::ocr::register_hotkey(h);
            bar::watch_autohide(h);
            backdrop::watch(h);
            widgets::sound::charger_init();
            widgets::screenshots::prune(h);
            widgets::clipboard::prune(h);
            widgets::quick::sync_tray(h);
            let n = h.clone();
            platform::wireless().watch_network(Box::new(move || { let _ = n.emit("wifi:changed", ()); widgets::location::poke(); }));
            bar::build_tray(h)?;
            bar::apply(h);
            if let Some(w) = app.get_webview_window("bar") { w.show()?; }
            // Windows settles the frameless window's frame on its first show, which changes
            // the size placed above; place it again so a fresh start matches every other time.
            bar::apply(h);
            widgets::start_watchers(h);
            scheduler::start(h.clone());
            widgets::location::start(h.clone());
            widgets::ipu::start(h.clone());
            widgets::sysmon::start(h.clone());
            widgets::screentime::start(h.clone());
            widgets::downloads::start(h.clone());
            widgets::aiusage::start(h.clone());
            updates::start(h.clone());
            Ok(())
        })
        .on_window_event(|w, e| match (w.label(), e) {
            ("popup", WindowEvent::Focused(false)) => bar::popup_hidden(w.app_handle()),
            ("bar", WindowEvent::ScaleFactorChanged { .. })
                if w.app_handle().state::<AppState>().dragging.load(std::sync::atomic::Ordering::Relaxed) => {}
            ("bar", WindowEvent::ThemeChanged(_)) => bar::apply(w.app_handle()),
            // Windows applies its own DPI resize after this event returns, which would undo
            // a size set here; re-apply a moment later instead.
            ("bar", WindowEvent::ScaleFactorChanged { .. }) => {
                let a = w.app_handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(120));
                    bar::apply(&a);
                });
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, export_config, import_config, apply_bar, restore_taskbar,
            bar::popup_open, bar::popup_hide, bar::banner_fit, bar::banner_hide, bar::jump_to, bar::open_settings_cmd, bar::set_edit_mode,
            bar::bar_preview, bar::bar_drag, bar::mouse_down, bar::transparency_effects, bar::appearance_support, backdrop::wallpaper_behind, bar::bar_reveal, bar::popup_leave,
            bar::thumbs_show, bar::thumbs_clear,
            widgets::todo::todo_list, widgets::todo::todo_add, widgets::todo::todo_update,
            widgets::todo::todo_delete, widgets::todo::todo_clear_done,
            widgets::notes::notes_list, widgets::notes::note_create, widgets::notes::note_update,
            widgets::notes::note_delete, widgets::notes::note_restore, widgets::notes::note_add_image,
            widgets::notes::note_remove_image, widgets::notes::note_image, widgets::notes::note_drag_image,
            widgets::focus::pomo_get, widgets::focus::pomo_start, widgets::focus::pomo_pause,
            widgets::focus::pomo_skip, widgets::focus::pomo_reset, widgets::focus::pomo_set_phase,
            widgets::focus::timers_list, widgets::focus::timer_add, widgets::focus::timer_pause,
            widgets::focus::timer_resume, widgets::focus::timer_delete, widgets::focus::timer_snooze,
            widgets::downloads::downloads_state, widgets::downloads::downloads_open, widgets::downloads::downloads_reveal,
            widgets::downloads::downloads_open_folder, widgets::downloads::downloads_open_with, widgets::downloads::downloads_rename,
            widgets::downloads::downloads_recycle, widgets::downloads::downloads_drag, widgets::downloads::file_icon,
            widgets::downloads::recycle_info, widgets::downloads::recycle_empty,
            widgets::focus::sw_get, widgets::focus::sw_toggle, widgets::focus::sw_lap, widgets::focus::sw_reset,
            widgets::location::location_status, widgets::location::location_clear, widgets::location::location_open_map, widgets::location::location_search, widgets::weather::weather_get,
            widgets::solat::solat_days, widgets::solat::solat_zones, widgets::solat::solat_test_sound,
            widgets::ipu::ipu_get,
            widgets::sysmon::sysmon_history, widgets::sysmon::battery_get, widgets::sysmon::disks_list,
            widgets::sysmon::power_get, widgets::sysmon::power_set_mode, widgets::sysmon::power_use_balanced,
            widgets::screentime::screentime_today, widgets::screentime::screentime_week,
            widgets::jumplist::jumplist,
            alerts::alerts_list, alerts::alerts_mark_read, alerts::alerts_dismiss, alerts::alerts_clear,
            widgets::apps::apps_list, widgets::apps::app_icon, widgets::apps::app_activate,
            widgets::apps::start_menu, widgets::apps::windows_search,
            widgets::apps::app_close, widgets::apps::app_launch,
            widgets::sound::audio_state, widgets::sound::audio_set_volume, widgets::sound::audio_set_mute,
            widgets::sound::audio_set_default, widgets::sound::audio_sessions,
            widgets::sound::audio_set_session_volume, widgets::sound::audio_set_session_mute,
            widgets::media::media_info, widgets::media::media_control, widgets::media::media_seek, widgets::media::media_pause_playing, widgets::media::media_resume,
            widgets::quick::radios_list, widgets::quick::radio_set, widgets::quick::airplane_set,
            widgets::quick::display_mode, widgets::quick::display_set_mode,
            widgets::quick::brightness_get, widgets::quick::brightness_set,
            widgets::quick::hotspot_get, widgets::quick::hotspot_set,
            widgets::currency::fx_rates,
            widgets::apps::app_pin_target, widgets::apps::app_browse, widgets::apps::apps_installed, widgets::apps::paths_exist,
            widgets::ai::ai_key_status, widgets::ai::ai_key_set, widgets::ai::ai_key_clear, widgets::ai::ai_usage, widgets::ai::ai_ask,
            widgets::aiusage::aiu_state, widgets::aiusage::aiu_refresh, widgets::aiusage::aiu_connect, widgets::aiusage::aiu_disconnect,
            widgets::ocr::ocr_hotkey_status, widgets::ocr::ocr_snap_start, widgets::ocr::ocr_shot, widgets::ocr::ocr_snap_cancel, widgets::ocr::ocr_snap_show,
            widgets::ocr::ocr_snap_finish, widgets::screenshots::shots_list, widgets::clipboard::clip_list, widgets::clipboard::clip_copy, widgets::clipboard::clip_delete, widgets::screenshots::shots_copy,
            widgets::screenshots::shots_delete, widgets::screenshots::shots_open, widgets::screenshots::shot_fullscreen, widgets::quick::record_screen, widgets::ocr::ocr_history, widgets::ocr::ocr_history_delete, widgets::ocr::ocr_copy,
            widgets::quick::wifi_list, widgets::quick::wifi_connect, widgets::quick::wifi_disconnect,
            widgets::quick::bt_list, widgets::quick::bt_pair, widgets::quick::bt_unpair, widgets::quick::open_settings_uri,
            bar::popup_fit, widgets::quick::tray_icons, widgets::quick::tray_click, widgets::quick::tray_refresh, widgets::sound::audio_meter,
            widgets::quick::clock_system_12h, widgets::quick::lang_list, widgets::quick::lang_set, widgets::quick::windows_notifications, widgets::quick::show_desktop,
        ])
        .build(tauri::generate_context!())
        .expect("error while building GlassBar")
        .run(|app, e| if let RunEvent::Exit = e { bar::cleanup(app) });
}
