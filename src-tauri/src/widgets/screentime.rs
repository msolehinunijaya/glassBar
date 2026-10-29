//! Screen Time: seconds per app per day, from the foreground window, sampled
//! every 5 s. Records only while a Screen Time widget is placed; keeps 30 days.
use crate::db::{now_ms, Db};
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const INTERVAL_MS: i64 = 5_000;
const IDLE_MS: u64 = 5 * 60_000;
const KEEP_DAYS: i64 = 30;

/// Seconds to credit for one sample. Idle, locked, or nothing in front = 0;
/// a long gap (PC asleep) counts as at most two intervals.
pub fn credit(gap_ms: i64, idle_ms: u64, exe: Option<&str>, interval_ms: i64) -> i64 {
    let Some(exe) = exe else { return 0 };
    if idle_ms >= IDLE_MS || exe.to_lowercase().ends_with("lockapp.exe") { return 0; }
    gap_ms.clamp(0, 2 * interval_ms) / 1000
}

/// The 7 days ending `today`, with 0 for days without data.
pub fn week(rows: &[(String, i64)], today: chrono::NaiveDate) -> Vec<(String, i64)> {
    (0..7).rev().map(|back| {
        let d = (today - chrono::Duration::days(back)).to_string();
        let s = rows.iter().find(|(day, _)| *day == d).map(|r| r.1).unwrap_or(0);
        (d, s)
    }).collect()
}

fn today() -> chrono::NaiveDate { chrono::Local::now().date_naive() }

fn flush(app: &AppHandle, day: &str, acc: &mut HashMap<String, i64>) {
    if acc.is_empty() { return; }
    let db = app.state::<Db>();
    let c = db.lock();
    for (exe, secs) in acc.drain() {
        let _ = c.execute(
            "INSERT INTO screen_time(day, exe, seconds) VALUES(?1, ?2, ?3)
             ON CONFLICT(day, exe) DO UPDATE SET seconds = seconds + excluded.seconds",
            params![day, exe, secs]);
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut acc: HashMap<String, i64> = HashMap::new();
        let mut day = today().to_string();
        let (mut last, mut last_flush) = (now_ms(), now_ms());
        loop {
            std::thread::sleep(Duration::from_millis(INTERVAL_MS as u64));
            let now = now_ms();
            let gap = now - last;
            last = now;
            if !app.state::<crate::AppState>().config.lock().unwrap().uses("screentime") {
                acc.clear();
                continue;
            }
            let shell = crate::platform::shell();
            let exe = shell.foreground_exe();
            let secs = credit(gap, shell.idle_ms(), exe.as_deref(), INTERVAL_MS);
            let d = today().to_string();
            if d != day { flush(&app, &day, &mut acc); day = d; }
            if secs > 0 { *acc.entry(exe.unwrap()).or_default() += secs; }
            if now - last_flush >= 60_000 { flush(&app, &day, &mut acc); last_flush = now; }
        }
    });
}

pub fn housekeeping(app: &AppHandle) {
    let cutoff = (today() - chrono::Duration::days(KEEP_DAYS)).to_string();
    let _ = app.state::<Db>().lock().execute("DELETE FROM screen_time WHERE day < ?1", [cutoff]);
}

#[derive(Serialize)]
pub struct AppTime { pub exe: String, pub name: String, pub seconds: i64 }

#[derive(Serialize)]
pub struct Today { pub total: i64, pub apps: Vec<AppTime> }

#[tauri::command]
pub fn screentime_today(db: State<Db>) -> Today {
    let c = db.lock();
    let mut q = c.prepare("SELECT exe, seconds FROM screen_time WHERE day = ?1 ORDER BY seconds DESC").unwrap();
    let rows: Vec<(String, i64)> = q.query_map([today().to_string()], |r| Ok((r.get(0)?, r.get(1)?)))
        .map(|r| r.filter_map(Result::ok).collect()).unwrap_or_default();
    let total = rows.iter().map(|r| r.1).sum();
    let apps = rows.into_iter().take(8).map(|(exe, seconds)| {
        let name = std::path::Path::new(&exe).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        AppTime { exe, name, seconds }
    }).collect();
    Today { total, apps }
}

#[tauri::command]
pub fn screentime_week(db: State<Db>) -> Vec<(String, i64)> {
    let c = db.lock();
    let mut q = c.prepare("SELECT day, SUM(seconds) FROM screen_time WHERE day >= ?1 GROUP BY day").unwrap();
    let rows: Vec<(String, i64)> = q.query_map([(today() - chrono::Duration::days(6)).to_string()], |r| Ok((r.get(0)?, r.get(1)?)))
        .map(|r| r.filter_map(Result::ok).collect()).unwrap_or_default();
    week(&rows, today())
}

#[cfg(test)]
mod tests {
    use super::*;
    const I: i64 = 5_000;

    #[test]
    fn idle_gives_zero() { assert_eq!(credit(I, 300_000, Some(r"C:\a\code.exe"), I), 0); }
    #[test]
    fn lockapp_gives_zero() {
        assert_eq!(credit(I, 0, Some(r"C:\Windows\SystemApps\LockApp.exe"), I), 0);
        assert_eq!(credit(I, 0, None, I), 0);
    }
    #[test]
    fn sleep_gap_is_capped() { assert_eq!(credit(3 * 3_600_000, 0, Some("a.exe"), I), 10); }
    #[test]
    fn normal_gap_credits_gap() { assert_eq!(credit(I, 1_000, Some("a.exe"), I), 5); }

    #[test]
    fn week_fills_missing_days_with_zero() {
        let rows = vec![("2026-10-21".to_string(), 60), ("2026-10-23".to_string(), 120)];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        let w = week(&rows, today);
        assert_eq!(w.len(), 7);
        assert_eq!(w[0], ("2026-10-17".to_string(), 0));
        assert_eq!(w[4], ("2026-10-21".to_string(), 60));
        assert_eq!(w[6], ("2026-10-23".to_string(), 120));
    }
}
