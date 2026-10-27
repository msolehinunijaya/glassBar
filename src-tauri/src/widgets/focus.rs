//! Focus tools: Pomodoro, countdown timers, stopwatch. State lives here (SQLite),
//! never in a webview, so alerts fire once even with every popup closed.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::scheduler::{self, Due};
use chrono::{Duration, NaiveDate};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

const MIN: i64 = 60_000;

// ---- Pomodoro ----

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Phase { #[default] Work, Short, Long }

/// States: idle/ready (no ends_at, no paused_ms), running (ends_at), paused (paused_ms).
/// Phases never start on their own: when one ends the timer stops, `completed` says which
/// phase just finished ("Work session complete"), and `phase` is the one waiting for a click.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Pomo {
    pub phase: Phase,
    /// Work sessions completed in the current set.
    pub done: u32,
    pub ends_at: Option<i64>,
    pub paused_ms: Option<i64>,
    pub completed: Option<Phase>,
}

/// Durations; `auto_start` starts the next phase when one ends; `sound`/`volume` (0–100) the
/// phase-end chime; `goal` = focus sessions a day; `dim_others` dims the dock while focusing.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PomoCfg {
    pub work_min: u32, pub short_min: u32, pub long_min: u32, pub every: u32,
    pub auto_start: bool, pub sound: bool, pub volume: u32, pub goal: u32, pub dim_others: bool,
}
impl Default for PomoCfg {
    fn default() -> Self {
        Self { work_min: 25, short_min: 5, long_min: 15, every: 4, auto_start: false, sound: true, volume: 60, goal: 4, dim_others: false }
    }
}

impl Pomo {
    pub fn duration(&self, c: &PomoCfg) -> i64 {
        let m = match self.phase { Phase::Work => c.work_min, Phase::Short => c.short_min, Phase::Long => c.long_min };
        m.max(1) as i64 * MIN
    }
    pub fn start(&mut self, now: i64, c: &PomoCfg) {
        if self.ends_at.is_some() { return; }
        self.completed = None;
        self.ends_at = Some(now + self.paused_ms.take().unwrap_or_else(|| self.duration(c)));
    }
    pub fn pause(&mut self, now: i64) {
        if let Some(e) = self.ends_at.take() { self.paused_ms = Some((e - now).max(0)); }
    }
    /// The phase after this one. `counted` = the work session was completed (not skipped).
    fn next(&self, c: &PomoCfg, counted: bool) -> Pomo {
        let (phase, done) = match self.phase {
            Phase::Work => {
                let done = self.done + counted as u32;
                if counted && done >= c.every.max(1) { (Phase::Long, done) } else { (Phase::Short, done) }
            }
            Phase::Short => (Phase::Work, self.done),
            Phase::Long => (Phase::Work, 0),
        };
        Pomo { phase, done, ends_at: None, paused_ms: None, completed: None }
    }
    /// A phase ran out: stop, remember what finished, and wait for the user to start the next.
    pub fn finish(&self, _now: i64, c: &PomoCfg) -> Pomo {
        Pomo { completed: Some(self.phase), ..self.next(c, true) }
    }
    /// Skip to the next phase (explicit click) without counting the work session; it waits idle.
    pub fn skip(&self, _now: i64, c: &PomoCfg) -> Pomo {
        self.next(c, false)
    }
    /// Switch to `phase` (the Focus/Short/Long control): the timer stops, the round count stays.
    pub fn set_phase(&self, phase: Phase) -> Pomo {
        Pomo { phase, done: self.done, ..Default::default() }
    }
    /// Reset: a started timer goes back to the start of its phase; an idle one resets the round.
    pub fn reset(&self) -> Pomo {
        if self.ends_at.is_some() || self.paused_ms.is_some() { self.set_phase(self.phase) } else { Pomo::default() }
    }
    /// "3/4" = third work session of four (breaks show the sessions done).
    pub fn label(&self, c: &PomoCfg) -> String {
        let n = if self.phase == Phase::Work { self.done + 1 } else { self.done };
        format!("{}/{}", n.min(c.every), c.every)
    }
}

