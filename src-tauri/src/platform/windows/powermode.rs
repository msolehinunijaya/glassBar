//! Windows power mode (Settings → System → Power & battery → Power mode) and
//! battery/energy saver status.
use super::super::{PowerMode, Res};
use windows::core::GUID;

// Exported by powrprof.dll (what Settings and the battery flyout call) but missing
// from the import library, so they are looked up at runtime.
type GetFn = unsafe extern "system" fn(*mut GUID) -> u32;
type SetFn = unsafe extern "system" fn(GUID) -> u32;

fn proc<T>(name: windows::core::PCSTR) -> Option<T> {
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    unsafe {
        let lib = LoadLibraryW(windows::core::w!("powrprof.dll")).ok()?;
        let f = GetProcAddress(lib, name)?;
        Some(std::mem::transmute_copy::<_, T>(&f))
    }
}

#[allow(non_snake_case)]
unsafe fn PowerGetEffectiveOverlayScheme(g: *mut GUID) -> u32 {
    match proc::<GetFn>(windows::core::s!("PowerGetEffectiveOverlayScheme")) { Some(f) => unsafe { f(g) }, None => 1 }
}

#[allow(non_snake_case)]
unsafe fn PowerSetActiveOverlayScheme(g: GUID) -> u32 {
    match proc::<SetFn>(windows::core::s!("PowerSetActiveOverlayScheme")) { Some(f) => unsafe { f(g) }, None => 1 }
}

const EFFICIENCY: GUID = GUID::from_u128(0x961cc777_2547_4f9d_8174_7d86181b8a7a);
const BALANCED: GUID = GUID::zeroed();
const PERFORMANCE: GUID = GUID::from_u128(0xded574b5_45a0_4f42_8737_46345c09c238);

pub fn mode() -> Option<PowerMode> {
    let mut g = GUID::zeroed();
    if unsafe { PowerGetEffectiveOverlayScheme(&mut g) } != 0 { return None; }
    Some(match g {
        EFFICIENCY => PowerMode::Efficiency,
        PERFORMANCE => PowerMode::Performance,
        _ => PowerMode::Balanced,
    })
}

pub fn set_mode(m: PowerMode) -> Res<()> {
    let g = match m { PowerMode::Efficiency => EFFICIENCY, PowerMode::Balanced => BALANCED, PowerMode::Performance => PERFORMANCE };
    match unsafe { PowerSetActiveOverlayScheme(g) } {
        0 => Ok(()),
        e => Err(format!("Windows refused the power mode change ({e})")),
    }
}

/// Battery saver / energy saver on? (Read-only: Windows offers no public switch.)
pub fn saver_on() -> Option<bool> {
    use windows::System::Power::{EnergySaverStatus, PowerManager};
    PowerManager::EnergySaverStatus().ok().map(|s| s == EnergySaverStatus::On)
}
