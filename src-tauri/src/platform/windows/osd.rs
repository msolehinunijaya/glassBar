//! Windows' own volume / brightness pop-up (the "OSD"), hidden while GlassBar shows its own
//! (the volume and brightness Dock alerts), so the two never show at once.
//!
//! Explorer hosts the pop-up in a small window of class `XamlExplorerHostIslandWindow`.
//! Keeping that window minimized stops the pop-up from drawing (checked on Windows 11 26200:
//! minimized, the volume keys showed nothing; restored, the pop-up was back). The same class
//! is used for a full-screen window too, so only the explorer one of pop-up size is touched.
//! Explorer creates the window the first time it shows a pop-up, so right after GlassBar
//! starts the very first one can still flash; after that, none.
//! Approach as in github.com/louis141414/hide-volume-brightness-osd (a Windhawk mod).
use super::windows_list::exe_path;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;

static HIDE: AtomicBool = AtomicBool::new(false);
/// The pop-up's window once found, and whether GlassBar has minimized it (so it is only ever
/// put back when GlassBar changed it).
static HOST: AtomicIsize = AtomicIsize::new(0);
static OURS: AtomicBool = AtomicBool::new(false);
static WAKE: OnceLock<Mutex<Sender<()>>> = OnceLock::new();
/// A round of follow-up checks is already on its way (holding a volume key sends many changes).
static NUDGING: AtomicBool = AtomicBool::new(false);

/// Pure: is a window of the right class the pop-up's host? Explorer's, and pop-up sized
/// (physical px) -- never the full-screen window of the same class.
fn is_popup_host(exe: &str, w: i32, h: i32) -> bool {
    exe.to_lowercase().ends_with("\\explorer.exe") && w > 0 && w < 800 && h > 0 && h < 300
}

fn find() -> Option<HWND> {
    let mut after: Option<HWND> = None;
    loop {
        let h = unsafe { FindWindowExW(None, after, w!("XamlExplorerHostIslandWindow"), None) }.ok()?;
        let mut r = RECT::default();
        let mut pid = 0u32;
        unsafe {
            let _ = GetWindowRect(h, &mut r);
            GetWindowThreadProcessId(h, Some(&mut pid));
        }
        if is_popup_host(&exe_path(pid), r.right - r.left, r.bottom - r.top) { return Some(h); }
        after = Some(h);
    }
}

fn host() -> Option<HWND> {
    let h = HWND(HOST.load(Ordering::Relaxed) as *mut _);
    if !h.is_invalid() && unsafe { IsWindow(Some(h)) }.as_bool() { return Some(h); }
    let found = find()?;
    HOST.store(found.0 as isize, Ordering::Relaxed);
    Some(found)
}

fn keep_hidden() {
    if let Some(h) = host() {
        if !unsafe { IsIconic(h) }.as_bool() {
            unsafe { let _ = ShowWindow(h, SW_MINIMIZE); }
            OURS.store(true, Ordering::Relaxed);
        }
    }
}

/// Put the pop-up back as Windows had it: not minimized, hidden until it next shows.
fn put_back() {
    if !OURS.swap(false, Ordering::Relaxed) { return; }
    if let Some(h) = host() {
        unsafe { let _ = ShowWindow(h, SW_SHOWNOACTIVATE); let _ = ShowWindow(h, SW_HIDE); }
    }
}

fn start() -> &'static Mutex<Sender<()>> {
    WAKE.get_or_init(|| {
        let (tx, rx) = channel::<()>();
        std::thread::spawn(move || loop {
            if HIDE.load(Ordering::Relaxed) { keep_hidden(); } else { put_back(); }
            // Once a second, or at once when nudged (a volume or brightness change).
            let _ = rx.recv_timeout(Duration::from_secs(1));
        });
        Mutex::new(tx)
    })
}

/// Hide (or stop hiding) Windows' own pop-up.
pub fn set(hide: bool) {
    HIDE.store(hide, Ordering::Relaxed);
    let _ = start().lock().unwrap().send(());
}

/// The volume or brightness just changed: Windows is about to show its pop-up; check now,
/// and again a moment later (explorer may create the window on this very change).
pub fn nudge() {
    if !HIDE.load(Ordering::Relaxed) { return; }
    let tx = start().lock().unwrap().clone();
    let _ = tx.send(());
    if NUDGING.swap(true, Ordering::Relaxed) { return; }
    std::thread::spawn(move || {
        for ms in [80, 250, 600] { std::thread::sleep(Duration::from_millis(ms)); let _ = tx.send(()); }
        NUDGING.store(false, Ordering::Relaxed);
    });
}

/// GlassBar is closing: put the pop-up back now (not on the next tick).
pub fn restore_now() {
    HIDE.store(false, Ordering::Relaxed);
    put_back();
}

#[cfg(test)]
mod tests {
    use super::is_popup_host;

    #[test]
    fn only_explorers_popup_sized_window_is_the_host() {
        assert!(is_popup_host(r"C:\Windows\explorer.exe", 292, 77), "the volume pop-up at 125 %");
        assert!(!is_popup_host(r"C:\Windows\explorer.exe", 1920, 1080), "the full-screen window of the same class");
        assert!(!is_popup_host(r"C:\Program Files\Other\app.exe", 292, 77), "another app's window");
        assert!(!is_popup_host(r"C:\Windows\explorer.exe", 0, 0));
    }
}
