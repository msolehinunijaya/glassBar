//! Windows 11 remembers per program whether its tray icon is "always shown" (promoted)
//! under HKCU\Control Panel\NotifyIconSettings\<id>: ExecutablePath + IsPromoted.
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Registry::*;

fn read_str(key: HKEY, name: &str) -> Option<String> {
    let mut buf = vec![0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    let r = unsafe { RegGetValueW(key, None, &HSTRING::from(name), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr() as *mut _), Some(&mut size)) };
    r.is_ok().then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)]))
}
fn read_dword(key: HKEY, name: &str) -> Option<u32> {
    let mut v = 0u32;
    let mut size = 4u32;
    let r = unsafe { RegGetValueW(key, None, &HSTRING::from(name), RRF_RT_REG_DWORD, None, Some(&mut v as *mut _ as *mut _), Some(&mut size)) };
    r.is_ok().then_some(v)
}

/// Known-folder GUID prefixes Windows uses in ExecutablePath -> real folders.
pub fn expand(path: &str) -> String {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let win = env("windir");
    let map = [
        ("{6D809377-6AF0-444B-8957-A3773F02200E}", env("ProgramW6432")),
        ("{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}", env("ProgramFiles(x86)")),
        ("{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}", format!(r"{win}\System32")),
        ("{F38BF404-1D43-42F2-9305-67DE0B28FC23}", win.clone()),
    ];
    for (g, real) in map {
        if path.len() >= g.len() && path[..g.len()].eq_ignore_ascii_case(g) { return format!("{real}{}", &path[g.len()..]); }
    }
    path.to_string()
}

/// Lower-case exe paths whose icon the user set to "always show".
pub fn load() -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    unsafe {
        let mut root = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(r"Control Panel\NotifyIconSettings"), Some(0), KEY_READ, &mut root).is_err() { return out; }
        let mut i = 0u32;
        loop {
            let mut name = [0u16; 64];
            let mut len = name.len() as u32;
            if RegEnumKeyExW(root, i, Some(windows::core::PWSTR(name.as_mut_ptr())), &mut len, None, None, None, None).is_err() { break; }
            i += 1;
            let mut sub = HKEY::default();
            if RegOpenKeyExW(root, PCWSTR(name.as_ptr()), Some(0), KEY_READ, &mut sub).is_ok() {
                if read_dword(sub, "IsPromoted") == Some(1) {
                    if let Some(p) = read_str(sub, "ExecutablePath") { out.insert(expand(&p).to_lowercase()); }
                }
                let _ = RegCloseKey(sub);
            }
        }
        let _ = RegCloseKey(root);
    }
    out
}

pub fn is_promoted(set: &std::collections::HashSet<String>, exe: &str) -> bool { set.contains(&exe.to_lowercase()) }

#[cfg(test)]
mod tests {
    #[test]
    fn expands_known_folder_guids() {
        std::env::set_var("ProgramW6432", r"C:\Program Files");
        assert_eq!(super::expand(r"{6D809377-6AF0-444B-8957-A3773F02200E}\App\app.exe"), r"C:\Program Files\App\app.exe");
        assert_eq!(super::expand(r"C:\x.exe"), r"C:\x.exe");
    }
}
