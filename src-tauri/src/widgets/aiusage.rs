//! AI Usage widget: how much of Claude and ChatGPT you have used. Five sources, one shape:
//! - `anthropic`: Anthropic Admin API usage + cost reports (Admin key in Credential Manager)
//! - `openai`: OpenAI Admin API usage + costs (Admin key in Credential Manager)
//! - `claudecode`: Claude Code's local logs (%USERPROFILE%\.claude\projects\**\*.jsonl), read
//!   on change, no sign-in
//! - `claudeplan`: the Claude plan's limits (session, weekly), asked for with Claude Code's own
//!   login on this PC, like Claude Code's /usage (unofficial endpoint, no sign-in)
//! - `chatgptweb`: the ChatGPT plan's limits, read inside a sign-in webview with its own
//!   cookie folder (unofficial: the site's own internal endpoint)
//! Only limits a source publishes are shown; none are made up. The last good reading is
//! cached; a failed refresh keeps it and adds `error`. Keys and cookies are never logged.
use crate::db::{kv_get, kv_set, now_ms, Db};
use chrono::{Datelike, Duration as Days, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const SOURCES: [&str; 5] = ["anthropic", "claudecode", "openai", "claudeplan", "chatgptweb"];
const EVERY_MS: i64 = 5 * 60_000;
/// Opening the flyout refreshes the APIs at once, but not more often than this.
const FORCE_GAP_MS: i64 = 30_000;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Totals { pub input: u64, pub output: u64, pub cache_read: u64, pub cache_write: u64 }
impl Totals {
    pub fn sum(&self) -> u64 { self.input + self.output + self.cache_read + self.cache_write }
    fn add(&mut self, o: &Totals) { self.input += o.input; self.output += o.output; self.cache_read += o.cache_read; self.cache_write += o.cache_write; }
}

/// A limit the source itself reports: % used and when it resets.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Limit { pub key: String, pub label: String, pub pct: f64, pub resets_at: Option<i64> }

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Named { pub name: String, pub tokens: u64, #[serde(default)] pub full: String }

/// Claude Code's current 5-hour block (starts on the hour of its first message).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Session { pub totals: Totals, pub started_at: i64, pub resets_at: i64 }

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Reading {
    pub source: String,
    pub connected: bool,
    /// Last 4 characters of the key, for "••••a91f".
    pub hint: Option<String>,
    /// Last good read (Unix ms); 0 = never.
    pub updated_at: i64,
    /// The latest refresh failed: the data below is the last good one.
    pub error: Option<String>,
    /// "auth" (key invalid / signed out), "offline", "broken" (web page changed).
    pub error_kind: Option<String>,
    pub has_usage: bool,
    pub limits: Vec<Limit>,
    pub session: Option<Session>,
    pub today: Totals,
    pub week: Totals,
    pub month: Totals,
    /// Last 30 days, oldest first: (YYYY-MM-DD, tokens).
    pub daily: Vec<(String, u64)>,
    /// Last 30 days, most first.
    pub models: Vec<Named>,
    pub projects: Vec<Named>,
    pub cost_today: Option<f64>,
    pub cost_month: Option<f64>,
    /// Cost from list prices (local logs), not a bill.
    pub cost_estimated: bool,
}

/// One day × model of token use.
#[derive(Clone, Debug, PartialEq)]
pub struct Row { pub date: String, pub model: String, pub t: Totals }

fn ymd(d: NaiveDate) -> String { d.format("%Y-%m-%d").to_string() }

/// Today / this week (from Monday) / this month, 30 daily totals and the model ranking.
/// `costs` is USD per day; empty = the source has no cost.
pub fn summarize(r: &mut Reading, rows: &[Row], costs: &BTreeMap<String, f64>, today: NaiveDate) {
    let t = ymd(today);
    let week = ymd(today - Days::days(today.weekday().num_days_from_monday() as i64));
    let month = format!("{}-01", &t[..7]);
    let from30 = ymd(today - Days::days(29));
    let mut daily: BTreeMap<String, u64> = (0..30).map(|i| (ymd(today - Days::days(29 - i)), 0)).collect();
    let mut models: HashMap<&str, u64> = HashMap::new();
    (r.today, r.week, r.month) = Default::default();
    for row in rows {
        // API days are UTC: west of UTC a bucket can be "tomorrow"; count it as today.
        let date = row.date.as_str().min(t.as_str());
        if date == t { r.today.add(&row.t); }
        if date >= week.as_str() { r.week.add(&row.t); }
        if date >= month.as_str() { r.month.add(&row.t); }
        if date >= from30.as_str() {
            *daily.entry(date.to_string()).or_default() += row.t.sum();
            if !row.model.is_empty() { *models.entry(&row.model).or_default() += row.t.sum(); }
        }
    }
    r.daily = daily.into_iter().collect();
    let mut m: Vec<Named> = models.into_iter().filter(|x| x.1 > 0).map(|(n, k)| Named { name: n.into(), tokens: k, full: String::new() }).collect();
    m.sort_by(|a, b| b.tokens.cmp(&a.tokens));
    r.models = m;
    (r.cost_today, r.cost_month) = if costs.is_empty() { (None, None) } else {
        let d = |d: &str| costs.iter().filter(|(k, _)| k.as_str().min(t.as_str()) >= d).map(|x| x.1).sum::<f64>();
        (Some(costs.iter().filter(|(k, _)| k.as_str() >= t.as_str()).map(|x| x.1).sum()), Some(d(&month)))
    };
    r.has_usage = true;
}

// ---------- Admin APIs ----------

type Fail = (String, &'static str);

fn http_fail(status: u16, v: &Value, what: &str) -> Fail {
    let msg = v["error"]["message"].as_str().unwrap_or("");
    match status {
        401 => (format!("{what} rejected this key (invalid or expired)"), "auth"),
        403 => (format!("This key can't read usage: it needs an Admin key{}", if msg.is_empty() { String::new() } else { format!(" ({msg})") }), "auth"),
        _ => (if msg.is_empty() { format!("{what}: HTTP {status}") } else { format!("{what}: {msg}") }, "other"),
    }
}

fn get(url: &str, headers: &[(&str, &str)], what: &str) -> Result<Value, Fail> {
    match crate::net::get_json_h(url, headers) {
        Ok((s, v)) if (200..300).contains(&s) => Ok(v),
        Ok((s, v)) => Err(http_fail(s, &v, what)),
        Err(e) => Err((e, "offline")),
    }
}

/// GET every page (`page=<next_page>`) of a report, at most 6.
fn pages(base: &str, headers: &[(&str, &str)], what: &str) -> Result<Vec<Value>, Fail> {
    let mut out = vec![];
    let mut url = base.to_string();
    for _ in 0..6 {
        let v = get(&url, headers, what)?;
        let next = (v["has_more"] == true).then(|| v["next_page"].as_str().map(String::from)).flatten();
        out.push(v);
        match next { Some(p) => url = format!("{base}&page={}", crate::net::enc(&p)), None => break }
    }
    Ok(out)
}

pub fn parse_anthropic_usage(v: &Value, rows: &mut Vec<Row>) {
    for b in v["data"].as_array().into_iter().flatten() {
        let date = b["starting_at"].as_str().and_then(|s| s.get(..10)).unwrap_or("").to_string();
        for x in b["results"].as_array().into_iter().flatten() {
            let n = |p: &str| x.pointer(p).and_then(Value::as_u64).unwrap_or(0);
            rows.push(Row { date: date.clone(), model: x["model"].as_str().unwrap_or("").into(), t: Totals {
                input: n("/uncached_input_tokens"), output: n("/output_tokens"), cache_read: n("/cache_read_input_tokens"),
                cache_write: n("/cache_creation/ephemeral_5m_input_tokens") + n("/cache_creation/ephemeral_1h_input_tokens"),
            } });
        }
    }
}

/// Anthropic cost amounts are decimal strings in cents.
pub fn parse_anthropic_cost(v: &Value, costs: &mut BTreeMap<String, f64>) {
    for b in v["data"].as_array().into_iter().flatten() {
        let date = b["starting_at"].as_str().and_then(|s| s.get(..10)).unwrap_or("").to_string();
        for x in b["results"].as_array().into_iter().flatten() {
            let cents = x["amount"].as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| x["amount"].as_f64()).unwrap_or(0.0);
            *costs.entry(date.clone()).or_default() += cents / 100.0;
        }
    }
}

fn unix_day(s: i64) -> String { Utc.timestamp_opt(s, 0).single().map(|d| ymd(d.date_naive())).unwrap_or_default() }

pub fn parse_openai_usage(v: &Value, rows: &mut Vec<Row>) {
    for b in v["data"].as_array().into_iter().flatten() {
        let date = unix_day(b["start_time"].as_i64().unwrap_or(0));
        for x in b["results"].as_array().into_iter().flatten() {
            let n = |k: &str| x[k].as_u64().unwrap_or(0);
            let cached = n("input_cached_tokens");
            rows.push(Row { date: date.clone(), model: x["model"].as_str().unwrap_or("").into(), t: Totals {
                input: n("input_tokens").saturating_sub(cached), output: n("output_tokens"), cache_read: cached, cache_write: 0,
            } });
        }
    }
}

pub fn parse_openai_cost(v: &Value, costs: &mut BTreeMap<String, f64>) {
    for b in v["data"].as_array().into_iter().flatten() {
        let date = unix_day(b["start_time"].as_i64().unwrap_or(0));
        for x in b["results"].as_array().into_iter().flatten() {
            *costs.entry(date.clone()).or_default() += x["amount"]["value"].as_f64().unwrap_or(0.0);
        }
    }
}

/// The first day to fetch: the 1st of the month or 29 days ago, whichever is earlier.
fn first_day(today: NaiveDate) -> NaiveDate { (today - Days::days(29)).min(today.with_day(1).unwrap()) }

const ANT: &str = "https://api.anthropic.com/v1/organizations";
fn ant_headers(key: &str) -> [(&str, &str); 2] { [("x-api-key", key), ("anthropic-version", "2023-06-01")] }
const OAI: &str = "https://api.openai.com/v1/organization";

fn fetch_api(source: &str, key: &str, today: NaiveDate) -> Result<Reading, Fail> {
    let from = first_day(today);
    let mut rows = vec![];
    let mut costs = BTreeMap::new();
    if source == "anthropic" {
        let h = ant_headers(key);
        let at = format!("{}T00:00:00Z", ymd(from));
        for v in pages(&format!("{ANT}/usage_report/messages?starting_at={at}&bucket_width=1d&limit=31&group_by[]=model"), &h, "Anthropic")? { parse_anthropic_usage(&v, &mut rows); }
        for v in pages(&format!("{ANT}/cost_report?starting_at={at}&limit=31"), &h, "Anthropic")? { parse_anthropic_cost(&v, &mut costs); }
    } else {
        let auth = format!("Bearer {key}");
        let h = [("Authorization", auth.as_str())];
        let at = from.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        for v in pages(&format!("{OAI}/usage/completions?start_time={at}&bucket_width=1d&limit=31&group_by=model"), &h, "OpenAI")? { parse_openai_usage(&v, &mut rows); }
        for v in pages(&format!("{OAI}/costs?start_time={at}&bucket_width=1d&limit=31"), &h, "OpenAI")? { parse_openai_cost(&v, &mut costs); }
    }
    let mut r = Reading::default();
    summarize(&mut r, &rows, &costs, today);
    Ok(r)
}

/// Plain words for a key of the wrong kind, before any network call.
pub fn key_shape_error(source: &str, key: &str) -> Option<&'static str> {
    match source {
        "anthropic" if key.starts_with("sk-ant-admin") => None,
        "anthropic" if key.starts_with("sk-ant-") => Some("This is a normal key; usage needs an Admin key (sk-ant-admin…)"),
        "anthropic" => Some("That isn't an Anthropic Admin key (it starts with sk-ant-admin…)"),
        "openai" if key.starts_with("sk-admin-") => None,
        "openai" if key.starts_with("sk-") => Some("This is a normal key; usage needs an Admin key (sk-admin-…)"),
        "openai" => Some("That isn't an OpenAI Admin key (it starts with sk-admin-…)"),
        _ => Some("This source doesn't use a key"),
    }
}

/// The one test call on save: a single small usage page.
fn validate(source: &str, key: &str) -> Result<(), String> {
    if let Some(e) = key_shape_error(source, key) { return Err(e.into()); }
    let day = Local::now().date_naive() - Days::days(1);
    let r = if source == "anthropic" {
        get(&format!("{ANT}/usage_report/messages?starting_at={}T00:00:00Z&bucket_width=1d&limit=1", ymd(day)), &ant_headers(key), "Anthropic")
    } else {
        let auth = format!("Bearer {key}");
        get(&format!("{OAI}/usage/completions?start_time={}&bucket_width=1d&limit=1", day.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()), &[("Authorization", auth.as_str())], "OpenAI")
    };
    r.map(|_| ()).map_err(|(m, kind)| if kind == "offline" { "Couldn't reach the server: check your connection".into() } else { m })
}

fn key_name(source: &str) -> &'static str { if source == "anthropic" { "anthropic-admin-key" } else { "openai-admin-key" } }
fn api_key(source: &str) -> Option<String> { crate::widgets::ai::cred(key_name(source)).ok()?.get_password().ok() }

// ---------- Claude Code logs ----------

/// List prices per million tokens: input, output, cache read. Cache writes are 1.25× input
/// (5 min) and 2× (1 h). Unknown models get no cost rather than a guess.
pub fn price(model: &str) -> Option<(f64, f64, f64)> {
    const P: [(&str, f64, f64, f64); 15] = [
        ("fable-5-1", 10.0, 50.0, 0.25), ("mythos-5-1", 10.0, 50.0, 0.25), ("fable-5", 10.0, 50.0, 1.0), ("mythos-5", 10.0, 50.0, 1.0),
        ("opus-5-5", 4.0, 20.0, 0.2), ("opus-5", 5.0, 25.0, 0.5), ("opus-4-8", 5.0, 25.0, 0.5), ("opus-4-7", 5.0, 25.0, 0.5), ("opus-4-6", 5.0, 25.0, 0.5),
        ("sonnet-5-5", 2.0, 10.0, 0.2), ("sonnet-5", 2.0, 10.0, 0.2), ("sonnet-4-6", 3.0, 15.0, 0.3),
        ("haiku-5-5", 0.1, 0.5, 0.01), ("haiku-4-5", 1.0, 5.0, 0.1), ("opus-4-5", 5.0, 25.0, 0.5),
    ];
    let m = model.trim_start_matches("claude-");
    P.iter().find(|p| m.starts_with(p.0)).map(|p| (p.1, p.2, p.3))
}

struct Entry { at: i64, model: Arc<str>, project: Arc<str>, t: Totals, write_1h: u64 }

fn cost_of(e: &Entry) -> Option<f64> {
    let (i, o, cr) = price(&e.model)?;
    let w5 = e.t.cache_write - e.write_1h;
    Some((e.t.input as f64 * i + e.t.output as f64 * o + e.t.cache_read as f64 * cr + w5 as f64 * i * 1.25 + e.write_1h as f64 * i * 2.0) / 1e6)
}

#[derive(Default)]
struct Logs { offsets: HashMap<PathBuf, u64>, seen: HashSet<u64>, entries: Vec<Entry>, names: HashSet<Arc<str>> }

#[derive(Deserialize)]
struct LLine { #[serde(rename = "type", default)] kind: String, #[serde(default)] timestamp: String, #[serde(default)] cwd: String,
               #[serde(rename = "requestId", default)] request_id: String, message: Option<LMsg> }
#[derive(Deserialize)]
struct LMsg { #[serde(default)] id: String, #[serde(default)] model: String, usage: Option<LUsage> }
#[derive(Deserialize, Default)]
#[serde(default)]
struct LUsage { input_tokens: u64, output_tokens: u64, cache_read_input_tokens: u64, cache_creation_input_tokens: u64, cache_creation: Option<LCache> }
#[derive(Deserialize, Default)]
#[serde(default)]
struct LCache { ephemeral_1h_input_tokens: u64 }

impl Logs {
    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(a) = self.names.get(s) { return a.clone(); }
        let a: Arc<str> = s.into();
        self.names.insert(a.clone());
        a
    }
    /// One log line → an entry, once per message (Claude Code writes a message's usage on
    /// several lines).
    fn line(&mut self, l: &str, fallback_project: &str) {
        if !l.contains("\"usage\"") || !l.contains("\"assistant\"") { return; }
        let Ok(v) = serde_json::from_str::<LLine>(l) else { return };
        let (Some(m), true) = (v.message, v.kind == "assistant") else { return };
        let (Some(u), false) = (m.usage, m.model.is_empty() || m.model.starts_with('<')) else { return };
        let Ok(at) = chrono::DateTime::parse_from_rfc3339(&v.timestamp) else { return };
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (m.id.as_str(), v.request_id.as_str()).hash(&mut h);
        if !m.id.is_empty() && !self.seen.insert(h.finish()) { return; }
        let project = if v.cwd.is_empty() { fallback_project } else { v.cwd.as_str() };
        let (model, project) = (self.intern(&m.model), self.intern(project));
        self.entries.push(Entry { at: at.timestamp_millis(), model, project,
            t: Totals { input: u.input_tokens, output: u.output_tokens, cache_read: u.cache_read_input_tokens, cache_write: u.cache_creation_input_tokens },
            write_1h: u.cache_creation.map(|c| c.ephemeral_1h_input_tokens).unwrap_or(0).min(u.cache_creation_input_tokens) });
    }

    /// Read what was added to each recent log since last time (whole lines only).
    fn scan(&mut self, root: &Path, cutoff: i64) {
        use std::io::{BufRead, Seek, SeekFrom};
        let mut stack = vec![(root.to_path_buf(), 0)];
        while let Some((d, depth)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                let Ok(md) = e.metadata() else { continue };
                if md.is_dir() { if depth < 4 { stack.push((p, depth + 1)); } continue; }
                if p.extension().is_none_or(|x| x != "jsonl") { continue; }
                let mtime = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
                if mtime < cutoff { continue; }
                let mut off = self.offsets.get(&p).copied().unwrap_or(0);
                if md.len() < off { off = 0; } // rewritten: the seen-set keeps it from counting twice
                if md.len() == off { continue; }
                let Ok(mut f) = std::fs::File::open(&p) else { continue };
                if f.seek(SeekFrom::Start(off)).is_err() { continue; }
                let fallback = p.parent().and_then(|x| x.file_name()).map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
                let mut r = std::io::BufReader::with_capacity(1 << 16, f);
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    match r.read_until(b'\n', &mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) if buf.last() != Some(&b'\n') => { let _ = n; break } // still being written
                        Ok(n) => { off += n as u64; if let Ok(s) = std::str::from_utf8(&buf) { self.line(s, &fallback); } }
                    }
                }
                self.offsets.insert(p, off);
            }
        }
        self.entries.retain(|e| e.at >= cutoff);
    }
}

