//! Privacy: which apps use the microphone, camera, location or screen capture, from Windows'
//! capability access store (the same data behind the privacy indicator in the Windows
//! taskbar and "Recent activity" in Settings > Privacy).
//! HKCU\...\CapabilityAccessManager\ConsentStore\<capability>\<app> and
//! ...\NonPackaged\<C:#path#to#app.exe>, each with LastUsedTimeStart / LastUsedTimeStop
//! (FILETIME). In use = started and not stopped (stop is 0). Its `Value` (Allow / Deny) is
//! the switch: HKCU = "Let apps access…", HKCU\...\NonPackaged = "Let desktop apps access…",
//! HKLM = "… access" for the whole device. Group policy: HKLM\SOFTWARE\Policies\...\AppPrivacy.
//! Also: cameras and microphones (Configuration Manager / Core Audio), the location service,
//! and turning a device on or off (pnputil, as administrator).
use super::super::{CapUse, Capability, PrivDevice, PrivacyFacts, Res, Service, UsingApp};
use super::wide;
use std::sync::mpsc::Sender;
use windows::core::{GUID, PCWSTR};
use windows::Win32::Devices::DeviceAndDriverInstallation::*;
use windows::Win32::Devices::Properties::{DEVPKEY_Device_DeviceDesc, DEVPKEY_Device_FriendlyName, DEVPKEY_Device_InstanceId, DEVPROPTYPE};
use windows::Win32::Foundation::{DEVPROPKEY, ERROR_SUCCESS};
use windows::Win32::System::Registry::*;

const BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";
const POLICY: &str = r"SOFTWARE\Policies\Microsoft\Windows\AppPrivacy";
/// Device interface classes: cameras (what camera apps open) and recording endpoints.
const KSCATEGORY_VIDEO_CAMERA: GUID = GUID::from_u128(0xe5323777_f976_4f5b_9b55_b94699c46e44);
const DEVINTERFACE_AUDIO_CAPTURE: GUID = GUID::from_u128(0x2eef81be_33fa_4800_9670_1cd474972c3f);

/// Pure: the name to show for a store key. Packaged "Microsoft.WindowsCamera_8wekyb3d8bbwe"
/// -> "Windows Camera"; non-packaged "C:#Program Files#Zoom#bin#Zoom.exe" -> "Zoom".
pub fn app_label(key: &str, packaged: bool) -> (String, String) {
    if !packaged {
        let path = key.replace('#', "\\");
        let stem = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| key.into());
        return (stem, path);
    }
    let id = key.split('_').next().unwrap_or(key);
    let last = id.rsplit('.').next().unwrap_or(id);
    // Split CamelCase: WindowsCamera -> Windows Camera (runs of capitals stay together).
    let mut out = String::new();
    let cs: Vec<char> = last.chars().collect();
    for (i, c) in cs.iter().enumerate() {
        if i > 0 && c.is_uppercase() && (cs[i - 1].is_lowercase() || cs.get(i + 1).is_some_and(|n| n.is_lowercase())) { out.push(' '); }
        out.push(*c);
    }
    (out, String::new())
}

/// Pure: (start, stop) FILETIMEs -> in use?
pub fn in_use_now(start: u64, stop: u64) -> bool { start > 0 && stop == 0 }

/// Pure: a store switch ("Allow" / "Deny" / "Prompt" / unset) -> allowed?
pub fn allowed(v: Option<&str>) -> Option<bool> {
    match v { Some("Allow") => Some(true), Some("Deny") => Some(false), _ => None }
}

/// Pure: an AppPrivacy policy value -> forced allow (1) / deny (2); 0 or unset = user decides.
pub fn policy_of(v: Option<u32>) -> Option<bool> {
    match v { Some(1) => Some(true), Some(2) => Some(false), _ => None }
}

fn open_in(root: HKEY, path: &str) -> Option<HKEY> {
    let mut k = HKEY::default();
    let w = wide(path);
    (unsafe { RegOpenKeyExW(root, PCWSTR(w.as_ptr()), None, KEY_READ, &mut k) } == ERROR_SUCCESS).then_some(k)
}
fn open(path: &str) -> Option<HKEY> { open_in(HKEY_CURRENT_USER, path) }

