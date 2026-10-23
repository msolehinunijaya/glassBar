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
    // Windows quietly ignores power modes on any plan but Balanced: say so instead.
    if let Some(plan) = other_plan() {
        return Err(format!("Power mode only works with Windows' Balanced power plan, and this PC uses {plan}."));
    }
    let g = match m { PowerMode::Efficiency => EFFICIENCY, PowerMode::Balanced => BALANCED, PowerMode::Performance => PERFORMANCE };
    match unsafe { PowerSetActiveOverlayScheme(g) } {
        0 if mode() == Some(m) => Ok(()),
        0 => Err("Windows didn't apply the power mode.".into()),
        e => Err(format!("Windows refused the power mode change ({e})")),
    }
}

/// The Balanced power plan (Control Panel › Power Options). Power modes need it.
const BALANCED_PLAN: GUID = GUID::from_u128(0x381b4222_f694_41f0_9685_ff5bb260df2e);

fn active_plan() -> Option<GUID> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::System::Registry::HKEY;
    type GetActive = unsafe extern "system" fn(HKEY, *mut *mut GUID) -> u32;
    let f = proc::<GetActive>(windows::core::s!("PowerGetActiveScheme"))?;
    let mut p: *mut GUID = std::ptr::null_mut();
    unsafe {
        if f(HKEY::default(), &mut p) != 0 || p.is_null() { return None; }
        let g = *p;
        let _ = LocalFree(Some(HLOCAL(p as _)));
        Some(g)
    }
}

/// Name of the active plan if it isn't Balanced (power modes then do nothing).
pub fn other_plan() -> Option<String> {
    let g = active_plan()?;
    if g == BALANCED_PLAN { return None; }
    Some(match g.to_u128() {
        0x8c5e7fda_e8bf_4a96_9a85_a6e23a8c635c => "the High performance plan",
        0xa1841308_3541_4fab_bc81_f71556f20b4a => "the Power saver plan",
        0xe9a42b02_d5df_448d_aa00_03f14749eb61 => "the Ultimate Performance plan",
        _ => "a custom power plan",
    }.into())
}

pub fn use_balanced_plan() -> Res<()> {
    use windows::Win32::System::Registry::HKEY;
    type SetActive = unsafe extern "system" fn(HKEY, *const GUID) -> u32;
    let f = proc::<SetActive>(windows::core::s!("PowerSetActiveScheme")).ok_or("PowerSetActiveScheme not found")?;
    match unsafe { f(HKEY::default(), &BALANCED_PLAN) } {
        0 => Ok(()),
        e => Err(format!("Windows couldn't switch to the Balanced plan ({e})")),
    }
}

/// Battery saver / energy saver on? (Read-only: Windows offers no public switch.)
pub fn saver_on() -> Option<bool> {
    use windows::System::Power::{EnergySaverStatus, PowerManager};
    PowerManager::EnergySaverStatus().ok().map(|s| s == EnergySaverStatus::On)
}