/// The 5-hour block `now` is in, if any: a block starts on the hour of the first message
/// after a block ended (or after 5 h of quiet) and lasts 5 h.
pub fn current_block(mut ts: Vec<(i64, Totals)>, now: i64) -> Option<Session> {
    const H5: i64 = 5 * 3_600_000;
    ts.sort_by_key(|x| x.0);
    let (mut start, mut last, mut tot) = (None::<i64>, 0, Totals::default());
    for (t, x) in ts {
        if start.is_none_or(|s| t >= s + H5 || t - last >= H5) { start = Some(t - t.rem_euclid(3_600_000)); tot = Totals::default(); }
        tot.add(&x);
        last = t;
    }
    let s = start?;
    (now < s + H5).then_some(Session { totals: tot, started_at: s, resets_at: s + H5 })
}

fn logs_reading(lg: &Logs, now: i64) -> Reading {
    let mut by: HashMap<(String, Arc<str>), Totals> = HashMap::new();
    let mut costs: BTreeMap<String, f64> = BTreeMap::new();
    let mut projects: HashMap<Arc<str>, u64> = HashMap::new();
    let from30 = now - 30 * 86_400_000;
    for e in &lg.entries {
        let date = Local.timestamp_millis_opt(e.at).single().map(|d| ymd(d.date_naive())).unwrap_or_default();
        if let Some(c) = cost_of(e) { *costs.entry(date.clone()).or_default() += c; }
        by.entry((date, e.model.clone())).or_default().add(&e.t);
        if e.at >= from30 { *projects.entry(e.project.clone()).or_default() += e.t.sum(); }
    }
    let rows: Vec<Row> = by.into_iter().map(|((date, model), t)| Row { date, model: model.to_string(), t }).collect();
    let mut r = Reading::default();
    summarize(&mut r, &rows, &costs, Local::now().date_naive());
    r.cost_estimated = true;
    let mut p: Vec<Named> = projects.into_iter().map(|(full, tokens)| {
        let name = full.rsplit(['\\', '/']).find(|s| !s.is_empty()).unwrap_or(&full).to_string();
        Named { name, tokens, full: full.to_string() }
    }).collect();
    p.sort_by(|a, b| b.tokens.cmp(&a.tokens));
    p.truncate(5);
    r.projects = p;
    r.session = current_block(lg.entries.iter().map(|e| (e.at, e.t)).collect(), now);
    r
}

