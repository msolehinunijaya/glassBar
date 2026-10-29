//! The one shared location for Weather, sunrise/sunset, Waktu Solat, IPU and the world clock.
//! Modes: auto (Windows Location, then IP) | windows | ip | manual. Never made up: when every
//! source fails the status is Unavailable with the reason, and widgets keep the last place
//! that worked (if there is one) until a check succeeds again.
//! Re-checked on start, every 30 min (auto/windows), on wake and on a network change; widgets
//! hear `location:changed` only when the place really changes (new name, or moved > 1 km).
use crate::config::LocationCfg;
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::{enc, get_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const RECHECK_MS: i64 = 30 * 60_000;
/// Retry an Unavailable result this soon (it isn't a known place, so nothing is reloaded).
const RETRY_MS: i64 = 5 * 60_000;
const KEY: &str = "location";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Place {
    pub lat: f64,
    pub lon: f64,
    /// "Town, State, Country"; empty when no source could name it.
    pub name: String,
    /// "windows" | "ip" | "manual"
    pub source: String,
    /// IANA time zone, when the source gave one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    /// Accuracy radius in metres, exactly as the source reported it (Windows only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<f64>,
    /// How Windows found it ("GPS", "Wi-Fi"…).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    /// Why Automatic fell back to IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub mode: String,
    /// None = Unavailable.
    pub place: Option<Place>,
    /// Why it is Unavailable.
    pub reason: Option<String>,
    /// The last place that worked. Widgets use it while the latest check failed.
    pub last_known: Option<Place>,
    /// When this was checked (ms).
    pub at: i64,
}

/// ~11 m grid.
pub fn round4(x: f64) -> f64 { (x * 10_000.0).round() / 10_000.0 }

pub fn valid_lat_lon(lat: f64, lon: f64) -> bool {
    lat.is_finite() && lon.is_finite() && (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon)
}

/// What widgets use: the place just found, else the last one that worked (after sleep or
/// before the network is up a check can fail; that must not blank every widget).
pub fn usable(s: &Status) -> Option<&Place> { s.place.as_ref().or(s.last_known.as_ref()) }

/// Widgets reload only for this: a place appeared or vanished, got a new name, or moved > 1 km.
pub fn is_real_change(old: Option<&Place>, new: Option<&Place>) -> bool {
    match (old, new) {
        (None, None) => false,
        (Some(a), Some(b)) => a.name != b.name || super::ipu::km(a.lat, a.lon, b.lat, b.lon) > 1.0,
        _ => true,
    }
}

/// Mode → source order. `windows` and `ip` are only called when the mode needs them.
pub fn resolve(cfg: &LocationCfg, windows: impl FnOnce() -> Result<Place, String>, ip: impl FnOnce() -> Result<Place, String>) -> Result<Place, String> {
    match cfg.mode.as_str() {
        "manual" => match (cfg.lat, cfg.lon) {
            (Some(lat), Some(lon)) if valid_lat_lon(lat, lon) => Ok(Place {
                lat, lon, name: cfg.name.clone(), source: "manual".into(), tz: cfg.tz.clone(), ..Default::default()
            }),
            _ => Err("No manual location set yet. Search for a city or enter coordinates.".into()),
        },
        "windows" => windows(),
        "ip" => ip(),
        _ => windows().or_else(|we| match ip() {
            Ok(p) => Ok(Place { note: Some(format!("Using IP because {}", lower_first(&we))), ..p }),
            Err(ie) => Err(format!("{we}. IP lookup also failed: {ie}.")),
        }),
    }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    // Keep proper names ("Windows Location") as they are.
    match c.next() { Some(f) if !s.starts_with("Windows") && !s.starts_with("IP") => f.to_lowercase().chain(c).collect(), _ => s.to_string() }
}

pub fn from_fix(f: crate::platform::Fix, name: Option<String>) -> Place {
    Place {
        lat: round4(f.lat), lon: round4(f.lon), name: name.unwrap_or_default(), source: "windows".into(),
        accuracy: f.accuracy_m.filter(|a| a.is_finite() && *a > 0.0).map(f64::round), via: Some(f.via.into()), ..Default::default()
    }
}

/// ipwho.is (primary) or ipapi.co (backup) response -> Place.
pub fn parse_ip(v: &Value) -> Option<Place> {
    if v["success"] == false || v["error"] == true { return None; }
    let lat = v["latitude"].as_f64()?;
    let lon = v["longitude"].as_f64()?;
    let country = v["country"].as_str().filter(|c| c.len() > 2).or(v["country_name"].as_str());
    let name = [v["city"].as_str(), v["region"].as_str(), country].into_iter().flatten()
        .filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
    let tz = v["timezone"]["id"].as_str().or(v["timezone"].as_str()).map(String::from);
    Some(Place { lat: round4(lat), lon: round4(lon), name, source: "ip".into(), tz, ..Default::default() })
}

fn ip_lookup() -> Result<Place, String> {
    ["https://ipwho.is/", "https://ipapi.co/json/"].iter()
        .find_map(|u| get_json(u).ok().as_ref().and_then(parse_ip))
        .ok_or_else(|| "no answer from the IP location services (offline?)".into())
}

/// Run `f` on its own thread and wait at most `limit`. A call that hangs is left to finish on
/// its own; until it does, new calls fail at once instead of starting another. Windows
/// Location can stop answering (after sleep, while networks and screens change), and its
/// access request has no timeout of its own.
pub fn with_deadline<T: Send + 'static>(running: &'static AtomicBool, limit: Duration, f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    if running.swap(true, Ordering::SeqCst) { return Err("Windows Location is still busy".into()); }
    let (tx, rx) = channel();
    std::thread::spawn(move || { let r = f(); running.store(false, Ordering::SeqCst); let _ = tx.send(r); });
    rx.recv_timeout(limit).unwrap_or_else(|_| Err("Windows Location didn't answer".into()))
}

