//! Recent files (from the shell's Recent folder) that open with a given program.
use super::super::RecentItem;
use super::{com_init, wide};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ};
use windows::Win32::UI::Shell::{AssocQueryStringW, IShellLinkW, ShellLink, ASSOCF_NONE, ASSOCSTR_EXECUTABLE, SLGP_RAWPATH};

/// Extension -> program that opens it (cached; associations rarely change).
static ASSOC: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn assoc_exe(ext: &str) -> String {
    let mut cache = ASSOC.lock().unwrap();
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(e) = cache.get(ext) { return e.clone(); }
    let w = wide(ext);
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    let exe = unsafe {
        AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_EXECUTABLE, PCWSTR(w.as_ptr()), PCWSTR::null(), Some(PWSTR(buf.as_mut_ptr())), &mut len)
    }.ok().map(|_| String::from_utf16_lossy(&buf[..len.saturating_sub(1) as usize])).unwrap_or_default();
    cache.insert(ext.to_string(), exe.clone());
    exe
}

fn resolve(link: &IShellLinkW, file: &IPersistFile, lnk: &PathBuf) -> Option<String> {
    let w = wide(&lnk.to_string_lossy());
    unsafe {
        file.Load(PCWSTR(w.as_ptr()), STGM_READ).ok()?;
        let mut buf = [0u16; 1024];
        link.GetPath(&mut buf, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32).ok()?;
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n])).filter(|s| !s.is_empty())
    }
}

pub fn recent_for(exe: &str, limit: usize) -> Vec<RecentItem> {
    com_init();
    let Some(dir) = std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join(r"Microsoft\Windows\Recent")) else { return vec![] };
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut lnks: Vec<(std::time::SystemTime, PathBuf)> = rd.flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    lnks.sort_by(|a, b| b.0.cmp(&a.0));
    let Ok(link) = (unsafe { CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER) }) else { return vec![] };
    let Ok(file) = link.cast::<IPersistFile>() else { return vec![] };
    let mut out = Vec::new();
    for (_, lnk) in lnks.into_iter().take(300) {
        let Some(target) = resolve(&link, &file, &lnk) else { continue };
        let p = PathBuf::from(&target);
        let Some(ext) = p.extension().map(|e| format!(".{}", e.to_string_lossy().to_lowercase())) else { continue };
        if !p.is_file() || !crate::widgets::jumplist::matches_exe(&assoc_exe(&ext), exe) { continue; }
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        out.push(RecentItem { name, path: target });
        if out.len() >= limit { break; }
    }
    out
}