fn subkeys(k: HKEY) -> Vec<String> {
    let mut out = vec![];
    for i in 0.. {
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        if unsafe { RegEnumKeyExW(k, i, Some(windows::core::PWSTR(buf.as_mut_ptr())), &mut len, None, None, None, None) } != ERROR_SUCCESS { break; }
        out.push(String::from_utf16_lossy(&buf[..len as usize]));
    }
    out
}

fn qword(k: HKEY, sub: &str, name: &str) -> u64 {
    let (s, n) = (wide(sub), wide(name));
    let mut v = 0u64;
    let mut size = 8u32;
    let ok = unsafe { RegGetValueW(k, PCWSTR(s.as_ptr()), PCWSTR(n.as_ptr()), RRF_RT_REG_QWORD, None, Some(&mut v as *mut u64 as _), Some(&mut size)) };
    if ok == ERROR_SUCCESS { v } else { 0 }
}

fn dword(root: HKEY, path: &str, name: &str) -> Option<u32> {
    let (s, n) = (wide(path), wide(name));
    let mut v = 0u32;
    let mut size = 4u32;
    let ok = unsafe { RegGetValueW(root, PCWSTR(s.as_ptr()), PCWSTR(n.as_ptr()), RRF_RT_REG_DWORD, None, Some(&mut v as *mut u32 as _), Some(&mut size)) };
    (ok == ERROR_SUCCESS).then_some(v)
}

fn text(root: HKEY, path: &str, name: &str) -> Option<String> {
    let (s, n) = (wide(path), wide(name));
    let mut buf = [0u16; 64];
    let mut size = (buf.len() * 2) as u32;
    let ok = unsafe { RegGetValueW(root, PCWSTR(s.as_ptr()), PCWSTR(n.as_ptr()), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr() as _), Some(&mut size)) };
    (ok == ERROR_SUCCESS).then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)]))
}

/// Every app in the store that ever used `what` (in use now or before). None: the record
/// can't be read (no such capability on this Windows, or access refused). `frozen`: Windows
/// has moved this record into its protected database (an entry marked PersistedInDatabase,
/// Windows 11 24H2 and later): the registry values stop updating, so they are history only.
pub fn uses(what: &str) -> (Option<Vec<CapUse>>, bool) {
    let mut out = vec![];
    let (mut readable, mut frozen) = (false, false);
    for (path, packaged) in [(format!("{BASE}\\{what}"), true), (format!("{BASE}\\{what}\\NonPackaged"), false)] {
        let Some(k) = open(&path) else { continue };
        readable = true;
        for key in subkeys(k) {
            if packaged && key == "NonPackaged" { continue; }
            frozen |= dword(k, &key, "PersistedInDatabase") == Some(1);
            let (start, stop) = (qword(k, &key, "LastUsedTimeStart"), qword(k, &key, "LastUsedTimeStop"));
            if start == 0 { continue; }
            let (label, exe) = app_label(&key, packaged);
            let name = friendly(&key, packaged, &exe).unwrap_or(label);
            let until = if in_use_now(start, stop) { 0 } else { super::files::filetime_ms(stop) };
            out.push(CapUse { id: key, name, exe, since: super::files::filetime_ms(start), until });
        }
        unsafe { let _ = RegCloseKey(k); }
    }
    (readable.then_some(out), frozen)
}

/// The name Windows shows for an app: a Store app's display name, a program's description
/// ("msedge.exe" -> "Microsoft Edge"). Remembered: these don't change while GlassBar runs.
fn friendly(key: &str, packaged: bool, exe: &str) -> Option<String> {
    use std::collections::HashMap;
    static NAMES: std::sync::Mutex<Option<HashMap<String, Option<String>>>> = std::sync::Mutex::new(None);
    if let Some(hit) = NAMES.lock().unwrap().get_or_insert_with(HashMap::new).get(key) { return hit.clone(); }
    let name = if packaged { super::packages::display_name(key) } else { file_description(exe) };
    NAMES.lock().unwrap().as_mut().unwrap().insert(key.to_string(), name.clone());
    name
}