fn logs_root() -> Option<PathBuf> { std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".claude").join("projects")) }

// ---------- Claude plan limits, via Claude Code's login (unofficial) ----------

const PLAN_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const EXPIRED: &str = "Claude Code login expired: open Claude Code once to renew it";

/// Claude Code's login on this PC: (access token, expires at ms). Read fresh each time and
/// never stored, logged or refreshed by GlassBar (refreshing it would sign Claude Code out).
fn claude_login() -> Option<(String, Option<i64>)> {
    let p = std::env::var_os("USERPROFILE").map(PathBuf::from)?.join(".claude").join(".credentials.json");
    let v: Value = serde_json::from_slice(&std::fs::read(p).ok()?).ok()?;
    let o = &v["claudeAiOauth"];
    Some((o["accessToken"].as_str().filter(|s| !s.is_empty())?.to_string(), o["expiresAt"].as_i64()))
}

/// The plan's limits: the numbers Claude Code's /usage shows. The token goes only to Anthropic.
fn fetch_plan(token: &str) -> Result<Vec<Limit>, Fail> {
    let auth = format!("Bearer {token}");
    match crate::net::get_json_h(PLAN_URL, &[("Authorization", auth.as_str()), ("anthropic-beta", "oauth-2025-04-20")]) {
        Ok((200..=299, v)) => {
            let l = parse_claude_web(&v);
            if l.is_empty() { Err(("Couldn't read Claude plan usage (unofficial source)".into(), "broken")) } else { Ok(l) }
        }
        Ok((401 | 403, _)) => Err((EXPIRED.into(), "auth")),
        Ok((s, _)) => Err((format!("Claude plan usage: HTTP {s}"), "other")),
        Err(e) => Err((e, "offline")),
    }
}

