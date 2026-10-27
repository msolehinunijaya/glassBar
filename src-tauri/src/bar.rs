//! Bar window placement (dock/float + AppBar), popup window, settings window, tray.
pub use crate::config::Edge;
pub use crate::platform::Rect;
use crate::config::{Config, TaskbarMode, ThemeMode};
use crate::platform::{shell, Glass};
use crate::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry};

/// Space between the bar and the screen edge, logical px.

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
    let thick = (bar.thickness() * s).round() as i32;
    let gap = (bar.gap.clamp(0.0, 48.0) * s).round() as i32;
    let (mut r, strip) = place(monitor_rect(&mon), bar.edge, length, thick, gap);
    if let Some(p) = bar.floating { r.x = p.x; r.y = p.y; }
    Some((r, strip))
}

/// Place, reserve and style the bar from the current config.
/// Bumped by every slide and every re-placement: a running slide that sees a newer number
/// stops, so two slides (or a slide and a re-placement) can never fight over the position.
static SLIDE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Where the bar was last put while reserving space, so it can be put back if Windows
/// shoves it (it moves windows out of a newly reserved strip, and the dock window is not
/// the reservation's owner).
static RESERVED_AT: std::sync::Mutex<Option<Rect>> = std::sync::Mutex::new(None);
static APPLY_TX: std::sync::OnceLock<std::sync::mpsc::Sender<()>> = std::sync::OnceLock::new();
static APPLY_PENDING: AtomicBool = AtomicBool::new(false);

/// Place, reserve and style the bar. Runs on a background worker: reserving space and
/// changing the taskbar are calls into Explorer, and when Explorer is slow to answer they
/// must not freeze the dock (Windows then reports GlassBar as "not responding").
/// Requests arriving while one is queued are merged.
pub fn apply(app: &AppHandle) {
    SLIDE.fetch_add(1, Ordering::SeqCst); // stop any slide at once
    let Some(tx) = APPLY_TX.get() else { return apply_now(app) };
    if !APPLY_PENDING.swap(true, Ordering::SeqCst) { let _ = tx.send(()); }
}

pub fn start_apply_worker(app: &AppHandle) {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    if APPLY_TX.set(tx).is_err() { return; }
    let app = app.clone();
    std::thread::Builder::new().name("bar-apply".into()).spawn(move || {
        while rx.recv().is_ok() {
            APPLY_PENDING.store(false, Ordering::SeqCst);
            apply_now(&app);
        }
    }).ok();
}

/// Windows changed the work area (any app bar moved): put the dock back in its strip if it
/// was pushed. Never re-registers, so it can't start a loop.
pub fn replace(app: &AppHandle) {
    let cfg = app.state::<AppState>().config.lock().unwrap().profile().bar.clone();
    if cfg.auto_hide || cfg.floating.is_some() || !cfg.reserve_space { return; }
    let Some(r) = *RESERVED_AT.lock().unwrap() else { return };
    let Some(win) = app.get_webview_window("bar") else { return };
    let at = win.outer_position().map(|p| (p.x, p.y)).unwrap_or((r.x, r.y));
    if at != (r.x, r.y) { set_rect(&win, r); }
}