/// A program's FileDescription from its version resource.
fn file_description(exe: &str) -> Option<String> {
    use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    let w = wide(exe);
    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(w.as_ptr()), None);
        if size == 0 { return None; }
        let mut buf = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(w.as_ptr()), None, size, buf.as_mut_ptr() as _).ok()?;
        let query = |q: &str| -> Option<(*mut std::ffi::c_void, u32)> {
            let q = wide(q);
            let (mut p, mut len) = (std::ptr::null_mut(), 0u32);
            VerQueryValueW(buf.as_ptr() as _, PCWSTR(q.as_ptr()), &mut p, &mut len).as_bool().then_some((p, len)).filter(|(p, l)| !p.is_null() && *l > 0)
        };
        let lang = query(r"\VarFileInfo\Translation").filter(|(_, l)| *l >= 4)
            .map(|(p, _)| { let t = p as *const u16; format!("{:04x}{:04x}", *t, *t.add(1)) }).unwrap_or_else(|| "040904b0".into());
        let (p, len) = query(&format!(r"\StringFileInfo\{lang}\FileDescription"))?;
        let s = String::from_utf16_lossy(std::slice::from_raw_parts(p as *const u16, len as usize));
        let s = s.trim_end_matches('\0').trim().to_string();
        (!s.is_empty()).then_some(s)
    }
}

/// Apps using `what` ("microphone" | "webcam") right now, newest first: read live; the
/// store's record only where that can't be (and the record still updates).
pub fn in_use(what: &str) -> Vec<UsingApp> {
    let live = if what == "webcam" { camera_now() } else { mic_now() };
    let list = live.unwrap_or_else(|_| match uses(what) { (Some(u), false) => u, _ => vec![] });
    let mut out: Vec<UsingApp> = list.into_iter().filter(|u| u.until == 0)
        .map(|u| UsingApp { name: u.name, exe: u.exe, since: u.since }).collect();
    out.sort_by(|a, b| b.since.cmp(&a.since));
    out
}

// ---- Live: who uses the microphone or a camera right now ----

/// pid -> when GlassBar first saw it using that resource (ms), so "since" stays put.
static FIRST_SEEN: std::sync::Mutex<Vec<(u32, &'static str, i64)>> = std::sync::Mutex::new(Vec::new());

/// Running processes as apps, one per program (browsers run several).
fn as_apps(pids: &[(u32, String)], kind: &'static str) -> Vec<CapUse> {
    let now = crate::db::now_ms();
    let mut seen = FIRST_SEEN.lock().unwrap();
    seen.retain(|(p, k, _)| *k != kind || pids.iter().any(|(q, _)| q == p));
    let mut out: Vec<CapUse> = vec![];
    for (pid, exe) in pids {
        let since = match seen.iter().find(|(p, k, _)| p == pid && *k == kind) { Some(s) => s.2, None => { seen.push((*pid, kind, now)); now } };
        let id = if exe.is_empty() { format!("pid:{pid}") } else { exe.to_lowercase() };
        if let Some(x) = out.iter_mut().find(|x| x.id == id) { x.since = x.since.min(since); continue; }
        let stem = std::path::Path::new(exe).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| format!("Process {pid}"));
        let name = if exe.is_empty() { stem } else { friendly(&id, false, exe).unwrap_or(stem) };
        out.push(CapUse { id, name, exe: exe.clone(), since, until: 0 });
    }
    out
}

fn mic_now() -> Res<Vec<CapUse>> { super::audio::recording_now().map(|p| as_apps(&p, "mic")) }

/// Camera streaming per camera (symbolic link) -> process ids, from Media Foundation's sensor
/// activity monitor (the camera frame server reports who streams). None until it started.
static CAMERAS: std::sync::Mutex<Option<std::collections::HashMap<String, Vec<u32>>>> = std::sync::Mutex::new(None);

fn camera_now() -> Res<Vec<CapUse>> {
    let pids: Vec<u32> = {
        let g = CAMERAS.lock().unwrap();
        let Some(map) = g.as_ref() else { return Err("Camera activity monitoring isn't running".into()) };
        map.values().flatten().copied().collect()
    };
    // A process that quit while streaming may not get a final report: only live ones count.
    let list: Vec<(u32, String)> = pids.into_iter().filter_map(|p| { let e = super::windows_list::exe_path(p); (!e.is_empty() || alive(p)).then_some((p, e)) }).collect();
    Ok(as_apps(&list, "cam"))
}

fn alive(pid: u32) -> bool {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map(|h| { let _ = windows::Win32::Foundation::CloseHandle(h); }).is_ok() }
}

#[windows::core::implement(windows::Win32::Media::MediaFoundation::IMFSensorActivitiesReportCallback)]
struct SensorReports(Sender<()>);

