//! Privacy Monitor: camera, microphone, location, screen recording and screen sharing as one
//! report the widget draws. Each resource gets one state from what Windows actually reports.
//! Something that can't be checked is "unknown" or "limited", never "not in use".
use super::blocking;
use crate::platform::{self, AppWindow, CapUse, Capability, PrivDevice, PrivacyFacts};
use serde::Serialize;
use tauri::AppHandle;

/// States: active (in use now, as Windows reports it), inactive (no use detected), blocked
/// (a privacy switch or policy stops apps), disabled (turned off), unavailable (nothing to
/// monitor, or a needed service is off), unknown (can't be checked, or a running app doesn't
/// report it), limited (nothing detected, but detection covers only some apps).
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub key: &'static str,
    pub state: &'static str,
    pub headline: String,
    /// Why, for any state but active / inactive ("" otherwise).
    pub reason: String,
    /// It can't be checked or a service has a problem: the dock icon shows a warning.
    pub attention: bool,
    /// Apps using it now (`verified` false: running, but whether it uses it can't be seen).
    pub apps: Vec<AppUse>,
    /// Apps that used it in the last 7 days, newest first (Windows' own record).
    pub recent: Vec<AppUse>,
    pub devices: Vec<PrivDevice>,
    /// Detail rows: (label, value).
    pub facts: Vec<(String, String)>,
    /// How it's detected, in a sentence.
    pub method: &'static str,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppUse { pub id: String, pub name: String, pub exe: String, pub since: i64, pub until: i64, pub verified: bool, pub window: Option<isize> }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Report { pub resources: Vec<Resource>, pub checked: i64, pub mic_muted: Option<bool> }

const WEEK_MS: i64 = 7 * 24 * 3600 * 1000;
const CAM_METHOD: &str = "Live from Windows' camera service (Media Foundation's sensor activity monitor): which programs are streaming from a camera now. Programs that open a camera without Windows' camera service (some older capture tools) aren't seen. Recent activity is Windows' privacy record.";
const MIC_METHOD: &str = "Live from Windows' audio system (Core Audio recording sessions): which programs are recording from a microphone now, the same thing Windows' microphone indicator shows. Recent activity is Windows' privacy record.";
const FROZEN: &str = "This version of Windows keeps its record of which apps use this in a protected database that only Windows can read.";
const LOCATION_METHOD: &str = "Windows' capability access record for location, the location switches, and the Windows Geolocation Service (lfsvc).";
const CAPTURE_METHOD: &str = "Windows' record of apps that capture the screen through its screen-capture API without the yellow border, plus a short list of apps known not to report it. Other ways of capturing the screen aren't visible to Windows or GlassBar.";

/// Screen capture by these is most likely sharing (meetings, browsers, remote help); by any
/// other app, recording. Windows doesn't say which.
const SHARE_APPS: [&str; 16] = ["teams", "zoom", "webex", "chrome", "msedge", "firefox", "brave", "opera", "vivaldi", "discord",
    "slack", "skype", "whatsapp", "telegram", "anydesk", "teamviewer"];
/// Running apps that capture the screen without telling Windows: shown as unverified.
const UNREPORTED_SHARE: [(&str, &str); 4] = [("zoom.exe", "Zoom"), ("ciscocollabhost.exe", "Webex"), ("anydesk.exe", "AnyDesk"), ("teamviewer.exe", "TeamViewer")];
const UNREPORTED_RECORD: [(&str, &str); 5] = [("obs64.exe", "OBS Studio"), ("obs32.exe", "OBS Studio"), ("bdcam.exe", "Bandicam"), ("camrecorder.exe", "Camtasia Recorder"), ("screentogif.exe", "ScreenToGif")];

fn app(u: &CapUse) -> AppUse { AppUse { id: u.id.clone(), name: u.name.clone(), exe: u.exe.clone(), since: u.since, until: u.until, verified: true, window: None } }

/// In use now, by the store's record. Nothing when that record is frozen: an old "start"
/// with no "stop" there would show as in use forever.
fn active(c: &Capability, keep: impl Fn(&CapUse) -> bool) -> Vec<AppUse> {
    if c.frozen { return vec![]; }
    let mut v: Vec<AppUse> = c.uses.iter().flatten().filter(|u| u.until == 0 && keep(u)).map(app).collect();
    v.sort_by(|a, b| b.since.cmp(&a.since));
    v
}

