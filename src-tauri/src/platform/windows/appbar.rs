//! AppBar registration, native taskbar hiding, and window glass styling.
use super::super::{Glass, Rect};
use crate::config::{Edge, TaskbarMode};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND, DWMWCP_ROUNDSMALL,
};
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Whether our AppBar is registered (it is owned by the hidden system window).
static REGISTERED: Mutex<bool> = Mutex::new(false);
/// Last taskbar mode applied, so unchanged config saves don't touch Explorer.
static LAST_MODE: Mutex<Option<TaskbarMode>> = Mutex::new(None);
/// True while the user wants the Windows taskbar hidden. Explorer re-shows its taskbar on
/// some events (layout changes, edge hover, restarts); the window watcher hides it again.
pub static KEEP_HIDDEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// GlassBar starts with Windows: on restart/sign-out, leave the taskbar on auto-hide so it
/// stays out of sight after the restart until GlassBar is back.
pub static KEEP_ON_RESTART: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Windows is shutting down or signing out: remove our reserved space; put the taskbar back
/// unless GlassBar hides it and will start again with Windows.
pub fn end_session() {
    remove();
    let keep = KEEP_HIDDEN.load(std::sync::atomic::Ordering::Relaxed) && KEEP_ON_RESTART.load(std::sync::atomic::Ordering::Relaxed);
    if !keep { set_taskbar(TaskbarMode::Show); }
}

/// Is `h` one of the Windows taskbars (main or secondary monitor)?
pub fn is_taskbar(h: HWND) -> bool {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(h, &mut buf) } as usize;
    matches!(String::from_utf16_lossy(&buf[..n]).as_str(), "Shell_TrayWnd" | "Shell_SecondaryTrayWnd")
}

/// Hide `h` again if it is a taskbar that the user asked to keep hidden.
pub fn rehide_if_taskbar(h: HWND) {
    if KEEP_HIDDEN.load(std::sync::atomic::Ordering::Relaxed) && is_taskbar(h) {
        unsafe { let _ = ShowWindow(h, SW_HIDE); }
    }
}
/// File holding the taskbar state from before we first changed it. On disk because
/// Windows persists auto-hide itself, so a crash would otherwise lose the real state.
static STATE_FILE: OnceLock<PathBuf> = OnceLock::new();

/// Message Windows sends to our system window about the AppBar.
pub const APPBAR_MSG: u32 = WM_USER + 77;

fn data(h: HWND) -> APPBARDATA {
    APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: h, ..Default::default() }
}

/// AppBar messages must reach Explorer directly: with GlassBar's tray host running, its
/// stand-in "Shell_TrayWnd" would receive them first, and the forwarded copy doesn't
/// reserve any space. The host steps aside for the duration of the call.
fn abmsg(msg: u32, abd: &mut APPBARDATA) -> usize {
    super::tray::bypass(|| unsafe { SHAppBarMessage(msg, abd) })
}

fn owner() -> Option<HWND> {
    let h = super::system::hwnd();
    (h != 0).then(|| HWND(h as *mut _))
}

pub fn set(edge: Edge, strip: Rect) -> Rect {
    let Some(h) = owner() else { return strip };
    let mut reg = REGISTERED.lock().unwrap();
    let mut abd = data(h);
    if !*reg {
        abd.uCallbackMessage = APPBAR_MSG;
        *reg = abmsg(ABM_NEW, &mut abd) != 0;
    }
    position(abd, edge, strip)
}

/// Extra docks (one per other monitor) reserve their strip with their own window, which
/// needs no notifications: fullscreen and Explorer restarts are handled for the main dock.
static EXTRAS: Mutex<Vec<isize>> = Mutex::new(Vec::new());

pub fn set_for(win: isize, edge: Edge, strip: Rect) -> Rect {
    if win == 0 { return strip; }
    let abd = data(HWND(win as *mut _));
    let mut extras = EXTRAS.lock().unwrap();
    if !extras.contains(&win) && abmsg(ABM_NEW, &mut data(HWND(win as *mut _))) != 0 { extras.push(win); }
    position(abd, edge, strip)
}

pub fn remove_for(win: isize) {
    let mut extras = EXTRAS.lock().unwrap();
    if let Some(i) = extras.iter().position(|&w| w == win) {
        extras.remove(i);
        abmsg(ABM_REMOVE, &mut data(HWND(win as *mut _)));
    }
}

pub fn remove_extras() {
    for w in std::mem::take(&mut *EXTRAS.lock().unwrap()) { abmsg(ABM_REMOVE, &mut data(HWND(w as *mut _))); }
}

