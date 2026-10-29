//! Self-check of the dock. Every 5 s the core pings the dock's page, which answers through
//! the same path its buttons use. No answer for 20 s while the PC is awake means the dock is
//! stuck (it still draws, but clicks do nothing): reload it, and restart GlassBar if a reload
//! didn't help. Time asleep never counts.
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const PING_MS: u64 = 5_000;
pub const STUCK_MS: u64 = 20_000;
/// A tick this late means the PC was asleep (or the whole process stalled): start over.
const ASLEEP_MS: u64 = 15_000;
/// After start (or a wake-up / reload) the page gets this long before it counts as stuck.
const GRACE_MS: u64 = 30_000;

static LAST_PONG: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Act { Fine, Reload, Restart }

#[derive(Debug, Default)]
pub struct State { pub last_tick: u64, pub reloaded_at: Option<u64>, pub grace_until: u64 }

impl State {
    pub fn new(now: u64) -> Self { Self { grace_until: now + GRACE_MS, ..Default::default() } }
}

/// Pure decision for one tick. `last_pong` = when the page last really answered.
pub fn decide(now: u64, last_pong: u64, st: &mut State) -> Act {
    let slept = st.last_tick != 0 && now.saturating_sub(st.last_tick) > ASLEEP_MS;
    st.last_tick = now;
    if slept { st.reloaded_at = None; st.grace_until = now + GRACE_MS; return Act::Fine; }
    // The page answered after the reload: the reload worked.
    if st.reloaded_at.is_some_and(|t| last_pong > t) { st.reloaded_at = None; }
    if now < st.grace_until || now.saturating_sub(last_pong) < STUCK_MS { return Act::Fine; }
    match st.reloaded_at {
        None => { st.reloaded_at = Some(now); st.grace_until = now + GRACE_MS; Act::Reload }
        Some(t) if now - t >= GRACE_MS + STUCK_MS => Act::Restart,
        Some(_) => Act::Fine,
    }
}

/// The dock is stuck: if GlassBar's own worker pool is full (every worker waiting on
/// something that never answers), a reload can't help, so restart straight away.
pub fn repair(act: Act, starved: bool) -> Act { if act == Act::Reload && starved { Act::Restart } else { act } }

/// Does a trivial job still get a worker within `limit`? Commands (app icons, media, widget
/// actions) all run on that pool.
fn pool_starved(limit: Duration) -> bool {
    let (tx, rx) = std::sync::mpsc::channel();
    tauri::async_runtime::spawn_blocking(move || { let _ = tx.send(()); });
    rx.recv_timeout(limit).is_err()
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut st = State::new(crate::db::now_ms() as u64);
        let mut seq = 0u64;
        loop {
            std::thread::sleep(Duration::from_millis(PING_MS));
            seq += 1;
            let _ = app.emit_to("bar", "health:ping", seq);
            let now = crate::db::now_ms() as u64;
            let act = decide(now, LAST_PONG.load(Ordering::Relaxed), &mut st);
            let starved = act != Act::Fine && pool_starved(Duration::from_secs(3));
            if starved { crate::applog::log("health: GlassBar's worker pool is full (something never answered)"); }
            match repair(act, starved) {
                Act::Fine => {}
                Act::Reload => {
                    crate::applog::log("health: the dock stopped answering for 20 s: reloading it");
                    for l in ["bar", "popup", "banner"] {
                        if let Some(w) = app.get_webview_window(l) { let _ = w.reload(); }
                    }
                    crate::bar::apply(&app);
                }
                Act::Restart => {
                    crate::applog::log("health: still not answering after a reload: restarting GlassBar");
                    relaunch(&app);
                }
            }
        }
    });
}

/// Start a fresh GlassBar and quit this one. The new one waits for this one to be gone
/// first (`--after <pid>`): GlassBar is single-instance, and would otherwise just close.
fn relaunch(app: &AppHandle) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).args(["--after", &std::process::id().to_string()]).spawn();
    }
    app.exit(0);
}

/// Started by `relaunch`: wait (up to 10 s) until the old GlassBar has exited.
pub fn wait_for_previous() {
    let args: Vec<String> = std::env::args().collect();
    let pid = args.iter().position(|a| a == "--after").and_then(|i| args.get(i + 1)).and_then(|p| p.parse::<u32>().ok());
    if let Some(pid) = pid { crate::platform::shell().wait_for_exit(pid, 10_000); }
}

/// The dock's page answered a ping (through the same worker pool its buttons use).
#[tauri::command]
pub async fn health_pong(_seq: u64) -> Result<(), String> {
    crate::widgets::blocking(|| LAST_PONG.store(crate::db::now_ms() as u64, Ordering::Relaxed)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    const T0: u64 = 1_000_000;

    /// Run ticks every 5 s for `ms`, the page answering at the times `answers` says.
    fn run(st: &mut State, from: u64, ms: u64, answers: impl Fn(u64) -> Option<u64>) -> Vec<(u64, Act)> {
        let mut last = 0;
        (1..=ms / PING_MS).map(|i| {
            let now = from + i * PING_MS;
            if let Some(a) = answers(now) { last = a; }
            (now, decide(now, last, st))
        }).collect()
    }

    #[test]
    fn a_starved_worker_pool_restarts_at_once_instead_of_reloading() {
        // A reload can't empty GlassBar's own worker pool; only a restart does.
        assert_eq!(repair(Act::Reload, true), Act::Restart);
        assert_eq!(repair(Act::Reload, false), Act::Reload);
        assert_eq!(repair(Act::Fine, true), Act::Fine);
    }

    #[test]
    fn answering_dock_is_fine() {
        let mut st = State::new(T0);
        assert!(run(&mut st, T0, 300_000, |now| Some(now - 100)).iter().all(|(_, a)| *a == Act::Fine));
    }

    #[test]
    fn stuck_dock_is_reloaded_once_then_restarted() {
        let mut st = State::new(T0);
        let acts = run(&mut st, T0, 200_000, |_| None); // never answers
        let reloads: Vec<u64> = acts.iter().filter(|(_, a)| *a == Act::Reload).map(|(t, _)| t - T0).collect();
        assert_eq!(reloads, vec![GRACE_MS], "never answered since start: one reload when the start-up grace ends");
        let restart = acts.iter().find(|(_, a)| *a == Act::Restart).map(|(t, _)| t - T0).unwrap();
        assert_eq!(restart, reloads[0] + GRACE_MS + STUCK_MS, "restart when the reload didn't help");
    }

    #[test]
    fn time_asleep_does_not_count() {
        let mut st = State::new(T0);
        decide(T0 + PING_MS, T0, &mut st);
        // The PC slept 8 hours: the next tick comes very late, the last answer is old.
        let woke = T0 + 8 * 3_600_000;
        assert_eq!(decide(woke, T0, &mut st), Act::Fine);
        assert_eq!(decide(woke + PING_MS, T0, &mut st), Act::Fine, "a fresh grace period after waking");
    }

    #[test]
    fn a_reload_that_works_resets_everything() {
        let mut st = State::new(T0);
        // Stuck until the reload, then answering again.
        let acts = run(&mut st, T0, 300_000, |now| (now > T0 + 60_000).then_some(now - 100));
        assert_eq!(acts.iter().filter(|(_, a)| *a == Act::Reload).count(), 1);
        assert!(!acts.iter().any(|(_, a)| *a == Act::Restart));
        assert!(st.reloaded_at.is_none());
    }
}
