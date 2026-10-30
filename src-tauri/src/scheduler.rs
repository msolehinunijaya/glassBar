//! One thread that sleeps until the next due item (to-do, timer, Pomodoro phase,
//! prayer, daily housekeeping), fires each exactly once, and goes back to sleep.
//! `poke()` wakes it early when data changes; a 60 s cap catches PC sleep/wake.
use crate::db::{now_ms, Db};
use std::collections::HashSet;
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Alerts later than this when we notice them are recorded as missed (no toast/sound).
pub const MISSED_AFTER_MS: i64 = 15 * 60 * 1000;
/// Items older than this are ignored outright (the `fired` table keeps 30 days,
/// so without this an old overdue to-do would re-appear as "missed" monthly).
const IGNORE_AFTER_MS: i64 = 25 * 86_400_000;

#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    /// Unique per occurrence; stored in `fired` so it never fires twice.
    pub key: String,
    pub at: i64,
    pub source: &'static str,
    pub title: String,
    pub body: String,
    /// Sound payload for the bar (None = chime).
    pub sound: Option<serde_json::Value>,
}

/// Pure: which items fire now (with missed flag), and when to wake next.
pub fn due_actions(now: i64, items: &[Due], fired: &HashSet<String>) -> (Vec<(Due, bool)>, Option<i64>) {
    let mut fire: Vec<(Due, bool)> = items.iter()
        .filter(|d| d.at <= now && d.at > now - IGNORE_AFTER_MS && !fired.contains(&d.key))
        .map(|d| (d.clone(), now - d.at > MISSED_AFTER_MS))
        .collect();
    fire.sort_by_key(|(d, _)| d.at);
    let next = items.iter().filter(|d| d.at > now).map(|d| d.at).min();
    (fire, next)
}

static WAKE: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

/// Record `key` as fired. True only if this call inserted it: a duplicate or a
/// failed write (DB locked, disk full) must not fire, or it would repeat forever.
pub fn mark_fired(c: &rusqlite::Connection, key: &str, at: i64) -> bool {
    matches!(c.execute("INSERT OR IGNORE INTO fired(key, at) VALUES(?1, ?2)", rusqlite::params![key, at]), Ok(1))
}

/// Re-evaluate now (call after any change to due data).
pub fn poke() {
    *WAKE.0.lock().unwrap() = true;
    WAKE.1.notify_one();
}

/// Every source of due items. Later tasks add their own.
fn collect(app: &AppHandle) -> Vec<Due> {
    let mut v = crate::widgets::todo::due_items(app);
    v.extend(crate::widgets::focus::due_items(app));
    v.extend(crate::widgets::solat::due_items(app));
    v.extend(crate::widgets::water::due_items(app));
    v.extend(crate::widgets::countdown::due_items(app));
    // Due "now" so it sorts after anything else that is due (alerts go first).
    v.push(Due {
        key: format!("house:{}", chrono::Local::now().format("%Y-%m-%d")),
        at: now_ms(), source: "house", title: String::new(), body: String::new(), sound: None,
    });
    v
}

fn handle(app: &AppHandle, d: &Due, missed: bool) {
    match d.source {
        "house" => housekeeping(app),
        // A Dock alert only: no banner, no sound, nothing in the Notification Center.
        "water" => crate::widgets::water::remind(app, missed),
        // Timer, Pomodoro and prayer time are told by the dock itself (a full-dock alert with
        // its own sound), not by a banner: alerts::DOCK_SOURCES. Still in the list.
        "timer" => {
            let id = crate::widgets::focus::timer_id(&d.key);
            if let Some(id) = id { crate::widgets::focus::on_timer_due(app, id); }
            let dnd = app.state::<crate::AppState>().config.lock().unwrap().dnd;
            crate::alerts::notify_full(app, d.source, &d.title, &d.body, missed, serde_json::Value::Null, serde_json::json!({ "timerId": id }));
            if !missed && !dnd { let _ = app.emit_to("bar", "sound:play", serde_json::json!({ "kind": "chime" })); }
            if let Some(a) = crate::widgets::focus::timer_alert(id, &d.body, missed, dnd) { let _ = app.emit_to("bar", "dock:alert", a); }
        }
        "pomodoro" => {
            let (sound, data) = crate::widgets::focus::on_pomo_due(app);
            let dnd = app.state::<crate::AppState>().config.lock().unwrap().dnd;
            let auto = data["auto"].as_bool().unwrap_or(false);
            crate::alerts::notify_full(app, d.source, &d.title, &d.body, missed, serde_json::Value::Null, data);
            if !missed && !dnd && !sound.is_null() { let _ = app.emit_to("bar", "sound:play", sound); }
            if let Some(a) = crate::widgets::focus::pomo_alert(&d.title, &d.body, missed, dnd, auto) { let _ = app.emit_to("bar", "dock:alert", a); }
        }
        "solat" => {
            // The adzan is the prayer widget's own sound setting, not a banner chime: it plays
            // whether or not prayer banners are muted or their sound is off. Only Do Not Disturb
            // (and a prayer missed while GlassBar was off) keeps it quiet.
            let dnd = app.state::<crate::AppState>().config.lock().unwrap().dnd;
            crate::alerts::notify_full(app, d.source, &d.title, &d.body, missed, serde_json::Value::Null, serde_json::json!({ "key": d.key }));
            if !missed && !dnd {
                let sound = d.sound.as_ref().map(crate::widgets::solat::sound_payload).unwrap_or(serde_json::json!({ "kind": "chime" }));
                let _ = app.emit_to("bar", "sound:play", sound);
            }
            // After the sound, so the dock already knows the adzan is playing (Stop button).
            if let Some(a) = crate::widgets::solat::prayer_alert(&d.key, d.at, missed, dnd) { let _ = app.emit_to("bar", "dock:alert", a); }
        }
        _ => {
            let sound = d.sound.as_ref().map(crate::widgets::solat::sound_payload).unwrap_or(serde_json::json!({ "kind": "chime" }));
            // The occurrence key ("todo:<id>:<at>", "solat:<date>:<zone>:<name>") lets the
            // banner's buttons act on exactly this item.
            crate::alerts::notify_full(app, d.source, &d.title, &d.body, missed, sound, serde_json::json!({ "key": d.key }));
        }
    }
}

