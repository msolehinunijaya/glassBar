//! Focus tools: Pomodoro, countdown timers, stopwatch. State lives here (SQLite),
//! never in a webview, so alerts fire once even with every popup closed.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::scheduler::{self, Due};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PomoCfg { pub work_min: u32, pub short_min: u32, pub long_min: u32, pub every: u32 }
impl Default for PomoCfg {
    fn default() -> Self { Self { work_min: 25, short_min: 5, long_min: 15, every: 4 } }
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
    /// "3/4" = third work session of four (breaks show the sessions done).
    pub fn label(&self, c: &PomoCfg) -> String {
        let n = if self.phase == Phase::Work { self.done + 1 } else { self.done };
        format!("{}/{}", n.min(c.every), c.every)
    }
}

fn pomo_cfg(app: &AppHandle) -> PomoCfg {
    let v = app.state::<crate::AppState>().config.lock().unwrap().widget_settings("pomodoro");
    serde_json::from_value(v).unwrap_or_default()
}

fn load_pomo(app: &AppHandle) -> Pomo {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    kv_get(&c, "pomodoro").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

#[derive(Serialize, Clone)]
pub struct PomoState { pub pomo: Pomo, pub cfg: PomoCfg, pub label: String }

fn save_pomo(app: &AppHandle, p: &Pomo) -> PomoState {
    {
        let db = app.state::<Db>();
        kv_set(&db.0.lock().unwrap(), "pomodoro", &serde_json::to_string(p).unwrap());
    }
    let cfg = pomo_cfg(app);
    let s = PomoState { label: p.label(&cfg), pomo: p.clone(), cfg };
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
pub fn pomo_get(app: AppHandle) -> PomoState {
    let cfg = pomo_cfg(&app);
    let p = load_pomo(&app);
    PomoState { label: p.label(&cfg), pomo: p, cfg }
}
#[tauri::command]
pub fn pomo_start(app: AppHandle) -> PomoState { pomo_do(&app, |mut p, now, c| { p.start(now, c); p }) }
#[tauri::command]
pub fn pomo_pause(app: AppHandle) -> PomoState { pomo_do(&app, |mut p, now, _| { p.pause(now); p }) }
#[tauri::command]
pub fn pomo_skip(app: AppHandle) -> PomoState { pomo_do(&app, |p, now, c| p.skip(now, c)) }
#[tauri::command]
pub fn pomo_reset(app: AppHandle) -> PomoState { pomo_do(&app, |_, _, _| Pomo::default()) }

/// Scheduler hook: the running phase ended.
pub fn on_pomo_due(app: &AppHandle) { pomo_do(app, |p, now, c| p.finish(now, c)); }

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

pub fn due_items(app: &AppHandle) -> Vec<Due> {
    let mut v = Vec::new();
    let p = load_pomo(app);
    if let Some(at) = p.ends_at {
        let (title, body) = match p.phase {
            Phase::Work => ("Work session complete", "Click Start Break when you're ready."),
            _ => ("Break complete", "Click Start Work Session when you're ready."),
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
    use super::*;

    fn cfg() -> PomoCfg { PomoCfg::default() }

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
