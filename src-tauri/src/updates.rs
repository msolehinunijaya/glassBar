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