fn apply_now(app: &AppHandle) {
    SLIDE.fetch_add(1, Ordering::SeqCst);
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let Some(win) = app.get_webview_window("bar") else { return };
    let bar = &cfg.profile().bar;
    // Style first: turning the window shadow on/off adds/removes Windows' frame, and the
    // size set below must be measured without it (else the first placement after a
    // start is a frame's worth too big: 16 px wider, 39 px taller).
    // The window's own light/dark mode decides the Windows 11 acrylic colour (the tint is
    // ignored there): pin it to GlassBar's theme, or follow Windows on Auto. Before
    // is_dark, which reads it on Auto.
    let theme = match cfg.theme.mode { ThemeMode::Auto => None, ThemeMode::Light => Some(tauri::Theme::Light), ThemeMode::Dark => Some(tauri::Theme::Dark) };
    let windows: Vec<_> = ["bar", "popup", "banner"].into_iter().filter_map(|l| app.get_webview_window(l)).collect();
    for w in &windows { let _ = w.set_theme(theme); }
    let glass = Glass { native: glass_native(cfg.theme.native_blur), dark: is_dark(app, &cfg),
                        material: cfg.theme.material.clone(), radius: cfg.theme.radius };
    // Blur/tint calls only work on the thread that owns the window (the main thread); from
    // the apply worker, hand them over and wait so the size below is measured after styling.
    let a = app.clone();
    // The banner window draws its own cards with gaps and corners between them: no material
    // there, or Windows fills the whole window behind them (a dark box around the banners).
    let plain = Glass { native: false, dark: glass.dark, material: String::new(), radius: glass.radius };
    let style = move || for w in &windows {
        let got = shell().style_window(w, if w.label() == "banner" { &plain } else { &glass });
        if w.label() == "bar" && *MATERIAL.lock().unwrap() != got {
            *MATERIAL.lock().unwrap() = got;
            let _ = a.emit("appearance:material", got);
        }
    };
    if std::thread::current().name() == Some("bar-apply") {
        let (tx, rx) = std::sync::mpsc::channel();
        if app.run_on_main_thread(move || { style(); let _ = tx.send(()); }).is_ok() { let _ = rx.recv_timeout(Duration::from_secs(2)); }
    } else {
        style();
    }
    if let Some((mut r, strip)) = bar_rect(app, bar, bar.length) {
        log_place(bar.edge, r);
        *RESERVED_AT.lock().unwrap() = None;
        if bar.auto_hide && bar.floating.is_none() {
            // Auto-hide never reserves space: windows use the whole screen. Without the
            // AppBar no "fullscreen ended" notice can arrive, so make sure it stays on top.
            shell().appbar_remove();
            let _ = win.set_always_on_top(true);
            if !app.state::<AppState>().revealed.load(Ordering::Relaxed) {
                if let Some(mon) = bar_monitor(app, bar.monitor) { r = hidden_rect(r, bar.edge, monitor_rect(&mon), PEEK); }
            }
            set_rect(&win, r);
        } else if bar.floating.is_none() && bar.reserve_space {
            let got = shell().appbar_set(bar.edge, strip);
            r.x += got.x - strip.x;
            r.y += got.y - strip.y;
            set_rect(&win, r);
            *RESERVED_AT.lock().unwrap() = Some(r);
            // Windows may shove the dock out of the strip a moment later: check once more.
            let a = app.clone();
            std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(350)); replace(&a); });
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
}

/// Log where the dock goes (only when it changes).
fn log_place(edge: Edge, r: Rect) {
    static LAST: std::sync::Mutex<Option<(Edge, Rect)>> = std::sync::Mutex::new(None);
    let mut last = LAST.lock().unwrap();
    if *last != Some((edge, r)) {
        *last = Some((edge, r));
        crate::applog::log(format!("dock: {edge:?} edge at {},{} {}x{}", r.x, r.y, r.w, r.h));
    }
}

/// Place the dock again after each delay (seconds), on a background thread.
pub fn apply_later(app: &AppHandle, after_s: &[u64]) {
    let (a, after) = (app.clone(), after_s.to_vec());
    std::thread::spawn(move || {
        let mut done = 0;
        for s in after { std::thread::sleep(Duration::from_secs(s - done)); done = s; apply(&a); }
    });
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
pub fn bar_reveal(app: AppHandle, show: bool) { reveal(&app, show) }

/// Slide an auto-hidden bar in or out (~120 ms). A newer slide or re-placement cancels it.
pub fn reveal(app: &AppHandle, show: bool) {
    let cfg = app.state::<AppState>().config.lock().unwrap().profile().bar.clone();
    if !cfg.auto_hide || cfg.floating.is_some() { return; }
    if app.state::<AppState>().revealed.swap(show, Ordering::Relaxed) == show { return; }
    let (Some(win), Some((r, _)), Some(mon)) = (app.get_webview_window("bar"), bar_rect(app, &cfg, cfg.length), bar_monitor(app, cfg.monitor)) else { return };
    let to = if show { r } else { hidden_rect(r, cfg.edge, monitor_rect(&mon), PEEK) };
    let from = win.outer_position().map(|p| (p.x, p.y)).unwrap_or((to.x, to.y));
    // Sliding in is always on purpose: come out above whatever window covers the edge.
    if show { let _ = win.set_always_on_top(true); shell().raise_top(&win); }
    let gen = SLIDE.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        const STEPS: i32 = 8;
        for i in 1..=STEPS {
            if SLIDE.load(Ordering::SeqCst) != gen { return; } // superseded
            // ease-out
            let t = 1.0 - (1.0 - i as f64 / STEPS as f64).powi(3);
            let x = from.0 + ((to.x - from.0) as f64 * t).round() as i32;
            let y = from.1 + ((to.y - from.1) as f64 * t).round() as i32;
            let _ = win.set_position(PhysicalPosition::new(x, y));
            std::thread::sleep(Duration::from_millis(15));
        }
    });
}