/// Focus done on one day: finished work sessions, their minutes and the longest one.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FocusDay { pub day: String, pub sessions: u32, pub minutes: u32, pub longest: u32 }
impl FocusDay {
    /// This record as of `day` (zero if it is from another day).
    pub fn on(self, day: &str) -> Self { if self.day == day { self } else { FocusDay { day: day.into(), ..Default::default() } } }
    /// After a finished work session of `minutes`.
    pub fn add(self, day: &str, minutes: u32) -> Self {
        let d = self.on(day);
        FocusDay { sessions: d.sessions + 1, minutes: d.minutes + minutes, longest: d.longest.max(minutes), ..d }
    }
}

/// Days kept in the focus history (a streak longer than this stops counting up).
const HISTORY_DAYS: usize = 400;

/// History (oldest first, one entry per day with focus) after a finished session.
pub fn history_add(mut h: Vec<FocusDay>, day: &str, minutes: u32) -> Vec<FocusDay> {
    match h.last_mut() {
        Some(last) if last.day == day => *last = std::mem::take(last).add(day, minutes),
        _ => h.push(FocusDay::default().add(day, minutes)),
    }
    let cut = h.len().saturating_sub(HISTORY_DAYS);
    h.split_off(cut)
}

fn date(s: &str) -> Option<NaiveDate> { NaiveDate::parse_from_str(s, "%Y-%m-%d").ok() }

/// The 7 days ending `today`, oldest first, zero where nothing was done.
pub fn history_week(h: &[FocusDay], today: NaiveDate) -> Vec<FocusDay> {
    (0..7).rev().map(|back| {
        let d = (today - Duration::days(back)).format("%Y-%m-%d").to_string();
        h.iter().find(|x| x.day == d).cloned().unwrap_or(FocusDay { day: d, ..Default::default() })
    }).collect()
}

/// Days in a row with at least one finished session, up to today. No focus yet today
/// does not break it: the streak then ends yesterday.
pub fn streak(h: &[FocusDay], today: NaiveDate) -> u32 {
    let mut days: Vec<NaiveDate> = h.iter().filter(|x| x.sessions > 0).filter_map(|x| date(&x.day)).collect();
    days.sort_unstable();
    days.dedup();
    let mut want = if days.last() == Some(&today) { today } else { today - Duration::days(1) };
    let mut n = 0;
    for d in days.iter().rev() {
        if *d == want { n += 1; want -= Duration::days(1); } else if *d < want { break; }
    }
    n
}

fn today_key() -> String { chrono::Local::now().format("%Y-%m-%d").to_string() }
/// The stored history; the old single-day record ("pomo:today") seeds it once.
fn load_history(c: &rusqlite::Connection) -> Vec<FocusDay> {
    kv_get(c, "pomo:history").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| {
        kv_get(c, "pomo:today").and_then(|s| serde_json::from_str::<FocusDay>(&s).ok()).filter(|d| d.sessions > 0).into_iter().collect()
    })
}

fn pomo_cfg(app: &AppHandle) -> PomoCfg {
    let v = app.state::<crate::AppState>().config.lock().unwrap().widget_settings("productivity");
    serde_json::from_value(v).unwrap_or_default()
}