impl windows::Win32::Media::MediaFoundation::IMFSensorActivitiesReportCallback_Impl for SensorReports_Impl {
    fn OnActivitiesReport(&self, report: windows::core::Ref<windows::Win32::Media::MediaFoundation::IMFSensorActivitiesReport>) -> windows::core::Result<()> {
        let r = report.ok()?;
        let mut update: Vec<(String, Vec<u32>)> = vec![];
        unsafe {
            for i in 0..r.GetCount()? {
                let a = r.GetActivityReport(i)?;
                let mut buf = [0u16; 512];
                let mut n = 0u32;
                a.GetSymbolicLink(&mut buf, &mut n)?;
                let link = String::from_utf16_lossy(&buf[..(n as usize).min(buf.len())]).trim_end_matches('\0').to_string();
                let mut pids = vec![];
                for j in 0..a.GetProcessCount()? {
                    let p = a.GetProcessActivity(j)?;
                    if p.GetStreamingState()?.as_bool() { pids.push(p.GetProcessId()?); }
                }
                update.push((link, pids));
            }
        }
        CAMERAS.lock().unwrap().get_or_insert_with(Default::default).extend(update);
        let _ = self.0.send(());
        Ok(())
    }
}

/// Start Media Foundation's sensor activity monitor (Windows 10 1903+) for the app's life.
fn watch_cameras(tx: Sender<()>) {
    use windows::Win32::Media::MediaFoundation::*;
    std::thread::spawn(move || unsafe {
        super::com_init();
        let started = MFStartup(MF_VERSION, MFSTARTUP_LITE).and_then(|_| {
            let cb: IMFSensorActivitiesReportCallback = SensorReports(tx).into();
            let m = MFCreateSensorActivityMonitor(&cb)?;
            m.Start()?;
            Ok(m)
        });
        match started {
            Ok(_monitor) => {
                // Monitoring works: no camera reported yet means none is streaming.
                CAMERAS.lock().unwrap().get_or_insert_with(Default::default);
                loop { std::thread::park(); } // keeps `_monitor` alive
            }
            Err(e) => crate::applog::log(format!("privacy: camera activity monitor failed ({})", e.message())),
        }
    });
}

fn capability(what: &str, policy: &str) -> Capability {
    let at = format!("{BASE}\\{what}");
    let (uses, frozen) = uses(what);
    Capability {
        uses,
        frozen,
        user: allowed(text(HKEY_CURRENT_USER, &at, "Value").as_deref()),
        device: allowed(text(HKEY_LOCAL_MACHINE, &at, "Value").as_deref()),
        desktop: allowed(text(HKEY_CURRENT_USER, &format!("{at}\\NonPackaged"), "Value").as_deref()),
        policy: policy_of(dword(HKEY_LOCAL_MACHINE, POLICY, policy)),
    }
}

/// Screen capture: apps that captured without the yellow border, and ones that did it
/// programmatically (Windows Graphics Capture). Switch: "Screenshots and screen recording".
fn capture() -> Capability {
    let mut c = capability("graphicsCaptureProgrammatic", "LetAppsAccessGraphicsCaptureProgrammatic");
    let (more, frozen) = uses("graphicsCaptureWithoutBorder");
    c.frozen |= frozen;
    if let Some(more) = more {
        let list = c.uses.get_or_insert_with(Vec::new);
        for u in more {
            match list.iter_mut().find(|x| x.id == u.id) {
                // The same app in both: keep its latest session.
                Some(x) => if u.since > x.since { *x = u },
                None => list.push(u),
            }
        }
    }
    c
}

// ---- Devices (Configuration Manager) ----

fn prop(node: u32, key: &DEVPROPKEY) -> Option<String> {
    let mut ty = DEVPROPTYPE::default();
    let mut buf = [0u16; 256];
    let mut size = (buf.len() * 2) as u32;
    let r = unsafe { CM_Get_DevNode_PropertyW(node, key, &mut ty, Some(buf.as_mut_ptr() as *mut u8), &mut size, 0) };
    (r == CR_SUCCESS).then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)])).filter(|s| !s.is_empty())
}

fn node(instance: &str) -> Option<u32> {
    let w = wide(instance);
    let mut n = 0u32;
    (unsafe { CM_Locate_DevNodeW(&mut n, PCWSTR(w.as_ptr()), CM_LOCATE_DEVNODE_NORMAL) } == CR_SUCCESS).then_some(n)
}

