//! IPU (Indeks Pencemaran Udara): latest hourly reading from the DOE station
//! nearest the user (DOE APIMS public map service). Outside Malaysia, or when
//! DOE is down, Open-Meteo's US AQI, labelled as such.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::get_json;
use crate::widgets::location::{self, Place};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

const TTL_MS: i64 = 15 * 60_000;
/// Farther than this from any DOE station = use the worldwide AQI instead.
const MAX_STATION_KM: f64 = 150.0;
const DOE_URL: &str = "https://eqms.doe.gov.my/api3/publicmapproxy/PUBLIC_DISPLAY/CAQM_MCAQM_Current_Reading/MapServer/0/query\
    ?where=1%3D1&outFields=STATION_ID,STATION_LOCATION,STATE_NAME,API,CLASS,PARAM_SELECTED,DATETIME,LATITUDE,LONGITUDE\
    &returnGeometry=false&f=json";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Band { Good, Moderate, Unhealthy, VeryUnhealthy, Hazardous }

/// Malaysian IPU bands.
pub fn band(v: f64) -> Band {
    match v {
        v if v <= 50.0 => Band::Good,
        v if v <= 100.0 => Band::Moderate,
        v if v <= 200.0 => Band::Unhealthy,
        v if v <= 300.0 => Band::VeryUnhealthy,
        _ => Band::Hazardous,
    }
}