fn recent(c: &Capability, now: i64, keep: impl Fn(&CapUse) -> bool) -> Vec<AppUse> {
    let mut v: Vec<AppUse> = c.uses.iter().flatten().filter(|u| u.until > 0 && now - u.until < WEEK_MS && keep(u)).map(app).collect();
    v.sort_by(|a, b| b.until.cmp(&a.until));
    v.truncate(5);
    v
}

/// "Teams", "Teams and Zoom", "Teams, Zoom and 2 more".
pub fn names(apps: &[AppUse]) -> String {
    match apps {
        [] => String::new(),
        [a] => a.name.clone(),
        [a, b] => format!("{} and {}", a.name, b.name),
        [a, b, rest @ ..] => format!("{}, {} and {} more", a.name, b.name, rest.len()),
    }
}

fn on_off(v: Option<bool>) -> String { match v { Some(true) => "On", Some(false) => "Off", None => "Not set" }.into() }

fn switch_facts(c: &Capability) -> Vec<(String, String)> {
    let mut f = vec![
        ("Access on this device".into(), on_off(c.device)),
        ("Let apps access".into(), on_off(c.user)),
        ("Let desktop apps access".into(), on_off(c.desktop)),
    ];
    if let Some(p) = c.policy { f.push(("Organisation policy".into(), if p { "Forces access on" } else { "Blocks access" }.into())); }
    f
}

/// Why apps can't use it, when a switch stops them. Policy first: it overrides the rest.
fn switch_block(c: &Capability, what: &str) -> Option<String> {
    if c.policy == Some(false) { return Some(format!("Your organisation's policy blocks apps from using the {what}.")); }
    if c.device == Some(false) { return Some(format!("{} access is off for this device in Windows' privacy settings.", cap(what))); }
    if c.user == Some(false) { return Some(format!("Apps aren't allowed to use the {what} (Windows' privacy settings).")); }
    None
}

fn cap(s: &str) -> String { let mut c = s.chars(); c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default() }

