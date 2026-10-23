//! One thread that sleeps until the next due item (to-do, timer, Pomodoro phase,
//! prayer, daily housekeeping), fires each exactly once, and goes back to sleep.
//! `poke()` wakes it early when data changes; a 60 s cap catches PC sleep/wake.
use crate::db::{now_ms, Db};
use std::collections::HashSet;
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Alerts later than this when we notice them are recorded as missed (no toast/sound).
pub const MISSED_AFTER_MS: i64 = 15 * 60 * 1000;

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
        .filter(|d| d.at <= now && !fired.contains(&d.key))
        .map(|d| (d.clone(), now - d.at > MISSED_AFTER_MS))
        .collect();
    fire.sort_by_key(|(d, _)| d.at);
    let next = items.iter().filter(|d| d.at > now).map(|d| d.at).min();
    (fire, next)
}

static WAKE: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

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
    v.push(Due {
        key: format!("house:{}", chrono::Local::now().format("%Y-%m-%d")),
        at: 0, source: "house", title: String::new(), body: String::new(), sound: None,
    });
    v
}

fn handle(app: &AppHandle, d: &Due, missed: bool) {
    match d.source {
        "house" => housekeeping(app),
        "pomodoro" => {
            crate::widgets::focus::on_pomo_due(app);
            crate::alerts::notify(app, d.source, &d.title, &d.body, missed);
        }
        _ => match &d.sound {
            Some(s) => crate::alerts::notify_with(app, d.source, &d.title, &d.body, missed, crate::widgets::solat::sound_payload(s)),
            None => crate::alerts::notify(app, d.source, &d.title, &d.body, missed),
        },
    }
}

/// Daily cleanup. Later tasks add note expiry and screen-time retention.
fn housekeeping(app: &AppHandle) {
    {
        let c = app.state::<Db>();
        let c = c.0.lock().unwrap();
        let _ = c.execute("DELETE FROM fired WHERE at < ?1", [now_ms() - 30 * 86_400_000]);
    }
    crate::widgets::notes::housekeeping(app);
    crate::widgets::solat::refresh(app);
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        let items = collect(&app);
        let fired: HashSet<String> = {
            let db = app.state::<Db>();
            let c = db.0.lock().unwrap();
            let mut q = c.prepare("SELECT key FROM fired").unwrap();
            q.query_map([], |r| r.get(0)).map(|r| r.filter_map(Result::ok).collect()).unwrap_or_default()
        };
        let now = now_ms();
        let (fire, next) = due_actions(now, &items, &fired);
        for (d, missed) in &fire {
            let _ = app.state::<Db>().0.lock().unwrap()
                .execute("INSERT OR IGNORE INTO fired(key, at) VALUES(?1, ?2)", rusqlite::params![d.key, now]);
            handle(&app, d, *missed);
        }
        if !fire.is_empty() { continue; } // firing may create new due items (next Pomodoro phase)
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
    fn empty_means_no_wake() {
        assert_eq!(due_actions(10, &[], &HashSet::new()).1, None);
    }
}