/// Alert levels: >100 and >200. Returns the level newly entered, if any.
pub fn crossing(prev: Option<Band>, now: Band) -> Option<Band> {
    let level = |b: Band| match b { Band::Good | Band::Moderate => 0, Band::Unhealthy => 1, _ => 2 };
    let (p, n) = (prev.map(level).unwrap_or(0), level(now));
    match n {
        n if n > p && n == 1 => Some(Band::Unhealthy),
        n if n > p && n == 2 => Some(Band::VeryUnhealthy),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Station {
    pub id: String, pub name: String, pub state: String, pub lat: f64, pub lon: f64,
    pub api: Option<f64>, pub class: String, pub pollutant: String,
    /// Reading time, Unix ms.
    pub at: i64,
}

pub fn parse_doe(v: &Value) -> Vec<Station> {
    v["features"].as_array().map(|f| f.iter().filter_map(|x| {
        let a = &x["attributes"];
        Some(Station {
            id: a["STATION_ID"].as_str()?.into(),
            name: a["STATION_LOCATION"].as_str()?.into(),
            state: a["STATE_NAME"].as_str().unwrap_or_default().into(),
            lat: a["LATITUDE"].as_f64()?,
            lon: a["LONGITUDE"].as_f64()?,
            api: a["API"].as_f64(),
            class: a["CLASS"].as_str().unwrap_or_default().into(),
            pollutant: a["PARAM_SELECTED"].as_str().unwrap_or_default().into(),
            // DOE stores Malaysia local time (UTC+8) as if it were UTC.
            at: a["DATETIME"].as_i64().map(|t| t - 8 * 3_600_000).unwrap_or(0),
        })
    }).collect()).unwrap_or_default()
}

fn km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let (dp, dl) = ((lat2 - lat1).to_radians(), (lon2 - lon1).to_radians());
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    6371.0 * 2.0 * a.sqrt().asin()
}

/// Stations with a reading, nearest first, with distance in km.
pub fn nearest(stations: &[Station], lat: f64, lon: f64) -> Vec<(Station, f64)> {
    let mut v: Vec<(Station, f64)> = stations.iter().filter(|s| s.api.is_some())
        .map(|s| (s.clone(), km(lat, lon, s.lat, s.lon))).collect();
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    v
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IpuReading {
    /// "ipu" (DOE) or "aqi_us" (Open-Meteo fallback)
    pub kind: String,
    pub value: f64,
    pub band: Band,
    pub station: Option<Station>,
    pub distance_km: Option<f64>,
    pub nearby: Vec<(Station, f64)>,
    pub place: Option<Place>,
    pub fetched_at: i64,
    pub offline: bool,
    /// All stations from the last DOE fetch (for the country/state/station browser).
    pub stations: Vec<Station>,
}

fn us_aqi(p: &Place) -> Option<IpuReading> {
    let v = get_json(&format!("https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=us_aqi", p.lat, p.lon)).ok()?;
    let value = v["current"]["us_aqi"].as_f64()?;
    // US AQI has its own scale; reuse the same colour bands as a rough guide.
    Some(IpuReading { kind: "aqi_us".into(), value, band: band(value), station: None, distance_km: None, nearby: vec![],
                      place: Some(p.clone()), fetched_at: now_ms(), offline: false, stations: vec![] })
}

pub enum Choice {
    /// A DOE station (and its distance from the user, if known).
    Station(Station, Option<f64>),
    /// No suitable DOE station: use the worldwide AQI (US) at the user's place.
    Aqi,
}

/// Which source to show. `country`: "auto" (from location), "MY" (Malaysia forced).
pub fn choose(country: &str, pick: Option<&str>, place: Option<(f64, f64)>, stations: &[Station]) -> Choice {
    let dist = |s: &Station| place.map(|(lat, lon)| km(lat, lon, s.lat, s.lon));
    if let Some(s) = pick.and_then(|id| stations.iter().find(|s| s.id == id && s.api.is_some())) {
        return Choice::Station(s.clone(), dist(s));
    }
    let ranked = place.map(|(lat, lon)| nearest(stations, lat, lon)).unwrap_or_default();
    match (ranked.first(), country) {
        (Some((s, d)), _) if *d <= MAX_STATION_KM => Choice::Station(s.clone(), Some(*d)),
        (Some((s, d)), "MY") => Choice::Station(s.clone(), Some(*d)),
        (None, "MY") => stations.iter().filter(|s| s.api.is_some()).min_by(|a, b| a.name.cmp(&b.name))
            .map(|s| Choice::Station(s.clone(), None)).unwrap_or(Choice::Aqi),
        _ => Choice::Aqi,
    }
}

fn fetch(app: &AppHandle) -> Option<IpuReading> {
    let s = app.state::<crate::AppState>().config.lock().unwrap().widget_settings("ipu");
    let country = s["country"].as_str().unwrap_or("auto").to_string();
    let stations = get_json(DOE_URL).map(|v| parse_doe(&v)).unwrap_or_default();
    // Another country: IPU (DOE) only covers Malaysia, so show that city's AQI (US), labelled as such.
    if country == "other" {
        if let (Some(lat), Some(lon)) = (s["city"]["lat"].as_f64(), s["city"]["lon"].as_f64()) {
            let name = s["city"]["name"].as_str().unwrap_or("Selected city").to_string();
            let p = Place { lat, lon, name, source: "manual".into(), tz: None };
            return us_aqi(&p).map(|r| IpuReading { stations, ..r });
        }
    }
    let place = location::current(app, false);
    let pick = s["station"].as_str().filter(|x| !x.is_empty() && *x != "auto");
    let coords = place.as_ref().map(|p| (p.lat, p.lon));
    match choose(if country == "MY" { "MY" } else { "auto" }, pick, coords, &stations) {
        Choice::Station(st, dist) => Some(IpuReading {
            kind: "ipu".into(), value: st.api.unwrap_or(0.0), band: band(st.api.unwrap_or(0.0)),
            nearby: coords.map(|(lat, lon)| nearest(&stations, lat, lon)).unwrap_or_default()
                .into_iter().filter(|(x, _)| x.id != st.id).take(3).collect(),
            station: Some(st), distance_km: dist, place, fetched_at: now_ms(), offline: false, stations,
        }),
        Choice::Aqi => place.as_ref().and_then(us_aqi).map(|r| IpuReading { stations, ..r }),
    }
}

fn alert_if_crossed(app: &AppHandle, r: &IpuReading) {
    if app.state::<crate::AppState>().config.lock().unwrap().widget_settings("ipu")["alerts"] == false { return; }
    let prev: Option<Band> = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let p = kv_get(&c, "ipu:band").and_then(|s| serde_json::from_str(&s).ok());
        kv_set(&c, "ipu:band", &serde_json::to_string(&r.band).unwrap());
        p
    };
    if let Some(level) = crossing(prev, r.band) {
        let what = if r.kind == "ipu" { "IPU" } else { "AQI (US)" };
        let at = r.station.as_ref().map(|s| s.name.as_str()).unwrap_or("your area");
        let title = if level == Band::Unhealthy { "Air quality is unhealthy" } else { "Air quality is very unhealthy" };
        crate::alerts::notify(app, "ipu", title, &format!("{what} {} at {at}. Limit time outdoors.", r.value.round()), false);
    }
}

fn get(app: &AppHandle, force: bool) -> Option<IpuReading> {
    let cached: Option<IpuReading> = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        kv_get(&c, "ipu").and_then(|s| serde_json::from_str(&s).ok())
    };
    if let Some(r) = &cached { if !force && now_ms() - r.fetched_at < TTL_MS { return Some(r.clone()); } }
    match fetch(app) {
        Some(r) => {
            { let db = app.state::<Db>(); kv_set(&db.0.lock().unwrap(), "ipu", &serde_json::to_string(&r).unwrap()); }
            alert_if_crossed(app, &r);
            Some(r)
        }
        None => cached.map(|r| IpuReading { offline: true, ..r }),
    }
}