/// Auto-hide driven by the real mouse position (see autohide.rs). Idle unless auto-hide is on.
pub fn watch_autohide(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let mut timers = crate::autohide::Timers::default();
        loop {
            std::thread::sleep(Duration::from_millis(crate::autohide::TICK_MS as u64));
            let cfg = app.state::<AppState>().config.lock().unwrap().profile().bar.clone();
            if !cfg.auto_hide || cfg.floating.is_some() { continue; }
            let (Some((shown, _)), Some(mon), Some((x, y))) = (bar_rect(&app, &cfg, cfg.length), bar_monitor(&app, cfg.monitor), shell().cursor_pos()) else { continue };
            let st = app.state::<AppState>();
            let popup_open = app.get_webview_window("popup").and_then(|p| p.is_visible().ok()).unwrap_or(false);
            let snap_open = app.get_webview_window("snap").and_then(|p| p.is_visible().ok()).unwrap_or(false);
            let held = HOLD_UNTIL.load(Ordering::Relaxed) > crate::db::now_ms() as u64; // a jump is showing a widget
            let busy = st.edit.load(Ordering::Relaxed) || st.dragging.load(Ordering::Relaxed) || held
                || popup_open || snap_open || shell().menu_open() || shell().mouse_down();
            let inputs = crate::autohide::Inputs {
                revealed: st.revealed.load(Ordering::Relaxed),
                over_bar: crate::autohide::contains(shown, x, y, 4),
                at_edge: crate::autohide::contains(crate::autohide::edge_zone(shown, cfg.edge, monitor_rect(&mon)), x, y, 0),
                busy,
            };
            match crate::autohide::step(&inputs, &mut timers) {
                crate::autohide::Act::Show => reveal(&app, true),
                crate::autohide::Act::Hide => reveal(&app, false),
                crate::autohide::Act::Stay => {}
            }
        }
    });
}

/// The frontend marks a window drag, so a DPI change mid-drag doesn't snap the bar back.
#[tauri::command]
pub fn bar_drag(app: AppHandle, on: bool) { app.state::<AppState>().dragging.store(on, Ordering::Relaxed); }

#[tauri::command]
pub fn mouse_down() -> bool { shell().mouse_down() }

/// Native blur only when it's on and Windows' Transparency effects are on; otherwise the
/// page draws the glass tint itself (else the bar is solid black in any theme).
fn glass_native(wanted: bool) -> bool { wanted && shell().transparency_effects() }

#[tauri::command]
pub fn transparency_effects() -> bool { shell().transparency_effects() }

/// The material Windows really draws behind the dock (after fallbacks), "none" if none.
static MATERIAL: std::sync::Mutex<&'static str> = std::sync::Mutex::new("none");

/// What this Windows can draw, for the Appearance page: no promise of an effect that can't show.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Support { build: u32, acrylic: bool, mica: bool, blur: bool, transparency: bool, applied: &'static str }

#[tauri::command]
pub fn appearance_support() -> Support {
    let build = shell().os_build();
    // 0 = unknown build: let Windows decide (the fallback reports what it really drew).
    let at = |b: u32| build == 0 || build >= b;
    Support { build, acrylic: at(17763), mica: at(22000), blur: at(17763),
              transparency: shell().transparency_effects(), applied: *MATERIAL.lock().unwrap() }
}

/// The panel measured its content: make the popup exactly that tall (CSS px), on the same
/// side of the same widget, at most 85 % of the screen (beyond that it scrolls).
#[tauri::command]
pub fn popup_fit(app: AppHandle, height: f64) {
    let Some(p) = app.state::<AppState>().popup.lock().unwrap().place else { return };
    let Some(win) = app.get_webview_window("popup") else { return };
    let h = ((height * p.scale).round() as i32).clamp(60, p.mon.h * 85 / 100);
    set_rect(&win, popup_rect(p.bar, p.anchor, p.width, h, p.edge, p.mon, p.gap));
}

