//! Bar window placement (dock/float + AppBar), popup window, settings window, tray.
pub use crate::config::Edge;
pub use crate::platform::Rect;
use crate::config::{Config, TaskbarMode, ThemeMode};
use crate::platform::{shell, Glass};
use crate::AppState;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry};

/// Space between the bar and the screen edge, logical px.
const GAP: f64 = 6.0;

/// Window rect and reserved AppBar strip for a docked bar, all in physical px.
pub fn place(mon: Rect, edge: Edge, length: f64, thick: i32, gap: i32) -> (Rect, Rect) {
    let strip = thick + 2 * gap;
    let along = |total: i32| (total as f64 * length).round() as i32;
    match edge {
        Edge::Top | Edge::Bottom => {
            let w = along(mon.w);
            let x = mon.x + (mon.w - w) / 2;
            let (y, ry) = if edge == Edge::Top { (mon.y + gap, mon.y) }
                          else { (mon.y + mon.h - gap - thick, mon.y + mon.h - strip) };
            (Rect { x, y, w, h: thick }, Rect { x: mon.x, y: ry, w: mon.w, h: strip })
        }
        Edge::Left | Edge::Right => {
            let h = along(mon.h);
            let y = mon.y + (mon.h - h) / 2;
            let (x, rx) = if edge == Edge::Left { (mon.x + gap, mon.x) }
                          else { (mon.x + mon.w - gap - thick, mon.x + mon.w - strip) };
            (Rect { x, y, w: thick, h }, Rect { x: rx, y: mon.y, w: strip, h: mon.h })
        }
    }
}

/// Where a popup goes: beside the bar on the side away from the screen edge,
/// centered on its widget, kept on screen.
pub fn popup_rect(bar: Rect, anchor: Rect, w: i32, h: i32, edge: Edge, mon: Rect, gap: i32) -> Rect {
    let cx = anchor.x + anchor.w / 2 - w / 2;
    let cy = anchor.y + anchor.h / 2 - h / 2;
    let (x, y) = match edge {
        Edge::Bottom => (cx, bar.y - gap - h),
        Edge::Top => (cx, bar.y + bar.h + gap),
        Edge::Left => (bar.x + bar.w + gap, cy),
        Edge::Right => (bar.x - gap - w, cy),
    };
    let x = x.clamp(mon.x + gap, (mon.x + mon.w - w - gap).max(mon.x + gap));
    let y = y.clamp(mon.y + gap, (mon.y + mon.h - h - gap).max(mon.y + gap));
    Rect { x, y, w, h }
}

fn hwnd(w: &WebviewWindow) -> isize { w.hwnd().map(|h| h.0 as isize).unwrap_or(0) }

fn set_rect(w: &WebviewWindow, r: Rect) {
    let _ = w.set_size(PhysicalSize::new(r.w.max(1) as u32, r.h.max(1) as u32));
    let _ = w.set_position(PhysicalPosition::new(r.x, r.y));
}

fn monitor_rect(m: &tauri::Monitor) -> Rect {
    Rect { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 }
}

fn bar_monitor(app: &AppHandle, idx: usize) -> Option<tauri::Monitor> {
    let mons = app.available_monitors().ok()?;
    mons.get(idx).cloned()
        .or_else(|| app.primary_monitor().ok().flatten())
        .or_else(|| mons.into_iter().next())
}

fn is_dark(app: &AppHandle, cfg: &Config) -> bool {
    match cfg.theme.mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::Auto => app.get_webview_window("bar")
            .and_then(|w| w.theme().ok())
            .map(|t| t == tauri::Theme::Dark)
            .unwrap_or(true),
    }
}

/// Place, reserve and style the bar from the current config.
pub fn apply(app: &AppHandle) {
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let Some(win) = app.get_webview_window("bar") else { return };
    let bar = &cfg.profile().bar;
    if let Some(mon) = bar_monitor(app, bar.monitor) {
        let s = mon.scale_factor();
        let thick = (bar.size.px() * s).round() as i32;
        let gap = (GAP * s).round() as i32;
        let (mut r, strip) = place(monitor_rect(&mon), bar.edge, bar.length, thick, gap);
        let id = hwnd(&win);
        match bar.floating {
            Some(p) => {
                shell().appbar_remove(id);
                r.x = p.x;
                r.y = p.y;
            }
            None if bar.reserve_space => {
                let got = shell().appbar_set(id, bar.edge, strip);
                r.x += got.x - strip.x;
                r.y += got.y - strip.y;
            }
            None => shell().appbar_remove(id),
        }
        set_rect(&win, r);
    }
    shell().set_taskbar(bar.native_taskbar);
    let glass = Glass { native: cfg.theme.native_blur, dark: is_dark(app, &cfg), opacity: cfg.theme.opacity };
    for label in ["bar", "popup"] {
        if let Some(w) = app.get_webview_window(label) { shell().style_window(&w, &glass); }
    }
}

