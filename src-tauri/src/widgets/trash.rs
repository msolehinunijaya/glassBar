//! Recycle Bin widget: the newest items in the bin with Restore, files dropped on the dock
//! moved to the bin (never deleted outright), and a watcher that keeps the dock's
//! empty/full state current without polling. Size and count are `downloads::recycle_info`.
use crate::platform::{files, BinItem};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrashResult { pub moved: u32, pub failed: Vec<(String, String)> }

/// Pure: how a batch of recycle attempts went.
pub fn summarize(results: Vec<Result<(), String>>, paths: &[String]) -> TrashResult {
    let mut moved = 0;
    let mut failed = vec![];
    for (r, p) in results.into_iter().zip(paths) {
        match r { Ok(()) => moved += 1, Err(e) => failed.push((p.clone(), e)) }
    }
    TrashResult { moved, failed }
}

fn changed(app: &AppHandle) {
    super::downloads::recycle_stale();
    let _ = app.emit("recycle:changed", ());
}

/// The newest `limit` items in the bin (Windows counts every item: a big bin takes a moment).
#[tauri::command]
pub async fn trash_items(limit: u32) -> Result<Vec<BinItem>, String> {
    let mut v = super::blocking(|| files().recycle_items()).await??;
    v.truncate(limit as usize);
    Ok(v)
}

/// Put items back (ids from `trash_items`, or the original paths of files just dropped).
#[tauri::command]
pub async fn trash_restore(app: AppHandle, keys: Vec<String>) -> Result<u32, String> {
    let n = super::blocking(move || files().recycle_restore(&keys)).await??;
    changed(&app);
    Ok(n)
}

/// Files dropped on the bin: each is moved to the Recycle Bin. One that fails doesn't stop
/// the others.
#[tauri::command]
pub async fn trash_recycle(app: AppHandle, paths: Vec<String>) -> Result<TrashResult, String> {
    let p = paths.clone();
    let results = super::blocking(move || p.iter().map(|x| {
        if !std::path::Path::new(x).exists() { return Err("it's no longer there".to_string()); }
        files().recycle(x)
    }).collect::<Vec<_>>()).await?;
    changed(&app);
    Ok(summarize(results, &paths))
}

/// Empty with Windows' own confirmation (the dock's right-click menu).
#[tauri::command]
pub async fn trash_empty_confirm(app: AppHandle) -> Result<(), String> {
    super::blocking(|| files().recycle_empty_ask()).await??;
    changed(&app);
    Ok(())
}

/// Watch the bins for the life of the app: one thread asleep in Windows until a bin changes
/// (cheap), so a widget added later is current at once. Only panels that listen reload.
pub fn start(app: AppHandle) {
    let a = app.clone();
    files().watch_recycle(Box::new(move || changed(&a)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recycle_result_counts_failures() {
        let paths: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let r = summarize(vec![Ok(()), Err("in use".into()), Ok(())], &paths);
        assert_eq!(r.moved, 2);
        assert_eq!(r.failed, vec![("b".to_string(), "in use".to_string())]);
    }
}