#[tauri::command]
pub async fn ipu_get(app: AppHandle, force: bool) -> Option<IpuReading> {
    crate::widgets::blocking(move || get(&app, force)).await.ok().flatten()
}

/// Background refresh so haze alerts arrive even when no popup is open.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        if app.state::<crate::AppState>().config.lock().unwrap().uses("ipu") { let _ = get(&app, false); }
        std::thread::sleep(std::time::Duration::from_millis(TTL_MS as u64));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOE: &str = include_str!("../../tests/fixtures/doe-apims.json");

    fn st(name: &str, lat: f64, lon: f64, api: Option<f64>) -> Station {
        Station { id: name.into(), name: name.into(), state: String::new(), lat, lon, api, class: String::new(), pollutant: String::new(), at: 0 }
    }

    #[test]
    fn choose_respects_country_and_station() {
        let s = parse_doe(&serde_json::from_str(DOE).unwrap());
        let kl = Some((3.139, 101.687));
        let london = Some((51.5, -0.12));
        // auto: nearest DOE station when close, worldwide AQI when far from Malaysia
        assert!(matches!(choose("auto", None, kl, &s), Choice::Station(_, Some(d)) if d < 15.0));
        assert!(matches!(choose("auto", None, london, &s), Choice::Aqi));
        // Malaysia chosen explicitly: always a DOE station, even from far away / no location
        assert!(matches!(choose("MY", None, london, &s), Choice::Station(..)));
        assert!(matches!(choose("MY", None, None, &s), Choice::Station(..)));
        // a picked station wins
        assert!(matches!(choose("MY", Some("CA16W"), london, &s), Choice::Station(st, _) if st.id == "CA16W"));
        // unknown station id falls back to nearest
        assert!(matches!(choose("MY", Some("nope"), kl, &s), Choice::Station(..)));
    }

    #[test]
    fn band_edges() {
        use Band::*;
        let got: Vec<Band> = [0.0, 50.0, 51.0, 100.0, 101.0, 200.0, 201.0, 300.0, 301.0].map(band).to_vec();
        assert_eq!(got, [Good, Good, Moderate, Moderate, Unhealthy, Unhealthy, VeryUnhealthy, VeryUnhealthy, Hazardous]);
    }

    #[test]
    fn parse_doe_reads_stations_and_fixes_local_time() {
        let s = parse_doe(&serde_json::from_str(DOE).unwrap());
        assert_eq!(s.len(), 68);
        let cheras = s.iter().find(|x| x.id == "CA16W").unwrap();
        assert_eq!(cheras.pollutant, "PM2.5");
        // DOE stores Malaysia local time as if it were UTC: 16:00 MYT = 08:00 UTC.
        assert_eq!(cheras.at, 1792771200000 - 8 * 3_600_000);
    }

    #[test]
    fn nearest_orders_by_distance() {
        let s = parse_doe(&serde_json::from_str(DOE).unwrap());
        let n = nearest(&s, 3.139, 101.687); // central Kuala Lumpur
        assert!(n[0].1 < 15.0, "nearest station should be within 15 km, got {}", n[0].1);
        assert!(n.windows(2).all(|w| w[0].1 <= w[1].1));
    }

    #[test]
    fn nearest_skips_null_api() {
        let n = nearest(&[st("off", 3.0, 101.0, None), st("on", 5.0, 100.0, Some(40.0))], 3.0, 101.0);
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].0.name, "on");
    }

    #[test]
    fn crossing_fires_on_entry_only() {
        use Band::*;
        assert_eq!(crossing(Some(Moderate), Unhealthy), Some(Unhealthy));
        assert_eq!(crossing(Some(Unhealthy), Unhealthy), None);
        assert_eq!(crossing(Some(Unhealthy), VeryUnhealthy), Some(VeryUnhealthy));
        assert_eq!(crossing(Some(VeryUnhealthy), Moderate), None);
        assert_eq!(crossing(None, Hazardous), Some(VeryUnhealthy));
        assert_eq!(crossing(Some(Good), Moderate), None);
    }
}
