//! One shared HTTP client for all widget fetches (weather, prayer times, IPU, location).
use std::sync::OnceLock;
use std::time::Duration;

fn client() -> &'static reqwest::blocking::Client {
    static C: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("GlassBar/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("http client")
    })
}

/// GET a URL and parse JSON. Blocking: call from the blocking pool or a worker thread.
pub fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let r = client().get(url).send().map_err(|e| format!("network: {e}"))?;
    if !r.status().is_success() { return Err(format!("HTTP {}", r.status())); }
    r.json().map_err(|e| format!("bad response: {e}"))
}

/// Percent-encode a query value.
pub fn enc(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}