type Verdict = (&'static str, String, String, bool);

/// Camera or microphone.
fn device(key: &'static str, what: &str, c: &Capability, live: &Result<Vec<CapUse>, String>, devs: &Result<Vec<PrivDevice>, String>, muted: Option<bool>, now: i64) -> Resource {
    // Live first; the store's record only when live can't be read and the record still updates.
    let apps = match live {
        Ok(v) => { let mut a: Vec<AppUse> = v.iter().map(app).collect(); a.sort_by(|x, y| y.since.cmp(&x.since)); a }
        Err(_) => active(c, |_| true),
    };
    let checkable = live.is_ok() || (c.uses.is_some() && !c.frozen);
    let mut facts = switch_facts(c);
    facts.push(("Live activity".into(), match live { Ok(_) => "Read from Windows now".into(), Err(e) => format!("Unavailable: {e}") }));
    facts.push(("Devices".into(), match devs {
        Ok(d) => match d.iter().filter(|x| !x.enabled).count() { 0 => format!("{} connected", d.len()), off => format!("{} connected, {off} turned off", d.len()) },
        Err(e) => format!("Couldn't be listed: {e}"),
    }));
    if let Some(m) = muted { facts.push(("Muted".into(), if m { "Yes: apps that record get silence" } else { "No" }.into())); }
    let none_on = devs.as_ref().is_ok_and(|d| !d.is_empty() && d.iter().all(|x| !x.enabled));
    let (state, mut headline, reason, attention): Verdict = if !checkable {
        ("unknown", "Status can't be checked".into(), format!("GlassBar couldn't read {what} activity from Windows, and Windows' own record {}, so it can't tell whether the {what} is in use.",
            if c.frozen { "is in a protected database" } else { "couldn't be read" }), true)
    } else if !apps.is_empty() {
        ("active", format!("In use by {}", names(&apps)), String::new(), false)
    } else if let Some(r) = switch_block(c, what) {
        ("blocked", "Permission denied".into(), r, false)
    } else if devs.as_ref().is_ok_and(|d| d.is_empty()) {
        ("unavailable", format!("No {what} found"), format!("No {what} is connected, or Windows doesn't see one."), false)
    } else if none_on {
        ("disabled", "Turned off".into(), format!("Every {what} is turned off in Windows, so no app can use one."), false)
    } else {
        ("inactive", "No active access detected".into(), String::new(), false)
    };
    if muted == Some(true) && matches!(state, "active" | "inactive") { headline.push_str(" · muted"); }
    Resource { key, state, headline, reason, attention, apps, recent: recent(c, now, |_| true), devices: devs.clone().unwrap_or_default(), facts, method: if key == "mic" { MIC_METHOD } else { CAM_METHOD } }
}

fn location(f: &PrivacyFacts, now: i64) -> Resource {
    let c = &f.location;
    let apps = active(c, |_| true);
    let mut facts = switch_facts(c);
    facts[0].0 = "Location services".into();
    if f.location_policy_off { facts.push(("Location policy".into(), "Location turned off".into())); }
    facts.push(("Windows Geolocation Service".into(), match &f.location_service {
        Ok(Some(s)) => format!("{}{}", if s.running { "Running" } else { "Stopped (Windows starts it when needed)" }, if s.disabled { ", set to Disabled" } else { "" }),
        Ok(None) => "Not installed".into(),
        Err(e) => e.clone(),
    }));
    let svc = f.location_service.as_ref().ok().copied();
    let (state, headline, reason, attention): Verdict = if c.uses.is_none() {
        ("unknown", "Status can't be checked".into(), "Windows' record of which apps use location couldn't be read, so GlassBar can't tell whether it's in use.".into(), true)
    } else if !apps.is_empty() {
        ("active", format!("In use by {}", names(&apps)), String::new(), false)
    } else if f.location_policy_off || c.policy == Some(false) {
        ("blocked", "Turned off by policy".into(), "Your organisation's policy turns location off for apps.".into(), false)
    } else if c.device == Some(false) {
        ("disabled", "Location services are off".into(), "Location is off for this device, so no app can get your location.".into(), false)
    } else if svc == Some(None) {
        ("unavailable", "Location service missing".into(), "The Windows Geolocation Service (lfsvc) isn't installed, so apps can't get your location.".into(), true)
    } else if svc.flatten().is_some_and(|s| s.disabled) {
        ("unavailable", "Location service is disabled".into(),
         "The Windows Geolocation Service (lfsvc) is set to Disabled, so apps can't get your location even though location is on. An administrator can set it back to Manual in Services.".into(), true)
    } else if c.user == Some(false) {
        ("blocked", "Permission denied".into(), "Apps aren't allowed to use your location (Windows' privacy settings).".into(), false)
    } else if c.frozen {
        ("unknown", "On · app activity can't be seen".into(),
         format!("{FROZEN} Windows has no other way for apps to see who uses location, so GlassBar can't tell whether an app is using it now. Location services, permissions and the location service shown here are current."), false)
    } else {
        ("inactive", "On · no active request detected".into(), String::new(), false)
    };
    Resource { key: "location", state, headline, reason, attention, apps, recent: recent(c, now, |_| true), devices: vec![], facts, method: LOCATION_METHOD }
}

fn is_share(u: &CapUse) -> bool {
    let h = format!("{} {} {}", u.id, u.name, u.exe).to_lowercase();
    SHARE_APPS.iter().any(|a| h.contains(a))
}

/// Screen recording (`sharing` false) or screen sharing (true): Windows records only one
/// thing, a screen capture; the app that does it decides which card it lands on.
fn capture(sharing: bool, f: &PrivacyFacts, now: i64) -> Resource {
    let c = &f.capture;
    let keep = |u: &CapUse| is_share(u) == sharing;
    let mut apps = active(c, keep);
    let list: &[(&str, &str)] = if sharing { &UNREPORTED_SHARE } else { &UNREPORTED_RECORD };
    let mut unverified: Vec<AppUse> = vec![];
    for (exe, name) in list {
        if f.running.iter().any(|r| r == exe) && !unverified.iter().any(|u| u.name == *name) {
            unverified.push(AppUse { id: exe.to_string(), name: name.to_string(), exe: String::new(), since: 0, until: 0, verified: false, window: None });
        }
    }
    let what = if sharing { "sharing" } else { "recording" };
    let mut facts = vec![("Screen capture permission (Store apps)".into(), on_off(c.user))];
    let mut checked: Vec<&str> = list.iter().map(|(_, n)| *n).collect();
    checked.dedup();
    facts.push(("Also checked (they don't report to Windows)".into(), checked.join(", ")));
    let (state, headline, reason, attention): Verdict = if !apps.is_empty() {
        ("active", format!("{} {} capturing your screen", names(&apps), if apps.len() == 1 { "is" } else { "are" }),
         if sharing { "Windows reports a screen capture by a meeting, browser or remote-help app: most likely screen sharing.".into() }
         else { "Windows reports a screen capture in progress.".into() }, false)
    } else if !unverified.is_empty() {
        ("unknown", format!("{} {} running", names(&unverified), if unverified.len() == 1 { "is" } else { "are" }),
         format!("{} doesn't tell Windows when it captures the screen, so GlassBar can't see whether {what} is happening.", names(&unverified)), false)
    } else if c.uses.is_none() || c.frozen {
        ("unknown", "Screen capture can't be seen".into(),
         if c.frozen { format!("{FROZEN} Windows has no other way for apps to see screen capture, so GlassBar can only check for the apps listed below.") }
         else { "This version of Windows doesn't keep a record of screen capture, so GlassBar can only check for the apps listed below.".into() }, false)
    } else {
        ("limited", format!("No screen {what} detected"),
         "Detection is limited: Windows only reports apps that capture the screen through its own API without the yellow border (for example Snipping Tool, Teams and Edge). Other methods aren't visible.".into(), false)
    };
    apps.extend(unverified);
    Resource { key: if sharing { "sharing" } else { "recording" }, state, headline, reason, attention, apps, recent: recent(c, now, keep), devices: vec![], facts, method: CAPTURE_METHOD }
}

pub fn build(f: &PrivacyFacts, now: i64) -> Report {
    Report {
        resources: vec![
            device("camera", "camera", &f.camera, &f.camera_live, &f.cameras, None, now),
            device("mic", "microphone", &f.mic, &f.mic_live, &f.mics, f.mic_muted, now),
            location(f, now),
            capture(false, f, now),
            capture(true, f, now),
        ],
        checked: now,
        mic_muted: f.mic_muted,
    }
}

/// The window of an app, so the panel can bring it forward (to stop sharing there).
fn window_for(a: &AppUse, wins: &[AppWindow]) -> Option<isize> {
    wins.iter().find(|w| {
        if !a.exe.is_empty() { w.exe.eq_ignore_ascii_case(&a.exe) }
        else if !a.verified { w.exe.to_lowercase().ends_with(&format!("\\{}", a.id)) }
        else { w.aumid.as_deref().is_some_and(|m| m.to_lowercase().starts_with(&format!("{}!", a.id.to_lowercase()))) }
    }).map(|w| w.id)
}

#[tauri::command]
pub async fn privacy_report() -> Result<Report, String> {
    blocking(|| {
        let mut r = build(&platform::privacy().facts(), crate::db::now_ms());
        let wins = platform::shell().list_windows();
        for a in r.resources.iter_mut().flat_map(|r| r.apps.iter_mut()) { a.window = window_for(a, &wins); }
        r
    }).await
}

/// Turn a camera or microphone on or off for every app (Windows asks for admin first), then
/// check it really changed. Ok = verified.
#[tauri::command]
pub async fn privacy_set_device(instance: String, enabled: bool) -> Result<String, String> {
    blocking(move || {
        let p = platform::privacy();
        let word = if enabled { "on" } else { "off" };
        match p.device_enabled(&instance) {
            None => return Err("That device isn't connected any more.".into()),
            Some(now) if now == enabled => return Ok(format!("It was already turned {word}.")),
            _ => {}
        }
        let code = p.set_device_enabled(&instance, enabled)?;
        for _ in 0..15 {
            if p.device_enabled(&instance) == Some(enabled) { return Ok(format!("Turned {word}. Windows confirms the change.")); }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        Err(if code == 3010 { format!("Windows will finish turning it {word} after a restart.") }
            else { format!("Windows accepted the request, but the device doesn't report being {word}.") })
    }).await?
}

/// Device Manager or Services (only those two).
#[tauri::command]
pub fn privacy_open_tool(tool: String) -> Result<(), String> {
    if !["devmgmt.msc", "services.msc"].contains(&tool.as_str()) { return Err("not a known tool".into()); }
    platform::shell().launch(&tool, "")
}

/// A privacy notification, at most once per `key` in 10 s (every dock asks).
#[tauri::command]
pub fn privacy_notify(app: AppHandle, key: String, title: String, body: String) {
    use std::time::{Duration, Instant};
    static LAST: std::sync::Mutex<Vec<(String, Instant)>> = std::sync::Mutex::new(Vec::new());
    {
        let mut last = LAST.lock().unwrap();
        last.retain(|(_, t)| t.elapsed() < Duration::from_secs(10));
        if last.iter().any(|(k, _)| *k == key) { return; }
        last.push((key, Instant::now()));
    }
    crate::alerts::notify(&app, "privacy", &title, &body, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Service;

    fn u(id: &str, since: i64, until: i64) -> CapUse { CapUse { id: id.into(), name: id.into(), exe: String::new(), since, until } }
    fn cap(uses: Vec<CapUse>) -> Capability { Capability { uses: Some(uses), frozen: false, user: Some(true), device: Some(true), desktop: Some(true), policy: None } }
    fn dev(on: bool) -> PrivDevice { PrivDevice { id: "d".into(), instance: "USB\\X".into(), name: "Cam".into(), enabled: on, default: false } }
    fn facts() -> PrivacyFacts {
        PrivacyFacts { camera: cap(vec![]), mic: cap(vec![]), location: cap(vec![]), capture: cap(vec![]), camera_live: Ok(vec![]), mic_live: Ok(vec![]),
            cameras: Ok(vec![dev(true)]), mics: Ok(vec![dev(true)]), mic_muted: Some(false),
            location_service: Ok(Some(Service { running: false, disabled: false })), location_policy_off: false, running: vec![] }
    }
    fn get<'a>(r: &'a Report, k: &str) -> &'a Resource { r.resources.iter().find(|x| x.key == k).unwrap() }
    const NOW: i64 = 1_000_000_000;

    #[test]
    fn all_quiet() {
        let r = build(&facts(), NOW);
        assert_eq!(get(&r, "camera").state, "inactive");
        assert_eq!(get(&r, "mic").state, "inactive");
        assert_eq!(get(&r, "location").state, "inactive");
        // Screen capture: nothing seen is "limited", never plain inactive.
        assert_eq!(get(&r, "recording").state, "limited");
        assert_eq!(get(&r, "sharing").state, "limited");
        assert!(r.resources.iter().all(|x| !x.attention));
    }

    #[test]
    fn live_activity_wins_and_a_frozen_record_never_shows_use() {
        let mut f = facts();
        f.mic_live = Ok(vec![u("Teams", NOW - 5, 0)]);
        f.camera.uses = Some(vec![u("Stuck", NOW - 99, 0)]);
        f.camera.frozen = true;
        let r = build(&f, NOW);
        assert_eq!(get(&r, "mic").headline, "In use by Teams");
        assert_eq!(get(&r, "camera").state, "inactive"); // live says nobody streams
        // Live unreadable and the record frozen: unknown, never "not in use".
        f.camera_live = Err("no monitor".into());
        let r = build(&f, NOW);
        assert_eq!(get(&r, "camera").state, "unknown");
        assert!(get(&r, "camera").attention);
        // Frozen location / screen capture: unknown (a Windows limit, no dock warning).
        let mut f = facts();
        f.location.frozen = true;
        f.capture.frozen = true;
        let r = build(&f, NOW);
        assert_eq!(get(&r, "location").state, "unknown");
        assert!(!get(&r, "location").attention);
        assert_eq!(get(&r, "recording").state, "unknown");
        assert_eq!(get(&r, "sharing").state, "unknown");
        // ...but a definite "off" still shows as off.
        f.location.device = Some(false);
        assert_eq!(get(&build(&f, NOW), "location").state, "disabled");
    }

    #[test]
    fn in_use_beats_everything_and_names_apps() {
        let mut f = facts();
        f.camera_live = Err("x".into());
        f.camera.uses = Some(vec![u("Zoom", NOW - 10, 0), u("Old", NOW - 1000, NOW - 500)]);
        f.camera.user = Some(false); // a stale switch read can't hide observed use
        let c = build(&f, NOW);
        let c = get(&c, "camera");
        assert_eq!(c.state, "active");
        assert_eq!(c.headline, "In use by Zoom");
        assert_eq!(c.recent.len(), 1);
    }

    #[test]
    fn unreadable_record_is_unknown_not_inactive() {
        let mut f = facts();
        f.mic.uses = None;
        f.mic_live = Err("x".into());
        let r = build(&f, NOW);
        assert_eq!(get(&r, "mic").state, "unknown");
        assert!(get(&r, "mic").attention);
    }

    #[test]
    fn switches_devices_and_mute() {
        let mut f = facts();
        f.camera.policy = Some(false);
        f.mic.device = Some(false);
        let r = build(&f, NOW);
        assert_eq!(get(&r, "camera").state, "blocked");
        assert!(get(&r, "camera").reason.contains("policy"));
        assert_eq!(get(&r, "mic").state, "blocked");

        let mut f = facts();
        f.cameras = Ok(vec![]);
        f.mics = Ok(vec![dev(false), dev(false)]);
        f.mic_muted = Some(true);
        let r = build(&f, NOW);
        assert_eq!(get(&r, "camera").state, "unavailable");
        assert_eq!(get(&r, "mic").state, "disabled");

        let mut f = facts();
        f.mic_muted = Some(true);
        assert_eq!(get(&build(&f, NOW), "mic").headline, "No active access detected · muted");
        // A device list that failed doesn't turn into "no camera".
        f.cameras = Err("boom".into());
        assert_eq!(get(&build(&f, NOW), "camera").state, "inactive");
    }

    #[test]
    fn location_states() {
        let mut f = facts();
        f.location.device = Some(false);
        assert_eq!(get(&build(&f, NOW), "location").state, "disabled");
        let mut f = facts();
        f.location_service = Ok(Some(Service { running: false, disabled: true }));
        let r = build(&f, NOW);
        assert_eq!(get(&r, "location").state, "unavailable");
        assert!(get(&r, "location").attention);
        let mut f = facts();
        f.location_policy_off = true;
        assert_eq!(get(&build(&f, NOW), "location").state, "blocked");
        let mut f = facts();
        f.location.user = Some(false);
        assert_eq!(get(&build(&f, NOW), "location").state, "blocked");
        let mut f = facts();
        f.location.uses = Some(vec![u("Maps", NOW - 5, 0)]);
        assert_eq!(get(&build(&f, NOW), "location").state, "active");
    }

    #[test]
    fn screen_capture_split_by_app() {
        let mut f = facts();
        f.capture.uses = Some(vec![u("MSTeams_8wekyb3d8bbwe", NOW - 5, 0), u("Microsoft.ScreenSketch_8wekyb3d8bbwe", NOW - 9, 0)]);
        let r = build(&f, NOW);
        assert_eq!(get(&r, "sharing").state, "active");
        assert_eq!(get(&r, "sharing").apps.len(), 1);
        assert_eq!(get(&r, "recording").state, "active");
        assert!(get(&r, "recording").apps[0].id.contains("ScreenSketch"));
    }

    #[test]
    fn unreported_apps_are_unverified() {
        let mut f = facts();
        f.running = vec!["obs64.exe".into(), "zoom.exe".into()];
        let r = build(&f, NOW);
        let rec = get(&r, "recording");
        assert_eq!(rec.state, "unknown");
        assert_eq!(rec.headline, "OBS Studio is running");
        assert!(!rec.apps[0].verified);
        assert_eq!(get(&r, "sharing").state, "unknown");
        // No record of screen capture at all (older Windows): unknown, not "none".
        let mut f = facts();
        f.capture.uses = None;
        assert_eq!(get(&build(&f, NOW), "recording").state, "unknown");
    }

    #[test]
    fn joins_names() {
        let a = |n: &str| AppUse { id: n.into(), name: n.into(), exe: String::new(), since: 0, until: 0, verified: true, window: None };
        assert_eq!(names(&[a("A")]), "A");
        assert_eq!(names(&[a("A"), a("B")]), "A and B");
        assert_eq!(names(&[a("A"), a("B"), a("C"), a("D")]), "A, B and 2 more");
    }

    #[test]
    fn finds_windows() {
        let w = |exe: &str, aumid: Option<&str>| AppWindow { id: 7, title: String::new(), exe: exe.into(), active: false, minimized: false, aumid: aumid.map(Into::into) };
        let teams = AppUse { id: "MSTeams_8wekyb3d8bbwe".into(), name: "MS Teams".into(), exe: String::new(), since: 0, until: 0, verified: true, window: None };
        assert_eq!(window_for(&teams, &[w("C:\\x\\ms-teams.exe", Some("MSTeams_8wekyb3d8bbwe!MSTeams"))]), Some(7));
        let obs = AppUse { id: "obs64.exe".into(), verified: false, ..teams.clone() };
        assert_eq!(window_for(&obs, &[w("C:\\OBS\\bin\\64bit\\obs64.exe", None)]), Some(7));
        assert_eq!(window_for(&obs, &[w("C:\\x\\notobs64.exe.bak", None)]), None);
    }
}