/// Connected whenever Claude Code is signed in here (and not switched off). Blocking.
fn refresh_plan(app: &AppHandle) {
    let off = off_list(app).iter().any(|s| s == "claudeplan");
    let mut r = cached(app, "claudeplan");
    let Some((token, exp)) = (if off { None } else { claude_login() }) else {
        if r.connected { put(app, Reading { source: "claudeplan".into(), ..Default::default() }); }
        return;
    };
    r.connected = true;
    let res = if exp.is_some_and(|e| e < now_ms()) { Err((EXPIRED.to_string(), "auth")) } else { fetch_plan(&token) };
    match res {
        Ok(l) => {
            let old = std::mem::replace(&mut r.limits, l);
            r.updated_at = now_ms(); r.error = None; r.error_kind = None;
            alerts(app, "claudeplan", &old, &r.limits);
        }
        Err((m, k)) => { r.error = Some(m); r.error_kind = Some(k.into()); }
    }
    put(app, r);
}

// ---------- ChatGPT plan limits, via a sign-in webview (unofficial) ----------

const WEB_URL: &str = "https://chatgpt.com/codex/settings/usage";
fn web_dir(source: &str) -> PathBuf { crate::data_dir().join("aiusage-web").join(source) }

/// Runs inside the signed-in page: fetches the site's own usage JSON (with the page's own
/// cookies, to the same site) and hands only that JSON back through the page title.
const JS_CHATGPT: &str = r#"(async()=>{const out=x=>{document.title='GBR:'+JSON.stringify(x)};try{const s=await fetch('/api/auth/session',{credentials:'include'});const j=s.ok?await s.json():{};if(!j.accessToken){out({signin:true});return}const u=await fetch('/backend-api/wham/usage',{headers:{Authorization:'Bearer '+j.accessToken}});out(u.ok?{usage:await u.json()}:{status:u.status})}catch(e){out({error:String(e)})}})()"#;

fn ms_of(s: &str) -> Option<i64> { chrono::DateTime::parse_from_rfc3339(s).ok().map(|d| d.timestamp_millis()) }

