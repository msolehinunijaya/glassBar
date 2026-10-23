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

/// Idle when both `ends_at` and `paused_ms` are None.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Pomo {
    pub phase: Phase,
    /// Work sessions completed in the current set.
    pub done: u32,
    pub ends_at: Option<i64>,
    pub paused_ms: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PomoCfg { pub work_min: u32, pub short_min: u32, pub long_min: u32, pub every: u32, pub auto_start: bool }
impl Default for PomoCfg {
    fn default() -> Self { Self { work_min: 25, short_min: 5, long_min: 15, every: 4, auto_start: true } }
}

impl Pomo {
    pub fn duration(&self, c: &PomoCfg) -> i64 {
        let m = match self.phase { Phase::Work => c.work_min, Phase::Short => c.short_min, Phase::Long => c.long_min };
        m.max(1) as i64 * MIN
    }
    pub fn start(&mut self, now: i64, c: &PomoCfg) {
        if self.ends_at.is_some() { return; }
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
        Pomo { phase, done, ends_at: None, paused_ms: None }
    }
    /// A phase ran out: move on, and keep going if auto-start is on.
    pub fn finish(&self, now: i64, c: &PomoCfg) -> Pomo {
        let mut p = self.next(c, true);
        if c.auto_start { p.start(now, c); }
        p
    }
    /// Skip to the next phase without counting the work session; keeps running if it was.
    pub fn skip(&self, now: i64, c: &PomoCfg) -> Pomo {
        let running = self.ends_at.is_some();
        let mut p = self.next(c, false);
        if running { p.start(now, c); }
        p
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
        "UPDATE timers SET ends_at = ?2 + remaining_ms, remaining_ms = NULL WHERE id = ?1 AND remaining_ms IS NOT NULL", params![id, now_ms()]);
    timers_changed(&app);
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

#[tauri::command]
pub fn sw_get(app: AppHandle) -> Stopwatch { sw_do(&app, |_, _| {}) }
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
            Phase::Work => ("Focus session complete", "Time for a break."),
            _ => ("Break is over", "Back to focus."),
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
        p.start(0, &c);
        let mut seen = vec![];
        for i in 1..=8 {
            p = p.finish(i * 100 * MIN, &c);
            seen.push((p.phase, p.done));
        }
        use Phase::*;
        assert_eq!(seen, vec![(Short, 1), (Work, 1), (Short, 2), (Work, 2), (Short, 3), (Work, 3), (Long, 4), (Work, 0)]);
        assert_eq!(p.ends_at, Some(800 * MIN + 25 * MIN), "auto-start runs the next phase");
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
    fn skip_advances_without_counting_work() {
        let c = cfg();
        let mut p = Pomo::default();
        p.start(0, &c);
        p = p.skip(MIN, &c);
        assert_eq!((p.phase, p.done), (Phase::Short, 0));
    }

    #[test]
    fn counter_label() {
        let c = cfg();
        let mut p = Pomo::default();
        assert_eq!(p.label(&c), "1/4");
        p.start(0, &c);
        for i in 1..=4 { p = p.finish(i * MIN, &c); } // work, short, work, short
        assert_eq!((p.phase, p.label(&c)), (Phase::Work, "3/4".to_string()));
    }

    #[test]
    fn no_auto_start_waits() {
        let c = PomoCfg { auto_start: false, ..cfg() };
        let mut p = Pomo::default();
        p.start(0, &c);
        p = p.finish(25 * MIN, &c);
        assert_eq!((p.phase, p.ends_at, p.paused_ms), (Phase::Short, None, None));
    }
}