/// Daily cleanup. Later tasks add note expiry and screen-time retention.
fn housekeeping(app: &AppHandle) {
    {
        let c = app.state::<Db>();
        let c = c.lock();
        let _ = c.execute("DELETE FROM fired WHERE at < ?1", [now_ms() - 30 * 86_400_000]);
    }
    crate::widgets::notes::housekeeping(app);
    crate::widgets::solat::refresh(app);
    crate::widgets::screentime::housekeeping(app);
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        let items = collect(&app);
        let fired: HashSet<String> = {
            let db = app.state::<Db>();
            let c = db.lock();
            c.prepare("SELECT key FROM fired")
                .and_then(|mut q| q.query_map([], |r| r.get(0)).map(|r| r.filter_map(Result::ok).collect()))
                .unwrap_or_default()
        };
        let now = now_ms();
        let (fire, next) = due_actions(now, &items, &fired);
        let mut handled = false;
        for (d, missed) in &fire {
            let first = mark_fired(&app.state::<Db>().lock(), &d.key, now);
            if first {
                handle(&app, d, *missed);
                handled = true;
            }
        }
        // Firing may create new due items (next Pomodoro phase): look again right away.
        // If nothing could be recorded (DB locked), fall through and wait instead of spinning.
        if handled { continue; }
        let wait = next.map(|n| (n - now).clamp(0, 60_000)).unwrap_or(60_000) as u64;
        let (lock, cv) = &WAKE;
        let mut woke = lock.lock().unwrap();
        if !*woke {
            woke = cv.wait_timeout(woke, Duration::from_millis(wait)).unwrap().0;
        }
        *woke = false;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    const MIN: i64 = 60_000;

    fn due(key: &str, at: i64) -> Due {
        Due { key: key.into(), at, source: "todo", title: key.into(), body: String::new(), sound: None }
    }

    #[test]
    fn fires_due_items_once() {
        let items = [due("a", 100)];
        let (fire, _) = due_actions(200, &items, &HashSet::new());
        assert_eq!(fire.len(), 1);
        let fired: HashSet<String> = ["a".to_string()].into();
        let (fire, _) = due_actions(200, &items, &fired);
        assert!(fire.is_empty());
    }

    #[test]
    fn late_over_15_min_is_missed() {
        let now = 100 * MIN;
        let (fire, _) = due_actions(now, &[due("late", now - 16 * MIN), due("ok", now - 14 * MIN)], &HashSet::new());
        let missed: Vec<_> = fire.iter().map(|(d, m)| (d.key.as_str(), *m)).collect();
        assert_eq!(missed, vec![("late", true), ("ok", false)]);
    }

    #[test]
    fn next_wake_is_earliest_future() {
        let (fire, next) = due_actions(10, &[due("b", 50), due("a", 30), due("past", 5)], &HashSet::new());
        assert_eq!(fire.len(), 1);
        assert_eq!(next, Some(30));
    }

    #[test]
    fn mark_fired_reports_duplicates_and_errors() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrate(&c).unwrap();
        assert!(mark_fired(&c, "k", 1));
        assert!(!mark_fired(&c, "k", 2), "second insert of the same key must not fire again");
        c.execute_batch("DROP TABLE fired").unwrap();
        assert!(!mark_fired(&c, "x", 3), "a failed write must not fire (it would repeat forever)");
    }

    #[test]
    fn items_older_than_25_days_are_ignored() {
        let now = 100 * 86_400_000;
        let (fire, _) = due_actions(now, &[due("ancient", now - 26 * 86_400_000), due("old", now - 2 * 86_400_000)], &HashSet::new());
        assert_eq!(fire.iter().map(|(d, _)| d.key.as_str()).collect::<Vec<_>>(), ["old"]);
    }

    #[test]
    fn empty_means_no_wake() {
        assert_eq!(due_actions(10, &[], &HashSet::new()).1, None);
    }
}