/// Pure: a device node's problem code -> turned off? 22 = disabled (Device Manager, pnputil,
/// Settings > Cameras); 29 = turned off in the firmware (BIOS/UEFI).
pub fn off_problem(problem: u32) -> bool { problem == 22 || problem == 29 }

/// On or off; None when the device isn't connected (no live node).
pub fn node_enabled(instance: &str) -> Option<bool> {
    let n = node(instance)?;
    let (mut status, mut problem) = (CM_DEVNODE_STATUS_FLAGS::default(), CM_PROB::default());
    if unsafe { CM_Get_DevNode_Status(&mut status, &mut problem, n, 0) } != CR_SUCCESS { return None; }
    Some(!(status.0 & DN_HAS_PROBLEM.0 != 0 && off_problem(problem.0)))
}

/// Connected cameras, also turned-off ones (their interfaces stay listed, disabled).
fn cameras() -> Res<Vec<PrivDevice>> {
    let mut out: Vec<PrivDevice> = vec![];
    for path in interfaces(&KSCATEGORY_VIDEO_CAMERA)? {
        let Some(inst) = interface_instance(&path) else { continue };
        if out.iter().any(|d| d.instance.eq_ignore_ascii_case(&inst)) { continue; }
        let Some(n) = node(&inst) else { continue }; // not connected
        let name = prop(n, &DEVPKEY_Device_FriendlyName).or_else(|| prop(n, &DEVPKEY_Device_DeviceDesc)).unwrap_or_else(|| "Camera".into());
        out.push(PrivDevice { id: inst.clone(), enabled: node_enabled(&inst).unwrap_or(false), instance: inst, name, default: false });
    }
    Ok(out)
}

fn interfaces(class: &GUID) -> Res<Vec<String>> {
    unsafe {
        let mut len = 0u32;
        let r = CM_Get_Device_Interface_List_SizeW(&mut len, class, PCWSTR::null(), CM_GET_DEVICE_INTERFACE_LIST_ALL_DEVICES);
        if r != CR_SUCCESS { return Err(format!("Windows couldn't list devices (CM error {})", r.0)); }
        let mut buf = vec![0u16; len as usize];
        let r = CM_Get_Device_Interface_ListW(class, PCWSTR::null(), &mut buf, CM_GET_DEVICE_INTERFACE_LIST_ALL_DEVICES);
        if r != CR_SUCCESS { return Err(format!("Windows couldn't list devices (CM error {})", r.0)); }
        Ok(buf.split(|c| *c == 0).filter(|s| !s.is_empty()).map(String::from_utf16_lossy).collect())
    }
}

fn interface_instance(path: &str) -> Option<String> {
    let w = wide(path);
    let mut ty = DEVPROPTYPE::default();
    let mut buf = [0u16; 512];
    let mut size = (buf.len() * 2) as u32;
    let r = unsafe { CM_Get_Device_Interface_PropertyW(PCWSTR(w.as_ptr()), &DEVPKEY_Device_InstanceId, &mut ty, Some(buf.as_mut_ptr() as *mut u8), &mut size, 0) };
    (r == CR_SUCCESS).then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)]))
}

// ---- Location service ----

/// lfsvc: Ok(None) when it isn't installed.
fn location_service() -> Res<Option<Service>> {
    use windows::Win32::System::Services::*;
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT).map_err(|e| format!("Windows didn't let GlassBar read its services ({})", e.message()))?;
        let name = wide("lfsvc");
        let svc = match OpenServiceW(scm, PCWSTR(name.as_ptr()), SERVICE_QUERY_STATUS) {
            Ok(s) => s,
            Err(e) => {
                let _ = CloseServiceHandle(scm);
                return if e.code() == windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() { Ok(None) } else { Err(format!("Couldn't read the location service ({})", e.message())) };
            }
        };
        let mut st = SERVICE_STATUS::default();
        let ok = QueryServiceStatus(svc, &mut st).is_ok();
        let _ = CloseServiceHandle(svc);
        let _ = CloseServiceHandle(scm);
        if !ok { return Err("Couldn't read the location service's status".into()); }
        // Start type 4 = Disabled (the registry is the service's own config).
        let disabled = dword(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Services\lfsvc", "Start") == Some(4);
        Ok(Some(Service { running: st.dwCurrentState == SERVICE_RUNNING, disabled }))
    }
}

