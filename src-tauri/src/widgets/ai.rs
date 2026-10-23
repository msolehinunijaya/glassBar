//! Claude (Anthropic Messages API, raw HTTP: there is no official Rust SDK) for OCR-by-AI
//! and quick questions. The API key lives in Windows Credential Manager, never in
//! config.json. Token counts come from the API's own `usage` field; no costs are guessed.
use crate::db::{kv_get, kv_set, Db};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

pub const DEFAULT_MODEL: &str = "claude-opus-5-5";
const API: &str = "https://api.anthropic.com/v1/messages";

fn entry() -> Result<keyring::Entry, String> { keyring::Entry::new("GlassBar", "anthropic-api-key").map_err(|e| e.to_string()) }
fn api_key() -> Result<String, String> {
    entry()?.get_password().map_err(|_| "No Anthropic API key yet: add one in the AI widget's settings".to_string())
}

#[tauri::command]
pub fn ai_key_status() -> bool { api_key().is_ok() }
#[tauri::command]
pub fn ai_key_set(key: String) -> Result<(), String> {
    let k = key.trim();
    if k.is_empty() { return Err("empty key".into()); }
    entry()?.set_password(k).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn ai_key_clear() -> Result<(), String> {
    match entry()?.delete_credential() { Ok(()) | Err(keyring::Error::NoEntry) => Ok(()), Err(e) => Err(e.to_string()) }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub day: String, pub day_in: u64, pub day_out: u64, pub day_requests: u32,
    pub month: String, pub month_in: u64, pub month_out: u64,
}

/// Add one request's tokens, rolling the day/month buckets over when the date changes.
pub fn rollover(mut u: Usage, day: &str) -> Usage {
    let month = &day[..7];
    if u.day != day { u.day = day.into(); u.day_in = 0; u.day_out = 0; u.day_requests = 0; }
    if u.month != month { u.month = month.into(); u.month_in = 0; u.month_out = 0; }
    u
}

pub fn add_usage(u: Usage, day: &str, input: u64, output: u64) -> Usage {
    let mut u = rollover(u, day);
    u.day_in += input; u.day_out += output; u.day_requests += 1;
    u.month_in += input; u.month_out += output;
    u
}

fn today() -> String { chrono::Local::now().format("%Y-%m-%d").to_string() }

fn usage_get(app: &AppHandle) -> Usage {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    let u: Usage = kv_get(&c, "ai_usage").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    rollover(u, &today()) // zeros, not yesterday's numbers, on a new day/month
}

#[tauri::command]
pub fn ai_usage(app: AppHandle) -> Usage { usage_get(&app) }

/// Text of the reply, or why there is none. Pure, so it's testable.
pub fn parse_reply(v: &Value) -> Result<(String, u64, u64), String> {
    if let Some(m) = v["error"]["message"].as_str() { return Err(m.to_string()); }
    if v["stop_reason"] == "refusal" {
        let why = v["stop_details"]["explanation"].as_str().unwrap_or("no reason given");
        return Err(format!("Claude declined this request ({why})"));
    }
    let text: String = v["content"].as_array().ok_or("unexpected response")?
        .iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("");
    let u = &v["usage"];
    let n = |k: &str| u[k].as_u64().unwrap_or(0);
    let input = n("input_tokens") + n("cache_read_input_tokens") + n("cache_creation_input_tokens");
    Ok((text, input, n("output_tokens")))
}

fn model(app: &AppHandle) -> String {
    let st = app.state::<crate::AppState>();
    let cfg = st.config.lock().unwrap();
    cfg.widget_settings("ai")["model"].as_str().filter(|s| !s.is_empty()).unwrap_or(DEFAULT_MODEL).to_string()
}

/// One request. `effort` per Messages API `output_config.effort`.
pub fn ask(app: &AppHandle, content: Value, effort: &str, max_tokens: u32) -> Result<String, String> {
    let key = api_key()?;
    let body = json!({
        "model": model(app),
        "max_tokens": max_tokens,
        "output_config": { "effort": effort },
        // On a safety decline, the API retries on Anthropic's recommended fallback model.
        "fallbacks": "default",
        "messages": [{ "role": "user", "content": content }],
    });
    let v = crate::net::post_json(API, &[
        ("x-api-key", key.as_str()),
        ("anthropic-version", "2026-10-21"),
        ("anthropic-beta", "server-side-fallback-2026-07-01"),
    ], &body, 120)?;
    let (text, input, output) = parse_reply(&v)?;
    {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        let u: Usage = kv_get(&c, "ai_usage").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        kv_set(&c, "ai_usage", &serde_json::to_string(&add_usage(u, &today(), input, output)).unwrap());
    }
    let _ = app.emit("ai:changed", ());
    Ok(text)
}

#[tauri::command]
pub async fn ai_ask(app: AppHandle, question: String) -> Result<String, String> {
    let q = question.trim().to_string();
    if q.is_empty() { return Err("type a question first".into()); }
    crate::widgets::blocking(move || ask(&app, json!([{ "type": "text", "text": q }]), "medium", 8000)).await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_rolls_over_by_day_and_month() {
        let u = add_usage(Usage::default(), "2026-10-23", 100, 20);
        let u = add_usage(u, "2026-10-23", 5, 5);
        assert_eq!((u.day_in, u.day_out, u.day_requests, u.month_in), (105, 25, 2, 105));
        let u = add_usage(u, "2026-10-24", 1, 1);
        assert_eq!((u.day_in, u.day_requests, u.month_in), (1, 1, 106));
        let u = add_usage(u, "2026-11-01", 1, 1);
        assert_eq!((u.month, u.month_in), ("2026-11".to_string(), 1));
    }

    #[test]
    fn parses_text_usage_refusal_and_errors() {
        let ok = json!({"content":[{"type":"thinking","thinking":""},{"type":"text","text":"Hi"},{"type":"text","text":"!"}],
            "stop_reason":"end_turn","usage":{"input_tokens":10,"cache_read_input_tokens":5,"output_tokens":3}});
        assert_eq!(parse_reply(&ok).unwrap(), ("Hi!".into(), 15, 3));
        let refusal = json!({"content":[],"stop_reason":"refusal","stop_details":{"type":"refusal","category":"cyber","explanation":"x"}});
        assert!(parse_reply(&refusal).unwrap_err().contains("declined"));
        let err = json!({"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}});
        assert_eq!(parse_reply(&err).unwrap_err(), "invalid x-api-key");
    }
}
