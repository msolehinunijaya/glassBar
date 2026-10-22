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

pub fn set_mode(m: DisplayMode) -> Res<()> {
    let topo = match m {
        DisplayMode::Internal => SDC_TOPOLOGY_INTERNAL,
        DisplayMode::Clone => SDC_TOPOLOGY_CLONE,
        DisplayMode::Extend => SDC_TOPOLOGY_EXTEND,
        DisplayMode::External => SDC_TOPOLOGY_EXTERNAL,
    };
    let r = unsafe { SetDisplayConfig(None, None, SDC_APPLY | topo) };
    if r == 0 { Ok(()) } else { Err(format!("display mode change failed ({r})")) }
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
            found = Some((cur - min) * 100 / (max - min));
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

pub fn set_brightness(v: u32) -> Res<()> {
    let v = v.min(100);
    let ddc = ddc_set(v);
    let laptop = wmi_set(v).is_ok();
    if ddc || laptop { Ok(()) } else { Err(err("No monitor here supports brightness control")) }
}
