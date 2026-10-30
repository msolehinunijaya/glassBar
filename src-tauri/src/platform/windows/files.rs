//! Files: the Downloads known folder, real file-type icons, show in folder / open with,
//! move to Recycle Bin (never a permanent delete), drag a file out, and the Recycle Bin
//! size / empty.
use super::super::{BinItem, RecycleDrive, RecycleInfo, Res};
use super::wide;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::UI::Shell::*;

/// The user's real Downloads folder (it may have been moved to another drive).
pub fn downloads_dir() -> Option<String> {
    unsafe {
        let p: PWSTR = SHGetKnownFolderPath(&FOLDERID_Downloads, KNOWN_FOLDER_FLAG(0), None).ok()?;
        let s = p.to_string().ok();
        windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as _));
        s
    }
}

/// The icon Windows shows for this file (its own icon for .exe/.lnk, the type's otherwise).
pub fn file_icon(path: &str) -> Option<String> {
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
    let p = wide(path);
    let mut info = SHFILEINFOW::default();
    unsafe {
        let ok = SHGetFileInfoW(PCWSTR(p.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut info), std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_ICON | SHGFI_LARGEICON);
        if ok == 0 || info.hIcon.is_invalid() { return None; }
        let png = super::windows_list::icon_to_png(info.hIcon);
        let _ = DestroyIcon(info.hIcon);
        png.map(super::windows_list::data_url)
    }
}

/// File Explorer with the file selected.
pub fn reveal(path: &str) -> Res<()> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer.exe").raw_arg(format!("/select,\"{path}\"")).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Windows' "Open with" chooser.
pub fn open_with(path: &str) -> Res<()> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("rundll32.exe").raw_arg(format!("shell32.dll,OpenAs_RunDLL {path}")).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Shell calls (Recycle Bin, file operations) expect a COM apartment on their thread;
/// without one they can wait forever. Run each on its own short-lived STA thread.
fn on_sta<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::spawn(move || {
        use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
        let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) }.is_ok();
        let r = f();
        if init { unsafe { CoUninitialize() } }
        r
    }).join().expect("shell thread")
}

/// Move to the Recycle Bin. FOF_WANTNUKEWARNING: if Windows can't recycle it (too big,
/// bin turned off), it asks before deleting for good; GlassBar never deletes outright.
pub fn recycle(path: &str) -> Res<()> {
    let path = path.to_string();
    on_sta(move || recycle_now(&path))
}
fn recycle_now(path: &str) -> Res<()> {
    let mut from: Vec<u16> = path.encode_utf16().collect();
    from.extend([0, 0]); // double-NUL-terminated list
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_WANTNUKEWARNING).0 as u16,
        ..Default::default()
    };
    match unsafe { SHFileOperationW(&mut op) } {
        0 if !op.fAnyOperationsAborted.as_bool() => Ok(()),
        0 => Err("Cancelled".into()),
        e => Err(format!("Windows couldn't move it to the Recycle Bin (code {e})")),
    }
}

/// Drag the file out of the dock (into a chat, an email…), with the shell's own drag.
/// Must run on the window's UI thread while the mouse button is down.
pub fn drag_out(hwnd: isize, path: &str) -> Res<()> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::IDataObject;
    use windows::Win32::System::Ole::{DROPEFFECT_COPY, DROPEFFECT_LINK};
    let p = wide(path);
    unsafe {
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(p.as_ptr()), None).map_err(|e| e.to_string())?;
        let data: IDataObject = item.BindToHandler(None, &BHID_DataObject).map_err(|e| e.to_string())?;
        SHDoDragDrop(Some(HWND(hwnd as _)), &data, None, DROPEFFECT_COPY | DROPEFFECT_LINK).map(|_| ()).map_err(|e| e.to_string())
    }
}

