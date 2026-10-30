//! Mirror: the webcam in the popup. The camera itself runs in the page (getUserMedia);
//! here we only let the popup use it and save photos and videos to the user's folders:
//! Pictures\GlassBar Mirror and Videos\GlassBar Mirror.
use std::path::{Path, PathBuf};
use tauri::ipc::{InvokeBody, Request};
use tauri::{AppHandle, Manager};

const FOLDER: &str = "GlassBar Mirror";

/// `mirror-2026-10-30-142233.png`, or `…-2.png`, `…-3.png` when that name is taken
/// (two photos in the same second).
fn free_name(dir: &Path, stamp: &str, ext: &str) -> PathBuf {
    let mut p = dir.join(format!("mirror-{stamp}.{ext}"));
    let mut n = 2;
    while p.exists() { p = dir.join(format!("mirror-{stamp}-{n}.{ext}")); n += 1; }
    p
}

/// Body: the file bytes (raw, not JSON, so a long video doesn't crawl through the IPC).
/// Header `kind`: `photo` (PNG) or `video` (WebM). Returns the saved path.
#[tauri::command]
pub async fn mirror_save(app: AppHandle, request: Request<'_>) -> Result<String, String> {
    let InvokeBody::Raw(bytes) = request.body() else { return Err("expected file bytes".into()) };
    let kind = request.headers().get("kind").and_then(|v| v.to_str().ok()).unwrap_or("");
    let (base, ext) = match kind {
        "photo" => (app.path().picture_dir(), "png"),
        "video" => (app.path().video_dir(), "webm"),
        _ => return Err("unknown kind".into()),
    };
    let dir = base.map_err(|e| e.to_string())?.join(FOLDER);
    let bytes = bytes.clone();
    super::blocking(move || -> Result<String, String> {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = free_name(&dir, &chrono::Local::now().format("%Y-%m-%d-%H%M%S").to_string(), ext);
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().into_owned())
    }).await?
}

/// Let the popup use the webcam without WebView2's own "allow camera?" prompt: that prompt
/// takes focus, and the popup hides when it loses focus. The popup only ever shows our own
/// page, and only the camera is allowed (no microphone, location, …).
#[cfg(windows)]
pub fn allow_camera(win: &tauri::WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_STATE_ALLOW,
    };
    use webview2_com::PermissionRequestedEventHandler;
    let _ = win.with_webview(|wv| unsafe {
        let Ok(core) = wv.controller().CoreWebView2() else { return };
        let handler = PermissionRequestedEventHandler::create(Box::new(|_, args| {
            if let Some(a) = args {
                let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                a.PermissionKind(&mut kind)?;
                if kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA { a.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?; }
            }
            Ok(())
        }));
        let mut token = 0i64;
        let _ = core.add_PermissionRequested(&handler, &mut token);
    });
}

#[cfg(test)]
mod tests {
    use super::free_name;

    #[test]
    fn a_taken_name_gets_a_number() {
        let dir = std::env::temp_dir().join(format!("glassbar-mirror-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = free_name(&dir, "2026-10-30-142233", "png");
        assert!(first.ends_with("mirror-2026-10-30-142233.png"));
        std::fs::write(&first, b"x").unwrap();
        assert!(free_name(&dir, "2026-10-30-142233", "png").ends_with("mirror-2026-10-30-142233-2.png"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
