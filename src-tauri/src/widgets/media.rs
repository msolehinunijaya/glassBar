use super::blocking;
use crate::platform::{media, MediaAction, MediaInfo, Res};

#[tauri::command]
pub async fn media_info() -> Res<Option<MediaInfo>> { blocking(move || media().info()).await? }
#[tauri::command]
pub async fn media_control(action: MediaAction) -> Res<()> { blocking(move || media().control(action)).await? }
#[tauri::command]
pub async fn media_seek(position_ms: i64) -> Res<()> { blocking(move || media().seek(position_ms)).await? }
#[tauri::command]
pub async fn media_pause_playing() -> Res<Vec<String>> { blocking(move || media().pause_playing()).await? }
#[tauri::command]
pub async fn media_resume(ids: Vec<String>) -> Res<()> { blocking(move || media().resume(&ids)).await? }

/// The app that is playing, for the music widget: a friendly name, and the window and program
/// behind it (for its icon, and to bring it to the front).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source { name: String, id: String, exe: Option<String>, window: Option<isize>, aumid: Option<String> }

/// Pure: does this window belong to the media session's app? Store and web apps carry the
/// same app id; programs report their exe ("Spotify.exe") or its name ("Chrome", "MSEdge").
pub fn source_matches(id: &str, exe: &str, window_aumid: Option<&str>) -> bool {
    let id = id.trim().to_lowercase();
    if window_aumid.is_some_and(|a| a.to_lowercase() == id) { return true; }
    let file = exe.rsplit(['\\', '/']).next().unwrap_or("").to_lowercase();
    let stem = file.strip_suffix(".exe").unwrap_or(&file);
    !file.is_empty() && (file == id || stem == id || stem == id.strip_suffix(".exe").unwrap_or(&id))
}

/// Pure: a name people know for the app.
pub fn source_name(id: &str) -> String {
    let base = id.rsplit('!').next().unwrap_or(id);
    let base = base.strip_suffix(".exe").unwrap_or(base);
    match base.to_lowercase().as_str() {
        "msedge" => "Microsoft Edge".into(),
        "chrome" => "Google Chrome".into(),
        "308046b0af4a39cb" | "firefox" => "Firefox".into(),
        "brave" => "Brave".into(),
        "opera" => "Opera".into(),
        "spotify" => "Spotify".into(),
        "microsoft.zunemusic" => "Media Player".into(),
        _ => base.to_string(),
    }
}

fn find_source() -> Option<Source> {
    let id = media().source_id()?;
    let w = crate::platform::shell().list_windows().into_iter().find(|w| source_matches(&id, &w.exe, w.aumid.as_deref()));
    Some(Source {
        name: source_name(&id),
        exe: w.as_ref().map(|w| w.exe.clone()),
        window: w.as_ref().map(|w| w.id),
        aumid: if id.contains('!') { Some(id.clone()) } else { w.as_ref().and_then(|w| w.aumid.clone()) },
        id,
    })
}

#[tauri::command]
pub async fn media_source() -> Option<Source> { blocking(find_source).await.ok().flatten() }

/// Bring the playing app to the front -- or start it, if it has no window (a Store app).
#[tauri::command]
pub async fn media_source_open() -> Res<()> {
    blocking(|| {
        let s = find_source().ok_or("nothing is playing")?;
        if let Some(w) = s.window { crate::platform::shell().activate(w); return Ok(()); }
        match &s.aumid {
            Some(a) => crate::platform::shell().launch_aumid(a),
            None => crate::platform::shell().launch(&s.id, ""),
        }
    }).await?
}

#[cfg(test)]
mod source_tests {
    use super::{source_matches, source_name};

    #[test]
    fn finds_the_window_of_the_app_that_is_playing() {
        assert!(source_matches("Spotify.exe", r"C:\Users\u\AppData\Roaming\Spotify\Spotify.exe", None));
        assert!(source_matches("Chrome", r"C:\Program Files\Google\Chrome\Application\chrome.exe", None));
        assert!(source_matches("MSEdge", r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", None));
        assert!(source_matches("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic", r"C:\x\Microsoft.Media.Player.exe", Some("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic")));
        assert!(!source_matches("Spotify.exe", r"C:\Program Files\Google\Chrome\Application\chrome.exe", None));
        assert!(!source_matches("Chrome", "", None));
    }

    /// The real playing app on this PC (read only): `cargo test -- --ignored source_live --nocapture`.
    #[test]
    #[ignore]
    fn source_live() {
        let s = super::find_source();
        println!("SOURCE {:?}", s.map(|s| (s.name, s.id, s.exe, s.window, s.aumid)));
    }

    /// Bring the playing app to the front, then say which app is in front:
    /// `cargo test -- --ignored source_open_live --nocapture`.
    #[test]
    #[ignore]
    fn source_open_live() {
        let before = crate::platform::shell().foreground_exe();
        let r = tauri::async_runtime::block_on(super::media_source_open());
        std::thread::sleep(std::time::Duration::from_millis(600));
        println!("OPEN {r:?} | before={before:?} | after={:?}", crate::platform::shell().foreground_exe());
    }

    #[test]
    fn names_the_app_the_way_people_know_it() {
        assert_eq!(source_name("Spotify.exe"), "Spotify");
        assert_eq!(source_name("MSEdge"), "Microsoft Edge");
        assert_eq!(source_name("308046B0AF4A39CB"), "Firefox");
        assert_eq!(source_name("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"), "Media Player");
        assert_eq!(source_name("SomePlayer.exe"), "SomePlayer");
    }
}