fn query(root: Option<&str>) -> Res<(u64, u64)> {
    let w = root.map(wide);
    let mut q = SHQUERYRBINFO { cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };
    unsafe { SHQueryRecycleBinW(w.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr())), &mut q) }
        .map_err(|e| e.message().to_string())?;
    Ok((q.i64Size.max(0) as u64, q.i64NumItems.max(0) as u64))
}

/// Recycle Bin size and item count per fixed drive (each has its own bin), and the total.
/// Windows counts every item, so this takes ~2 s for a bin of ~18,000 items: one query per
/// drive, summed (no extra all-drives query).
pub fn recycle_info() -> Res<RecycleInfo> { on_sta(recycle_info_now) }
fn recycle_info_now() -> Res<RecycleInfo> {
    use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives};
    const DRIVE_FIXED: u32 = 3;
    let mut first_err = None;
    let mask = unsafe { GetLogicalDrives() };
    let mut drives = vec![];
    for i in 0..26u32 {
        if mask & (1 << i) == 0 { continue; }
        let root = format!("{}:\\", (b'A' + i as u8) as char);
        let w = wide(&root);
        if unsafe { GetDriveTypeW(PCWSTR(w.as_ptr())) } != DRIVE_FIXED { continue; }
        let (s, n) = match query(Some(&root)) { Ok(v) => v, Err(e) => { first_err.get_or_insert(e); continue } };
        let mut free = 0u64;
        unsafe { let _ = GetDiskFreeSpaceExW(PCWSTR(w.as_ptr()), None, None, Some(&mut free)); }
        drives.push(RecycleDrive { root, size: s, items: n, free });
    }
    // Every drive failed: that is Windows' answer (e.g. a policy), not an empty bin.
    if drives.is_empty() { if let Some(e) = first_err { return Err(e); } }
    Ok(RecycleInfo { size: drives.iter().map(|d| d.size).sum(), items: drives.iter().map(|d| d.items).sum(), drives })
}

/// Empty the bin (all drives, or one). GlassBar asked first; Windows shows its own progress
/// and plays its sound.
pub fn recycle_empty(root: Option<&str>) -> Res<()> {
    let root = root.map(str::to_string);
    on_sta(move || recycle_empty_now(root.as_deref()))
}
fn recycle_empty_now(root: Option<&str>) -> Res<()> {
    let w = root.map(wide);
    unsafe { SHEmptyRecycleBinW(None, w.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr())), SHERB_NOCONFIRMATION) }
        .map_err(|e| e.message().to_string())
}

/// Empty the bin with Windows' own confirmation dialog (right-click › Empty on the dock).
pub fn recycle_empty_ask() -> Res<()> {
    on_sta(|| unsafe { SHEmptyRecycleBinW(None, PCWSTR::null(), 0) }.map_err(|e| e.message().to_string()))
}

// ---- Items in the bin ----

use windows::Win32::Foundation::PROPERTYKEY;
/// "System.Recycle.DeletedFrom" / "System.Recycle.DateDeleted" (not in windows-rs).
const PKEY_DELETED_FROM: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from_u128(0x9b174b33_40ff_11d2_a27e_00c04fc30871), pid: 2 };
const PKEY_DATE_DELETED: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from_u128(0x9b174b33_40ff_11d2_a27e_00c04fc30871), pid: 3 };
/// "System.FileName": the original name with its extension (display names may hide it).
const PKEY_FILE_NAME: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from_u128(0x41cf5ae0_f75a_4806_bd87_59c7d9248eb9), pid: 100 };
const PKEY_SIZE: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac), pid: 12 };

/// FILETIME (100 ns since 1601) -> Unix ms.
pub fn filetime_ms(ft: u64) -> i64 { (ft as i64 - 116_444_736_000_000_000) / 10_000 }

