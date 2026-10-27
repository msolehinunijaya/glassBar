//! Projection mode (Win+P) and brightness.
use super::super::{DisplayMode, Res};
use super::{com_init, err};
use windows::core::{BOOL, BSTR, PCWSTR, w};
use windows::Win32::Devices::Display::*;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use windows::Win32::System::Com::*;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::*;

pub fn mode() -> Option<DisplayMode> {
    unsafe {
        let (mut np, mut nm) = (0u32, 0u32);
        GetDisplayConfigBufferSizes(QDC_DATABASE_CURRENT, &mut np, &mut nm).ok().ok()?;
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); np as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); nm as usize];
        let mut topo = DISPLAYCONFIG_TOPOLOGY_ID::default();
        QueryDisplayConfig(QDC_DATABASE_CURRENT, &mut np, paths.as_mut_ptr(), &mut nm, modes.as_mut_ptr(), Some(&mut topo))
            .ok().ok()?;
        match topo {
            DISPLAYCONFIG_TOPOLOGY_INTERNAL => Some(DisplayMode::Internal),
            DISPLAYCONFIG_TOPOLOGY_CLONE => Some(DisplayMode::Clone),
            DISPLAYCONFIG_TOPOLOGY_EXTEND => Some(DisplayMode::Extend),
            DISPLAYCONFIG_TOPOLOGY_EXTERNAL => Some(DisplayMode::External),
            _ => None,
        }
    }
}

/// How many displays are plugged in, also ones switched off right now ("PC only" hides the
/// second one but it's still connected).
pub fn connected() -> u32 {
    unsafe {
        let (mut np, mut nm) = (0u32, 0u32);
        if GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut np, &mut nm).ok().is_err() { return 1; }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); np as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); nm as usize];
        if QueryDisplayConfig(QDC_ALL_PATHS, &mut np, paths.as_mut_ptr(), &mut nm, modes.as_mut_ptr(), None).ok().is_err() { return 1; }
        let mut seen: Vec<(u32, i32, u32)> = paths[..np as usize].iter()
            .filter(|p| p.targetInfo.targetAvailable.as_bool())
            .map(|p| (p.targetInfo.adapterId.LowPart, p.targetInfo.adapterId.HighPart, p.targetInfo.id))
            .collect();
        seen.sort();
        seen.dedup();
        (seen.len() as u32).max(1)
    }
}

pub fn set_mode(m: DisplayMode) -> Res<()> {
    let topo = match m {
        DisplayMode::Internal => SDC_TOPOLOGY_INTERNAL,
        DisplayMode::Clone => SDC_TOPOLOGY_CLONE,
        DisplayMode::Extend => SDC_TOPOLOGY_EXTEND,
        DisplayMode::External => SDC_TOPOLOGY_EXTERNAL,
    };
    let r = unsafe { SetDisplayConfig(None, None, SDC_APPLY | topo) };
    if r == 0 { return Ok(()); }
    Err(if connected() < 2 { "No second display is connected".into() }
        else { "Windows couldn't switch the display. Check the second display's cable, or try again with Win+P".into() })
}

// ---- External monitors: DDC/CI ----

fn physical_monitors() -> Vec<PHYSICAL_MONITOR> {
    unsafe extern "system" fn each(m: HMONITOR, _: HDC, _: *mut RECT, out: LPARAM) -> BOOL {
        unsafe { (*(out.0 as *mut Vec<HMONITOR>)).push(m) };
        true.into()
    }
    let mut hmons: Vec<HMONITOR> = Vec::new();
    let mut out = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(each), LPARAM(&mut hmons as *mut _ as isize));
        for m in hmons {
            let mut n = 0u32;
            if GetNumberOfPhysicalMonitorsFromHMONITOR(m, &mut n).is_err() || n == 0 { continue; }
            let mut arr = vec![PHYSICAL_MONITOR::default(); n as usize];
            if GetPhysicalMonitorsFromHMONITOR(m, &mut arr).is_ok() { out.extend(arr); }
        }
    }
    out
}

fn ddc_get() -> Option<u32> {
    let mons = physical_monitors();
    let mut found = None;
    for pm in &mons {
        let (mut min, mut cur, mut max) = (0u32, 0u32, 0u32);
        if found.is_none() && unsafe { GetMonitorBrightness(pm.hPhysicalMonitor, &mut min, &mut cur, &mut max) } != 0 && max > min {
            found = Some(cur.saturating_sub(min) * 100 / (max - min));
        }
    }
    unsafe { let _ = DestroyPhysicalMonitors(&mons); }
    found
}

fn ddc_set(v: u32) -> bool {
    let mons = physical_monitors();
    let mut any = false;
    for pm in &mons {
        let (mut min, mut cur, mut max) = (0u32, 0u32, 0u32);
        unsafe {
            if GetMonitorBrightness(pm.hPhysicalMonitor, &mut min, &mut cur, &mut max) != 0 && max > min {
                any |= SetMonitorBrightness(pm.hPhysicalMonitor, min + (max - min) * v / 100) != 0;
            }
        }
    }
    unsafe { let _ = DestroyPhysicalMonitors(&mons); }
    any
}

// ---- Laptop panels: WMI ----

