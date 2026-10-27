//! File Shelf: a few files parked by the dock, to pick up again later.
//!
//! The shelf holds nothing but paths (in the widget's settings, like the App Launcher's pins),
//! so it never moves, copies or deletes a file of yours. Opening, revealing and the icons all
//! reuse the Downloads widget's commands, which are generic over any path.
//!
//! What lives here is the bit that doesn't: handing a drag over to Windows so a shelved file
//! can land in another app, and a file picker for adding one without a drag.
use super::blocking;
use crate::platform::files;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

/// The file's own Windows icon as the thing the pointer carries. `Image::File` would only work
/// for a file that is itself a picture, and a shelf holds anything.
fn drag_image(path: &str) -> drag::Image {
    use base64::Engine as _;
    let png = files()
        .file_icon(path)
        .and_then(|url| url.split_once(',').map(|(_, b64)| b64.to_string()))
        .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok());
    match png {
        Some(bytes) => drag::Image::Raw(bytes),
        None => drag::Image::File(PathBuf::from(path)),
    }
}

/// Hand a drag to Windows so the files can be dropped into Explorer, a chat or an email.
/// Must run on the window's UI thread while the mouse button is still down.
///
/// When the drop actually happens, `shelf:dropped` goes back to the bar with the paths, and
/// the widget takes them off the shelf -- a cancelled drag leaves the shelf alone, so a stray
/// drag can't lose anything. The files themselves are copied by whatever received them;
/// GlassBar never moves or deletes them.
#[tauri::command]
pub fn shelf_drag(app: AppHandle, window: WebviewWindow, paths: Vec<String>) -> Result<(), String> {
    let Some(first) = paths.first().cloned() else { return Err("nothing to drag".into()) };
    let items: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let back = app.clone();
    let sent = paths.clone();
    app.run_on_main_thread(move || {
        let _ = drag::start_drag(
            &window,
            drag::DragItem::Files(items),
            drag_image(&first),
            move |result, _| {
                if matches!(result, drag::DragResult::Dropped) {
                    let _ = back.emit_to("bar", "shelf:dropped", &sent);
                }
            },
            Default::default(),
        );
    })
    .map_err(|e| e.to_string())
}

/// Add files without a drag. Empty = cancelled.
#[tauri::command]
pub async fn shelf_pick(app: AppHandle) -> Result<Vec<String>, String> {
    blocking(move || {
        let Some(files) = app.dialog().file().set_title("Add to the shelf").blocking_pick_files() else { return vec![] };
        files
            .into_iter()
            .filter_map(|f| f.into_path().ok())
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    })
    .await
}