pub fn parse_claude_web(u: &Value) -> Vec<Limit> {
    [("five_hour", "session", "5-hour limit"), ("seven_day", "week", "Weekly limit"),
     ("seven_day_opus", "week_opus", "Weekly · Opus"), ("seven_day_sonnet", "week_sonnet", "Weekly · Sonnet")]
        .iter().filter_map(|(k, key, label)| {
            let w = &u[*k];
            Some(Limit { key: (*key).into(), label: (*label).into(), pct: w["utilization"].as_f64()?, resets_at: w["resets_at"].as_str().and_then(ms_of) })
        }).collect()
}

pub fn parse_chatgpt_web(u: &Value, now: i64) -> Vec<Limit> {
    ["primary_window", "secondary_window"].iter().filter_map(|k| {
        let w = &u["rate_limit"][*k];
        let pct = w["used_percent"].as_f64()?;
        let secs = w["limit_window_seconds"].as_i64().unwrap_or(0);
        let (key, label) = match secs {
            s if s > 0 && s <= 12 * 3600 => ("session".to_string(), format!("{}-hour limit", (s + 1800) / 3600)),
            s if (6 * 86_400..=8 * 86_400).contains(&s) => ("week".into(), "Weekly limit".into()),
            s if s > 0 => (format!("w{s}"), format!("{}-day limit", (s + 43_200) / 86_400)),
            _ => (k.to_string(), "Usage limit".into()),
        };
        let resets_at = w["reset_at"].as_i64().map(|s| s * 1000).or_else(|| w["reset_after_seconds"].as_i64().map(|s| now + s * 1000));
        Some(Limit { key, label, pct, resets_at })
    }).collect()
}

/// What the ChatGPT page script sent back → limits, or why not (message, kind).
pub fn parse_web(v: &Value, now: i64) -> Result<Vec<Limit>, Fail> {
    let out = || ("Signed out of ChatGPT: sign in again".to_string(), "auth");
    if v["signin"] == true || v["status"].as_i64().is_some_and(|s| s == 401 || s == 403) { return Err(out()); }
    let limits = parse_chatgpt_web(&v["usage"], now);
    if limits.is_empty() { return Err(("Couldn't read ChatGPT usage page (unofficial source)".into(), "broken")); }
    Ok(limits)
}

static WEB_BUSY: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Open (visible, to sign in) or quietly load (hidden, to refresh) the plan's usage page in
/// a webview with its own cookie folder. The result arrives via the page title.
fn web_open(app: &AppHandle, source: &str, visible: bool) -> Result<(), String> {
    let label = format!("aiu-{source}");
    if let Some(w) = app.get_webview_window(&label) {
        if visible { let _ = w.show(); let _ = w.set_focus(); }
        return Ok(());
    }
    { let mut b = WEB_BUSY.lock().unwrap(); if !b.iter().any(|s| s == source) { b.push(source.into()); } }
    let src = source.to_string();
    let js = JS_CHATGPT;
    let url = WEB_URL.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    let a = app.clone();
    let title = "Sign in to ChatGPT · GlassBar reads only your usage";
    let w = tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::External(url))
        .title(title)
        .inner_size(960.0, 720.0)
        .visible(visible)
        .data_directory(web_dir(source))
        .on_page_load(move |w, p| { if matches!(p.event(), tauri::webview::PageLoadEvent::Finished) { let _ = w.eval(js); } })
        .on_document_title_changed(move |w, t| {
            let Some(json) = t.strip_prefix("GBR:") else { return };
            let v: Value = serde_json::from_str(json).unwrap_or(Value::Null);
            let res = parse_web(&v, now_ms());
            // Signing in: keep the window open until the usage numbers come through.
            if w.is_visible().unwrap_or(false) && matches!(&res, Err((_, "auth"))) { return; }
            // Off the UI thread: DB, alerts and banners.
            let (a, src) = (a.clone(), src.clone());
            std::thread::spawn(move || {
                let mut r = cached(&a, &src);
                match res {
                    Ok(l) => {
                        let old = std::mem::replace(&mut r.limits, l);
                        r.connected = true; r.updated_at = now_ms(); r.error = None; r.error_kind = None;
                        alerts(&a, &src, &old, &r.limits);
                    }
                    Err((m, k)) => { r.error = Some(m); r.error_kind = Some(k.into()); }
                }
                put(&a, r);
                let _ = w.close();
            });
        })
        .build()
        .map_err(|e| e.to_string());
    // A hidden refresh that never answers (no network, page changed) is given up after 45 s.
    let (a, l, s) = (app.clone(), label.clone(), source.to_string());
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(45));
        if let Some(w) = a.get_webview_window(&l) {
            if !w.is_visible().unwrap_or(true) {
                let _ = w.close();
                let mut r = cached(&a, &s);
                r.error = Some("Couldn't read ChatGPT usage page (unofficial source)".into());
                r.error_kind = Some("broken".into());
                put(&a, r);
            }
        }
    });
    let s = source.to_string();
    match w {
        Ok(w) => { w.on_window_event(move |e| if matches!(e, tauri::WindowEvent::Destroyed) { WEB_BUSY.lock().unwrap().retain(|x| x != &s); }); Ok(()) }
        Err(e) => { WEB_BUSY.lock().unwrap().retain(|x| x != &s); Err(e) }
    }
}

// ---------- State, cache, alerts ----------

static STATE: Mutex<Option<HashMap<String, Reading>>> = Mutex::new(None);
static LAST_TRY: Mutex<Vec<(String, i64)>> = Mutex::new(Vec::new());

fn cached(app: &AppHandle, source: &str) -> Reading {
    let mut st = STATE.lock().unwrap();
    let m = st.get_or_insert_with(HashMap::new);
    if let Some(r) = m.get(source) { return r.clone(); }
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    let mut r: Reading = kv_get(&c, &format!("aiu:r:{source}")).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    r.source = source.into();
    m.insert(source.into(), r.clone());
    r
}

fn put(app: &AppHandle, r: Reading) {
    let changed = {
        let mut st = STATE.lock().unwrap();
        let m = st.get_or_insert_with(HashMap::new);
        let changed = m.get(&r.source) != Some(&r);
        m.insert(r.source.clone(), r.clone());
        changed
    };
    if changed {
        let db = app.state::<Db>();
        kv_set(&db.0.lock().unwrap(), &format!("aiu:r:{}", r.source), &serde_json::to_string(&r).unwrap());
        let _ = app.emit("aiusage:changed", ());
    }
}