/// Lower-case exe names of the running processes.
fn running() -> Vec<String> {
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    let mut out = vec![];
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let n = e.szExeFile.iter().position(|c| *c == 0).unwrap_or(e.szExeFile.len());
            out.push(String::from_utf16_lossy(&e.szExeFile[..n]).to_lowercase());
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
    }
    out.sort();
    out.dedup();
    out
}

pub fn facts() -> PrivacyFacts {
    let mics = super::audio::capture_endpoints();
    PrivacyFacts {
        camera: capability("webcam", "LetAppsAccessCamera"),
        mic: capability("microphone", "LetAppsAccessMicrophone"),
        location: capability("location", "LetAppsAccessLocation"),
        capture: capture(),
        camera_live: camera_now(),
        mic_live: mic_now(),
        cameras: cameras(),
        mic_muted: super::audio::mic_state().ok().map(|m| m.muted),
        mics,
        location_service: location_service(),
        location_policy_off: dword(HKEY_LOCAL_MACHINE, r"SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors", "DisableLocation") == Some(1),
        running: running(),
    }
}

// ---- Turning a device on or off ----

/// pnputil /enable-device or /disable-device, as administrator (Windows shows its UAC
/// prompt), waiting until it finished. The exit code: 0 done, 3010 after a restart.
pub fn set_enabled(instance: &str, on: bool) -> Res<u32> {
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
    if instance.contains('"') { return Err("Not a device id".into()); }
    let (verb, file) = (wide("runas"), wide("pnputil.exe"));
    let params = wide(&format!("{} \"{instance}\"", if on { "/enable-device" } else { "/disable-device" }));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32, fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()), lpFile: PCWSTR(file.as_ptr()), lpParameters: PCWSTR(params.as_ptr()), nShow: SW_HIDE.0, ..Default::default()
    };
    unsafe {
        if let Err(e) = ShellExecuteExW(&mut info) {
            return Err(if e.code() == windows::Win32::Foundation::ERROR_CANCELLED.to_hresult() {
                "Not changed: the Windows administrator prompt was cancelled".into()
            } else { format!("Windows couldn't start the device tool ({})", e.message()) });
        }
        if info.hProcess.is_invalid() { return Err("Windows didn't report the device tool's result".into()); }
        let waited = WaitForSingleObject(info.hProcess, 60_000);
        let mut code = 0u32;
        let got = GetExitCodeProcess(info.hProcess, &mut code).is_ok();
        let _ = CloseHandle(info.hProcess);
        if waited != WAIT_OBJECT_0 || !got { return Err("The device tool didn't finish within a minute".into()); }
        match code {
            0 | 3010 => Ok(code),
            5 => Err("Windows refused: administrator rights are needed".into()),
            c => Err(format!("Windows couldn't change the device (pnputil error {c:#x})")),
        }
    }
}

// ---- Watching ----

/// Calls `f` when anything the Privacy Monitor shows changes: an app starts or stops using
/// the microphone, camera, location or screen capture, a privacy switch is flipped, or a
/// camera or microphone comes, goes, or is turned on or off. Asleep in Windows meanwhile
/// (RegNotifyChangeKeyValue, CM_Register_Notification); one thread for the app's life.
pub fn watch(f: Box<dyn Fn() + Send>) {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    watch_registry(tx.clone());
    watch_devices(tx.clone());
    watch_cameras(tx.clone());
    watch_mic(tx);
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            // Apps write start and stop separately, devices come with several interfaces:
            // let a burst settle.
            std::thread::sleep(std::time::Duration::from_millis(300));
            while rx.try_recv().is_ok() {}
            f();
        }
    });
}

/// Who records from a microphone: Core Audio's capture sessions, compared every 2 s.
// ponytail: polling; IAudioSessionNotification + per-session events if 2 s ever feels slow.
fn watch_mic(tx: Sender<()>) {
    std::thread::spawn(move || {
        let mut last: Vec<u32> = vec![];
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let mut now: Vec<u32> = super::audio::recording_now().unwrap_or_default().into_iter().map(|x| x.0).collect();
            now.sort();
            if now != last { last = now; if tx.send(()).is_err() { return; } }
        }
    });
}

