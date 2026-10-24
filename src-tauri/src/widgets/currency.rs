//! Exchange rates: open.er-api.com (free, daily), falling back to Frankfurter (ECB).
//! Cached per base currency for 1 hour; when both fail, the cache is returned marked
//! `cached`. These are indicative mid-market rates, not bank rates.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::{enc, get_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use tauri::{AppHandle, Manager};

const TTL_MS: i64 = 60 * 60_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rates {
    pub base: String,
    /// 1 base = rate units of each currency.
    pub rates: BTreeMap<String, f64>,
    /// When the provider last updated the rates (Unix ms).
    pub rate_time: i64,
    pub fetched_at: i64,
    pub source: String,
    /// True when this is old cached data because the latest fetch failed.
    #[serde(default)]
    pub cached: bool,
}

fn rates_of(v: &Value) -> Option<BTreeMap<String, f64>> {
    let m: BTreeMap<String, f64> = v.as_object()?.iter().filter_map(|(k, x)| Some((k.clone(), x.as_f64()?))).collect();
    (!m.is_empty()).then_some(m)
}

pub fn parse_er_api(v: &Value) -> Option<Rates> {
    if v["result"] != "success" { return None; }
    Some(Rates {
        base: v["base_code"].as_str()?.into(),
        rates: rates_of(&v["rates"])?,
        rate_time: v["time_last_update_unix"].as_i64()? * 1000,
        fetched_at: now_ms(),
        source: "ExchangeRate-API".into(),
        cached: false,
    })
}

pub fn parse_frankfurter(v: &Value) -> Option<Rates> {
    let base: String = v["base"].as_str()?.into();
    let mut rates = rates_of(&v["rates"])?;
    rates.insert(base.clone(), 1.0);
    let date = chrono::NaiveDate::parse_from_str(v["date"].as_str()?, "%Y-%m-%d").ok()?;
    // ECB publishes around 16:00 CET; noon UTC is a fair "as of" for a date-only stamp.
    let rate_time = date.and_hms_opt(12, 0, 0)?.and_utc().timestamp_millis();
    Some(Rates { base, rates, rate_time, fetched_at: now_ms(), source: "Frankfurter (ECB)".into(), cached: false })
}

fn fetch(base: &str) -> Result<Rates, String> {
    let b = enc(base);
    get_json(&format!("https://open.er-api.com/v6/latest/{b}")).ok().as_ref().and_then(parse_er_api)
        .or_else(|| get_json(&format!("https://api.frankfurter.app/latest?from={b}")).ok().as_ref().and_then(parse_frankfurter))
        .ok_or_else(|| "couldn't reach either exchange-rate service".into())
}

fn get(app: &AppHandle, base: &str, force: bool) -> Result<Rates, String> {
    let base = base.trim().to_uppercase();
    if base.len() != 3 || !base.chars().all(|c| c.is_ascii_alphabetic()) { return Err("invalid currency code".into()); }
    let key = format!("fx:{base}");
    let cached: Option<Rates> = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        kv_get(&c, &key).and_then(|s| serde_json::from_str(&s).ok())
    };
    let force = force || crate::widgets::first_since_start(&key);
    if let Some(r) = &cached { if !force && now_ms() - r.fetched_at < TTL_MS { return Ok(r.clone()); } }
    match fetch(&base) {
        Ok(r) => {
            let db = app.state::<Db>();
            kv_set(&db.0.lock().unwrap(), &key, &serde_json::to_string(&r).unwrap());
            Ok(r)
        }
        Err(e) => cached.map(|r| Rates { cached: true, ..r }).ok_or(e),
    }
}

#[tauri::command]
pub async fn fx_rates(app: AppHandle, base: String, force: bool) -> Result<Rates, String> {
    crate::widgets::blocking(move || get(&app, &base, force)).await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_er_api() {
        let v = serde_json::json!({"result":"success","base_code":"USD","time_last_update_unix":1792713601,
            "rates":{"USD":1,"MYR":4.21,"EUR":0.92}});
        let r = parse_er_api(&v).unwrap();
        assert_eq!(r.base, "USD");
        assert_eq!(r.rates["MYR"], 4.21);
        assert_eq!(r.rate_time, 1792713601000);
        assert!(parse_er_api(&serde_json::json!({"result":"error","error-type":"unsupported-code"})).is_none());
    }

    #[test]
    fn parses_frankfurter_and_adds_base() {
        let v = serde_json::json!({"amount":1.0,"base":"EUR","date":"2026-10-22","rates":{"USD":1.08,"MYR":4.6}});
        let r = parse_frankfurter(&v).unwrap();
        assert_eq!(r.rates["EUR"], 1.0);
        assert_eq!(r.rates["USD"], 1.08);
        assert!(r.rate_time > 0);
    }
}