fn provider_of(source: &str) -> (&'static str, &'static str) {
    if matches!(source, "openai" | "chatgptweb") { ("chatgpt", "ChatGPT") } else { ("claude", "Claude") }
}

/// Banners at 80 % and 95 % of a limit (once per limit window), and when a limit that was
/// past 80 % resets.
fn alerts(app: &AppHandle, source: &str, old: &[Limit], new: &[Limit]) {
    let (p, name) = provider_of(source);
    let s = app.state::<crate::AppState>().config.lock().unwrap().widget_settings("aiusage");
    let on = |k: &str| s[k].as_bool().unwrap_or(true);
    let key = if p == "claude" { "alertsClaude" } else { "alertsChatgpt" };
    let db = app.state::<Db>();
    let mut sent: Vec<String> = kv_get(&db.0.lock().unwrap(), "aiu:alerted").and_then(|x| serde_json::from_str(&x).ok()).unwrap_or_default();
    for (title, body, id) in alert_plan(source, name, old, new, &sent) {
        if !on(key) || (id.ends_with("|reset") && !on("alertReset")) { sent.push(id); continue; }
        crate::alerts::notify(app, "aiusage", &title, &body, false);
        sent.push(id);
    }
    let n = sent.len().saturating_sub(100);
    sent.drain(..n);
    kv_set(&db.0.lock().unwrap(), "aiu:alerted", &serde_json::to_string(&sent).unwrap());
}

/// Which alerts to send now: (title, body, id). `sent` holds the ids already sent; an id
/// names the limit window, so the same alert never repeats within it. Pure.
pub fn alert_plan(source: &str, name: &str, old: &[Limit], new: &[Limit], sent: &[String]) -> Vec<(String, String, String)> {
    // resets_at can wobble by seconds between reads: name the window by its 15-min slot.
    let win = |l: &Limit| l.resets_at.map(|r| (r + 450_000) / 900_000).unwrap_or(0);
    let mut out = vec![];
    for l in new {
        if let Some(th) = [95.0, 80.0].into_iter().find(|t| l.pct >= *t) {
            let id = format!("{source}|{}|{}|{th}", l.key, win(l));
            if !sent.contains(&id) && !(th == 80.0 && sent.contains(&format!("{source}|{}|{}|95", l.key, win(l)))) {
                let when = l.resets_at.and_then(|r| Local.timestamp_millis_opt(r).single()).map(|d| format!(" · resets at {}", d.format("%-I:%M %P"))).unwrap_or_default();
                out.push((format!("{name} {:.0}% used", l.pct), format!("{}{when}", l.label), id));
            }
        }
        if let Some(o) = old.iter().find(|o| o.key == l.key) {
            if o.pct >= 80.0 && win(o) != win(l) && l.pct < o.pct {
                let what = if l.key == "session" { "session".to_string() } else { l.label.to_lowercase() };
                let id = format!("{source}|{}|{}|reset", l.key, win(l));
                if !sent.contains(&id) { out.push((format!("{name} {what} reset"), "Full limit available".into(), id)); }
            }
        }
    }
    out
}

fn off_list(app: &AppHandle) -> Vec<String> {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    kv_get(&c, "aiu:off").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
fn set_off(app: &AppHandle, source: &str, off: bool) {
    let mut l = off_list(app);
    l.retain(|s| s != source);
    if off { l.push(source.into()); }
    kv_set(&app.state::<Db>().0.lock().unwrap(), "aiu:off", &serde_json::to_string(&l).unwrap());
}

fn due(source: &str, gap: i64) -> bool {
    let mut t = LAST_TRY.lock().unwrap();
    let now = now_ms();
    match t.iter_mut().find(|x| x.0 == source) {
        Some(x) if now - x.1 < gap => false,
        Some(x) => { x.1 = now; true }
        None => { t.push((source.into(), now)); true }
    }
}

fn refresh_api(app: &AppHandle, source: &str) {
    let mut r = cached(app, source);
    if !r.connected { return; }
    let Some(key) = api_key(source) else {
        r.error = Some("Key missing from Credential Manager: reconnect".into()); r.error_kind = Some("auth".into());
        return put(app, r);
    };
    match fetch_api(source, &key, Local::now().date_naive()) {
        Ok(n) => put(app, Reading { source: source.into(), connected: true, hint: r.hint, updated_at: now_ms(), ..n }),
        Err((m, k)) => { r.error = Some(m); r.error_kind = Some(k.into()); put(app, r); }
    }
}

/// Refresh what is due: APIs every 5 min (or now when `force`, at most every 30 s), web
/// pages every 5 min. Blocking.
fn refresh(app: &AppHandle, force: bool) {
    for s in ["anthropic", "openai"] {
        if cached(app, s).connected && due(s, if force { FORCE_GAP_MS } else { EVERY_MS }) { refresh_api(app, s); }
    }
    // The Claude plan is a cheap API call: as often as the APIs (it also notices a new login).
    if due("claudeplan", if force { FORCE_GAP_MS } else { EVERY_MS }) { refresh_plan(app); }
    let s = "chatgptweb";
    if cached(app, s).connected && !WEB_BUSY.lock().unwrap().iter().any(|x| x == s) && due(s, EVERY_MS) {
        let _ = web_open(app, s, false);
    }
}

/// Background: APIs and web every 5 min while the widget is on a dock; Claude Code logs
/// whenever a log file changes (Windows change notifications, no polling).
pub fn start(app: AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || loop {
        let on = a.state::<crate::AppState>().config.lock().unwrap().uses("aiusage");
        if on { refresh(&a, false); }
        std::thread::sleep(Duration::from_secs(30));
    });
    std::thread::spawn(move || watch_logs(app));
}

