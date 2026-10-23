//! Real tray icons on the dock, the way alternative shells (RetroBar/Cairo) do it:
//! a hidden window of class "Shell_TrayWnd" sits above Explorer's in z-order, so apps'
//! Shell_NotifyIcon calls (sent to FindWindow("Shell_TrayWnd")) reach it first. Every
//! message is forwarded to Explorer's real tray unchanged, so Windows' own tray keeps
//! working. Off unless the user turns it on.
use super::super::TrayIcon;
use super::wide;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::core::{GUID, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

static DECOY: AtomicIsize = AtomicIsize::new(0);
static DIRTY: AtomicBool = AtomicBool::new(false);
static STEPPED_ASIDE: AtomicBool = AtomicBool::new(false);
const ASK_TIMER: usize = 2;
static ON_CHANGE: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

struct Icon { hwnd: u32, uid: u32, guid: Option<GUID>, callback: u32, version: u32, tip: String, hicon: isize, hidden: bool, png: Option<String>, rev: u64 }
unsafe impl Send for Icon {}
static ICONS: Mutex<Option<HashMap<String, Icon>>> = Mutex::new(None);

// The 32-bit-handle layout Shell_NotifyIcon marshals into WM_COPYDATA (dwData = 1).
#[repr(C)]
#[derive(Clone, Copy)]
struct Nid32 {
    cb_size: u32, hwnd: u32, uid: u32, flags: u32, callback: u32, hicon: u32,
    tip: [u16; 128], state: u32, state_mask: u32, info: [u16; 256], version: u32,
    info_title: [u16; 64], info_flags: u32, guid: GUID, balloon_icon: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct TrayData { signature: u32, message: u32, nid: Nid32 }

const NIF_MESSAGE: u32 = 1;
const NIF_ICON: u32 = 2;
const NIF_TIP: u32 = 4;
const NIF_STATE: u32 = 8;
const NIF_GUID: u32 = 0x20;
const NIS_HIDDEN: u32 = 1;
const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;
const NIM_SETVERSION: u32 = 4;
const GUID_OFFSET: usize = std::mem::offset_of!(Nid32, guid);

fn utf16z(b: &[u16]) -> String { String::from_utf16_lossy(&b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())]) }

/// Pure: apply one Shell_NotifyIcon message to the icon map. Returns true if anything changed.
fn apply(map: &mut HashMap<String, Icon>, message: u32, nid: &Nid32, copy_icon: &dyn Fn(u32) -> isize, free: &dyn Fn(isize)) -> bool {
    let has_guid = nid.cb_size as usize >= GUID_OFFSET + 16 && nid.flags & NIF_GUID != 0;
    let key = if has_guid { format!("{:?}", nid.guid) } else { format!("{:x}:{}", nid.hwnd, nid.uid) };
    match message {
        NIM_ADD | NIM_MODIFY => {
            if message == NIM_MODIFY && !map.contains_key(&key) { return false; }
            let e = map.entry(key).or_insert(Icon { hwnd: nid.hwnd, uid: nid.uid, guid: has_guid.then_some(nid.guid), callback: 0, version: 0,
                tip: String::new(), hicon: 0, hidden: false, png: None, rev: 0 });
            e.hwnd = nid.hwnd;
            e.uid = nid.uid;
            if nid.flags & NIF_MESSAGE != 0 { e.callback = nid.callback; }
            if nid.flags & NIF_TIP != 0 { e.tip = utf16z(&nid.tip); }
            if nid.flags & NIF_STATE != 0 && nid.state_mask & NIS_HIDDEN != 0 { e.hidden = nid.state & NIS_HIDDEN != 0; }
            if nid.flags & NIF_ICON != 0 {
                if e.hicon != 0 { free(e.hicon); }
                e.hicon = copy_icon(nid.hicon);
                e.png = None;
                e.rev += 1;
            }
            true
        }
        NIM_SETVERSION => map.get_mut(&key).map(|e| e.version = nid.version).is_some(),
        NIM_DELETE => map.remove(&key).map(|e| if e.hicon != 0 { free(e.hicon) }).is_some(),
        _ => false,
    }
}

