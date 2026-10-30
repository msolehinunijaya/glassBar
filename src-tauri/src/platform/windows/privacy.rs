//! Which apps are using the microphone or the camera right now: Windows' capability access
//! store (the same data behind the privacy indicator in the Windows taskbar).
//! HKCU\...\CapabilityAccessManager\ConsentStore\<microphone|webcam>\<app> and
//! ...\NonPackaged\<C:#path#to#app.exe>, each with LastUsedTimeStart / LastUsedTimeStop
//! (FILETIME). In use = started and not stopped (stop is 0).
use super::super::UsingApp;
use super::wide;
use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::*;

const BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";

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

fn open(path: &str) -> Option<HKEY> {
    let mut k = HKEY::default();
    let w = wide(path);
    (unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(w.as_ptr()), None, KEY_READ, &mut k) } == ERROR_SUCCESS).then_some(k)
}

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

/// Apps using `what` ("microphone" | "webcam") right now, newest first.
pub fn in_use(what: &str) -> Vec<UsingApp> {
    let mut out = vec![];
    for (path, packaged) in [(format!("{BASE}\\{what}"), true), (format!("{BASE}\\{what}\\NonPackaged"), false)] {
        let Some(k) = open(&path) else { continue };
        for key in subkeys(k) {
            if packaged && key == "NonPackaged" { continue; }
            let (start, stop) = (qword(k, &key, "LastUsedTimeStart"), qword(k, &key, "LastUsedTimeStop"));
            if !in_use_now(start, stop) { continue; }
            let (name, exe) = app_label(&key, packaged);
            out.push(UsingApp { name, exe, since: super::files::filetime_ms(start) });
        }
        unsafe { let _ = RegCloseKey(k); }
    }
    out.sort_by(|a, b| b.since.cmp(&a.since));
    out
}

/// Calls `f` whenever an app starts or stops using the microphone or camera. One thread
/// asleep in Windows (RegNotifyChangeKeyValue) for the life of the app.
pub fn watch(f: Box<dyn Fn() + Send>) {
    use windows::Win32::System::Threading::{CreateEventW, WaitForMultipleObjects};
    std::thread::spawn(move || {
        let keys: Vec<HKEY> = ["microphone", "webcam"].iter().filter_map(|w| open(&format!("{BASE}\\{w}"))).collect();
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
            // Apps write start and stop separately: let a burst settle.
            std::thread::sleep(std::time::Duration::from_millis(300));
            arm(i);
            f();
        }
    });
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
}

#[cfg(test)]
mod live {
    /// Read-only. `cargo test privacy::live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn reads_the_store() {
        println!("mic: {:?}", super::in_use("microphone"));
        println!("cam: {:?}", super::in_use("webcam"));
        println!("mic state: {:?}", super::super::audio::mic_state().map(|m| (m.muted, m.volume, m.devices.len())));
    }
}
