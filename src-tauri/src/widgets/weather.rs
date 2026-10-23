//! Weather from Open-Meteo (free, no key). Cached 30 min; offline shows the cache.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::get_json;
use crate::widgets::location::{self, Place};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

const TTL_MS: i64 = 30 * 60_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Current {
    pub temp: f64, pub code: i32, pub is_day: bool,
    // Details are optional: older caches don't have them and the API may omit any.
    #[serde(default)] pub feels_like: Option<f64>,
    #[serde(default)] pub humidity: Option<f64>,
    /// km/h, or mph when the unit is °F.
    #[serde(default)] pub wind_speed: Option<f64>,
    /// Degrees the wind comes FROM (0 = north).
    #[serde(default)] pub wind_dir: Option<f64>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hour { pub time: i64, pub temp: f64, pub code: i32, #[serde(default)] pub pop: Option<f64> }
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Day {
    pub date: i64, pub min: f64, pub max: f64, pub code: i32,
    /// Unix ms; None during polar day/night.
    #[serde(default)] pub sunrise: Option<i64>,
    #[serde(default)] pub sunset: Option<i64>,
    /// Max precipitation probability %.
    #[serde(default)] pub pop: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub fetched_at: i64,
    pub unit: String,
    pub place: Option<Place>,
    /// True when this is cached data because the latest fetch failed.
    pub offline: bool,
    pub current: Current,
    pub hourly: Vec<Hour>,
    pub daily: Vec<Day>,
}

/// Parse an Open-Meteo forecast (timeformat=unixtime). Times become Unix ms.
pub fn parse_weather(v: &Value, unit: &str) -> Option<Weather> {
    let cur = &v["current"];
    let now = cur["time"].as_i64()?;
    let num = |x: &Value| x.as_f64();
    let h = &v["hourly"];
    let hourly: Vec<Hour> = h["time"].as_array()?.iter().enumerate()
        .filter_map(|(i, t)| Some(Hour { time: t.as_i64()?, temp: num(&h["temperature_2m"][i])?, code: h["weather_code"][i].as_i64()? as i32,
            pop: num(&h["precipitation_probability"][i]) }))
        .filter(|x| x.time + 3600 > now)
        .take(12)
        .map(|x| Hour { time: x.time * 1000, ..x })
        .collect();
    let d = &v["daily"];
    let daily: Vec<Day> = d["time"].as_array()?.iter().enumerate()
        .filter_map(|(i, t)| Some(Day {
            date: t.as_i64()? * 1000,
            min: num(&d["temperature_2m_min"][i])?,
            max: num(&d["temperature_2m_max"][i])?,
            code: d["weather_code"][i].as_i64()? as i32,
            sunrise: d["sunrise"][i].as_i64().map(|s| s * 1000),
            sunset: d["sunset"][i].as_i64().map(|s| s * 1000),
            pop: num(&d["precipitation_probability_max"][i]),
        }))
        .collect();
    Some(Weather {
        fetched_at: now_ms(),
        unit: unit.into(),
        place: None,
        offline: false,
        current: Current {
            temp: num(&cur["temperature_2m"])?, code: cur["weather_code"].as_i64()? as i32, is_day: cur["is_day"] == 1,
            feels_like: num(&cur["apparent_temperature"]),
            humidity: num(&cur["relative_humidity_2m"]),
            wind_speed: num(&cur["wind_speed_10m"]),
            wind_dir: num(&cur["wind_direction_10m"]),
        },
        hourly,
        daily,
    })
}

fn fetch(p: &Place, unit: &str) -> Result<Weather, String> {
    let f = if unit == "f" { "&temperature_unit=fahrenheit&wind_speed_unit=mph" } else { "" };
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,is_day,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_direction_10m\
         &hourly=temperature_2m,weather_code,precipitation_probability\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max\
         &forecast_days=3&timezone=auto&timeformat=unixtime{f}", p.lat, p.lon);
    let mut w = parse_weather(&get_json(&url)?, unit).ok_or("unexpected weather response")?;
    w.place = Some(p.clone());
    Ok(w)
}

fn get(app: &AppHandle, force: bool) -> Option<Weather> {
    let unit = app.state::<crate::AppState>().config.lock().unwrap()
        .widget_settings("clock")["weatherUnit"].as_str().unwrap_or("c").to_string();
    let place = location::current(app).ok();
    let cached: Option<Weather> = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        kv_get(&c, "weather").and_then(|s| serde_json::from_str(&s).ok())
    };
    let fresh = |w: &Weather| now_ms() - w.fetched_at < TTL_MS && w.unit == unit && !location::is_real_change(w.place.as_ref(), place.as_ref());
    if let Some(w) = &cached { if !force && fresh(w) { return Some(w.clone()); } }
    match place.as_ref().map(|p| fetch(p, &unit)) {
        Some(Ok(w)) => {
            let db = app.state::<Db>();
            kv_set(&db.0.lock().unwrap(), "weather", &serde_json::to_string(&w).unwrap());
            Some(w)
        }
        // Only the same place's old forecast, marked offline; never another place's.
        Some(Err(_)) => cached.filter(|w| !location::is_real_change(w.place.as_ref(), place.as_ref())).map(|w| Weather { offline: true, ..w }),
        None => None, // location unavailable: nothing rather than a guess
    }
}