/// Every item in the Recycle Bin (as the shell sees them) with the item itself.
fn bin_items() -> Res<Vec<(BinItem, IShellItem2)>> {
    use windows::core::Interface;
    use windows::Win32::System::SystemServices::{SFGAO_FOLDER, SFGAO_STREAM};
    let e = |e: windows::core::Error| e.message().to_string();
    unsafe {
        let bin: IShellItem = SHGetKnownFolderItem(&FOLDERID_RecycleBinFolder, KNOWN_FOLDER_FLAG(0), None).map_err(e)?;
        let list: IEnumShellItems = bin.BindToHandler(None, &BHID_EnumItems).map_err(e)?;
        let mut out = vec![];
        loop {
            let mut got = [None];
            let mut n = 0u32;
            if list.Next(&mut got, Some(&mut n)).is_err() || n == 0 { break; }
            let Some(it) = got[0].take() else { break };
            let Ok(it2) = it.cast::<IShellItem2>() else { continue };
            let pw = |p: windows::core::PWSTR| { let s = p.to_string().unwrap_or_default(); windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as _)); s };
            let id = it.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING).map(pw).unwrap_or_default();
            let display = it.GetDisplayName(SIGDN_NORMALDISPLAY).map(pw).unwrap_or_default();
            let name = it2.GetString(&PKEY_FILE_NAME).map(pw).ok().filter(|s| !s.is_empty()).unwrap_or(display);
            let from = it2.GetString(&PKEY_DELETED_FROM).map(pw).unwrap_or_default();
            let deleted_at = it2.GetFileTime(&PKEY_DATE_DELETED).map(|f| filetime_ms(((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64)).unwrap_or(0);
            let size = it2.GetUInt64(&PKEY_SIZE).unwrap_or(0);
            // A folder, not a .zip (zips are browsable folders that are also files: STREAM).
            let folder = it.GetAttributes(SFGAO_FOLDER | SFGAO_STREAM).map(|a| a.0 & SFGAO_FOLDER.0 != 0 && a.0 & SFGAO_STREAM.0 == 0).unwrap_or(false);
            let path = if from.is_empty() { name.clone() } else { format!("{}\\{}", from.trim_end_matches('\\'), name) };
            out.push((BinItem { id, name, path, from, deleted_at, size, folder }, it2));
        }
        out.sort_by(|a, b| b.0.deleted_at.cmp(&a.0.deleted_at));
        Ok(out)
    }
}

pub fn recycle_items() -> Res<Vec<BinItem>> { on_sta(|| bin_items().map(|v| v.into_iter().map(|(i, _)| i).collect())) }

/// Restore through the bin's own context menu ("undelete" = Restore), item by item. For an
/// original path that was deleted more than once, the newest copy comes back.
pub fn recycle_restore(keys: &[String]) -> Res<u32> {
    let keys: Vec<String> = keys.iter().map(|k| k.to_lowercase()).collect();
    on_sta(move || {
        let items = bin_items()?;
        let mut done = 0u32;
        let mut last_err = None;
        let mut used = std::collections::HashSet::new();
        for k in &keys {
            // Newest first, so the first match is the most recent deletion.
            let Some((i, (_, it))) = items.iter().enumerate()
                .find(|(n, (b, _))| !used.contains(n) && (b.id.to_lowercase() == *k || b.path.to_lowercase() == *k)) else { continue };
            used.insert(i);
            match undelete(it) { Ok(()) => done += 1, Err(e) => last_err = Some(e) }
        }
        match (done, last_err) { (0, Some(e)) => Err(e), _ => Ok(done) }
    })
}

fn undelete(item: &IShellItem2) -> Res<()> {
    use windows::Win32::UI::WindowsAndMessaging::{CreatePopupMenu, DestroyMenu};
    let e = |e: windows::core::Error| e.message().to_string();
    unsafe {
        let cm: IContextMenu = item.BindToHandler(None, &BHID_SFUIObject).map_err(e)?;
        let menu = CreatePopupMenu().map_err(e)?;
        let r = (|| {
            cm.QueryContextMenu(menu, 0, 1, 0x7FFF, CMF_NORMAL).ok().map_err(e)?;
            let verb = c"undelete";
            let info = CMINVOKECOMMANDINFO {
                cbSize: std::mem::size_of::<CMINVOKECOMMANDINFO>() as u32,
                lpVerb: windows::core::PCSTR(verb.as_ptr() as _),
                nShow: 1,
                fMask: 0x0000_0400, // CMIC_MASK_FLAG_NO_UI
                ..Default::default()
            };
            cm.InvokeCommand(&info).map_err(e)
        })();
        let _ = DestroyMenu(menu);
        r
    }
}

