//! Where the user is, for Weather, Waktu Solat and IPU.
//! Order: manual (when auto is off) → Windows location → IP lookup. Cached 6 h.
use crate::config::LocationCfg;
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::{enc, get_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

const TTL_MS: i64 = 6 * 3_600_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Place {
    pub lat: f64,
    pub lon: f64,
    pub name: String,
    /// "windows" | "ip" | "manual"
    pub source: String,
    /// IANA time zone, when known (city search results include it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
}

pub fn resolve_location(cfg: &LocationCfg, windows: Option<(f64, f64)>, ip: Option<Place>) -> Option<Place> {
    if !cfg.auto {
        return match (cfg.lat, cfg.lon) {
            (Some(lat), Some(lon)) => Some(Place { lat, lon, name: cfg.name.clone(), source: "manual".into(), tz: None }),
            _ => None,
        };
    }
    match windows {
        Some((lat, lon)) => Some(Place { lat, lon, name: "Current location".into(), source: "windows".into(), tz: None }),
        None => ip,
    }
}

/// ipwho.is (primary) or ipapi.co (backup) response -> Place.
pub fn parse_ip(v: &Value) -> Option<Place> {
    if v["success"] == false || v["error"] == true { return None; }
    let lat = v["latitude"].as_f64()?;
    let lon = v["longitude"].as_f64()?;
    let city = v["city"].as_str().unwrap_or_default();
    let region = v["region"].as_str().unwrap_or_default();
    let name = [city, region].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(", ");
    Some(Place { lat, lon, name, source: "ip".into(), tz: None })
}

fn ip_lookup() -> Option<Place> {
    ["https://ipwho.is/", "https://ipapi.co/json/"].iter().find_map(|u| get_json(u).ok().as_ref().and_then(parse_ip))
}

#[derive(Serialize, Deserialize)]
struct Cached { place: Place, at: i64 }

/// Current place (blocking: may call Windows location and the network).
pub fn current(app: &AppHandle, force: bool) -> Option<Place> {
    let cfg = app.state::<crate::AppState>().config.lock().unwrap().location.clone();
    if !cfg.auto { return resolve_location(&cfg, None, None); }
    let cached: Option<Cached> = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        kv_get(&c, "place").and_then(|s| serde_json::from_str(&s).ok())
    };
    if let Some(c) = &cached {
        if !force && now_ms() - c.at < TTL_MS { return Some(c.place.clone()); }
    }
    let win = crate::platform::location().locate().ok();
    let place = resolve_location(&cfg, win, if win.is_none() { ip_lookup() } else { None });
    match place {
        Some(p) => {
            let db = app.state::<Db>();
            kv_set(&db.0.lock().unwrap(), "place", &serde_json::to_string(&Cached { place: p.clone(), at: now_ms() }).unwrap());
            Some(p)
        }
        None => cached.map(|c| c.place), // offline: last known place
    }
}

#[tauri::command]
pub async fn location_get(app: AppHandle, force: bool) -> Option<Place> {
    crate::widgets::blocking(move || current(&app, force)).await.ok().flatten()
}

/// City search for the manual location picker (Open-Meteo geocoding).
#[tauri::command]
pub async fn location_search(query: String) -> Result<Vec<Place>, String> {
    if query.trim().len() < 2 { return Ok(vec![]); }
    crate::widgets::blocking(move || {
        let v = get_json(&format!("https://geocoding-api.open-meteo.com/v1/search?count=6&name={}", enc(query.trim())))?;
        Ok(v["results"].as_array().map(|a| a.iter().filter_map(|r| {
            let name = [r["name"].as_str(), r["admin1"].as_str(), r["country"].as_str()]
                .iter().flatten().cloned().collect::<Vec<_>>().join(", ");
            Some(Place { lat: r["latitude"].as_f64()?, lon: r["longitude"].as_f64()?, name, source: "manual".into(),
                         tz: r["timezone"].as_str().map(String::from) })
        }).collect()).unwrap_or_default())
    }).await?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip() -> Place { Place { lat: 3.08, lon: 101.58, name: "Petaling".into(), source: "ip".into(), tz: None } }

    #[test]
    fn resolve_prefers_manual_when_auto_off() {
        let cfg = LocationCfg { auto: false, lat: Some(1.5), lon: Some(103.7), name: "Johor Bahru".into() };
        let p = resolve_location(&cfg, Some((3.0, 101.0)), Some(ip())).unwrap();
        assert_eq!((p.lat, p.name.as_str(), p.source.as_str()), (1.5, "Johor Bahru", "manual"));
    }

    #[test]
    fn resolve_uses_windows_then_ip() {
        let cfg = LocationCfg::default();
        assert_eq!(resolve_location(&cfg, Some((3.0, 101.0)), Some(ip())).unwrap().source, "windows");
        assert_eq!(resolve_location(&cfg, None, Some(ip())).unwrap().source, "ip");
    }

    #[test]
    fn resolve_falls_back_to_none() {
        assert!(resolve_location(&LocationCfg::default(), None, None).is_none());
        let manual_without_coords = LocationCfg { auto: false, ..Default::default() };
        assert!(resolve_location(&manual_without_coords, Some((3.0, 101.0)), None).is_none());
    }

    #[test]
    fn parse_ipwho() {
        let v = serde_json::json!({"success": true, "city": "Petaling", "region": "Selangor", "latitude": 3.08, "longitude": 101.58});
        assert_eq!(parse_ip(&v).unwrap().name, "Petaling, Selangor");
        assert!(parse_ip(&serde_json::json!({"success": false})).is_none());
    }
}