/// Ask Explorer for `strip` on `edge` and take it; returns what was granted.
fn position(mut abd: APPBARDATA, edge: Edge, strip: Rect) -> Rect {
    unsafe {
        abd.uEdge = match edge { Edge::Left => ABE_LEFT, Edge::Top => ABE_TOP, Edge::Right => ABE_RIGHT, Edge::Bottom => ABE_BOTTOM };
        abd.rc = RECT { left: strip.x, top: strip.y, right: strip.x + strip.w, bottom: strip.y + strip.h };
        abmsg(ABM_QUERYPOS, &mut abd);
        // QUERYPOS may move the strip's outer side to clear other bars; keep our thickness.
        match edge {
            Edge::Top => abd.rc.bottom = abd.rc.top + strip.h,
            Edge::Bottom => abd.rc.top = abd.rc.bottom - strip.h,
            Edge::Left => abd.rc.right = abd.rc.left + strip.w,
            Edge::Right => abd.rc.left = abd.rc.right - strip.w,
        }
        abmsg(ABM_SETPOS, &mut abd);
    }
    let r = abd.rc;
    Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
}

pub fn remove() {
    let mut reg = REGISTERED.lock().unwrap();
    if let (true, Some(h)) = (*reg, owner()) {
        let mut abd = data(h);
        unsafe { abmsg(ABM_REMOVE, &mut abd) };
    }
    *reg = false;
}

/// Explorer restarted: our registration and its taskbar state are gone, re-apply everything.
pub fn reset() {
    *REGISTERED.lock().unwrap() = false;
    *LAST_MODE.lock().unwrap() = None;
}

pub fn set_state_dir(dir: &Path) {
    let _ = STATE_FILE.set(dir.join("taskbar-original.txt"));
}

fn saved_state() -> Option<u32> {
    std::fs::read_to_string(STATE_FILE.get()?).ok()?.trim().parse().ok()
}

fn taskbars() -> Vec<HWND> {
    unsafe extern "system" fn collect(h: HWND, out: LPARAM) -> BOOL {
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(h, &mut buf) } as usize;
        if String::from_utf16_lossy(&buf[..n]) == "Shell_SecondaryTrayWnd" {
            unsafe { (*(out.0 as *mut Vec<HWND>)).push(h) };
        }
        true.into()
    }
    let mut v = Vec::new();
    unsafe {
        if let Some(h) = super::tray::real_tray() { v.push(h); }
        let _ = EnumWindows(Some(collect), LPARAM(&mut v as *mut _ as isize));
    }
    v
}

fn taskbar_state() -> u32 {
    let Some(&tray) = taskbars().first() else { return ABS_ALWAYSONTOP };
    let mut abd = data(tray);
    unsafe { abmsg(ABM_GETSTATE, &mut abd) as u32 }
}

fn set_taskbar_state(state: u32) {
    if let Some(&tray) = taskbars().first() {
        let mut abd = data(tray);
        abd.lParam = LPARAM(state as isize);
        unsafe { abmsg(ABM_SETSTATE, &mut abd) };
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Plan {
    pub set_state: Option<u32>,
    /// Remember this as the user's original state (only if nothing is saved yet).
    pub save: Option<u32>,
    pub clear_saved: bool,
    /// Some(true) = show taskbar windows, Some(false) = hide, None = leave alone.
    pub show: Option<bool>,
}

/// Pure decision for `set_taskbar`. `saved` = state stored before we first changed it.
pub(crate) fn taskbar_plan(mode: TaskbarMode, saved: Option<u32>, current: u32) -> Plan {
    match mode {
        TaskbarMode::Show => match saved {
            Some(s) => Plan { set_state: Some(s), save: None, clear_saved: true, show: Some(true) },
            None => Plan { set_state: None, save: None, clear_saved: false, show: None },
        },
        TaskbarMode::Autohide | TaskbarMode::Hidden => Plan {
            set_state: Some(ABS_AUTOHIDE),
            save: if saved.is_none() { Some(current) } else { None },
            clear_saved: false,
            // Hidden = auto-hide (frees the work area) + window hidden.
            show: Some(mode == TaskbarMode::Autohide),
        },
    }
}

pub fn set_taskbar(mode: TaskbarMode) {
    KEEP_HIDDEN.store(mode == TaskbarMode::Hidden, std::sync::atomic::Ordering::Relaxed);
    let mut last = LAST_MODE.lock().unwrap();
    if *last == Some(mode) { return; }
    *last = Some(mode);
    let p = taskbar_plan(mode, saved_state(), taskbar_state());
    if let (Some(s), Some(f)) = (p.save, STATE_FILE.get()) {
        if let Some(dir) = f.parent() { let _ = std::fs::create_dir_all(dir); }
        let _ = std::fs::write(f, s.to_string());
    }
    if let Some(s) = p.set_state { set_taskbar_state(s); }
    if let Some(show) = p.show {
        for h in taskbars() { unsafe { let _ = ShowWindow(h, if show { SW_SHOW } else { SW_HIDE }); } }
    }
    if p.clear_saved {
        if let Some(f) = STATE_FILE.get() { let _ = std::fs::remove_file(f); }
    }
}

/// Draw the requested material, falling back to the next one this Windows has
/// (mica → acrylic → blur). Returns what is really drawn.
///
/// Acrylic and blur use the older composition API: Windows 11's system backdrops
/// (DWMSBT_*) turn flat whenever the window isn't active, and the dock never is.
/// Mica only exists as a system backdrop, so it is flat while the dock is inactive.
pub fn style_window(win: &tauri::WebviewWindow, g: &Glass) -> &'static str {
    let Ok(h) = win.hwnd() else { return "none" };
    let hwnd = HWND(h.0);
    let order: &[&'static str] = match (g.native, g.material.as_str()) {
        (false, _) => &[],
        (true, "mica") => &["mica", "acrylic", "blur"],
        (true, "blur") => &["blur", "acrylic"],
        _ => &["acrylic", "blur"],
    };
    // Acrylic's own tint, ABGR. Light: Windows needs some colour or acrylic turns black.
    let tint: u32 = if g.dark { 0x40_20_1C_1C } else { 0x40_FA_F6_F6 };
    let applied = order.iter().copied().find(|m| match *m {
        "mica" => { accent(hwnd, 0, 0, 0); window_vibrancy::apply_mica(win, Some(g.dark)).is_ok() }
        "acrylic" => { let _ = window_vibrancy::clear_mica(win); accent(hwnd, 4, 2, tint) }
        // Blur with a colour (flag 2) paints black on Windows 11: plain blur, the page tints it.
        _ => { let _ = window_vibrancy::clear_mica(win); accent(hwnd, 3, 0, 0) }
    }).unwrap_or("none");
    if applied == "none" { accent(hwnd, 0, 0, 0); let _ = window_vibrancy::clear_mica(win); }
    let _ = win.set_shadow(applied != "none");
    // Transparency effects off: see-through pixels show black, so the page draws a solid
    // bar and Windows rounds its corners (CSS-rounded corners would leave black wedges).
    // With a material Windows rounds the window too: 8 px, 4 px or square, nearest to the style.
    let solid = !crate::platform::shell().transparency_effects();
    let pref = if applied != "none" || solid {
        match g.radius { 0 => DWMWCP_DONOTROUND, 1..=5 => DWMWCP_ROUNDSMALL, _ => DWMWCP_ROUND }
    } else { DWMWCP_DONOTROUND };
    unsafe {
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const _ as *const _, std::mem::size_of_val(&pref) as u32);
    }
    applied
}