// ---- Popup ----

#[derive(Default)]
pub struct PopupState {
    pub instance: Option<String>, pub hidden_at: Option<Instant>, pub place: Option<Placement>,
    /// Opened by hovering (never focused): closes when the pointer leaves, not on blur.
    pub hover: bool,
}

/// Where the open popup was put, so it can be re-sized to its content without moving sides.
#[derive(Clone, Copy)]
pub struct Placement { bar: Rect, anchor: Rect, width: i32, edge: Edge, mon: Rect, gap: i32, scale: f64 }

/// The popup lost focus: close it, unless it's a hover panel. Those can't take focus, and
/// Windows reports the focus a clicked panel had before as lost just as one opens, which
/// used to close it at once (now and then). Hover panels close when the pointer leaves.
pub fn popup_blurred(app: &AppHandle) {
    if app.state::<AppState>().popup.lock().unwrap().hover { return; }
    popup_hidden(app);
}

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
        p.hover = !focus;
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
    let gap = (8.0 * s) as i32;
    let r = popup_rect(br, anchor, (width * s).round() as i32, (height * s).round() as i32, edge, mon, gap);
    st.popup.lock().unwrap().place = Some(Placement { bar: br, anchor, width: r.w, edge, mon, gap, scale: s });
    let _ = app.emit("popup:show", &instance);
    set_rect(&popup, r);
    // Hover previews must not steal focus from the user's app: make the window
    // non-activating for them, then show it through Tauri so its visible state stays right.
    popup.set_focusable(focus).map_err(e)?;
    // The card after a screenshot carries a big thumbnail: it shows itself (popup_reveal) once
    // that is drawn, or here after 300 ms at the latest. Shown at once, it was an empty box.
    if instance == FEEDBACK {
        let a = app.clone();
        std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(300)); popup_reveal(a); });
        return Ok(());
    }
    popup.show().map_err(e)?;
    if focus { popup.set_focus().map_err(e)?; }
    Ok(())
}

const FEEDBACK: &str = "__feedback";

/// The feedback card has drawn: show it (if it's still the one to show).
#[tauri::command]
pub fn popup_reveal(app: AppHandle) {
    let current = app.state::<AppState>().popup.lock().unwrap().instance.clone();
    if current.as_deref() != Some(FEEDBACK) { return; }
    if let Some(p) = app.get_webview_window("popup") { if !p.is_visible().unwrap_or(true) { let _ = p.show(); } }
}

#[tauri::command]
pub fn popup_hide(app: AppHandle) { popup_hidden(&app) }

/// The pointer left a hover panel or its button: close it, unless the pointer is in fact
/// still over one of them. Stray "left" events (a tooltip or a window appearing under the
/// pointer) used to close a hover panel just as it opened.
#[tauri::command]
pub fn popup_leave_close(app: AppHandle) {
    let st = app.state::<AppState>();
    let anchor = { let p = st.popup.lock().unwrap(); if !p.hover { return; } p.place.as_ref().map(|pl| pl.anchor) };
    if let (Some((x, y)), Some(win)) = (shell().cursor_pos(), app.get_webview_window("popup")) {
        let over_panel = match (win.outer_position(), win.outer_size()) {
            (Ok(p), Ok(s)) => crate::autohide::contains(Rect { x: p.x, y: p.y, w: s.width as i32, h: s.height as i32 }, x, y, 2),
            _ => false,
        };
        let over_button = anchor.is_some_and(|a| crate::autohide::contains(a, x, y, 2));
        if over_panel || over_button { return; }
    }
    popup_hidden(&app);
}

// ---- Notification banners ----