/// Undo everything that outlives the process: reserved space and hidden taskbar.
pub fn cleanup(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("bar") { shell().appbar_remove(hwnd(&w)); }
    shell().set_taskbar(TaskbarMode::Show);
}

// ---- Popup ----

#[derive(Default)]
pub struct PopupState { pub instance: Option<String>, pub hidden_at: Option<Instant> }

pub fn popup_hidden(app: &AppHandle) {
    if let Some(p) = app.get_webview_window("popup") { let _ = p.hide(); }
    app.state::<AppState>().popup.lock().unwrap().hidden_at = Some(Instant::now());
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn popup_open(app: AppHandle, instance: String, x: f64, y: f64, w: f64, h: f64, width: f64, height: f64) -> Result<(), String> {
    let st = app.state::<AppState>();
    {
        // Clicking the open widget again: the popup already hid on blur, so this click means "close".
        let mut p = st.popup.lock().unwrap();
        let just_hid = p.hidden_at.is_some_and(|t| t.elapsed() < Duration::from_millis(300));
        if just_hid && p.instance.as_deref() == Some(instance.as_str()) {
            p.hidden_at = None;
            return Ok(());
        }
        p.instance = Some(instance.clone());
    }
    let e = |e: tauri::Error| e.to_string();
    let (Some(bar), Some(popup)) = (app.get_webview_window("bar"), app.get_webview_window("popup")) else {
        return Err("windows not ready".into());
    };
    let s = bar.scale_factor().map_err(e)?;
    let pos = bar.outer_position().map_err(e)?;
    let size = bar.outer_size().map_err(e)?;
    let br = Rect { x: pos.x, y: pos.y, w: size.width as i32, h: size.height as i32 };
    let anchor = Rect { x: pos.x + (x * s) as i32, y: pos.y + (y * s) as i32, w: (w * s) as i32, h: (h * s) as i32 };
    let mon = bar.current_monitor().ok().flatten().map(|m| monitor_rect(&m)).unwrap_or(br);
    let cfg = st.config.lock().unwrap().profile().bar.clone();
    let edge = match (cfg.floating, cfg.edge) {
        (None, e) | (Some(_), e @ (Edge::Left | Edge::Right)) => e,
        (Some(_), _) if br.y + br.h / 2 > mon.y + mon.h / 2 => Edge::Bottom,
        _ => Edge::Top,
    };
    let r = popup_rect(br, anchor, (width * s).round() as i32, (height * s).round() as i32, edge, mon, (8.0 * s) as i32);
    let _ = app.emit_to("popup", "popup:show", &instance);
    set_rect(&popup, r);
    popup.show().map_err(e)?;
    popup.set_focus().map_err(e)?;
    Ok(())
}

#[tauri::command]
pub fn popup_hide(app: AppHandle) { popup_hidden(&app) }

// ---- Settings window, edit mode, tray ----

pub fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("settings") {
        w.unminimize()?;
        w.show()?;
        return w.set_focus();
    }
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("GlassBar Settings")
        .inner_size(880.0, 640.0)
        .min_inner_size(640.0, 480.0)
        .center()
        .build()?;
    Ok(())
}

/// Async on purpose: creating a window inside a sync command deadlocks on Windows.
#[tauri::command]
pub async fn open_settings_cmd(app: AppHandle) -> Result<(), String> {
    open_settings(&app).map_err(|e| e.to_string())
}

pub fn set_edit(app: &AppHandle, on: bool) {
    app.state::<AppState>().edit.store(on, Ordering::Relaxed);
    let _ = app.emit("editmode", on);
}

#[tauri::command]
pub fn set_edit_mode(app: AppHandle, on: bool) { set_edit(&app, on) }

fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let profiles = Submenu::with_id(app, "profiles", "Profiles", true)?;
    for name in cfg.profiles.keys() {
        let item = CheckMenuItem::with_id(app, format!("profile:{name}"), name, true, *name == cfg.active_profile, None::<&str>)?;
        profiles.append(&item)?;
    }
    Menu::with_items(app, &[
        &MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?,
        &MenuItem::with_id(app, "edit", "Edit widgets", true, None::<&str>)?,
        &profiles,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "restore", "Restore Windows taskbar", true, None::<&str>)?,
        &MenuItem::with_id(app, "quit", "Quit GlassBar", true, None::<&str>)?,
    ])
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut b = TrayIconBuilder::with_id("main")
        .tooltip("GlassBar")
        .menu(&tray_menu(app)?)
        .on_menu_event(|app, e| on_menu(app, e.id().as_ref()));
    if let Some(icon) = app.default_window_icon() { b = b.icon(icon.clone()); }
    b.build(app)?;
    Ok(())
}

pub fn refresh_tray(app: &AppHandle) {
    if let (Some(t), Ok(m)) = (app.tray_by_id("main"), tray_menu(app)) { let _ = t.set_menu(Some(m)); }
}

fn on_menu(app: &AppHandle, id: &str) {
    let edit_cfg = |f: &dyn Fn(&mut Config)| {
        let mut c = app.state::<AppState>().config.lock().unwrap().clone();
        f(&mut c);
        let _ = crate::commit(app, c);
    };
    match id {
        "settings" => { let _ = open_settings(app); }
        "edit" => set_edit(app, !app.state::<AppState>().edit.load(Ordering::Relaxed)),
        "restore" => edit_cfg(&|c| c.profile_mut().bar.native_taskbar = TaskbarMode::Show),
        "quit" => { cleanup(app); app.exit(0); }
        p => if let Some(name) = p.strip_prefix("profile:") {
            let name = name.to_string();
            edit_cfg(&|c| c.active_profile = name.clone());
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MON: Rect = Rect { x: 0, y: 0, w: 1920, h: 1080 };

    #[test]
    fn bottom_bar_is_centered_and_reserves_full_strip() {
        let (win, res) = place(MON, Edge::Bottom, 0.5, 48, 6);
        assert_eq!(win, Rect { x: 480, y: 1080 - 6 - 48, w: 960, h: 48 });
        assert_eq!(res, Rect { x: 0, y: 1080 - 60, w: 1920, h: 60 });
    }

    #[test]
    fn left_bar_is_vertical() {
        let (win, res) = place(MON, Edge::Left, 1.0, 48, 6);
        assert_eq!(win, Rect { x: 6, y: 0, w: 48, h: 1080 });
        assert_eq!(res, Rect { x: 0, y: 0, w: 60, h: 1080 });
    }

    #[test]
    fn second_monitor_offsets_apply() {
        let m2 = Rect { x: 1920, y: -200, w: 2560, h: 1440 };
        let (win, res) = place(m2, Edge::Top, 0.5, 36, 6);
        assert_eq!(win, Rect { x: 1920 + 640, y: -194, w: 1280, h: 36 });
        assert_eq!(res, Rect { x: 1920, y: -200, w: 2560, h: 48 });
        let (win, _) = place(m2, Edge::Right, 0.5, 36, 6);
        assert_eq!(win, Rect { x: 1920 + 2560 - 42, y: -200 + 360, w: 36, h: 720 });
    }

    #[test]
    fn popup_sits_beside_the_bar_and_stays_on_screen() {
        let bar = Rect { x: 480, y: 1026, w: 960, h: 48 };
        let r = popup_rect(bar, Rect { x: 500, y: 1026, w: 40, h: 48 }, 360, 400, Edge::Bottom, MON, 8);
        assert_eq!(r, Rect { x: 340, y: 1026 - 8 - 400, w: 360, h: 400 });
        let r = popup_rect(bar, Rect { x: 1900, y: 1026, w: 20, h: 48 }, 360, 400, Edge::Bottom, MON, 8);
        assert_eq!(r.x, 1920 - 360 - 8);
        let left = Rect { x: 6, y: 0, w: 48, h: 1080 };
        let r = popup_rect(left, Rect { x: 6, y: 0, w: 48, h: 40 }, 300, 300, Edge::Left, MON, 8);
        assert_eq!((r.x, r.y), (62, 8));
    }
}