/// Watch every fixed drive's `$Recycle.Bin` (with its subfolders): a change, once things
/// have been quiet for 1.5 s, calls `f`. One thread for the life of the app.
pub fn watch_recycle(f: Box<dyn Fn() + Send>) {
    use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows::Win32::Storage::FileSystem::{FindFirstChangeNotificationW, FindNextChangeNotification, GetDriveTypeW, GetLogicalDrives,
        FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_SIZE};
    use windows::Win32::System::Threading::WaitForMultipleObjects;
    std::thread::spawn(move || {
        let mut handles: Vec<HANDLE> = vec![];
        let mask = unsafe { GetLogicalDrives() };
        for i in 0..26u32 {
            if mask & (1 << i) == 0 { continue; }
            let root = format!("{}:\\", (b'A' + i as u8) as char);
            if unsafe { GetDriveTypeW(PCWSTR(wide(&root).as_ptr())) } != 3 { continue; } // DRIVE_FIXED
            let dir = wide(&format!("{root}$Recycle.Bin"));
            let flags = FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_SIZE;
            if let Ok(h) = unsafe { FindFirstChangeNotificationW(PCWSTR(dir.as_ptr()), true, flags) } { handles.push(h); }
        }
        if handles.is_empty() { return; }
        let mut pending = false;
        loop {
            let wait = if pending { 1500 } else { u32::MAX };
            let r = unsafe { WaitForMultipleObjects(&handles, false, wait) };
            if r == WAIT_TIMEOUT { pending = false; f(); continue; }
            let i = (r.0 - WAIT_OBJECT_0.0) as usize;
            if i >= handles.len() { std::thread::sleep(std::time::Duration::from_secs(5)); continue; }
            pending = true;
            unsafe { let _ = FindNextChangeNotification(handles[i]); }
        }
    });
}

#[cfg(test)]
mod bin_tests {
    #[test]
    fn filetime_converts_to_unix_ms() {
        assert_eq!(super::filetime_ms(116_444_736_000_000_000), 0);
        assert_eq!(super::filetime_ms(116_444_736_000_000_000 + 10_000 * 1_500), 1_500);
    }
}

#[cfg(test)]
mod bin_live {
    /// Read-only: lists the real bin. `cargo test bin_live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn lists_the_real_bin() {
        let t = std::time::Instant::now();
        let v = super::recycle_items().unwrap();
        println!("{} items in {:?}", v.len(), t.elapsed());
        for i in v.iter().take(4) { println!("{:?}", i); }
    }

    /// Recycles a throwaway file, finds it, restores it, removes it.
    #[test]
    #[ignore]
    fn round_trip() {
        let dir = std::env::temp_dir().join("glassbar-bin-test");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join(format!("probe-{}.txt", std::process::id()));
        std::fs::write(&f, "x").unwrap();
        let p = f.to_string_lossy().to_string();
        super::recycle(&p).unwrap();
        assert!(!f.exists());
        let v = super::recycle_items().unwrap();
        let hit = v.iter().find(|i| i.path.eq_ignore_ascii_case(&p)).cloned();
        println!("found: {:?}", hit);
        let n = super::recycle_restore(&[p.clone()]).unwrap();
        println!("restored {n}; exists again: {}", f.exists());
        assert_eq!(n, 1);
        assert!(f.exists());
        let _ = std::fs::remove_file(&f);
    }
}