/// Explorer's real taskbar (never our decoy).
pub fn real_tray() -> Option<HWND> {
    let cls = wide("Shell_TrayWnd");
    let decoy = DECOY.load(Ordering::Relaxed);
    let mut after: Option<HWND> = None;
    loop {
        let h = unsafe { FindWindowExW(None, after, PCWSTR(cls.as_ptr()), PCWSTR::null()) }.ok()?;
        if h.0 as isize != decoy { return Some(h); }
        after = Some(h);
    }
}

fn forward(msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
    let real = real_tray()?;
    let mut res = 0usize;
    let ok = unsafe { SendMessageTimeoutW(real, msg, wp, lp, SMTO_ABORTIFHUNG | SMTO_NORMAL, 1000, Some(&mut res)) };
    (ok.0 != 0).then_some(LRESULT(res as isize))
}

unsafe extern "system" fn proc(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_COPYDATA => {
            let cds = unsafe { &*(lp.0 as *const COPYDATASTRUCT) };
            if cds.dwData == 1 && cds.cbData as usize >= 8 + 24 && !cds.lpData.is_null() {
                // Copy what was sent; shorter (older) structs are zero-padded.
                let mut buf = vec![0u8; std::mem::size_of::<TrayData>()];
                let n = (cds.cbData as usize).min(buf.len());
                unsafe { std::ptr::copy_nonoverlapping(cds.lpData as *const u8, buf.as_mut_ptr(), n) };
                let td: TrayData = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const TrayData) };
                if td.signature == 0x34753423 {
                    let mut g = ICONS.lock().unwrap();
                    let map = g.get_or_insert_with(HashMap::new);
                    let copy = |hi: u32| unsafe { CopyIcon(HICON(hi as usize as *mut _)).map(|c| c.0 as isize).unwrap_or(0) };
                    let free = |hi: isize| unsafe { let _ = DestroyIcon(HICON(hi as *mut _)); };
                    if apply(map, td.message, &td.nid, &copy, &free) { DIRTY.store(true, Ordering::Relaxed); }
                }
                let _ = forward(msg, wp, lp);
                return LRESULT(1); // tell the app it worked (Explorer may refuse a re-add)
            }
            forward(msg, wp, lp).unwrap_or(LRESULT(0)) // AppBar and icon-position queries
        }
        WM_TIMER if wp.0 == ASK_TIMER => {
            unsafe { let _ = KillTimer(Some(h), ASK_TIMER); }
            super::system::ignore_next_taskbar_created();
            broadcast_taskbar_created();
            LRESULT(0)
        }
        WM_TIMER => {
            // Stay first in line: Explorer re-raises its taskbar now and then.
            let cls = wide("Shell_TrayWnd");
            if !STEPPED_ASIDE.load(Ordering::Relaxed) && unsafe { FindWindowW(PCWSTR(cls.as_ptr()), PCWSTR::null()) }.map(|w| w != h).unwrap_or(true) {
                unsafe { let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE); }
            }
            if DIRTY.swap(false, Ordering::Relaxed) {
                if let Some(f) = ON_CHANGE.lock().unwrap().as_ref() { f(); }
            }
            LRESULT(0)
        }
        WM_CLOSE => { unsafe { let _ = DestroyWindow(h); } LRESULT(0) }
        WM_DESTROY => { unsafe { PostQuitMessage(0) }; LRESULT(0) }
        m if m >= WM_USER || m == WM_COMMAND || m == WM_ACTIVATEAPP => {
            forward(msg, wp, lp).unwrap_or_else(|| unsafe { DefWindowProcW(h, msg, wp, lp) })
        }
        _ => unsafe { DefWindowProcW(h, msg, wp, lp) },
    }
}

fn broadcast_taskbar_created() {
    unsafe {
        let m = RegisterWindowMessageW(windows::core::w!("TaskbarCreated"));
        let _ = SendNotifyMessageW(HWND_BROADCAST, m, WPARAM(0), LPARAM(0));
    }
}

