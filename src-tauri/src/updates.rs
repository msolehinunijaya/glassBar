//! Auto-update from GitHub Releases. Checks 10 s after start and every 24 h;
//! a new version raises one alert. Installing happens from Settings → Updates.
use crate::db::{now_ms, Db};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

async fn latest(app: &AppHandle) -> Result<Option<String>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    Ok(updater.check().await.map_err(|e| e.to_string())?.map(|u| u.version))
}

/// Alert once per version.
fn announce(app: &AppHandle, version: &str) {
    let first = app.state::<Db>().lock()
        .execute("INSERT OR IGNORE INTO fired(key, at) VALUES(?1, ?2)", rusqlite::params![format!("update:{version}"), now_ms()])
        .map(|n| n > 0).unwrap_or(false);
    if first {
        crate::alerts::notify(app, "update", "Update available",
            &format!("GlassBar {version} is ready. Open Settings → Updates to install."), false);
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(10));
        loop {
            if let Ok(Some(v)) = tauri::async_runtime::block_on(latest(&app)) { announce(&app, &v); }
            std::thread::sleep(Duration::from_secs(24 * 3600));
        }
    });
}

/// Tray "Check for updates…": always answers, so the user sees a result.
pub fn check_now(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match latest(&app).await {
            Ok(Some(v)) => {
                let _ = app.state::<Db>().lock().execute("DELETE FROM fired WHERE key = ?1", [format!("update:{v}")]);
                announce(&app, &v);
            }
            Ok(None) => crate::alerts::notify(&app, "update", "GlassBar is up to date",
                                              &format!("You have version {}.", app.package_info().version), false),
            Err(e) => crate::alerts::notify(&app, "update", "Couldn't check for updates", &e, false),
        }
    });
}

// ---- What's new: once per version, after an update ----

/// The version whose "What's new" was last shown ("" on a fresh install).
const SEEN_KEY: &str = "whatsnew:seen";

/// Pure: the version to show "What's new" for. `seen` = the last version shown (None = never
/// recorded); `had_config` = GlassBar was used before this start (a config file existed). A fresh
/// install shows nothing; someone updating from a version before this feature sees it once.
pub fn whats_new_for(current: &str, seen: Option<&str>, had_config: bool) -> Option<String> {
    match seen {
        Some(s) if s == current => None,
        Some(_) => Some(current.to_string()),
        None if had_config => Some(current.to_string()),
        None => None,
    }
}

/// Did a config file exist when GlassBar started? Set once in main() before loading it.
pub static HAD_CONFIG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The version to show "What's new" for now, if any (the dock asks once it is up). A fresh
/// install is recorded as seen at once.
#[tauri::command]
pub fn whats_new_pending(app: AppHandle) -> Option<String> {
    let current = app.package_info().version.to_string();
    let db = app.state::<Db>();
    let c = db.lock();
    let seen = crate::db::kv_get(&c, SEEN_KEY);
    let show = whats_new_for(&current, seen.as_deref(), HAD_CONFIG.load(std::sync::atomic::Ordering::Relaxed));
    if show.is_none() && seen.as_deref() != Some(current.as_str()) { crate::db::kv_set(&c, SEEN_KEY, &current); }
    show
}

/// The card was shown: don't show it again for this version.
#[tauri::command]
pub fn whats_new_seen(app: AppHandle) {
    let current = app.package_info().version.to_string();
    let db = app.state::<Db>();
    crate::db::kv_set(&db.lock(), SEEN_KEY, &current);
}

#[cfg(test)]
mod whats_new_tests {
    use super::whats_new_for;
    #[test]
    fn once_per_version_and_never_on_a_fresh_install() {
        assert_eq!(whats_new_for("0.6.8", Some("0.6.7"), true).as_deref(), Some("0.6.8"));
        assert_eq!(whats_new_for("0.6.8", Some("0.6.8"), true), None);
        assert_eq!(whats_new_for("0.6.8", None, true).as_deref(), Some("0.6.8"), "updated from before this feature");
        assert_eq!(whats_new_for("0.6.8", None, false), None, "fresh install");
    }
}
