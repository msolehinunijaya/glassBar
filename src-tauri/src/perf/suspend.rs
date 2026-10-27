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

/// A hide: start a new generation (cancels older timers). Returns it.
fn arm(label: &str) -> u64 { with(label, |g, _| { *g += 1; *g }) }

/// On the main thread, right before suspending: still that hide, and not woken since?
/// Marks it suspended if so. Running here, in order with `wake`, closes the gap where a
/// show slipped in between the timer and the queued suspend (a blank popup).
fn claim(label: &str, gen: u64) -> bool {
    with(label, |g, asleep| { let ok = *g == gen && !*asleep; if ok { *asleep = true; } ok })
}

/// A show: cancel pending timers. True if the window was suspended and needs resuming.
fn disarm(label: &str) -> bool { with(label, |g, asleep| { *g += 1; std::mem::replace(asleep, false) }) }

/// The window was just hidden: free it in 30 s if it is still hidden and the mode is Low.
pub fn later(win: WebviewWindow) {
    let gen = arm(win.label());
    if crate::perf::effective() != PerfMode::Low { return; } // hover panels hide often: no idle timers
    std::thread::spawn(move || {
        std::thread::sleep(AFTER);
        if crate::perf::effective() != PerfMode::Low { return; }
        let w = win.clone();
        let _ = win.with_webview(move |wv| unsafe {
            if w.is_visible().unwrap_or(true) || !claim(w.label(), gen) { return; }
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
    if !disarm(win.label()) { return; }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wake_before_the_queued_suspend_cancels_it() {
        // The timer queued a suspend; popup_open's wake ran first on the main thread.
        let g = arm("t1");
        assert!(!disarm("t1")); // it wasn't suspended yet
        assert!(!claim("t1", g)); // so the queued suspend must not run now
    }

    #[test]
    fn suspend_then_wake() {
        let g = arm("t2");
        assert!(claim("t2", g));
        assert!(disarm("t2")); // was suspended: resume it
        assert!(!disarm("t2"));
    }

    #[test]
    fn a_newer_hide_cancels_the_older_timer() {
        let g1 = arm("t3");
        let g2 = arm("t3");
        assert!(!claim("t3", g1));
        assert!(claim("t3", g2));
    }
}