pub fn running() -> bool { DECOY.load(Ordering::Relaxed) != 0 }

/// Ask every app to send its tray icon again (they all re-add on TaskbarCreated).
pub fn refresh() {
    let h = DECOY.load(Ordering::Relaxed);
    if h != 0 { unsafe { let _ = SetTimer(Some(HWND(h as *mut _)), ASK_TIMER, 50, None); } }
}

/// Run `f` with the stand-in tray out of the way, so FindWindow("Shell_TrayWnd") inside
/// it (e.g. SHAppBarMessage) reaches Explorer's real taskbar. No-op when not hosting.
pub fn bypass<T>(f: impl FnOnce() -> T) -> T {
    let h = DECOY.load(Ordering::Relaxed);
    if h == 0 { return f(); }
    let w = HWND(h as *mut _);
    STEPPED_ASIDE.store(true, Ordering::Relaxed);
    unsafe {
        let _ = SetWindowPos(w, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
        let _ = SetWindowPos(w, Some(HWND_BOTTOM), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
    }
    let r = f();
    unsafe { let _ = SetWindowPos(w, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE); }
    STEPPED_ASIDE.store(false, Ordering::Relaxed);
    r
}

/// Start hosting (idempotent). Apps re-add their icons after the TaskbarCreated broadcast.
pub fn start(on_change: Box<dyn Fn() + Send>) {
    *ON_CHANGE.lock().unwrap() = Some(on_change);
    if running() { return; }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || unsafe {
        let Ok(module) = GetModuleHandleW(None) else { let _ = tx.send(()); return };
        let class = windows::core::w!("Shell_TrayWnd");
        let wc = WNDCLASSW { lpfnWndProc: Some(proc), hInstance: module.into(), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        let Ok(h) = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW, class, windows::core::w!(""), WS_POPUP, 0, 0, 0, 0, None, None, Some(module.into()), None)
        else { let _ = tx.send(()); return };
        DECOY.store(h.0 as isize, Ordering::Relaxed);
        SetTimer(Some(h), 1, 250, None);
        // Ask apps to re-send their icons once GlassBar has finished placing the dock: while
        // it reserves screen space this window briefly steps aside, and icons sent in that
        // moment would only reach Explorer.
        SetTimer(Some(h), ASK_TIMER, 1500, None);
        let _ = tx.send(());
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() { let _ = TranslateMessage(&msg); DispatchMessageW(&msg); }
        DECOY.store(0, Ordering::Relaxed);
        if let Some(map) = ICONS.lock().unwrap().take() {
            for e in map.values() { if e.hicon != 0 { let _ = DestroyIcon(HICON(e.hicon as *mut _)); } }
        }
    });
    let _ = rx.recv();
}

pub fn stop() {
    let h = DECOY.load(Ordering::Relaxed);
    if h != 0 { unsafe { let _ = PostMessageW(Some(HWND(h as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0)); } }
}

fn exe_of(hwnd: u32) -> String {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(HWND(hwnd as usize as *mut _), Some(&mut pid)) };
    super::windows_list::exe_path(pid)
}

pub fn icons(promoted: &dyn Fn(&str) -> bool) -> Vec<TrayIcon> {
    let mut g = ICONS.lock().unwrap();
    let Some(map) = g.as_mut() else { return vec![] };
    // Drop icons whose window died without removing them (crashed apps).
    map.retain(|_, e| unsafe { IsWindow(Some(HWND(e.hwnd as usize as *mut _))).as_bool() });
    let mut v: Vec<TrayIcon> = map.iter_mut().filter(|(_, e)| !e.hidden && e.hicon != 0).map(|(k, e)| {
        if e.png.is_none() { e.png = super::windows_list::hicon_data_url(e.hicon); }
        let exe = exe_of(e.hwnd);
        TrayIcon { id: k.clone(), tip: e.tip.clone(), icon: e.png.clone(), promoted: promoted(&exe), exe, rev: e.rev }
    }).collect();
    v.sort_by(|a, b| a.exe.to_lowercase().cmp(&b.exe.to_lowercase()).then(a.id.cmp(&b.id)));
    v
}

/// Forward a click the way Explorer does, so the app shows its own menu/window.
pub fn click(id: &str, button: &str) -> bool {
    let g = ICONS.lock().unwrap();
    let Some(e) = g.as_ref().and_then(|m| m.get(id)) else { return false };
    let hwnd = HWND(e.hwnd as usize as *mut _);
    let mut pt = POINT::default();
    unsafe { let _ = GetCursorPos(&mut pt); }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); let _ = AllowSetForegroundWindow(pid); }
    let v4 = e.version >= 4;
    // NOTIFYICON_VERSION_4: wParam = anchor x/y, lParam = (message, icon id). Older: (id, message).
    let params = |m: u32| -> (WPARAM, LPARAM) {
        if v4 { (WPARAM(((pt.x as u16 as u32) | ((pt.y as u16 as u32) << 16)) as usize), LPARAM((m | (e.uid << 16)) as isize)) }
        else { (WPARAM(e.uid as usize), LPARAM(m as isize)) }
    };
    let send = |m: u32| { let (w, l) = params(m); unsafe { let _ = SendNotifyMessageW(hwnd, e.callback, w, l); } };
    match button {
        "right" => { send(WM_RBUTTONDOWN); send(WM_RBUTTONUP); if v4 { send(WM_CONTEXTMENU); } }
        "double" => { send(WM_LBUTTONDBLCLK); send(WM_LBUTTONUP); }
        _ => { send(WM_LBUTTONDOWN); send(WM_LBUTTONUP); if v4 { send(WM_USER); /* NIN_SELECT */ } }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn nid(flags: u32, uid: u32, tip: &str) -> Nid32 {
        let mut n: Nid32 = unsafe { std::mem::zeroed() };
        n.cb_size = std::mem::size_of::<Nid32>() as u32;
        n.hwnd = 0x1234; n.uid = uid; n.flags = flags; n.callback = 0x8001; n.hicon = 7;
        for (i, c) in tip.encode_utf16().enumerate() { n.tip[i] = c; }
        n
    }

    #[test]
    fn layout_matches_shell_marshalling() {
        // cbSize of the 32-bit-handle NOTIFYICONDATAW (Vista+ layout) is 956 bytes.
        assert_eq!(std::mem::size_of::<Nid32>(), 956);
        assert_eq!(GUID_OFFSET, 936);
    }

    #[test]
    fn add_modify_hide_delete() {
        let mut m = HashMap::new();
        let copy = |h: u32| h as isize * 10;
        let freed = std::cell::RefCell::new(vec![]);
        let free = |h: isize| freed.borrow_mut().push(h);
        assert!(!apply(&mut m, NIM_MODIFY, &nid(NIF_TIP, 1, "x"), &copy, &free), "modify of unknown icon is ignored");
        assert!(apply(&mut m, NIM_ADD, &nid(NIF_MESSAGE | NIF_ICON | NIF_TIP, 1, "Volume"), &copy, &free));
        let e = m.get("1234:1").unwrap();
        assert_eq!((e.tip.as_str(), e.hicon, e.callback, e.rev), ("Volume", 70, 0x8001, 1));
        let mut hide = nid(NIF_STATE, 1, "");
        hide.state = NIS_HIDDEN; hide.state_mask = NIS_HIDDEN;
        apply(&mut m, NIM_MODIFY, &hide, &copy, &free);
        assert!(m["1234:1"].hidden);
        assert_eq!(m["1234:1"].tip, "Volume", "fields not in uFlags keep their value");
        let mut ver = nid(0, 1, ""); ver.version = 4;
        apply(&mut m, NIM_SETVERSION, &ver, &copy, &free);
        assert_eq!(m["1234:1"].version, 4);
        assert!(apply(&mut m, NIM_DELETE, &nid(0, 1, ""), &copy, &free));
        assert!(m.is_empty());
        assert_eq!(*freed.borrow(), vec![70], "copied icon is freed on delete");
    }
}