fn watch_registry(tx: Sender<()>) {
    use windows::Win32::System::Threading::{CreateEventW, WaitForMultipleObjects};
    std::thread::spawn(move || {
        let mut keys: Vec<HKEY> = ["microphone", "webcam", "location", "graphicsCaptureProgrammatic", "graphicsCaptureWithoutBorder"]
            .iter().filter_map(|w| open(&format!("{BASE}\\{w}"))).collect();
        // The device-wide switches.
        keys.extend(open_in(HKEY_LOCAL_MACHINE, BASE));
        if keys.is_empty() { return; }
        let events: Vec<_> = keys.iter().filter_map(|_| unsafe { CreateEventW(None, false, false, None) }.ok()).collect();
        if events.len() != keys.len() { return; }
        let arm = |i: usize| unsafe {
            let _ = RegNotifyChangeKeyValue(keys[i], true, REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET, Some(events[i]), true);
        };
        for i in 0..keys.len() { arm(i); }
        loop {
            let r = unsafe { WaitForMultipleObjects(&events, false, u32::MAX) };
            let i = r.0 as usize;
            if i >= keys.len() { std::thread::sleep(std::time::Duration::from_secs(5)); continue; }
            arm(i);
            if tx.send(()).is_err() { return; }
        }
    });
}

unsafe extern "system" fn on_device(_: HCMNOTIFICATION, ctx: *const std::ffi::c_void, _: CM_NOTIFY_ACTION, _: *const CM_NOTIFY_EVENT_DATA, _: u32) -> u32 {
    let tx = unsafe { &*(ctx as *const Sender<()>) };
    let _ = tx.send(());
    ERROR_SUCCESS.0
}

/// Camera and microphone interfaces arriving or leaving (plugged, removed, turned on/off).
fn watch_devices(tx: Sender<()>) {
    // Lives for the app's life: Windows calls back with it from its own threads.
    let ctx = Box::into_raw(Box::new(tx)) as *const std::ffi::c_void;
    for class in [KSCATEGORY_VIDEO_CAMERA, DEVINTERFACE_AUDIO_CAPTURE] {
        let mut filter = CM_NOTIFY_FILTER { cbSize: std::mem::size_of::<CM_NOTIFY_FILTER>() as u32, FilterType: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE, ..Default::default() };
        filter.u.DeviceInterface.ClassGuid = class;
        let mut h = HCMNOTIFICATION::default();
        let r = unsafe { CM_Register_Notification(&filter, Some(ctx), Some(on_device), &mut h) };
        if r != CR_SUCCESS { crate::applog::log(&format!("privacy: device watch failed ({})", r.0)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_store_keys() {
        assert_eq!(app_label("Microsoft.WindowsCamera_8wekyb3d8bbwe", true).0, "Windows Camera");
        assert_eq!(app_label("MSTeams_8wekyb3d8bbwe", true).0, "MS Teams");
        let (n, p) = app_label("C:#Program Files#Zoom#bin#Zoom.exe", false);
        assert_eq!(n, "Zoom");
        assert_eq!(p, r"C:\Program Files\Zoom\bin\Zoom.exe");
    }
    #[test]
    fn in_use_means_started_and_not_stopped() {
        assert!(in_use_now(5, 0));
        assert!(!in_use_now(5, 6));
        assert!(!in_use_now(0, 0));
    }
    #[test]
    fn reads_switches_and_policy() {
        assert_eq!(allowed(Some("Allow")), Some(true));
        assert_eq!(allowed(Some("Deny")), Some(false));
        assert_eq!(allowed(Some("Prompt")), None);
        assert_eq!(allowed(None), None);
        assert_eq!(policy_of(Some(2)), Some(false));
        assert_eq!(policy_of(Some(1)), Some(true));
        assert_eq!(policy_of(Some(0)), None);
        assert!(off_problem(22) && off_problem(29) && !off_problem(10));
    }
}

#[cfg(test)]
mod live {
    /// Read-only. `cargo test privacy::live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn reads_the_store() {
        println!("mic: {:?}", super::in_use("microphone"));
        println!("cam: {:?}", super::in_use("webcam"));
        let f = super::facts();
        println!("camera: {:?}\ncameras: {:?}", f.camera, f.cameras);
        println!("mic: {:?}\nmics: {:?} muted {:?}", f.mic.user, f.mics, f.mic_muted);
        println!("location: {:?} service {:?} policy off {}", f.location, f.location_service, f.location_policy_off);
        println!("capture: {:?}", f.capture.uses);
        println!("running: {}", f.running.len());
    }
}