fn load_pomo(app: &AppHandle) -> Pomo {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    kv_get(&c, "pomodoro").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Everything the Pomodoro UI shows, from one read. `week` = the last 7 days (oldest
/// first, today last), `streak` = days in a row, `longest` = longest session in minutes.
#[derive(Serialize, Clone)]
pub struct PomoState { pub pomo: Pomo, pub cfg: PomoCfg, pub label: String, pub today: FocusDay, pub week: Vec<FocusDay>, pub streak: u32, pub longest: u32 }

fn pomo_state(app: &AppHandle, p: Pomo) -> PomoState {
    let cfg = pomo_cfg(app);
    let h = { let db = app.state::<Db>(); let c = db.0.lock().unwrap(); load_history(&c) };
    let today = chrono::Local::now().date_naive();
    let week = history_week(&h, today);
    PomoState {
        label: p.label(&cfg), pomo: p, cfg, today: week[6].clone(), streak: streak(&h, today),
        longest: h.iter().map(|d| d.longest).max().unwrap_or(0), week,
    }
}

fn save_pomo(app: &AppHandle, p: &Pomo) -> PomoState {
    {
        let db = app.state::<Db>();
        kv_set(&db.0.lock().unwrap(), "pomodoro", &serde_json::to_string(p).unwrap());
    }
    let s = pomo_state(app, p.clone());
    let _ = app.emit("pomodoro:changed", ());
    scheduler::poke();
    s
}

fn pomo_do(app: &AppHandle, f: impl FnOnce(Pomo, i64, &PomoCfg) -> Pomo) -> PomoState {
    let cfg = pomo_cfg(app);
    let p = f(load_pomo(app), now_ms(), &cfg);
    save_pomo(app, &p)
}

#[tauri::command]
pub fn pomo_get(app: AppHandle) -> PomoState { let p = load_pomo(&app); pomo_state(&app, p) }
#[tauri::command]
pub fn pomo_start(app: AppHandle) -> PomoState { pomo_do(&app, |mut p, now, c| { p.start(now, c); p }) }
#[tauri::command]
pub fn pomo_pause(app: AppHandle) -> PomoState { pomo_do(&app, |mut p, now, _| { p.pause(now); p }) }
#[tauri::command]
pub fn pomo_skip(app: AppHandle) -> PomoState { pomo_do(&app, |p, now, c| p.skip(now, c)) }
#[tauri::command]
pub fn pomo_reset(app: AppHandle) -> PomoState { pomo_do(&app, |p, _, _| p.reset()) }
#[tauri::command]
pub fn pomo_set_phase(app: AppHandle, phase: Phase) -> PomoState { pomo_do(&app, |p, _, _| p.set_phase(phase)) }

/// Scheduler hook: the running phase ended. Records a finished focus session, moves on
/// (starting the next phase with Auto-start), and returns the alert's sound (Null = silent)
/// and banner data.
pub fn on_pomo_due(app: &AppHandle) -> (serde_json::Value, serde_json::Value) {
    let p = load_pomo(app);
    let cfg = pomo_cfg(app);
    if p.phase == Phase::Work && p.ends_at.is_some() {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let h = history_add(load_history(&c), &today_key(), cfg.work_min);
        kv_set(&c, "pomo:history", &serde_json::to_string(&h).unwrap());
    }
    pomo_do(app, |p, now, c| { let mut n = p.finish(now, c); if c.auto_start { n.start(now, c); } n });
    let sound = if cfg.sound { serde_json::json!({ "kind": "chime", "volume": cfg.volume.min(100) as f64 / 100.0 }) } else { serde_json::Value::Null };
    (sound, serde_json::json!({ "auto": cfg.auto_start }))
}

// ---- Timers ----

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Timer { pub id: i64, pub label: String, pub duration_ms: i64, pub ends_at: Option<i64>, pub remaining_ms: Option<i64> }

fn timers_changed(app: &AppHandle) {
    let _ = app.emit("timers:changed", ());
    scheduler::poke();
}

#[tauri::command]
pub fn timers_list(db: State<Db>) -> Vec<Timer> {
    let c = db.0.lock().unwrap();
    let mut q = c.prepare("SELECT id, label, duration_ms, ends_at, remaining_ms FROM timers ORDER BY id").unwrap();
    q.query_map([], |r| Ok(Timer { id: r.get(0)?, label: r.get(1)?, duration_ms: r.get(2)?, ends_at: r.get(3)?, remaining_ms: r.get(4)? }))
        .map(|r| r.filter_map(Result::ok).collect()).unwrap_or_default()
}

#[tauri::command]
pub fn timer_add(app: AppHandle, label: String, duration_ms: i64) -> Result<(), String> {
    if !(1_000..=24 * 60 * MIN).contains(&duration_ms) { return Err("Timer must be between 1 second and 24 hours".into()); }
    app.state::<Db>().0.lock().unwrap()
        .execute("INSERT INTO timers(label, duration_ms, ends_at) VALUES(?1, ?2, ?3)", params![label.trim(), duration_ms, now_ms() + duration_ms])
        .map_err(|e| e.to_string())?;
    timers_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn timer_pause(app: AppHandle, id: i64) {
    let _ = app.state::<Db>().0.lock().unwrap().execute(
        "UPDATE timers SET remaining_ms = MAX(ends_at - ?2, 0), ends_at = NULL WHERE id = ?1 AND ends_at > ?2", params![id, now_ms()]);
    timers_changed(&app);
}

#[tauri::command]
pub fn timer_resume(app: AppHandle, id: i64) {
    let _ = app.state::<Db>().0.lock().unwrap().execute(
        "UPDATE timers SET ends_at = ?2 + remaining_ms, remaining_ms = NULL WHERE id = ?1 AND remaining_ms > 0", params![id, now_ms()]);
    timers_changed(&app);
}

/// Restart a timer for `ms` more (the "+1 min" on a finished timer's banner).
#[tauri::command]
pub fn timer_snooze(app: AppHandle, id: i64, ms: i64) {
    let _ = app.state::<Db>().0.lock().unwrap().execute(
        "UPDATE timers SET duration_ms = ?2, ends_at = ?3, remaining_ms = NULL WHERE id = ?1", params![id, ms, now_ms() + ms]);
    timers_changed(&app);
}

/// Timer id from its scheduler key "timer:{id}:{ends_at}".
pub fn timer_id(key: &str) -> Option<i64> { key.strip_prefix("timer:")?.split(':').next()?.parse().ok() }

/// A timer ran out: mark it finished (ends_at NULL, remaining 0) so nothing keeps ticking.
pub fn on_timer_due(app: &AppHandle, id: i64) {
    let _ = app.state::<Db>().0.lock().unwrap()
        .execute("UPDATE timers SET ends_at = NULL, remaining_ms = 0 WHERE id = ?1", [id]);
    let _ = app.emit("timers:changed", ());
}

#[tauri::command]
pub fn timer_delete(app: AppHandle, id: i64) {
    let _ = app.state::<Db>().0.lock().unwrap().execute("DELETE FROM timers WHERE id = ?1", [id]);
    timers_changed(&app);
}

// ---- Stopwatch ----

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stopwatch { pub started_at: Option<i64>, pub acc_ms: i64, pub laps: Vec<i64> }

impl Stopwatch {
    pub fn elapsed(&self, now: i64) -> i64 { self.acc_ms + self.started_at.map(|s| now - s).unwrap_or(0) }
}

fn sw_do(app: &AppHandle, f: impl FnOnce(&mut Stopwatch, i64)) -> Stopwatch {
    let sw = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let mut sw: Stopwatch = kv_get(&c, "stopwatch").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        f(&mut sw, now_ms());
        kv_set(&c, "stopwatch", &serde_json::to_string(&sw).unwrap());
        sw
    };
    let _ = app.emit("stopwatch:changed", ());
    sw
}

/// Read only. (It used to go through sw_do, which announced a change on every read; each
/// widget answered by reading again, an endless event loop that froze the timer buttons.)
#[tauri::command]
pub fn sw_get(app: AppHandle) -> Stopwatch {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    kv_get(&c, "stopwatch").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
#[tauri::command]
pub fn sw_toggle(app: AppHandle) -> Stopwatch {
    sw_do(&app, |sw, now| match sw.started_at.take() {
        Some(s) => sw.acc_ms += now - s,
        None => sw.started_at = Some(now),
    })
}
#[tauri::command]
pub fn sw_lap(app: AppHandle) -> Stopwatch { sw_do(&app, |sw, now| { let e = sw.elapsed(now); sw.laps.push(e); }) }
#[tauri::command]
pub fn sw_reset(app: AppHandle) -> Stopwatch { sw_do(&app, |sw, _| *sw = Stopwatch::default()) }

// ---- Scheduler items ----

/// The dock alert for a Pomodoro phase that just ended: which one ("focus" / "break") and
/// what happens now. None for one missed while GlassBar was off, and in Do Not Disturb.
pub fn pomo_alert(title: &str, body: &str, missed: bool, dnd: bool) -> Option<serde_json::Value> {
    if missed || dnd { return None; }
    let done = if title.starts_with("Focus") { "focus" } else { "break" };
    Some(serde_json::json!({ "kind": "pomodoro", "done": done, "body": body }))
}

pub fn due_items(app: &AppHandle) -> Vec<Due> {
    let mut v = Vec::new();
    let p = load_pomo(app);
    if let Some(at) = p.ends_at {
        let (title, body) = match (p.phase, pomo_cfg(app).auto_start) {
            (Phase::Work, false) => ("Focus session complete", "Start your break when you're ready."),
            (Phase::Work, true) => ("Focus session complete", "Your break has started."),
            (_, false) => ("Break complete", "Start focus when you're ready."),
            (_, true) => ("Break complete", "Focus has started."),
        };
        v.push(Due { key: format!("pomo:{at}"), at, source: "pomodoro", title: title.into(), body: body.into(), sound: None });
    }
    for t in timers_list(app.state::<Db>()) {
        if let Some(at) = t.ends_at {
            let body = if t.label.is_empty() { "Time's up.".to_string() } else { t.label.clone() };
            v.push(Due { key: format!("timer:{}:{at}", t.id), at, source: "timer", title: "Timer done".into(), body, sound: None });
        }
    }
    v
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_finished_phase_becomes_a_dock_alert_saying_which() {
        let a = super::pomo_alert("Focus session complete", "Your break has started.", false, false).unwrap();
        assert_eq!((a["done"].as_str(), a["body"].as_str()), (Some("focus"), Some("Your break has started.")));
        assert_eq!(super::pomo_alert("Break complete", "Focus has started.", false, false).unwrap()["done"], "break");
        assert!(super::pomo_alert("Break complete", "x", true, false).is_none(), "missed while off");
        assert!(super::pomo_alert("Break complete", "x", false, true).is_none(), "Do Not Disturb");
    }

    use super::*;

    fn cfg() -> PomoCfg { PomoCfg::default() }

    fn day(d: &str, sessions: u32) -> FocusDay { FocusDay { day: d.into(), sessions, minutes: sessions * 25, longest: 25 } }
    fn ymd(s: &str) -> NaiveDate { date(s).unwrap() }

    #[test]
    fn focus_today_adds_and_resets_each_day() {
        let d = FocusDay::default().add("2026-10-24", 25).add("2026-10-24", 50);
        assert_eq!((d.sessions, d.minutes, d.longest), (2, 75, 50));
        assert_eq!(d.clone().on("2026-10-25"), FocusDay { day: "2026-10-25".into(), ..Default::default() });
        assert_eq!(d.add("2026-10-25", 30), FocusDay { day: "2026-10-25".into(), sessions: 1, minutes: 30, longest: 30 });
    }

    #[test]
    fn history_adds_per_day_and_week_fills_gaps() {
        let h = history_add(history_add(history_add(vec![], "2026-10-20", 25), "2026-10-22", 25), "2026-10-22", 50);
        assert_eq!(h.len(), 2);
        assert_eq!((h[1].sessions, h[1].minutes, h[1].longest), (2, 75, 50));
        let w = history_week(&h, ymd("2026-10-24"));
        assert_eq!(w.iter().map(|d| d.day.as_str()).collect::<Vec<_>>(),
            ["2026-10-18", "2026-10-19", "2026-10-20", "2026-10-21", "2026-10-22", "2026-10-23", "2026-10-24"]);
        assert_eq!(w.iter().map(|d| d.minutes).collect::<Vec<_>>(), [0, 0, 25, 0, 75, 0, 0]);
        let long = (0..500).fold(vec![], |h, i| history_add(h, &format!("d{i:03}"), 1));
        assert_eq!(long.len(), HISTORY_DAYS);
    }

    #[test]
    fn streak_counts_days_in_a_row_up_to_today() {
        let t = ymd("2026-10-24");
        let h = [day("2026-10-20", 1), day("2026-10-22", 2), day("2026-10-23", 1), day("2026-10-24", 1)];
        assert_eq!(streak(&h, t), 3);
        assert_eq!(streak(&h[..3], t), 2, "nothing yet today: the streak ends yesterday");
        assert_eq!(streak(&h[..2], t), 0, "a missed day breaks it");
        assert_eq!(streak(&[day("2026-10-24", 0)], t), 0, "a day with no session does not count");
        assert_eq!(streak(&[], t), 0);
    }

    #[test]
    fn reset_restarts_a_started_phase_or_the_round() {
        let c = cfg();
        let mut p = Pomo { phase: Phase::Work, done: 2, ..Default::default() };
        p.start(0, &c);
        assert_eq!(p.reset(), Pomo { phase: Phase::Work, done: 2, ..Default::default() });
        assert_eq!(p.reset().reset(), Pomo::default());
        let s = p.set_phase(Phase::Long);
        assert_eq!((s.phase, s.done, s.ends_at), (Phase::Long, 2, None));
    }

    #[test]
    fn old_settings_get_new_defaults() {
        let c: PomoCfg = serde_json::from_value(serde_json::json!({ "workMin": 50 })).unwrap();
        assert_eq!((c.work_min, c.goal, c.auto_start, c.sound), (50, 4, false, true));
    }

    #[test]
    fn work_to_short_then_long_on_fourth() {
        let c = cfg();
        let mut p = Pomo::default();
        let mut seen = vec![];
        for i in 1..=8 {
            p.start(i * 100 * MIN, &c); // every phase needs a click
            p = p.finish(i * 100 * MIN + 30 * MIN, &c);
            seen.push((p.phase, p.done));
        }
        use Phase::*;
        assert_eq!(seen, vec![(Short, 1), (Work, 1), (Short, 2), (Work, 2), (Short, 3), (Work, 3), (Long, 4), (Work, 0)]);
    }

    #[test]
    fn finish_stops_and_waits_for_a_click() {
        let c = cfg();
        let mut p = Pomo::default();
        p.start(0, &c);
        let p = p.finish(25 * MIN, &c);
        assert_eq!((p.phase, p.ends_at, p.paused_ms, p.completed), (Phase::Short, None, None, Some(Phase::Work)));
        let mut p = p;
        p.start(30 * MIN, &c);
        assert_eq!((p.ends_at, p.completed), (Some(35 * MIN), None), "starting clears the 'complete' state");
        let p = p.finish(35 * MIN, &c);
        assert_eq!((p.phase, p.ends_at, p.completed), (Phase::Work, None, Some(Phase::Short)));
    }

    #[test]
    fn pause_keeps_remaining() {
        let c = cfg();
        let mut p = Pomo::default();
        p.start(0, &c);
        p.pause(10 * MIN);
        assert_eq!((p.ends_at, p.paused_ms), (None, Some(15 * MIN)));
        p.start(100 * MIN, &c);
        assert_eq!(p.ends_at, Some(115 * MIN));
    }

    #[test]
    fn skip_advances_without_counting_work_and_does_not_start() {
        let c = cfg();
        let mut p = Pomo::default();
        p.start(0, &c);
        p = p.skip(MIN, &c);
        assert_eq!((p.phase, p.done, p.ends_at), (Phase::Short, 0, None));
    }

    #[test]
    fn counter_label() {
        let c = cfg();
        let mut p = Pomo::default();
        assert_eq!(p.label(&c), "1/4");
        for i in 1..=4 { p.start(i * MIN, &c); p = p.finish(i * MIN + 1, &c); } // work, short, work, short
        assert_eq!((p.phase, p.label(&c)), (Phase::Work, "3/4".to_string()));
    }

    #[test]
    fn timer_id_from_due_key() {
        assert_eq!(timer_id("timer:42:1792744673748"), Some(42));
        assert_eq!(timer_id("pomo:1"), None);
    }
}