/// SetWindowCompositionAttribute(WCA_ACCENT_POLICY): state 0 off, 3 blur, 4 acrylic.
/// Undocumented but stable since Windows 10 1803; returns false where it's missing.
fn accent(hwnd: HWND, state: u32, flags: u32, color: u32) -> bool {
    #[repr(C)] struct Policy { state: u32, flags: u32, color: u32, anim: u32 }
    #[repr(C)] struct Data { attr: u32, data: *mut Policy, size: usize }
    type Swca = unsafe extern "system" fn(HWND, *mut Data) -> BOOL;
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    unsafe {
        let Ok(user32) = GetModuleHandleW(windows::core::w!("user32.dll")) else { return false };
        let Some(f) = GetProcAddress(user32, windows::core::s!("SetWindowCompositionAttribute")) else { return false };
        let f: Swca = std::mem::transmute(f);
        let mut p = Policy { state, flags, color, anim: 0 };
        let mut d = Data { attr: 19, data: &mut p, size: std::mem::size_of::<Policy>() };
        f(hwnd, &mut d).as_bool()
    }
}

/// Windows build number from the registry (e.g. 22631), 0 if unreadable.
pub fn os_build() -> u32 {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let mut buf = [0u16; 16];
    let mut size = std::mem::size_of_val(&buf) as u32;
    let r = unsafe { RegGetValueW(HKEY_LOCAL_MACHINE, windows::core::w!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"),
        windows::core::w!("CurrentBuildNumber"), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr() as *mut _), Some(&mut size)) };
    if r.is_err() { return 0; }
    String::from_utf16_lossy(&buf).trim_end_matches(char::from(0)).parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    const ON_TOP: u32 = ABS_ALWAYSONTOP;
    const AUTO: u32 = ABS_AUTOHIDE;

    #[test]
    fn first_hide_saves_the_users_real_state() {
        let p = taskbar_plan(TaskbarMode::Hidden, None, ON_TOP);
        assert_eq!(p, Plan { set_state: Some(AUTO), save: Some(ON_TOP), clear_saved: false, show: Some(false) });
    }

    #[test]
    fn hiding_again_after_a_crash_keeps_the_saved_state() {
        // Killed while hidden: Windows still says auto-hide, the file still says always-on-top.
        let p = taskbar_plan(TaskbarMode::Autohide, Some(ON_TOP), AUTO);
        assert_eq!(p.save, None);
    }

    #[test]
    fn show_restores_the_saved_state_even_after_a_crash() {
        let p = taskbar_plan(TaskbarMode::Show, Some(ON_TOP), AUTO);
        assert_eq!(p, Plan { set_state: Some(ON_TOP), save: None, clear_saved: true, show: Some(true) });
    }

    #[test]
    fn show_never_touches_a_taskbar_we_did_not_change() {
        let p = taskbar_plan(TaskbarMode::Show, None, AUTO);
        assert_eq!(p, Plan { set_state: None, save: None, clear_saved: false, show: None });
    }
}