#[tauri::command]
pub async fn weather_get(app: AppHandle, force: bool) -> Option<Weather> {
    crate::widgets::blocking(move || get(&app, force)).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIX: &str = include_str!("../../tests/fixtures/open-meteo-forecast.json");

    #[test]
    fn parse_weather_reads_current_hourly_daily() {
        let v: serde_json::Value = serde_json::from_str(FIX).unwrap();
        let w = parse_weather(&v, "c").unwrap();
        assert_eq!(w.current.temp, v["current"]["temperature_2m"].as_f64().unwrap());
        assert_eq!(w.current.code, v["current"]["weather_code"].as_i64().unwrap() as i32);
        assert_eq!(w.hourly.len(), 12);
        assert!(w.hourly[0].time >= v["current"]["time"].as_i64().unwrap() * 1000 - 3_600_000);
        assert_eq!(w.daily.len(), 3);
        assert!(w.daily.iter().all(|d| d.min <= d.max));
    }

    #[test]
    fn parse_weather_reads_details_and_sun() {
        let v: serde_json::Value = serde_json::from_str(FIX).unwrap();
        let w = parse_weather(&v, "c").unwrap();
        assert_eq!(w.current.feels_like, v["current"]["apparent_temperature"].as_f64());
        assert_eq!(w.current.humidity, v["current"]["relative_humidity_2m"].as_f64());
        assert_eq!(w.current.wind_speed, v["current"]["wind_speed_10m"].as_f64());
        assert_eq!(w.current.wind_dir, v["current"]["wind_direction_10m"].as_f64());
        assert!(w.hourly.iter().all(|h| h.pop.is_some()));
        let d = &w.daily[0];
        assert_eq!(d.sunrise, v["daily"]["sunrise"][0].as_i64().map(|s| s * 1000));
        assert!(d.sunrise.unwrap() < d.sunset.unwrap());
        assert_eq!(d.pop, v["daily"]["precipitation_probability_max"][0].as_f64());
    }

    #[test]
    fn parse_weather_tolerates_missing_sun_and_details() {
        // Polar day/night: Open-Meteo returns no sunrise/sunset; older caches lack details.
        let mut v: serde_json::Value = serde_json::from_str(FIX).unwrap();
        v["daily"]["sunrise"] = serde_json::json!([null, null, null]);
        v["current"].as_object_mut().unwrap().remove("apparent_temperature");
        let w = parse_weather(&v, "c").unwrap();
        assert_eq!(w.daily[0].sunrise, None);
        assert_eq!(w.current.feels_like, None);
        let old: Weather = serde_json::from_str(r#"{"fetchedAt":1,"unit":"c","place":null,"offline":false,
            "current":{"temp":1.0,"code":0,"isDay":true},"hourly":[{"time":1,"temp":1.0,"code":0}],
            "daily":[{"date":1,"min":0.0,"max":1.0,"code":0}]}"#).unwrap();
        assert_eq!(old.daily[0].sunset, None);
    }

    #[test]
    fn parse_weather_rejects_garbage() {
        assert!(parse_weather(&serde_json::json!({"error": true}), "c").is_none());
    }
}