fn wmi() -> windows::core::Result<IWbemServices> {
    com_init();
    unsafe {
        let loc: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
        let svc = loc.ConnectServer(&BSTR::from("ROOT\\WMI"), &BSTR::new(), &BSTR::new(), &BSTR::new(), 0, &BSTR::new(), None)?;
        // RPC_C_AUTHN_WINNT = 10, RPC_C_AUTHZ_NONE = 0
        CoSetProxyBlanket(&svc, 10, 0, PCWSTR::null(), RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, None, EOAC_NONE)?;
        Ok(svc)
    }
}

fn wmi_first(svc: &IWbemServices, query: &str) -> windows::core::Result<IWbemClassObject> {
    unsafe {
        let en = svc.ExecQuery(&BSTR::from("WQL"), &BSTR::from(query), WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, None)?;
        let mut objs = [None];
        let mut n = 0u32;
        en.Next(WBEM_INFINITE, &mut objs, &mut n).ok()?;
        objs[0].take().ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))
    }
}

fn wmi_get() -> Option<u32> {
    let svc = wmi().ok()?;
    let obj = wmi_first(&svc, "SELECT CurrentBrightness FROM WmiMonitorBrightness").ok()?;
    let mut v = VARIANT::default();
    unsafe { obj.Get(w!("CurrentBrightness"), 0, &mut v, None, None).ok()? };
    i32::try_from(&v).ok().map(|b| b as u32)
}

fn wmi_set(value: u32) -> windows::core::Result<()> {
    let svc = wmi()?;
    let inst = wmi_first(&svc, "SELECT * FROM WmiMonitorBrightnessMethods")?;
    let mut path = VARIANT::default();
    unsafe {
        inst.Get(w!("__PATH"), 0, &mut path, None, None)?;
        let mut class = None;
        svc.GetObject(&BSTR::from("WmiMonitorBrightnessMethods"), Default::default(), None, Some(&mut class), None)?;
        let class = class.ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))?;
        let mut in_def = None;
        class.GetMethod(w!("WmiSetBrightness"), 0, &mut in_def, std::ptr::null_mut())?;
        let params = in_def.ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))?.SpawnInstance(0)?;
        params.Put(w!("Timeout"), 0, &VARIANT::from(0i32), 0)?;
        params.Put(w!("Brightness"), 0, &VARIANT::from(value as i32), 0)?;
        let path = BSTR::try_from(&path)?;
        svc.ExecMethod(&path, &BSTR::from("WmiSetBrightness"), Default::default(), None, &params, None, None)?;
    }
    Ok(())
}

pub fn brightness() -> Option<u32> { ddc_get().or_else(wmi_get) }

/// When GlassBar last set the brightness (ms). WMI's change event carries no "who", so a
/// change arriving shortly after ours is taken to be ours.
static OURS_AT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
const OURS_FOR_MS: u64 = 1500;
fn now_ms() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0) }

/// Pure: should a brightness event become an alert? Only a real change, not one we just made.
fn outside_change(last: Option<u32>, now: u32, since_ours_ms: u64) -> bool {
    last.is_some_and(|l| l != now) && since_ours_ms > OURS_FOR_MS
}

/// The laptop panel's brightness, as WMI announces it (WmiMonitorBrightnessEvent). Blocks
/// on the event, so it costs nothing between changes. External monitors (DDC/CI) send no
/// event and are not covered. No laptop panel or no WMI: try again every few minutes.
pub fn watch_brightness(on_change: Box<dyn Fn(u32) + Send>) {
    std::thread::spawn(move || {
        let mut last = wmi_get();
        loop {
            if let Ok(svc) = wmi() {
                let q = unsafe { svc.ExecNotificationQuery(&BSTR::from("WQL"), &BSTR::from("SELECT * FROM WmiMonitorBrightnessEvent"),
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, None) };
                if let Ok(en) = q {
                    loop {
                        let mut objs = [None];
                        let mut n = 0u32;
                        if unsafe { en.Next(WBEM_INFINITE, &mut objs, &mut n) }.is_err() || n == 0 { break; }
                        let Some(o) = objs[0].take() else { continue };
                        let mut v = VARIANT::default();
                        if unsafe { o.Get(w!("Brightness"), 0, &mut v, None, None) }.is_err() { continue; }
                        let Ok(b) = (unsafe { windows::Win32::System::Variant::VariantToUInt32(&v) }) else { continue };
                        let since = now_ms().saturating_sub(OURS_AT.load(std::sync::atomic::Ordering::Relaxed));
                        if outside_change(last, b, since) { on_change(b); }
                        last = Some(b);
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(300));
        }
    });
}

pub fn set_brightness(v: u32) -> Res<()> {
    let v = v.min(100);
    OURS_AT.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
    let ddc = ddc_set(v);
    // Again here: DDC/CI to external monitors can take longer than the window, and the
    // laptop panel's change event follows this call, not the first stamp.
    OURS_AT.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
    let laptop = wmi_set(v).is_ok();
    if ddc || laptop { Ok(()) } else { Err(err("No monitor here supports brightness control")) }
}

#[cfg(test)]
mod tests {
    use super::outside_change;

    #[test]
    fn brightness_alerts_only_for_real_changes_from_outside() {
        assert!(outside_change(Some(40), 60, 10_000), "a brightness key");
        assert!(!outside_change(Some(40), 40, 10_000), "same level again");
        assert!(!outside_change(None, 60, 10_000), "first reading is the baseline");
        assert!(!outside_change(Some(40), 60, 300), "our own slider, a moment ago");
    }
}