/// Keep an auto-hidden dock out until then (ms), while a jump highlights a widget.
static HOLD_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A notification for the banner window: hand it over and show the window (it sizes
/// itself with `banner_fit`). Never takes focus from the app you're typing in.
pub fn banner_push(app: &AppHandle, note: crate::alerts::Note) {
    let Some(win) = app.get_webview_window("banner") else { return };
    let _ = app.emit_to("banner", "banner:push", &note);
    if !win.is_visible().unwrap_or(false) {
        if let Some(r) = banner_rect(app, 340.0, 96.0) { set_rect(&win, r); }
        let _ = win.set_focusable(false);
        let _ = win.show();
    }
}

/// Where the banner goes: next to the dock (at its far end, on its screen side), or in the
/// screen corner on the dock's side. Works on every edge and monitor, auto-hidden or not.
fn banner_rect(app: &AppHandle, w_css: f64, h_css: f64) -> Option<Rect> {
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let bar = cfg.profile().bar.clone();
    let mon = bar_monitor(app, bar.monitor)?;
    let s = mon.scale_factor();
    let m = monitor_rect(&mon);
    let (w, h, gap) = ((w_css * s).round() as i32, (h_css * s).round() as i32, (10.0 * s).round() as i32);
    // Where the dock is when shown (also while auto-hidden or floating).
    let r = if bar.floating.is_some() {
        let win = app.get_webview_window("bar")?;
        let (p, z) = (win.outer_position().ok()?, win.outer_size().ok()?);
        Rect { x: p.x, y: p.y, w: z.width as i32, h: z.height as i32 }
    } else { bar_rect(app, &bar, bar.length)?.0 };
    let edge = match (bar.floating, bar.edge) {
        (Some(_), Edge::Top | Edge::Bottom) => if r.y + r.h / 2 > m.y + m.h / 2 { Edge::Bottom } else { Edge::Top },
        (_, e) => e,
    };
    let corner = cfg.banners.position == "corner";
    let (x, y) = match edge {
        Edge::Bottom => (if corner { m.x + m.w - w - gap } else { r.x + r.w - w }, r.y - gap - h),
        Edge::Top => (if corner { m.x + m.w - w - gap } else { r.x + r.w - w }, r.y + r.h + gap),
        Edge::Left => (r.x + r.w + gap, if corner { m.y + m.h - h - gap } else { r.y + r.h - h }),
        Edge::Right => (r.x - gap - w, if corner { m.y + m.h - h - gap } else { r.y + r.h - h }),
    };
    Some(Rect { x: x.clamp(m.x, m.x + m.w - w), y: y.clamp(m.y, m.y + m.h - h), w, h })
}

/// The banner measured its stack: fit the window to it (CSS px).
#[tauri::command]
pub fn banner_fit(app: AppHandle, width: f64, height: f64) {
    let (Some(win), Some(r)) = (app.get_webview_window("banner"), banner_rect(&app, width, height)) else { return };
    set_rect(&win, r);
}

/// Nothing left to show.
#[tauri::command]
pub fn banner_hide(app: AppHandle) {
    if let Some(w) = app.get_webview_window("banner") { let _ = w.hide(); }
}

/// Go to the widget that sent a notification (banner click, or the Notification Center
/// list): reveal an auto-hidden dock, then the bar scrolls to it, highlights it and (if set)
/// opens its panel.
#[tauri::command]
pub fn jump_to(app: AppHandle, source: String, data: serde_json::Value) {
    let open = app.state::<AppState>().config.lock().unwrap().banners.on_click == "open";
    HOLD_UNTIL.store(crate::db::now_ms() as u64 + 3_500, Ordering::Relaxed);
    reveal(&app, true);
    let _ = app.emit_to("bar", "bar:jump", serde_json::json!({ "source": source, "data": data, "open": open }));
}

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

pub fn open_settings(app: &AppHandle) -> tauri::Result<()> { open_settings_at(app, None) }

/// Open Settings, optionally on one section ("Location"…).
pub fn open_settings_at(app: &AppHandle, section: Option<String>) -> tauri::Result<()> {
    let section = section.filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric()));
    if let Some(w) = app.get_webview_window("settings") {
        if let Some(s) = &section { let _ = app.emit_to("settings", "settings:section", s); }
        w.unminimize()?;
        w.show()?;
        return w.set_focus();
    }
    let url = section.map_or("index.html".into(), |s| format!("index.html?section={s}"));
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(url.into()))
        .title("GlassBar Settings")
        .inner_size(880.0, 640.0)
        .min_inner_size(640.0, 480.0)
        .center()
        .build()?;
    Ok(())
}

/// Async on purpose: creating a window inside a sync command deadlocks on Windows.
#[tauri::command]
pub async fn open_settings_cmd(app: AppHandle, section: Option<String>) -> Result<(), String> {
    open_settings_at(&app, section).map_err(|e| e.to_string())
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

