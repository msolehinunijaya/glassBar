use super::blocking;
use crate::platform::{audio, AudioSession, AudioState, Res};

#[tauri::command]
pub async fn audio_state() -> Res<AudioState> { blocking(move || audio().state()).await? }
#[tauri::command]
pub async fn audio_set_volume(volume: f32) -> Res<()> { blocking(move || audio().set_volume(volume)).await? }
#[tauri::command]
pub async fn audio_set_mute(muted: bool) -> Res<()> { blocking(move || audio().set_mute(muted)).await? }
#[tauri::command]
pub async fn audio_set_default(id: String) -> Res<()> { blocking(move || audio().set_default(&id)).await? }
#[tauri::command]
pub async fn audio_sessions() -> Res<Vec<AudioSession>> { blocking(move || audio().sessions()).await? }
#[tauri::command]
pub async fn audio_set_session_volume(pid: u32, volume: f32) -> Res<()> { blocking(move || audio().set_session_volume(pid, volume)).await? }
#[tauri::command]
pub async fn audio_set_session_mute(pid: u32, muted: bool) -> Res<()> { blocking(move || audio().set_session_mute(pid, muted)).await? }

// ---- Device connection sounds ----

/// Play at most one sound per burst: a hub or a flaky cable can fire several events.
/// Same kind within 1.5 s, or anything within 0.7 s of the last sound, is skipped.
pub fn should_play(last: Option<(bool, i64)>, connected: bool, now: i64) -> bool {
    match last {
        Some((k, at)) if now - at < 700 || (k == connected && now - at < 1500) => false,
        _ => true,
    }
}

static LAST_DEVICE: std::sync::Mutex<Option<(bool, i64)>> = std::sync::Mutex::new(None);

pub fn device_event(app: &tauri::AppHandle, connected: bool) {
    use tauri::{Emitter, Manager};
    let s = {
        let st = app.state::<crate::AppState>();
        let c = st.config.lock().unwrap();
        if c.dnd { return; }
        c.device_sounds.clone()
    };
    if !(if connected { s.connect } else { s.disconnect }) { return; }
    let now = crate::db::now_ms();
    let mut last = LAST_DEVICE.lock().unwrap();
    if !should_play(*last, connected, now) { return; }
    *last = Some((connected, now));
    let kind = if connected { "connect" } else { "disconnect" };
    let _ = app.emit_to("bar", "sound:play", serde_json::json!({ "kind": kind, "volume": s.volume }));
}

#[cfg(test)]
mod tests {
    use super::should_play;
    #[test]
    fn device_sounds_debounce_bursts() {
        assert!(should_play(None, true, 0));
        assert!(!should_play(Some((true, 0)), true, 1000), "same kind within 1.5 s");
        assert!(should_play(Some((true, 0)), true, 1600));
        assert!(!should_play(Some((true, 0)), false, 500), "anything within 0.7 s");
        assert!(should_play(Some((true, 0)), false, 800), "unplug shortly after plug-in still sounds");
    }
}
