//! Tasbih counter (Waktu Solat): the count lives in the database (`kv`, key "tasbih"), never in
//! config.json, since every tap writes it. The target and dhikr are widget settings; the dock
//! and the panel send the targets of the current dhikr with each tap (e.g. [33, 33, 34]).
use crate::db::{kv_get, kv_set, Db};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// count: in the current step; step: which phrase (Tasbih Fatimah has three); round: full
/// cycles done; today: taps today (local date `day`).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TasbihState { pub count: u32, pub round: u32, pub step: u32, pub today: u32, pub day: String }

/// Pure: one tap (+1) or undo (-1). Reaching the step's target shows as complete (count ==
/// target); the next tap starts the next step at 1 (and the next round after the last step).
/// Undo never goes below 0 or back into a finished step. A new day resets `today` only.
pub fn apply(mut s: TasbihState, delta: i32, steps: &[u32], day: &str) -> TasbihState {
    let steps: Vec<u32> = if steps.is_empty() { vec![33] } else { steps.iter().map(|t| (*t).max(1)).collect() };
    if s.day != day { s.day = day.to_string(); s.today = 0; }
    if s.step as usize >= steps.len() { s.step = 0; }
    if delta > 0 {
        if s.count >= steps[s.step as usize] {
            s.count = 1;
            s.step = (s.step + 1) % steps.len() as u32;
            if s.step == 0 { s.round += 1; }
        } else {
            s.count += 1;
        }
        s.today += 1;
    } else if delta < 0 && s.count > 0 {
        s.count -= 1;
        s.today = s.today.saturating_sub(1);
    }
    s
}

fn today() -> String { chrono::Local::now().format("%Y-%m-%d").to_string() }

fn load(app: &AppHandle) -> TasbihState {
    let db = app.state::<Db>();
    let c = db.lock();
    let mut s: TasbihState = kv_get(&c, "tasbih").and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default();
    if s.day != today() { s.day = today(); s.today = 0; }
    s
}

fn store(app: &AppHandle, s: &TasbihState) {
    {
        let db = app.state::<Db>();
        let c = db.lock();
        kv_set(&c, "tasbih", &serde_json::to_string(s).unwrap_or_default());
    }
    let _ = app.emit("tasbih:changed", s);
}

#[tauri::command]
pub fn tasbih_get(app: AppHandle) -> TasbihState { load(&app) }

#[tauri::command]
pub fn tasbih_add(app: AppHandle, delta: i32, steps: Vec<u32>) -> TasbihState {
    let s = apply(load(&app), delta.signum(), &steps, &today());
    store(&app, &s);
    s
}

/// Back to 0 at the first phrase (today's total stays).
#[tauri::command]
pub fn tasbih_reset(app: AppHandle) -> TasbihState {
    let s = TasbihState { count: 0, round: 0, step: 0, ..load(&app) };
    store(&app, &s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    const D: &str = "2026-10-30";
    fn at(count: u32, step: u32) -> TasbihState { TasbihState { count, step, round: 0, today: 5, day: D.into() } }

    #[test]
    fn counts_to_the_target_then_starts_the_next_round() {
        let s = apply(at(32, 0), 1, &[33], D);
        assert_eq!((s.count, s.round, s.today), (33, 0, 6), "33 shows as complete");
        let s = apply(s, 1, &[33], D);
        assert_eq!((s.count, s.step, s.round), (1, 0, 1), "next tap: round 2 from 1");
    }
    #[test]
    fn tasbih_fatimah_moves_through_its_three_phrases() {
        let s = apply(at(33, 0), 1, &[33, 33, 34], D);
        assert_eq!((s.count, s.step, s.round), (1, 1, 0));
        let s = apply(at(34, 2), 1, &[33, 33, 34], D);
        assert_eq!((s.count, s.step, s.round), (1, 0, 1));
    }
    #[test]
    fn undo_stops_at_zero_and_a_new_day_resets_today_only() {
        let s = apply(at(0, 0), -1, &[33], D);
        assert_eq!((s.count, s.today), (0, 5));
        let s = apply(at(10, 0), 1, &[33], "2026-10-31");
        assert_eq!((s.count, s.today, s.day.as_str()), (11, 1, "2026-10-31"));
    }
    #[test]
    fn a_smaller_target_or_fewer_steps_never_breaks() {
        let s = apply(at(50, 2), 1, &[33], D);
        assert_eq!((s.count, s.step, s.round), (1, 0, 1), "past a lowered target: next round");
    }
}
