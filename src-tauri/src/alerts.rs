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
pub struct Alert { pub id: i64, pub source: String, pub title: String, pub body: String, pub at: i64, pub missed: bool, pub read: bool, pub data: serde_json::Value }

/// What one notification does besides its list entry (which it always gets).
#[derive(Debug, PartialEq, Eq)]
pub struct Delivery { pub banner: bool, pub toast: bool, pub sound: bool }

/// Missed alerts and Do Not Disturb: list entry only. Banners on: a banner (none for a
/// muted widget), with the sound if enabled. Banners off: a Windows toast and the sound.
pub fn delivery(dnd: bool, missed: bool, b: &crate::config::Banners, source: &str) -> Delivery {
    if missed || dnd { return Delivery { banner: false, toast: false, sound: false }; }
    if !b.show { return Delivery { banner: false, toast: true, sound: true }; }
    let banner = !b.muted.iter().any(|m| m == source);
    Delivery { banner, toast: false, sound: banner && b.sound }
}

/// The banner (and the click-to-jump) payload: which widget sent it, and its details.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note { pub id: i64, pub source: String, pub title: String, pub body: String, pub at: i64, pub data: serde_json::Value }

/// Record an alert (with `data`, e.g. which timer); then banner/toast and `sound` as
/// `delivery` says.
pub fn notify_full(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool, sound: serde_json::Value, data: serde_json::Value) {
    let at = now_ms();
    let id = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let _ = c.execute(
            "INSERT INTO alerts(source, title, body, at, missed, data) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![source, title, body, at, missed, data.to_string()],
        );
        let id = c.last_insert_rowid();
        // Keep history bounded.
        let _ = c.execute("DELETE FROM alerts WHERE id NOT IN (SELECT id FROM alerts ORDER BY id DESC LIMIT 200)", []);
        id
    };
    let _ = app.emit("alerts:changed", ());
    let (dnd, banners) = { let st = app.state::<AppState>(); let c = st.config.lock().unwrap(); (c.dnd, c.banners.clone()) };
    let d = delivery(dnd, missed, &banners, source);
    if d.toast { let _ = app.notification().builder().title(title).body(body).show(); }
    if d.banner {
        crate::bar::banner_push(app, Note { id, source: source.into(), title: title.into(), body: body.into(), at, data });
    }
    if d.sound { let _ = app.emit_to("bar", "sound:play", sound); }
}

/// Record an alert; banner (or toast) and play `sound` ("chime" or "azan") unless silenced.
pub fn notify_with(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool, sound: serde_json::Value) {
    notify_full(app, source, title, body, missed, sound, serde_json::Value::Null);
}

pub fn notify(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool) {
    notify_with(app, source, title, body, missed, serde_json::json!({ "kind": "chime" }));
}

#[tauri::command]
pub fn alerts_list(db: State<Db>) -> Vec<Alert> {
    let c = db.0.lock().unwrap();
    let mut q = c.prepare("SELECT id, source, title, body, at, missed, read, data FROM alerts ORDER BY id DESC LIMIT 200").unwrap();
    q.query_map([], |r| Ok(Alert {
        id: r.get(0)?, source: r.get(1)?, title: r.get(2)?, body: r.get(3)?, at: r.get(4)?, missed: r.get(5)?, read: r.get(6)?,
        data: r.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::Value::Null),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Banners;

    #[test]
    fn dnd_skips_banners_but_keeps_list_entries() {
        // (The list entry is always written; delivery only decides banner/toast/sound.)
        let b = Banners::default();
        assert_eq!(delivery(true, false, &b, "timer"), Delivery { banner: false, toast: false, sound: false });
        assert_eq!(delivery(false, true, &b, "timer"), Delivery { banner: false, toast: false, sound: false }, "missed while away");
    }

    #[test]
    fn one_notification_one_banner_no_duplicate_toast() {
        let b = Banners::default();
        assert_eq!(delivery(false, false, &b, "timer"), Delivery { banner: true, toast: false, sound: true });
        let off = Banners { show: false, ..Banners::default() };
        assert_eq!(delivery(false, false, &off, "timer"), Delivery { banner: false, toast: true, sound: true });
        let quiet = Banners { muted: vec!["ipu".into()], sound: false, ..Banners::default() };
        assert_eq!(delivery(false, false, &quiet, "ipu"), Delivery { banner: false, toast: false, sound: false }, "muted widget: list only");
        assert_eq!(delivery(false, false, &quiet, "timer"), Delivery { banner: true, toast: false, sound: false });
    }
}
