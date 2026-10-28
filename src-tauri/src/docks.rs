//! "All displays": a copy of the dock on every other monitor. The window labelled `bar`
//! stays the main dock (sounds, alerts and the Windows-taskbar handling stay with it); the
//! copies are `bar-<monitor>` windows that show the same widgets.

use crate::bar::{hidden_rect, monitor_rect, rect_on, set_rect, PEEK};
use crate::config::BarCfg;
use crate::platform::{shell, Rect};
use crate::AppState;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Window label for a monitor's extra dock: its name without the `\.\` prefix and other
/// punctuation, so labels survive monitors being renumbered.
fn label_for(name: &str) -> String {
    format!("bar-{}", name.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>())
}

/// Which dock goes on which monitor: the main dock (`bar`) on the chosen monitor (or the
/// primary when none is chosen or it's gone), plus with `all` a copy on every other one.
pub fn dock_monitors(all: bool, chosen: Option<usize>, mons: &[(String, Rect)], primary: usize) -> Vec<(String, usize)> {
    if mons.is_empty() { return vec![]; }
    let main = chosen.filter(|&i| i < mons.len()).unwrap_or(primary.min(mons.len() - 1));
    let mut out = vec![("bar".to_string(), main)];
    if all { out.extend(mons.iter().enumerate().filter(|(i, _)| *i != main).map(|(i, m)| (label_for(&m.0), i))); }
    out
}

/// The dock closest to a window (distance from the window's centre to the dock's rect).
pub fn nearest_dock<'a>(docks: &'a [(String, Rect)], win: Rect) -> Option<&'a str> {
    let (cx, cy) = (win.x + win.w / 2, win.y + win.h / 2);
    let gap = |r: &Rect| {
        let dx = (r.x - cx).max(cx - (r.x + r.w)).max(0) as i64;
        let dy = (r.y - cy).max(cy - (r.y + r.h)).max(0) as i64;
        dx * dx + dy * dy
    };
    docks.iter().min_by_key(|d| gap(&d.1)).map(|d| d.0.as_str())
}


// ---- The copies at run time ----

/// The extra docks wanted now, with their monitors (never the main `bar`).
fn wanted(app: &AppHandle, bar: &BarCfg) -> Vec<(String, tauri::Monitor)> {
    if !bar.all_displays || bar.floating.is_some() { return vec![]; }
    let Ok(mons) = app.available_monitors() else { return vec![] };
    let primary = app.primary_monitor().ok().flatten()
        .and_then(|p| mons.iter().position(|m| m.position() == p.position())).unwrap_or(0);
    let list: Vec<(String, Rect)> = mons.iter().map(|m| (m.name().cloned().unwrap_or_default(), monitor_rect(m))).collect();
    dock_monitors(true, bar.monitor, &list, primary).into_iter().skip(1).map(|(l, i)| (l, mons[i].clone())).collect()
}

fn extras(app: &AppHandle) -> Vec<WebviewWindow> {
    app.webview_windows().into_values().filter(|w| w.label().starts_with("bar-")).collect()
}

fn cfg(app: &AppHandle) -> BarCfg { app.state::<AppState>().config.lock().unwrap().profile().bar.clone() }

/// Create the missing copies and close the unwanted ones (runs on the apply worker: building
/// a window on the main thread from a command deadlocks).
pub fn sync(app: &AppHandle) {
    let want = wanted(app, &cfg(app));
    for w in extras(app) {
        if !want.iter().any(|(l, _)| l == w.label()) {
            if let Ok(h) = w.hwnd() { shell().appbar_remove_window(h.0 as isize); }
            crate::effects::forget(w.label());
            if let Some(m) = REVEALED.lock().unwrap().as_mut() { m.remove(w.label()); }
            let _ = w.destroy();
        }
    }
    for (label, _) in &want {
        if app.get_webview_window(label).is_some() { continue; }
        let built = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
            .title("GlassBar").decorations(false).transparent(true).shadow(true)
            .always_on_top(true).skip_taskbar(true).resizable(false).focused(false).visible(false)
            .build();
        if let Err(e) = built { crate::applog::log(format!("dock on {label}: {e}")); }
    }
}

/// Place every copy on its monitor: reserve its strip, or tuck it away when auto-hidden.
pub fn place_all(app: &AppHandle) {
    let bar = cfg(app);
    for (label, mon) in wanted(app, &bar) {
        let Some(w) = app.get_webview_window(&label) else { continue };
        let (mut r, strip) = rect_on(&bar, &mon, bar.length, &label);
        let h = w.hwnd().map(|h| h.0 as isize).unwrap_or(0);
        if bar.auto_hide {
            shell().appbar_remove_window(h);
            if !revealed(&label) { r = hidden_rect(r, bar.edge, monitor_rect(&mon), PEEK); }
        } else if bar.reserve_space {
            let got = shell().appbar_set_window(h, bar.edge, strip);
            r.x += got.x - strip.x;
            r.y += got.y - strip.y;
        } else {
            shell().appbar_remove_window(h);
        }
        set_rect(&w, r);
        let _ = w.set_always_on_top(true);
        if !w.is_visible().unwrap_or(true) { let _ = w.show(); }
        let _ = w.emit_to(label.as_str(), "bar:away", bar.auto_hide && !revealed(&label));
    }
}

