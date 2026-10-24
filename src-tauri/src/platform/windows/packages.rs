//! Store (MSIX) apps can't be started from their .exe inside C:\Program Files\WindowsApps
//! (Windows denies it), and that folder name changes with every update. Map such a path to
//! the app's AppUserModelID through Windows' package manager, which survives updates.
use windows::core::HSTRING;
use windows::Management::Deployment::PackageManager;

/// "…\WindowsApps\5319275A.WhatsAppDesktop_2.2640.100.0_x64__cv1g1gvanyjgm\WhatsApp.Root.exe"
/// -> ("5319275A.WhatsAppDesktop_cv1g1gvanyjgm", "WhatsApp.Root.exe"). Pure, so it's tested.
pub fn family_of(path: &str) -> Option<(String, String)> {
    let low = path.to_lowercase();
    let at = low.find("\\windowsapps\\")? + "\\windowsapps\\".len();
    let rest = &path[at..];
    let (folder, inner) = rest.split_once('\\')?;
    let (left, publisher) = folder.rsplit_once("__")?;
    let name = left.split('_').next()?;
    (!name.is_empty() && !publisher.is_empty()).then(|| (format!("{name}_{publisher}"), inner.to_string()))
}

pub fn aumid_for_path(path: &str) -> Option<String> {
    let (family, inner) = family_of(path)?;
    super::com_init();
    let pm = PackageManager::new().ok()?;
    let pkgs = pm.FindPackagesByUserSecurityIdPackageFamilyName(&HSTRING::new(), &HSTRING::from(&family)).ok()?;
    for pkg in pkgs {
        let Ok(entries) = pkg.GetAppListEntriesAsync().and_then(|o| o.join()) else { continue };
        let ids: Vec<String> = entries.into_iter().filter_map(|e| e.AppUserModelId().ok().map(|s| s.to_string())).collect();
        // Several apps in one package: prefer the entry whose manifest Executable is our .exe.
        let exe = inner.to_lowercase();
        let manifest = pkg.InstalledPath().ok()
            .and_then(|p| std::fs::read_to_string(format!("{p}\\AppxManifest.xml")).ok());
        if let Some(m) = manifest {
            for id in &ids {
                let app = id.rsplit('!').next().unwrap_or("");
                let tag = format!("Id=\"{app}\"");
                if let Some(i) = m.find(&tag) {
                    let el = &m[i..m[i..].find('>').map(|e| i + e).unwrap_or(m.len())];
                    if el.to_lowercase().contains(&format!("executable=\"{}\"", exe.replace('\\', "\\"))) { return Some(id.clone()); }
                }
            }
        }
        if let Some(first) = ids.into_iter().next() { return Some(first); }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn family_from_versioned_path() {
        assert_eq!(super::family_of(r"C:\Program Files\WindowsApps\5319275A.WhatsAppDesktop_2.2640.100.0_x64__cv1g1gvanyjgm\WhatsApp.Root.exe"),
            Some(("5319275A.WhatsAppDesktop_cv1g1gvanyjgm".into(), "WhatsApp.Root.exe".into())));
        assert_eq!(super::family_of(r"C:\Program Files\WindowsApps\Claude_3.1623.3.0_x64__pzs8sxrjxfjjc\app\claude.exe"),
            Some(("Claude_pzs8sxrjxfjjc".into(), r"app\claude.exe".into())));
        assert_eq!(super::family_of(r"C:\Program Files\Google\Chrome\Application\chrome.exe"), None);
    }
}
