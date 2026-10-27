//! Low performance mode: a popup or banner window that has stayed hidden for 30 s gives its
//! memory back (WebView2 suspend + low memory target). It wakes before it is shown again.
//! An older WebView2 without these calls simply isn't freed.
use crate::config::PerfMode;
use std::sync::Mutex;
use std::time::Duration;
use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, ICoreWebView2_3, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use webview2_com::TrySuspendCompletedHandler;
use windows::core::Interface;

/// Hidden this long before it is freed: quick re-opens stay instant.
const AFTER: Duration = Duration::from_secs(30);

/// Per window label: a generation (any newer hide or show cancels an older timer) and
/// whether it is suspended now.
static STATE: Mutex<Vec<(String, u64, bool)>> = Mutex::new(Vec::new());

fn with<R>(label: &str, f: impl FnOnce(&mut u64, &mut bool) -> R) -> R {
    let mut s = STATE.lock().unwrap();
    let i = match s.iter().position(|e| e.0 == label) {
        Some(i) => i,
        None => { s.push((label.to_string(), 0, false)); s.len() - 1 }
    };
    let e = &mut s[i];
    f(&mut e.1, &mut e.2)
}

/// The window was just hidden: free it in 30 s if it is still hidden and the mode is Low.
pub fn later(win: WebviewWindow) {
    let gen = with(win.label(), |g, _| { *g += 1; *g });
    if crate::perf::effective() != PerfMode::Low { return; } // hover panels hide often: no idle timers
    std::thread::spawn(move || {
        std::thread::sleep(AFTER);
        if crate::perf::effective() != PerfMode::Low || win.is_visible().unwrap_or(true) { return; }
        let go = with(win.label(), |g, asleep| { let go = *g == gen && !*asleep; if go { *asleep = true; } go });
        if !go { return; }
        let _ = win.with_webview(|wv| unsafe {
            let c = wv.controller();
            let _ = c.SetIsVisible(false); // suspending needs an invisible WebView
            if let Ok(core) = c.CoreWebView2() {
                if let Ok(c3) = core.cast::<ICoreWebView2_3>() {
                    let _ = c3.TrySuspend(&TrySuspendCompletedHandler::create(Box::new(|_, _| Ok(()))));
                }
                if let Ok(c19) = core.cast::<ICoreWebView2_19>() {
                    let _ = c19.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW);
                }
            }
        });
    });
}

/// About to hand the window something or show it: undo `later` first. Cheap when it wasn't
/// freed. Runs on the main thread ahead of the emit/show that follows, so content arrives.
pub fn wake(win: &WebviewWindow) {
    let was = with(win.label(), |g, asleep| { *g += 1; std::mem::replace(asleep, false) });
    if !was { return; }
    let _ = win.with_webview(|wv| unsafe {
        let c = wv.controller();
        if let Ok(core) = c.CoreWebView2() {
            if let Ok(c19) = core.cast::<ICoreWebView2_19>() {
                let _ = c19.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL);
            }
            if let Ok(c3) = core.cast::<ICoreWebView2_3>() { let _ = c3.Resume(); }
        }
        let _ = c.SetIsVisible(true);
    });
}