/// Re-place one copy only (magnification lengthened or shortened it).
pub fn place_one(app: &AppHandle, label: &str) {
    let bar = cfg(app);
    let Some((_, mon)) = wanted(app, &bar).into_iter().find(|(l, _)| l == label) else { return };
    let Some(w) = app.get_webview_window(label) else { return };
    if bar.auto_hide && !revealed(label) { return; }
    let (mut r, _) = rect_on(&bar, &mon, bar.length, label);
    // Keep where Windows put it across the edge (it may have nudged it when reserving).
    if let Ok(p) = w.outer_position() {
        match bar.edge { crate::config::Edge::Top | crate::config::Edge::Bottom => r.y = p.y, _ => r.x = p.x }
    }
    set_rect(&w, r);
}

// ---- Auto-hide for the copies (the main dock's is in bar.rs) ----

static REVEALED: Mutex<Option<HashMap<String, (crate::autohide::Timers, bool)>>> = Mutex::new(None);
static SLIDE: AtomicU64 = AtomicU64::new(0);

fn revealed(label: &str) -> bool { REVEALED.lock().unwrap().as_ref().and_then(|m| m.get(label)).is_some_and(|v| v.1) }

/// One tick of the auto-hide watcher for every copy.
pub fn autohide_tick(app: &AppHandle, bar: &BarCfg, (x, y): (i32, i32), busy: bool) {
    for (label, mon) in wanted(app, bar) {
        let (shown, _) = rect_on(bar, &mon, bar.length, &label);
        let act = {
            let mut map = REVEALED.lock().unwrap();
            let e = map.get_or_insert_with(HashMap::new).entry(label.clone()).or_default();
            let inputs = crate::autohide::Inputs {
                revealed: e.1,
                over_bar: crate::autohide::contains(shown, x, y, 4),
                at_edge: crate::autohide::contains(crate::autohide::edge_zone(shown, bar.edge, monitor_rect(&mon)), x, y, 0),
                busy,
            };
            crate::autohide::step(&inputs, &mut e.0)
        };
        match act {
            crate::autohide::Act::Show => slide(app, &label, shown, true),
            crate::autohide::Act::Hide => slide(app, &label, hidden_rect(shown, bar.edge, monitor_rect(&mon), PEEK), false),
            crate::autohide::Act::Stay => {}
        }
    }
}

fn slide(app: &AppHandle, label: &str, to: Rect, show: bool) {
    if let Some(e) = REVEALED.lock().unwrap().get_or_insert_with(HashMap::new).get_mut(label) { e.1 = show; }
    let Some(w) = app.get_webview_window(label) else { return };
    let _ = w.emit_to(label, "bar:away", !show);
    if show { shell().raise_top(&w); }
    let from = w.outer_position().map(|p| (p.x, p.y)).unwrap_or((to.x, to.y));
    // ponytail: one slide counter for all copies; a second dock sliding at the same moment snaps instead.
    let gen = SLIDE.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        const STEPS: i32 = 8;
        for i in 1..=STEPS {
            if SLIDE.load(Ordering::SeqCst) != gen { let _ = w.set_position(PhysicalPosition::new(to.x, to.y)); return; }
            let t = 1.0 - (1.0 - i as f64 / STEPS as f64).powi(3);
            let _ = w.set_position(PhysicalPosition::new(from.0 + ((to.x - from.0) as f64 * t).round() as i32, from.1 + ((to.y - from.1) as f64 * t).round() as i32));
            std::thread::sleep(Duration::from_millis(15));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Rect;

    fn mons(n: usize) -> Vec<(String, Rect)> {
        (0..n).map(|i| (format!(r"\\.\DISPLAY{}", i + 1), Rect { x: i as i32 * 1920, y: 0, w: 1920, h: 1080 })).collect()
    }

    #[test]
    fn one_monitor_is_just_the_main_dock() {
        assert_eq!(dock_monitors(true, None, &mons(1), 0), vec![("bar".to_string(), 0)]);
    }

    #[test]
    fn main_display_only_puts_the_dock_on_the_primary() {
        assert_eq!(dock_monitors(false, None, &mons(2), 1), vec![("bar".to_string(), 1)]);
    }

    #[test]
    fn all_displays_adds_a_copy_on_every_other_monitor() {
        assert_eq!(dock_monitors(true, Some(0), &mons(2), 1), vec![("bar".to_string(), 0), ("bar-DISPLAY2".to_string(), 1)]);
    }

    #[test]
    fn a_chosen_monitor_that_is_gone_falls_back_to_the_primary() {
        assert_eq!(dock_monitors(false, Some(5), &mons(2), 1), vec![("bar".to_string(), 1)]);
    }

    #[test]
    fn an_unplugged_monitor_loses_its_copy() {
        let three = dock_monitors(true, None, &mons(3), 0);
        let two = dock_monitors(true, None, &mons(2), 0);
        assert_eq!(three.len(), 3);
        assert!(!two.iter().any(|d| d.0 == "bar-DISPLAY3"));
    }

    #[test]
    fn windows_fly_to_the_nearest_dock() {
        let docks = vec![("bar".to_string(), Rect { x: 100, y: 1000, w: 1700, h: 60 }), ("bar-DISPLAY2".to_string(), Rect { x: 2020, y: 1000, w: 1700, h: 60 })];
        assert_eq!(nearest_dock(&docks, Rect { x: 2200, y: 100, w: 800, h: 600 }), Some("bar-DISPLAY2"));
        assert_eq!(nearest_dock(&docks, Rect { x: 300, y: 100, w: 800, h: 600 }), Some("bar"));
        assert_eq!(nearest_dock(&[], Rect::default()), None);
    }
}
