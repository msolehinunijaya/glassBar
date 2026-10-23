//! Hidden top-level window that receives system broadcasts (display changes,
//! Explorer restarts, logoff) and owns our AppBar registration (fullscreen notices).
use super::super::SystemEvent;
use super::appbar;
use crate::config::TaskbarMode;
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::OnceLock;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::ABN_FULLSCREENAPP;
use windows::Win32::UI::WindowsAndMessaging::*;

static HWND_SYS: AtomicIsize = AtomicIsize::new(0);
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
static TX: OnceLock<Sender<SystemEvent>> = OnceLock::new();
static IGNORE_CREATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// GlassBar's tray host broadcasts TaskbarCreated so apps re-add icons; that isn't an Explorer restart.
pub fn ignore_next_taskbar_created() { IGNORE_CREATED.store(true, Ordering::Relaxed); }

pub fn hwnd() -> isize { HWND_SYS.load(Ordering::Relaxed) }

unsafe extern "system" fn proc(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let ev = match msg {
        WM_DISPLAYCHANGE => Some(SystemEvent::DisplayChanged),
        WM_DEVICECHANGE if wp.0 as u32 == DBT_DEVICEARRIVAL || wp.0 as u32 == DBT_DEVICEREMOVECOMPLETE => {
            // Only USB device interfaces are registered, so one event per device.
            let hdr = lp.0 as *const DEV_BROADCAST_HDR;
            (!hdr.is_null() && unsafe { (*hdr).dbch_devicetype } == DBT_DEVTYP_DEVICEINTERFACE)
                .then(|| SystemEvent::Device(wp.0 as u32 == DBT_DEVICEARRIVAL))
        }
        WM_POWERBROADCAST if wp.0 as u32 == PBT_APMPOWERSTATUSCHANGE => Some(SystemEvent::PowerChanged),
        appbar::APPBAR_MSG if wp.0 as u32 == ABN_FULLSCREENAPP => Some(SystemEvent::Fullscreen(lp.0 != 0)),
        WM_ENDSESSION if wp.0 != 0 => {
            // Logoff/shutdown: the process may die right after this returns, so clean up here.
            appbar::remove();
            appbar::set_taskbar(TaskbarMode::Show);
            None
        }
        m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) && IGNORE_CREATED.swap(false, Ordering::Relaxed) => None,
        m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            appbar::reset();
            Some(SystemEvent::ShellRestarted)
        }
        _ => None,
    };
    // Hand off to a worker: handlers call SHAppBarMessage, which can re-enter this proc.
    if let (Some(e), Some(tx)) = (ev, TX.get()) { let _ = tx.send(e); }
    unsafe { DefWindowProcW(h, msg, wp, lp) }
}

/// Create the window (blocks until it exists) and deliver events to `on_event` on a worker thread.
pub fn start(on_event: Box<dyn Fn(SystemEvent) + Send>) {
    let (tx, rx) = channel();
    let _ = TX.set(tx);
    std::thread::spawn(move || { while let Ok(e) = rx.recv() { on_event(e); } });

    let (ready_tx, ready_rx) = channel();
    std::thread::spawn(move || unsafe {
        let Ok(module) = GetModuleHandleW(None) else { let _ = ready_tx.send(()); return };
        let class = w!("GlassBarSystem");
        let wc = WNDCLASSW { lpfnWndProc: Some(proc), hInstance: module.into(), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        // A real (hidden) top-level window: message-only windows don't get broadcasts.
        if let Ok(h) = CreateWindowExW(WS_EX_TOOLWINDOW, class, w!("GlassBar"), WS_POPUP, 0, 0, 0, 0, None, None, Some(module.into()), None) {
            HWND_SYS.store(h.0 as isize, Ordering::Relaxed);
            // USB plug/unplug notifications (GUID_DEVINTERFACE_USB_DEVICE). Event-driven, no polling.
            let mut filter = DEV_BROADCAST_DEVICEINTERFACE_W {
                dbcc_size: std::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
                dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE.0,
                dbcc_classguid: windows::core::GUID::from_u128(0xA5DCBF10_6530_11D2_901F_00C04FB951ED),
                ..Default::default()
            };
            let _ = RegisterDeviceNotificationW(windows::Win32::Foundation::HANDLE(h.0), &mut filter as *mut _ as *const _, DEVICE_NOTIFY_WINDOW_HANDLE);
        }
        TASKBAR_CREATED.store(RegisterWindowMessageW(w!("TaskbarCreated")), Ordering::Relaxed);
        let _ = ready_tx.send(());
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
    let _ = ready_rx.recv();
}
