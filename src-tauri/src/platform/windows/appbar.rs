//! AppBar registration, native taskbar hiding, and window glass styling.
use super::super::{Glass, Rect};
use super::wide;
use crate::config::{Edge, TaskbarMode};
use std::sync::{Mutex, OnceLock};
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND,
};
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static REGISTERED: Mutex<Option<isize>> = Mutex::new(None);
static ORIGINAL_STATE: OnceLock<u32> = OnceLock::new();

fn hwnd(h: isize) -> HWND { HWND(h as *mut _) }

fn data(h: HWND) -> APPBARDATA {
    APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: h, ..Default::default() }
}

pub fn set(win: isize, edge: Edge, strip: Rect) -> Rect {
    let mut reg = REGISTERED.lock().unwrap();
    let mut abd = data(hwnd(win));
    unsafe {
        if *reg != Some(win) {
            abd.uCallbackMessage = WM_USER + 77;
            SHAppBarMessage(ABM_NEW, &mut abd);
            *reg = Some(win);
        }
        abd.uEdge = match edge { Edge::Left => ABE_LEFT, Edge::Top => ABE_TOP, Edge::Right => ABE_RIGHT, Edge::Bottom => ABE_BOTTOM };
        abd.rc = RECT { left: strip.x, top: strip.y, right: strip.x + strip.w, bottom: strip.y + strip.h };
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        // QUERYPOS may move the strip's outer side to clear other bars; keep our thickness.
        match edge {
            Edge::Top => abd.rc.bottom = abd.rc.top + strip.h,
            Edge::Bottom => abd.rc.top = abd.rc.bottom - strip.h,
            Edge::Left => abd.rc.right = abd.rc.left + strip.w,
            Edge::Right => abd.rc.left = abd.rc.right - strip.w,
        }
        SHAppBarMessage(ABM_SETPOS, &mut abd);
    }
    let r = abd.rc;
    Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
}

pub fn remove(win: isize) {
    let mut reg = REGISTERED.lock().unwrap();
    if reg.take().is_some() {
        let mut abd = data(hwnd(win));
        unsafe { SHAppBarMessage(ABM_REMOVE, &mut abd) };
    }
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
        let cls = wide("Shell_TrayWnd");
        if let Ok(h) = FindWindowW(PCWSTR(cls.as_ptr()), PCWSTR::null()) { v.push(h); }
        let _ = EnumWindows(Some(collect), LPARAM(&mut v as *mut _ as isize));
    }
    v
}

fn taskbar_state() -> u32 {
    let Some(&tray) = taskbars().first() else { return ABS_ALWAYSONTOP };
    let mut abd = data(tray);
    unsafe { SHAppBarMessage(ABM_GETSTATE, &mut abd) as u32 }
}

fn set_taskbar_state(state: u32) {
    if let Some(&tray) = taskbars().first() {
        let mut abd = data(tray);
        abd.lParam = LPARAM(state as isize);
        unsafe { SHAppBarMessage(ABM_SETSTATE, &mut abd) };
    }
}

/// `Hidden` = auto-hide (frees the work area) + window hidden.
/// ponytail: Explorer restarts re-show the taskbar; re-apply on TaskbarCreated if it bites.
pub fn set_taskbar(mode: TaskbarMode) {
    let original = *ORIGINAL_STATE.get_or_init(taskbar_state);
    let (state, show) = match mode {
        TaskbarMode::Show => (original, SW_SHOW),
        TaskbarMode::Autohide => (ABS_AUTOHIDE, SW_SHOW),
        TaskbarMode::Hidden => (ABS_AUTOHIDE, SW_HIDE),
    };
    set_taskbar_state(state);
    for h in taskbars() { unsafe { let _ = ShowWindow(h, show); } }
}

pub fn style_window(win: &tauri::WebviewWindow, g: &Glass) {
    let a = (g.opacity.clamp(0.0, 1.0) * 255.0) as u8;
    let tint = if g.dark { (28, 28, 32, a) } else { (246, 246, 250, a) };
    if g.native {
        let _ = window_vibrancy::apply_acrylic(win, Some(tint));
    } else {
        let _ = window_vibrancy::clear_acrylic(win);
    }
    let _ = win.set_shadow(g.native);
    if let Ok(h) = win.hwnd() {
        let pref = if g.native { DWMWCP_ROUND } else { DWMWCP_DONOTROUND };
        unsafe {
            let _ = DwmSetWindowAttribute(HWND(h.0), DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const _ as *const _, std::mem::size_of_val(&pref) as u32);
        }
    }
}