fn watch_logs(app: AppHandle) {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
    use windows::Win32::Storage::FileSystem::*;
    use windows::Win32::System::Threading::WaitForSingleObject;
    let mut logs = Logs::default();
    let mut watch: Option<HANDLE> = None;
    loop {
        let on = app.state::<crate::AppState>().config.lock().unwrap().uses("aiusage");
        let root = logs_root().filter(|r| r.is_dir());
        let off = off_list(&app).iter().any(|s| s == "claudecode");
        let Some(root) = root.filter(|_| on && !off) else {
            if let Some(h) = watch.take() { unsafe { let _ = FindCloseChangeNotification(h); } }
            if !on { logs = Logs::default(); }
            let mut r = cached(&app, "claudecode");
            if r.connected { r.connected = false; put(&app, r); }
            std::thread::sleep(Duration::from_secs(10));
            continue;
        };
        if watch.is_none() {
            let w: Vec<u16> = root.to_string_lossy().encode_utf16().chain(Some(0)).collect();
            watch = unsafe { FindFirstChangeNotificationW(PCWSTR(w.as_ptr()), true, FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_FILE_NAME) }.ok();
        }
        let now = now_ms();
        logs.scan(&root, now - 35 * 86_400_000);
        let prev = cached(&app, "claudecode");
        put(&app, Reading { source: "claudecode".into(), connected: true, updated_at: now, ..logs_reading(&logs, now) }.keep_time_if_same(&prev));
        match watch {
            // Wake on a change (then let a burst of writes land), or each minute so the
            // session window and "today" roll over.
            Some(h) => unsafe {
                if WaitForSingleObject(h, 60_000) == WAIT_OBJECT_0 {
                    let _ = FindNextChangeNotification(h);
                    std::thread::sleep(Duration::from_secs(5));
                }
            },
            None => std::thread::sleep(Duration::from_secs(30)),
        }
    }
}

impl Reading {
    /// Unchanged numbers keep their "updated" time, so a quiet minute doesn't re-render.
    fn keep_time_if_same(mut self, prev: &Reading) -> Reading {
        let t = self.updated_at;
        self.updated_at = prev.updated_at;
        if &self != prev { self.updated_at = t; }
        self
    }
}

// ---------- Commands ----------

#[tauri::command]
pub fn aiu_state(app: AppHandle) -> Vec<Reading> { SOURCES.iter().map(|s| cached(&app, s)).collect() }

/// Refresh what is due now (the flyout calls this when it opens).
#[tauri::command]
pub async fn aiu_refresh(app: AppHandle) -> Result<(), String> { crate::widgets::blocking(move || refresh(&app, true)).await }

/// Save an Admin key after one test call; Claude Code / Claude plan: switch back on;
/// ChatGPT plan: open the sign-in window.
#[tauri::command]
pub async fn aiu_connect(app: AppHandle, source: String, key: Option<String>) -> Result<(), String> {
    match source.as_str() {
        "claudecode" => { set_off(&app, "claudecode", false); Ok(()) }
        "anthropic" | "openai" => crate::widgets::blocking(move || {
            let k = key.unwrap_or_default().trim().to_string();
            if k.is_empty() { return Err("Paste a key first".to_string()); }
            validate(&source, &k)?;
            crate::widgets::ai::cred(key_name(&source))?.set_password(&k).map_err(|e| e.to_string())?;
            let hint: String = k.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
            put(&app, Reading { source: source.clone(), connected: true, hint: Some(hint), ..Default::default() });
            LAST_TRY.lock().unwrap().retain(|x| x.0 != source);
            refresh_api(&app, &source);
            Ok(())
        }).await?,
        "claudeplan" => crate::widgets::blocking(move || {
            set_off(&app, "claudeplan", false);
            LAST_TRY.lock().unwrap().retain(|x| x.0 != "claudeplan");
            refresh_plan(&app);
            if cached(&app, "claudeplan").connected { Ok(()) } else { Err("Claude Code isn't signed in on this PC: run claude and sign in first".to_string()) }
        }).await?,
        "chatgptweb" => web_open(&app, &source, true),
        _ => Err("unknown source".into()),
    }
}