static WINDOWS_BUSY: AtomicBool = AtomicBool::new(false);

fn windows_lookup() -> Result<Place, String> {
    with_deadline(&WINDOWS_BUSY, Duration::from_secs(20), || crate::platform::location().locate().map(|f| from_fix(f, None)))
}

/// OpenStreetMap (Nominatim) address -> "Town, State, Country".
pub fn parse_osm(v: &Value) -> Option<String> {
    let a = &v["address"];
    let town = ["city", "town", "village", "suburb", "municipality", "district", "county"].iter().find_map(|k| a[k].as_str());
    let parts: Vec<&str> = [town, a["state"].as_str(), a["country"].as_str()].into_iter().flatten().filter(|s| !s.is_empty()).collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// City-level name for a Windows fix (Windows gives coordinates only).
fn osm_name(lat: f64, lon: f64) -> Option<String> {
    let url = format!("https://nominatim.openstreetmap.org/reverse?format=jsonv2&zoom=14&accept-language=en&lat={lat}&lon={lon}");
    get_json(&url).ok().as_ref().and_then(parse_osm)
}

/// Give an unnamed place a name: the previous one when it is within 1 km (no lookup),
/// else `lookup`. Left empty when nothing answers; never guessed.
pub fn name_place(p: &mut Place, prev: Option<&Place>, lookup: impl FnOnce(f64, f64) -> Option<String>) {
    if !p.name.is_empty() { return; }
    match prev.filter(|q| !q.name.is_empty() && super::ipu::km(q.lat, q.lon, p.lat, p.lon) <= 1.0) {
        Some(q) => p.name = q.name.clone(),
        None => p.name = lookup(p.lat, p.lon).unwrap_or_default(),
    }
}

fn read(app: &AppHandle) -> Option<Status> {
    let db = app.state::<Db>();
    let c = db.lock();
    kv_get(&c, KEY).and_then(|s| serde_json::from_str(&s).ok())
}

fn fresh(s: &Status, cfg: &LocationCfg, now: i64) -> bool {
    let age = now - s.at;
    s.mode == cfg.mode && match (cfg.mode.as_str(), &s.place) {
        ("manual", _) => false, // instant, and must follow edits
        (_, None) => age < RETRY_MS,
        ("ip", _) => true, // only start-up and network changes re-check the IP
        _ => age < RECHECK_MS,
    }
}

static BUSY: Mutex<()> = Mutex::new(());

/// What callers get while another check is running: the cached status, or "still finding".
/// They never wait for it: widgets poll the location, and each waiting call used to hold a
/// worker thread, so one stuck check starved the dock (no app icons, dead widgets).
pub fn while_busy(cached: Option<Status>, cfg: &LocationCfg, now: i64) -> Status {
    cached.unwrap_or_else(|| Status { mode: cfg.mode.clone(), reason: Some("Still finding your location".into()), at: now, ..Default::default() })
}

/// The current status; checks the sources when the cached one is stale (or `force`).
/// Blocking: may wait for Windows Location and the network. Only one check runs at a time.
pub fn status(app: &AppHandle, force: bool) -> Status {
    let cfg = app.state::<crate::AppState>().config.lock().unwrap().location.clone();
    let _one_at_a_time = match BUSY.try_lock() {
        Ok(g) => g,
        Err(TryLockError::Poisoned(p)) => p.into_inner(),
        Err(TryLockError::WouldBlock) => return while_busy(read(app), &cfg, now_ms()),
    };
    let old = read(app);
    if let Some(s) = old.as_ref().filter(|s| !force && fresh(s, &cfg, now_ms())) { return s.clone(); }
    let mut r = resolve(&cfg, windows_lookup, ip_lookup);
    if let Ok(p) = r.as_mut() {
        let prev = old.as_ref().and_then(|o| o.place.as_ref().or(o.last_known.as_ref()));
        name_place(p, prev, osm_name);
    }
    let last_known = r.as_ref().ok().cloned().or_else(|| old.as_ref().and_then(|o| o.place.clone().or(o.last_known.clone())));
    let new = Status { mode: cfg.mode.clone(), reason: r.as_ref().err().cloned(), place: r.ok(), last_known, at: now_ms() };
    {
        let db = app.state::<Db>();
        kv_set(&db.lock(), KEY, &serde_json::to_string(&new).unwrap());
    }
    if is_real_change(old.as_ref().and_then(usable), usable(&new)) {
        let _ = app.emit("location:changed", &new);
    }
    let _ = app.emit("location:status", &new);
    new
}

/// The place for widgets (the last known one while a check fails), or why there is none.
pub fn current(app: &AppHandle) -> Result<Place, String> {
    let s = status(app, false);
    usable(&s).cloned().ok_or_else(|| s.reason.unwrap_or_else(|| "Location unavailable".into()))
}

static POKE: OnceLock<Sender<()>> = OnceLock::new();

/// Re-check soon (network change, wake from sleep, location settings changed).
pub fn poke() { if let Some(tx) = POKE.get() { let _ = tx.send(()); } }

pub fn start(app: AppHandle) {
    let (tx, rx) = channel();
    let _ = POKE.set(tx);
    std::thread::spawn(move || {
        status(&app, true);
        loop {
            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(()) => {
                    // Network changes come in bursts; let them settle, then check once.
                    std::thread::sleep(Duration::from_secs(3));
                    while rx.try_recv().is_ok() {}
                    status(&app, true);
                    // Woke up or the network changed: widgets fetch fresh data now, not at their next tick.
                    let _ = app.emit("widgets:refresh", ());
                }
                Err(RecvTimeoutError::Timeout) => { status(&app, false); }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    });
}

#[tauri::command]
pub async fn location_status(app: AppHandle, force: bool) -> Result<Status, String> {
    crate::widgets::blocking(move || status(&app, force)).await
}

/// Forget the cached location and any manual place; back to Automatic.
#[tauri::command]
pub async fn location_clear(app: AppHandle) -> Result<Status, String> {
    {
        let db = app.state::<Db>();
        let c = db.lock();
        kv_set(&c, KEY, "");
        kv_set(&c, "place", ""); // cache key before v0.2
    }
    let mut cfg = app.state::<crate::AppState>().config.lock().unwrap().clone();
    cfg.location = LocationCfg::default();
    crate::commit(&app, cfg)?;
    crate::widgets::blocking(move || status(&app, true)).await
}

/// Show the point on OpenStreetMap in the browser (only when the user asks).
#[tauri::command]
pub fn location_open_map(lat: f64, lon: f64) -> Result<(), String> {
    if !valid_lat_lon(lat, lon) { return Err("bad coordinates".into()); }
    crate::platform::shell().launch(&format!("https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=13/{lat}/{lon}"), "")
}

/// City search for a manual location (Open-Meteo geocoding).
#[tauri::command]
pub async fn location_search(query: String) -> Result<Vec<Place>, String> {
    if query.trim().len() < 2 { return Ok(vec![]); }
    crate::widgets::blocking(move || {
        let v = get_json(&format!("https://geocoding-api.open-meteo.com/v1/search?count=6&name={}", enc(query.trim())))?;
        Ok(v["results"].as_array().map(|a| a.iter().filter_map(|r| {
            let name = [r["name"].as_str(), r["admin1"].as_str(), r["country"].as_str()]
                .iter().flatten().cloned().collect::<Vec<_>>().join(", ");
            Some(Place { lat: round4(r["latitude"].as_f64()?), lon: round4(r["longitude"].as_f64()?), name, source: "manual".into(),
                         tz: r["timezone"].as_str().map(String::from), ..Default::default() })
        }).collect()).unwrap_or_default())
    }).await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Fix;

    fn ip() -> Result<Place, String> { Ok(Place { lat: 3.0812, lon: 101.5834, name: "Petaling, Selangor, Malaysia".into(), source: "ip".into(), ..Default::default() }) }
    fn win() -> Result<Place, String> { Ok(from_fix(Fix { lat: 3.123456, lon: 101.654321, accuracy_m: Some(35.4), via: "Wi-Fi" }, Some("Kuala Lumpur".into()))) }
    fn off() -> Result<Place, String> { Err("Windows Location is turned off for desktop apps".into()) }
    fn down() -> Result<Place, String> { Err("offline".into()) }
    fn never() -> Result<Place, String> { panic!("source should not be asked") }

    #[test]
    fn a_hung_lookup_gives_up_at_the_deadline() {
        static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let t = std::time::Instant::now();
        let r: Result<u8, String> = with_deadline(&RUNNING, Duration::from_millis(200), || { std::thread::sleep(Duration::from_secs(3)); Ok(1) });
        assert!(r.is_err());
        assert!(t.elapsed() < Duration::from_secs(1));
        // While the hung one is still running, another isn't started (no pile-up of threads).
        let again: Result<u8, String> = with_deadline(&RUNNING, Duration::from_secs(5), || Ok(2));
        assert!(again.is_err());
    }

    #[test]
    fn a_quick_lookup_returns_its_result() {
        static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        assert_eq!(with_deadline(&RUNNING, Duration::from_secs(5), || Ok(7)), Ok(7));
        assert_eq!(with_deadline(&RUNNING, Duration::from_secs(5), || Err::<u8, _>("no".to_string())), Err("no".to_string()));
    }

    #[test]
    fn while_a_check_runs_others_get_the_cached_place_at_once() {
        let cfg = LocationCfg::default();
        let cached = Status { mode: cfg.mode.clone(), place: ip().ok(), at: 5, ..Default::default() };
        assert_eq!(while_busy(Some(cached.clone()), &cfg, 99), cached);
        let none = while_busy(None, &cfg, 99);
        assert!(none.place.is_none() && none.reason.is_some());
    }
    #[test]
    fn a_failed_check_keeps_widgets_on_the_last_known_place() {
        let failed = Status { mode: "auto".into(), place: None, reason: Some("offline".into()), last_known: ip().ok(), at: 1 };
        assert_eq!(usable(&failed), ip().ok().as_ref());
        let ok = Status { place: win().ok(), ..failed.clone() };
        assert_eq!(usable(&ok), win().ok().as_ref(), "a fresh place wins over the last known one");
        // Going from working to failed is not a change widgets reload for.
        assert!(!is_real_change(usable(&Status { place: ip().ok(), ..failed.clone() }), usable(&failed)));
        assert_eq!(usable(&Status { last_known: None, ..failed }), None, "nothing known yet: nothing made up");
    }

    fn mode(m: &str) -> LocationCfg { LocationCfg { mode: m.into(), ..Default::default() } }

    #[test]
    fn auto_prefers_windows() {
        let p = resolve(&mode("auto"), win, never).unwrap();
        assert_eq!((p.source.as_str(), p.lat, p.lon), ("windows", 3.1235, 101.6543));
        assert_eq!((p.accuracy, p.via.as_deref(), p.name.as_str()), (Some(35.0), Some("Wi-Fi"), "Kuala Lumpur"));
    }

    #[test]
    fn auto_falls_back_to_ip_and_says_why() {
        let p = resolve(&mode("auto"), off, ip).unwrap();
        assert_eq!(p.source, "ip");
        assert_eq!(p.note.as_deref(), Some("Using IP because Windows Location is turned off for desktop apps"));
        assert_eq!(p.accuracy, None, "IP gives no accuracy, so none is claimed");
    }

    #[test]
    fn auto_is_unavailable_when_both_fail() {
        let e = resolve(&mode("auto"), off, down).unwrap_err();
        assert!(e.contains("turned off") && e.contains("offline"), "{e}");
    }

    #[test]
    fn single_source_modes_never_fall_back() {
        assert!(resolve(&mode("windows"), off, never).is_err());
        assert_eq!(resolve(&mode("ip"), never, ip).unwrap().source, "ip");
        assert!(resolve(&mode("ip"), never, down).is_err());
    }

    #[test]
    fn manual_uses_exactly_what_was_set() {
        let cfg = LocationCfg { mode: "manual".into(), lat: Some(1.4927), lon: Some(103.7414), name: "Johor Bahru".into(), tz: Some("Asia/Kuala_Lumpur".into()), ..Default::default() };
        let p = resolve(&cfg, never, never).unwrap();
        assert_eq!((p.lat, p.lon, p.name.as_str(), p.source.as_str(), p.tz.as_deref()), (1.4927, 103.7414, "Johor Bahru", "manual", Some("Asia/Kuala_Lumpur")));
        assert!(resolve(&mode("manual"), never, never).is_err());
        let bad = LocationCfg { mode: "manual".into(), lat: Some(91.0), lon: Some(0.0), ..Default::default() };
        assert!(resolve(&bad, never, never).is_err());
    }

    #[test]
    fn real_change_is_a_new_name_or_more_than_1_km() {
        let a = ip().unwrap();
        assert!(!is_real_change(Some(&a), Some(&Place { lat: a.lat + 0.005, ..a.clone() })), "~550 m, same name");
        assert!(is_real_change(Some(&a), Some(&Place { lat: a.lat + 0.02, ..a.clone() })), "~2.2 km");
        assert!(is_real_change(Some(&a), Some(&Place { name: "Shah Alam".into(), ..a.clone() })));
        assert!(!is_real_change(Some(&a), Some(&Place { accuracy: Some(20.0), via: Some("GPS".into()), ..a.clone() })));
        assert!(is_real_change(None, Some(&a)) && is_real_change(Some(&a), None) && !is_real_change(None, None));
    }

    #[test]
    fn lat_lon_validation() {
        assert!(valid_lat_lon(90.0, -180.0) && valid_lat_lon(-90.0, 180.0) && valid_lat_lon(0.0, 0.0));
        assert!(!valid_lat_lon(90.1, 0.0) && !valid_lat_lon(0.0, -180.5) && !valid_lat_lon(f64::NAN, 0.0));
    }

    #[test]
    fn parse_ip_services() {
        let v = serde_json::json!({"success": true, "city": "Petaling", "region": "Selangor", "country": "Malaysia",
                                   "latitude": 3.081234, "longitude": 101.58, "timezone": {"id": "Asia/Kuala_Lumpur"}});
        let p = parse_ip(&v).unwrap();
        assert_eq!((p.name.as_str(), p.lat, p.tz.as_deref()), ("Petaling, Selangor, Malaysia", 3.0812, Some("Asia/Kuala_Lumpur")));
        let v = serde_json::json!({"city": "Ipoh", "region": "Perak", "country": "MY", "country_name": "Malaysia",
                                   "latitude": 4.6, "longitude": 101.1, "timezone": "Asia/Kuala_Lumpur"});
        let p = parse_ip(&v).unwrap();
        assert_eq!((p.name.as_str(), p.tz.as_deref()), ("Ipoh, Perak, Malaysia", Some("Asia/Kuala_Lumpur")));
        assert!(parse_ip(&serde_json::json!({"success": false})).is_none());
    }

    /// Live check of the OpenStreetMap lookup: `cargo test osm_name_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn osm_name_live() {
        println!("{:?}", osm_name(3.216, 101.4362));
    }

    #[test]
    fn naming_reuses_a_nearby_name_and_never_guesses() {
        let prev = Place { lat: 3.2160, lon: 101.4362, name: "Kuang, Selangor, Malaysia".into(), ..Default::default() };
        let mut p = win().unwrap();
        p.name.clear();
        p.lat = 3.2190; // ~330 m away
        p.lon = 101.4362;
        name_place(&mut p, Some(&prev), |_, _| panic!("no lookup within 1 km"));
        assert_eq!(p.name, "Kuang, Selangor, Malaysia");
        let mut far = Place { lat: 3.30, ..p.clone() };
        far.name.clear();
        name_place(&mut far, Some(&prev), |_, _| Some("Rawang, Selangor, Malaysia".into()));
        assert_eq!(far.name, "Rawang, Selangor, Malaysia");
        let mut none = Place { lat: 4.0, name: String::new(), ..p };
        name_place(&mut none, None, |_, _| None);
        assert_eq!(none.name, "", "nothing answered: no name, not a guess");
    }

    #[test]
    fn parse_osm_address() {
        let v = serde_json::json!({"address": {"village": "Kuang", "state": "Selangor", "country": "Malaysia", "postcode": "48050"}});
        assert_eq!(parse_osm(&v).as_deref(), Some("Kuang, Selangor, Malaysia"));
        let v = serde_json::json!({"address": {"district": "Kuala Selangor", "state": "Selangor", "country": "Malaysia"}});
        assert_eq!(parse_osm(&v).as_deref(), Some("Kuala Selangor, Selangor, Malaysia"));
        assert_eq!(parse_osm(&serde_json::json!({"error": "Unable to geocode"})), None);
    }

    #[test]
    fn freshness_per_mode() {
        let ok = |m: &str, at| Status { mode: m.into(), place: ip().ok(), at, ..Default::default() };
        let now = 100 * RECHECK_MS;
        assert!(fresh(&ok("auto", now - RECHECK_MS + 1), &mode("auto"), now));
        assert!(!fresh(&ok("auto", now - RECHECK_MS), &mode("auto"), now));
        assert!(fresh(&ok("ip", 0), &mode("ip"), now), "IP only re-checks on start and network change");
        assert!(!fresh(&ok("ip", now), &mode("auto"), now), "a mode switch re-checks");
        assert!(!fresh(&ok("manual", now), &mode("manual"), now));
        let none = Status { mode: "ip".into(), at: now - RETRY_MS, ..Default::default() };
        assert!(!fresh(&none, &mode("ip"), now), "Unavailable is retried");
    }
}
