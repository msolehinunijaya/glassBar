//! GlassBar's own alert center: every alert is recorded; toast + sound unless
//! Do Not Disturb is on or the alert was missed while the app was closed/asleep.
use crate::db::{now_ms, Db};
use crate::AppState;
use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Alert { pub id: i64, pub source: String, pub title: String, pub body: String, pub at: i64, pub missed: bool, pub read: bool }

/// Record an alert; show a toast and play `sound` ("chime" or "azan") unless silenced.
pub fn notify_with(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool, sound: serde_json::Value) {
    {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let _ = c.execute(
            "INSERT INTO alerts(source, title, body, at, missed) VALUES(?1, ?2, ?3, ?4, ?5)",
            params![source, title, body, now_ms(), missed],
        );
        // Keep history bounded.
        let _ = c.execute("DELETE FROM alerts WHERE id NOT IN (SELECT id FROM alerts ORDER BY id DESC LIMIT 200)", []);
    }
    let _ = app.emit("alerts:changed", ());
    let dnd = app.state::<AppState>().config.lock().unwrap().dnd;
    if missed || dnd { return; }
    let _ = app.notification().builder().title(title).body(body).show();
    let _ = app.emit_to("bar", "sound:play", sound);
}

pub fn notify(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool) {
    notify_with(app, source, title, body, missed, serde_json::json!({ "kind": "chime" }));
}

#[tauri::command]
pub fn alerts_list(db: State<Db>) -> Vec<Alert> {
    let c = db.0.lock().unwrap();
    let mut q = c.prepare("SELECT id, source, title, body, at, missed, read FROM alerts ORDER BY id DESC LIMIT 200").unwrap();
    q.query_map([], |r| Ok(Alert {
        id: r.get(0)?, source: r.get(1)?, title: r.get(2)?, body: r.get(3)?, at: r.get(4)?, missed: r.get(5)?, read: r.get(6)?,
    }))
    .map(|rows| rows.filter_map(Result::ok).collect())
    .unwrap_or_default()
}

fn exec(app: &AppHandle, sql: &str, p: impl rusqlite::Params) {
    let _ = app.state::<Db>().0.lock().unwrap().execute(sql, p);
    let _ = app.emit("alerts:changed", ());
}

#[tauri::command]
pub fn alerts_mark_read(app: AppHandle) { exec(&app, "UPDATE alerts SET read = 1 WHERE read = 0", []) }
#[tauri::command]
pub fn alerts_dismiss(app: AppHandle, id: i64) { exec(&app, "DELETE FROM alerts WHERE id = ?1", [id]) }
#[tauri::command]
pub fn alerts_clear(app: AppHandle) { exec(&app, "DELETE FROM alerts", []) }
