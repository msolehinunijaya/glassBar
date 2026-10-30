//! Keep Awake: a Windows power request ("GlassBar: Keep awake", visible in
//! `powercfg /requests`). It belongs to this process, so it ends on Quit and on a crash.
use super::super::Res;
use super::wide;
use std::sync::Mutex;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Power::{PowerClearRequest, PowerCreateRequest, PowerRequestDisplayRequired, PowerRequestSystemRequired, PowerSetRequest};
use windows::Win32::System::Threading::{POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0};

/// The open request and whether it also keeps the screen on.
static REQ: Mutex<Option<(isize, bool)>> = Mutex::new(None);

fn clear_locked(r: &mut Option<(isize, bool)>) {
    if let Some((h, display)) = r.take() {
        let h = HANDLE(h as _);
        unsafe {
            let _ = PowerClearRequest(h, PowerRequestSystemRequired);
            if display { let _ = PowerClearRequest(h, PowerRequestDisplayRequired); }
            let _ = CloseHandle(h);
        }
    }
}

/// Keep the PC awake (and the screen on, with `display`). Replaces any earlier request.
pub fn set(display: bool) -> Res<()> {
    let mut r = REQ.lock().unwrap();
    clear_locked(&mut r);
    let mut reason = wide("GlassBar: Keep awake");
    let ctx = REASON_CONTEXT {
        Version: 0, // POWER_REQUEST_CONTEXT_VERSION
        Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
        Reason: REASON_CONTEXT_0 { SimpleReasonString: windows::core::PWSTR(reason.as_mut_ptr()) },
    };
    let e = |e: windows::core::Error| e.message().to_string();
    unsafe {
        let h = PowerCreateRequest(&ctx).map_err(e)?;
        if let Err(x) = PowerSetRequest(h, PowerRequestSystemRequired) { let _ = CloseHandle(h); return Err(e(x)); }
        if display { let _ = PowerSetRequest(h, PowerRequestDisplayRequired); }
        *r = Some((h.0 as isize, display));
    }
    Ok(())
}

/// Let the PC sleep again.
pub fn clear() { clear_locked(&mut REQ.lock().unwrap()) }
