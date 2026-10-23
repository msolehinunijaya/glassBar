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

/// Pixels of an auto-hidden bar left on screen, so the mouse can find it.
const PEEK: i32 = 3;

/// An auto-hidden bar: pushed past its edge with only `peek` pixels showing.
pub fn hidden_rect(r: Rect, edge: Edge, mon: Rect, peek: i32) -> Rect {
    match edge {
        Edge::Bottom => Rect { y: mon.y + mon.h - peek, ..r },
        Edge::Top => Rect { y: mon.y - r.h + peek, ..r },
        Edge::Left => Rect { x: mon.x - r.w + peek, ..r },
        Edge::Right => Rect { x: mon.x + mon.w - peek, ..r },
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

/// Move first, then size: moving onto a monitor with another DPI rescales the
/// window, and the size we set afterwards must win.
fn set_rect(w: &WebviewWindow, r: Rect) {
    let _ = w.set_position(PhysicalPosition::new(r.x, r.y));
    let _ = w.set_size(PhysicalSize::new(r.w.max(1) as u32, r.h.max(1) as u32));
}

fn monitor_rect(m: &tauri::Monitor) -> Rect {
    Rect { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 }
}

fn bar_monitor(app: &AppHandle, idx: Option<usize>) -> Option<tauri::Monitor> {
    let mons = app.available_monitors().ok()?;
    idx.and_then(|i| mons.get(i).cloned())
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

/// Window rect for the bar with the given length; None if no monitor is found.
fn bar_rect(app: &AppHandle, bar: &crate::config::BarCfg, length: f64) -> Option<(Rect, Rect)> {
    let mon = bar_monitor(app, bar.monitor)?;
    let s = mon.scale_factor();
    let thick = (bar.size.px() * s).round() as i32;
    let gap = (GAP * s).round() as i32;
    let (mut r, strip) = place(monitor_rect(&mon), bar.edge, length, thick, gap);
    if let Some(p) = bar.floating { r.x = p.x; r.y = p.y; }
    Some((r, strip))
}

/// Place, reserve and style the bar from the current config.
pub fn apply(app: &AppHandle) {
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let Some(win) = app.get_webview_window("bar") else { return };
    let bar = &cfg.profile().bar;
    if let Some((mut r, strip)) = bar_rect(app, bar, bar.length) {
        if bar.auto_hide && bar.floating.is_none() {
            // Auto-hide never reserves space: windows use the whole screen.
            shell().appbar_remove();
            if !app.state::<AppState>().revealed.load(Ordering::Relaxed) {
                if let Some(mon) = bar_monitor(app, bar.monitor) { r = hidden_rect(r, bar.edge, monitor_rect(&mon), PEEK); }
            }
            set_rect(&win, r);
        } else if bar.floating.is_none() && bar.reserve_space {
            let got = shell().appbar_set(bar.edge, strip);
            r.x += got.x - strip.x;
            r.y += got.y - strip.y;
            set_rect(&win, r);
        } else {
            shell().appbar_remove();
            set_rect(&win, r);
        }
    }
    // While GlassBar auto-hides, the Windows taskbar must stay fully hidden: in its own
    // auto-hide mode it would pop up from the same edge whenever GlassBar is revealed.
    let native = if bar.auto_hide && bar.floating.is_none() && bar.native_taskbar == TaskbarMode::Autohide {
        TaskbarMode::Hidden
    } else {
        bar.native_taskbar
    };
    shell().set_taskbar(native);
    let glass = Glass { native: cfg.theme.native_blur, dark: is_dark(app, &cfg), opacity: cfg.theme.opacity };
    for label in ["bar", "popup"] {
        if let Some(w) = app.get_webview_window(label) { shell().style_window(&w, &glass); }
    }
}

/// Undo everything that outlives the process: reserved space and hidden taskbar.
pub fn cleanup(_app: &AppHandle) {
    crate::platform::tray().stop();
    shell().appbar_remove();
    shell().set_taskbar(TaskbarMode::Show);
}

/// Live preview while dragging a resize handle: only the window moves. The AppBar,
/// taskbar and config are updated once, by the save on release.
#[tauri::command]
pub fn bar_preview(app: AppHandle, length: f64) {
    let cfg = app.state::<AppState>().config.lock().unwrap().profile().bar.clone();
    if let (Some(win), Some((r, _))) = (app.get_webview_window("bar"), bar_rect(&app, &cfg, length.clamp(0.2, 1.0))) {
        set_rect(&win, r);
    }
}

/// Slide an auto-hidden bar in (show) or out. Animated over ~120 ms.
#[tauri::command]
pub fn bar_reveal(app: AppHandle, show: bool) {
    let cfg = app.state::<AppState>().config.lock().unwrap().profile().bar.clone();
    if !cfg.auto_hide || cfg.floating.is_some() { return; }
    if app.state::<AppState>().revealed.swap(show, Ordering::Relaxed) == show { return; }
    let (Some(win), Some((r, _)), Some(mon)) = (app.get_webview_window("bar"), bar_rect(&app, &cfg, cfg.length), bar_monitor(&app, cfg.monitor)) else { return };
    let to = if show { r } else { hidden_rect(r, cfg.edge, monitor_rect(&mon), PEEK) };
    let from = win.outer_position().map(|p| (p.x, p.y)).unwrap_or((to.x, to.y));
    std::thread::spawn(move || {
        const STEPS: i32 = 8;
        for i in 1..=STEPS {
            // ease-out
            let t = 1.0 - (1.0 - i as f64 / STEPS as f64).powi(3);
            let x = from.0 + ((to.x - from.0) as f64 * t).round() as i32;
            let y = from.1 + ((to.y - from.1) as f64 * t).round() as i32;
            let _ = win.set_position(PhysicalPosition::new(x, y));
            std::thread::sleep(Duration::from_millis(15));
        }
    });
}

/// The frontend marks a window drag, so a DPI change mid-drag doesn't snap the bar back.
#[tauri::command]
pub fn bar_drag(app: AppHandle, on: bool) { app.state::<AppState>().dragging.store(on, Ordering::Relaxed); }

#[tauri::command]
pub fn mouse_down() -> bool { shell().mouse_down() }

// ---- Popup ----

#[derive(Default)]
pub struct PopupState { pub instance: Option<String>, pub hidden_at: Option<Instant> }

pub fn popup_hidden(app: &AppHandle) {
    shell().thumbs_clear(); // never leave live thumbnails behind
    if let Some(p) = app.get_webview_window("popup") {
        let _ = p.hide();
        shell().hide_window(&p); // belt and braces: never leave an empty preview on screen
    }
    app.state::<AppState>().popup.lock().unwrap().hidden_at = Some(Instant::now());
    let _ = app.emit("popup:hide", ());
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn popup_open(app: AppHandle, instance: String, x: f64, y: f64, w: f64, h: f64, width: f64, height: f64, focus: Option<bool>) -> Result<(), String> {
    let focus = focus.unwrap_or(true);
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
    let _ = app.emit("popup:show", &instance);
    set_rect(&popup, r);
    // Hover previews must not steal focus from the user's app: make the window
    // non-activating for them, then show it through Tauri so its visible state stays right.
    popup.set_focusable(focus).map_err(e)?;
    popup.show().map_err(e)?;
    if focus { popup.set_focus().map_err(e)?; }
    Ok(())
}

#[tauri::command]
pub fn popup_hide(app: AppHandle) { popup_hidden(&app) }

#[derive(serde::Deserialize)]
pub struct ThumbArea { pub id: isize, pub x: f64, pub y: f64, pub w: f64, pub h: f64 }

/// Draw live previews of windows into the popup at the given CSS-pixel areas.
#[tauri::command]
pub fn thumbs_show(app: AppHandle, items: Vec<ThumbArea>) -> Vec<isize> {
    let Some(p) = app.get_webview_window("popup") else { return vec![] };
    let (Ok(h), Ok(s)) = (p.hwnd(), p.scale_factor()) else { return vec![] };
    let px = |v: f64| (v * s).round() as i32;
    let items: Vec<(isize, Rect)> = items.iter()
        .map(|a| (a.id, Rect { x: px(a.x), y: px(a.y), w: px(a.w), h: px(a.h) }))
        .collect();
    shell().thumbs_show(h.0 as isize, &items)
}

#[tauri::command]
pub fn thumbs_clear() { shell().thumbs_clear() }

/// The pointer left the widget that opened a hover popup; the popup closes soon
/// unless the pointer moves into it.
#[tauri::command]
pub fn popup_leave(app: AppHandle) { let _ = app.emit_to("popup", "popup:leave", ()); }

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
        let item = CheckMenuItem::with_id(app, format!("tray:profile:{name}"), name, true, *name == cfg.active_profile, None::<&str>)?;
        profiles.append(&item)?;
    }
    Menu::with_items(app, &[
        &MenuItem::with_id(app, "tray:settings", "Settings…", true, None::<&str>)?,
        &MenuItem::with_id(app, "tray:edit", "Edit widgets", true, None::<&str>)?,
        &profiles,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "tray:restore", "Restore Windows taskbar", true, None::<&str>)?,
        &MenuItem::with_id(app, "tray:update", "Check for updates…", true, None::<&str>)?,
        &MenuItem::with_id(app, "tray:quit", "Quit GlassBar", true, None::<&str>)?,
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

/// Tauri delivers every menu click (including the bar's right-click menus) to this
/// handler, so tray items are namespaced and anything else is ignored. Without this,
/// "Edit widgets" in the bar menu toggled edit mode on and straight back off.
fn on_menu(app: &AppHandle, id: &str) {
    let Some(id) = id.strip_prefix("tray:") else { return };
    let edit_cfg = |f: &dyn Fn(&mut Config)| {
        let mut c = app.state::<AppState>().config.lock().unwrap().clone();
        f(&mut c);
        let _ = crate::commit(app, c);
    };
    match id {
        "settings" => { let _ = open_settings(app); }
        "edit" => set_edit(app, !app.state::<AppState>().edit.load(Ordering::Relaxed)),
        "restore" => edit_cfg(&|c| c.profile_mut().bar.native_taskbar = TaskbarMode::Show),
        "update" => crate::updates::check_now(app.clone()),
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
    fn hidden_bar_leaves_a_sliver_on_its_edge() {
        let (win, _) = place(MON, Edge::Bottom, 0.5, 48, 6);
        assert_eq!(hidden_rect(win, Edge::Bottom, MON, 3), Rect { y: 1080 - 3, ..win });
        let (win, _) = place(MON, Edge::Top, 0.5, 48, 6);
        assert_eq!(hidden_rect(win, Edge::Top, MON, 3), Rect { y: 3 - 48, ..win });
        let (win, _) = place(MON, Edge::Left, 0.5, 48, 6);
        assert_eq!(hidden_rect(win, Edge::Left, MON, 3), Rect { x: 3 - 48, ..win });
        let (win, _) = place(MON, Edge::Right, 0.5, 48, 6);
        assert_eq!(hidden_rect(win, Edge::Right, MON, 3), Rect { x: 1920 - 3, ..win });
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
