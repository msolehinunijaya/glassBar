//! Files: the Downloads known folder, real file-type icons, show in folder / open with,
//! move to Recycle Bin (never a permanent delete), drag a file out, and the Recycle Bin
//! size / empty.
use super::super::{RecycleDrive, RecycleInfo, Res};
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