/// Remove a source's key (or cookies) and cached data.
#[tauri::command]
pub async fn aiu_disconnect(app: AppHandle, source: String) -> Result<(), String> {
    if !SOURCES.contains(&source.as_str()) { return Err("unknown source".into()); }
    match source.as_str() {
        "anthropic" | "openai" => match crate::widgets::ai::cred(key_name(&source))?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.to_string()),
        },
        "claudecode" | "claudeplan" => set_off(&app, &source, true),
        _ => {
            if let Some(w) = app.get_webview_window(&format!("aiu-{source}")) { let _ = w.destroy(); }
            let dir = web_dir(&source);
            // The browser process may hold the folder for a moment after the window closes.
            crate::widgets::blocking(move || for _ in 0..10 {
                if std::fs::remove_dir_all(&dir).is_ok() || !dir.exists() { break; }
                std::thread::sleep(Duration::from_millis(500));
            }).await?;
        }
    }
    put(&app, Reading { source: source.clone(), ..Default::default() });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
    fn t(n: u64) -> Totals { Totals { input: n, ..Default::default() } }

    #[test]
    fn summarizes_today_week_month_and_models() {
        // 2026-10-24 is a Saturday: week = from Mon 19th.
        let rows = vec![
            Row { date: "2026-10-24".into(), model: "a".into(), t: t(10) },
            Row { date: "2026-10-20".into(), model: "b".into(), t: t(5) },
            Row { date: "2026-10-02".into(), model: "a".into(), t: t(1) },
            Row { date: "2026-09-28".into(), model: "b".into(), t: t(100) },
            Row { date: "2026-10-25".into(), model: "a".into(), t: t(2) }, // UTC tomorrow → today
        ];
        let mut costs = BTreeMap::new();
        costs.insert("2026-10-24".to_string(), 1.5);
        costs.insert("2026-09-30".to_string(), 9.0);
        let mut r = Reading::default();
        summarize(&mut r, &rows, &costs, d("2026-10-24"));
        assert_eq!((r.today.sum(), r.week.sum(), r.month.sum()), (12, 17, 18));
        assert_eq!(r.daily.len(), 30);
        assert_eq!(r.daily.last().unwrap(), &("2026-10-24".to_string(), 12));
        assert_eq!(r.models.iter().map(|m| (m.name.as_str(), m.tokens)).collect::<Vec<_>>(), vec![("b", 105), ("a", 13)]);
        assert_eq!((r.cost_today, r.cost_month), (Some(1.5), Some(1.5)));
        let mut r = Reading::default();
        summarize(&mut r, &rows, &BTreeMap::new(), d("2026-10-24"));
        assert_eq!(r.cost_today, None, "no cost source: no made-up cost");
    }

    #[test]
    fn parses_admin_reports() {
        let u = json!({"data":[{"starting_at":"2026-10-24T00:00:00Z","results":[{"model":"claude-opus-5-5","uncached_input_tokens":10,"output_tokens":3,
            "cache_read_input_tokens":100,"cache_creation":{"ephemeral_5m_input_tokens":4,"ephemeral_1h_input_tokens":6}}]}],"has_more":false});
        let mut rows = vec![];
        parse_anthropic_usage(&u, &mut rows);
        assert_eq!(rows[0].t, Totals { input: 10, output: 3, cache_read: 100, cache_write: 10 });
        let mut c = BTreeMap::new();
        parse_anthropic_cost(&json!({"data":[{"starting_at":"2026-10-24T00:00:00Z","results":[{"amount":"250.5","currency":"USD"}]}]}), &mut c);
        assert_eq!(c["2026-10-24"], 2.505);
        let o = json!({"data":[{"start_time":1792800000,"results":[{"model":"gpt-x","input_tokens":50,"input_cached_tokens":20,"output_tokens":7}]}]});
        let mut rows = vec![];
        parse_openai_usage(&o, &mut rows);
        assert_eq!(rows[0].t, Totals { input: 30, output: 7, cache_read: 20, cache_write: 0 });
        let mut c = BTreeMap::new();
        parse_openai_cost(&json!({"data":[{"start_time":1792800000,"results":[{"amount":{"value":0.25,"currency":"usd"}}]}]}), &mut c);
        assert_eq!(c.values().sum::<f64>(), 0.25);
    }

    #[test]
    fn rejects_normal_keys_in_plain_words() {
        assert!(key_shape_error("anthropic", "sk-ant-api03-xyz").unwrap().contains("normal key"));
        assert!(key_shape_error("anthropic", "sk-ant-admin01-xyz").is_none());
        assert!(key_shape_error("openai", "sk-proj-abc").unwrap().contains("normal key"));
        assert!(key_shape_error("openai", "sk-admin-abc").is_none());
    }

    #[test]
    fn reads_claude_code_lines_once_per_message() {
        let mut lg = Logs::default();
        let l = r#"{"type":"assistant","timestamp":"2026-10-24T07:28:22.588Z","cwd":"C:\\laragon\\www\\glassBar","requestId":"r1","message":{"model":"claude-opus-5-5","id":"m1","content":[{"type":"text","text":"usage"}],"usage":{"input_tokens":2,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":50,"cache_creation":{"ephemeral_1h_input_tokens":100,"ephemeral_5m_input_tokens":0}}}}"#;
        lg.line(l, "x");
        lg.line(l, "x"); // same message written twice
        lg.line(&l.replace("assistant\",\"timestamp", "user\",\"timestamp"), "x");
        lg.line(&l.replace("claude-opus-5-5", "<synthetic>").replace("m1", "m2"), "x");
        assert_eq!(lg.entries.len(), 1);
        let e = &lg.entries[0];
        assert_eq!((e.t.sum(), e.write_1h, &*e.project), (1152, 100, "C:\\laragon\\www\\glassBar"));
        // 2*4 + 50*20 + 1000*0.2 + 100*4*2 = 2008 per million
        assert!((cost_of(e).unwrap() - 0.002008).abs() < 1e-9);
        assert_eq!(price("claude-opus-5"), Some((5.0, 25.0, 0.5)));
        assert_eq!(price("gpt-5"), None);
    }

    #[test]
    fn five_hour_block_starts_on_the_hour_and_ends() {
        let h = 3_600_000;
        let base = 1_000 * h; // on the hour
        let ts = vec![(base + 10 * 60_000, t(1)), (base + 2 * h, t(2)), (base - 9 * h, t(50))];
        let s = current_block(ts.clone(), base + 3 * h).unwrap();
        assert_eq!((s.totals.sum(), s.started_at, s.resets_at), (3, base, base + 5 * h));
        assert!(current_block(ts, base + 5 * h).is_none(), "block over");
    }

    #[test]
    fn web_limits_and_signed_out() {
        let c = json!({"usage":{"five_hour":{"utilization":62.0,"resets_at":"2026-10-24T07:40:00Z"},"seven_day":{"utilization":18.5,"resets_at":null},"seven_day_opus":null}});
        let l = parse_claude_web(&c["usage"]);
        assert_eq!(l.iter().map(|x| (x.key.as_str(), x.pct)).collect::<Vec<_>>(), vec![("session", 62.0), ("week", 18.5)]);
        assert_eq!(l[0].resets_at, Some(1792827600000));
        let g = json!({"usage":{"rate_limit":{"primary_window":{"used_percent":40,"limit_window_seconds":18000,"reset_after_seconds":60},
            "secondary_window":{"used_percent":5,"limit_window_seconds":604800,"reset_at":1792900000}}}});
        let l = parse_web(&g, 1000).unwrap();
        assert_eq!((l[0].label.as_str(), l[0].resets_at), ("5-hour limit", Some(61_000)));
        assert_eq!((l[1].key.as_str(), l[1].resets_at), ("week", Some(1792900000000)));
        assert_eq!(parse_web(&json!({"signin":true}), 0).unwrap_err().1, "auth");
        assert_eq!(parse_web(&json!({"usage":{"other":1}}), 0).unwrap_err().1, "broken");
    }

    #[test]
    fn alerts_once_per_window_and_on_reset() {
        let l = |pct: f64, r: i64| vec![Limit { key: "session".into(), label: "5-hour limit".into(), pct, resets_at: Some(r) }];
        let w = 10 * 900_000;
        let a = alert_plan("claudeplan", "Claude", &[], &l(82.0, w), &[]);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].0, "Claude 82% used");
        assert!(a[0].1.starts_with("5-hour limit · resets at "));
        let sent: Vec<String> = a.into_iter().map(|x| x.2).collect();
        assert!(alert_plan("claudeplan", "Claude", &l(82.0, w), &l(85.0, w + 5_000), &sent).is_empty(), "same window: no repeat");
        let b = alert_plan("claudeplan", "Claude", &l(85.0, w), &l(96.0, w), &sent);
        assert_eq!(b.len(), 1, "crossing 95 is its own alert");
        let r = alert_plan("claudeplan", "Claude", &l(96.0, w), &l(0.0, w + 5 * 3_600_000), &sent);
        assert_eq!(r[0].0, "Claude session reset");
        assert_eq!(r[0].1, "Full limit available");
    }
}
