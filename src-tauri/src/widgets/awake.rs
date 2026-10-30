//! Keep Awake (Quick Settings): stop the PC sleeping (and the screen dimming) for a while or
//! until turned off. In memory only: it never survives a restart, and the power request dies
//! with the process, so a crash can't leave the PC awake for good.
use crate::db::now_ms;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AwakeState { pub on: bool, /** Unix ms; None = until turned off */ pub until: Option<i64>, pub screen: bool }

/// The state, and a generation number so an old timer can't end a newer request.
static STATE: Mutex<(AwakeState, u64)> = Mutex::new((AwakeState { on: false, until: None, screen: true }, 0));

/// When a request for `minutes` (None = until turned off) made at `now` ends.
pub fn until_of(now: i64, minutes: Option<u32>) -> Option<i64> { minutes.filter(|m| *m > 0).map(|m| now + m as i64 * 60_000) }

fn publish(app: &AppHandle, s: AwakeState) { let _ = app.emit("awake:changed", s); }

#[tauri::command]
pub fn awake_get() -> AwakeState { STATE.lock().unwrap().0 }

/// Turn on (or change) Keep Awake: for `minutes`, or until turned off when None / 0.
#[tauri::command]
pub fn awake_set(app: AppHandle, minutes: Option<u32>, screen: bool) -> Result<AwakeState, String> {
    crate::platform::power().keep_awake(screen)?;
    let s = AwakeState { on: true, until: until_of(now_ms(), minutes), screen };
    let gen = {
        let mut g = STATE.lock().unwrap();
        g.1 += 1;
        g.0 = s;
        g.1
    };
    if let Some(until) = s.until {
        let a = app.clone();
        std::thread::spawn(move || {
            // Wake at least once a minute: the PC may have slept anyway (lid closed).
            while now_ms() < until {
                std::thread::sleep(std::time::Duration::from_millis(((until - now_ms()).clamp(0, 60_000)) as u64));
                if STATE.lock().unwrap().1 != gen { return; }
            }
            if STATE.lock().unwrap().1 == gen { off(&a); }
        });
    }
    crate::applog::log(format!("keep awake on ({})", minutes.map_or("until off".into(), |m| format!("{m} min"))));
    publish(&app, s);
    Ok(s)
}

fn off(app: &AppHandle) -> AwakeState {
    crate::platform::power().allow_sleep();
    let s = {
        let mut g = STATE.lock().unwrap();
        g.1 += 1;
        g.0 = AwakeState { on: false, until: None, screen: g.0.screen };
        g.0
    };
    publish(app, s);
    s
}

#[tauri::command]
pub fn awake_off(app: AppHandle) -> AwakeState { off(&app) }

#[cfg(test)]
mod tests {
    #[test]
    fn until_counts_minutes_from_now() {
        assert_eq!(super::until_of(1_000, Some(30)), Some(1_000 + 30 * 60_000));
        assert_eq!(super::until_of(1_000, None), None);
        assert_eq!(super::until_of(1_000, Some(0)), None, "0 = until turned off");
    }
}
