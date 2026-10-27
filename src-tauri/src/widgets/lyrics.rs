//! Synced lyrics for whatever is playing.
//!
//! A local `.lrc` in `%APPDATA%\GlassBar\lyrics` always wins; otherwise LRCLIB
//! (lrclib.net, free, no key). Windows' media API gives no file path for Spotify or browser
//! playback, so there is nothing to look "next to" -- that folder is both the offline source
//! and the fix when LRCLIB is wrong.
//!
//! Only *synced* lyrics are used. A plain block of text can't follow the music, and a static
//! verse on a dock is noise, so a track with only plain lyrics counts as having none.
//!
//! Results are cached in the `kv` table: a hit for 30 days, a miss for a day, so an unknown
//! song isn't looked up again on every play. A network failure is never cached, or one moment
//! offline would hide lyrics until tomorrow.
use crate::db::{kv_get, kv_set, now_ms, Db};
use crate::net::{enc, get_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const TTL_MS: i64 = 30 * 24 * 60 * 60_000;
const MISS_TTL_MS: i64 = 24 * 60 * 60_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub at_ms: i64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    /// Timed lines, earliest first. Empty means this track has no synced lyrics.
    pub lines: Vec<LyricLine>,
    /// "local" (a .lrc on this PC), "lrclib", or "none".
    pub source: String,
    pub fetched_at: i64,
}

fn none() -> Lyrics {
    Lyrics { lines: vec![], source: "none".into(), fetched_at: now_ms() }
}

/// One `[..]` tag as milliseconds, or None when it isn't a timestamp (so `[ar:Dave]`,
/// `[length:03:20]` and `[offset:+500]` all fall through to the metadata branch).
/// Accepts `mm:ss`, `mm:ss.c`, `mm:ss.cc` and `mm:ss.mmm`, with `:` also allowed as the
/// fraction separator because some files are written that way.
fn stamp_ms(tag: &str) -> Option<i64> {
    let (m, rest) = tag.split_once(':')?;
    let min: i64 = m.trim().parse().ok()?;
    if min < 0 {
        return None;
    }
    let (s, frac) = match rest.split_once(['.', ':']) {
        Some((s, f)) => (s, f),
        None => (rest, ""),
    };
    let sec: i64 = s.trim().parse().ok()?;
    if !(0..60).contains(&sec) {
        return None;
    }
    let frac_ms = if frac.trim().is_empty() {
        0
    } else {
        let digits: String = frac.trim().chars().take_while(|c| c.is_ascii_digit()).take(3).collect();
        if digits.is_empty() {
            return None;
        }
        let v: i64 = digits.parse().ok()?;
        match digits.len() {
            1 => v * 100,
            2 => v * 10,
            _ => v,
        }
    };
    Some(min * 60_000 + sec * 1000 + frac_ms)
}

/// Timed lines from LRC text, earliest first. Metadata tags are skipped. `[offset:+N]` shifts
/// every line by N ms; the LRC convention is that a positive offset makes the lyrics appear
/// *earlier*, so it is subtracted.
pub fn parse_lrc(text: &str) -> Vec<LyricLine> {
    let mut offset = 0i64;
    let mut out: Vec<LyricLine> = Vec::new();
    for raw in text.lines() {
        let mut rest = raw.trim();
        let mut stamps: Vec<i64> = Vec::new();
        while rest.starts_with('[') {
            let Some(end) = rest.find(']') else { break };
            let tag = &rest[1..end];
            if let Some(ms) = stamp_ms(tag) {
                stamps.push(ms);
            } else if let Some(v) = tag.strip_prefix("offset:") {
                offset = v.trim().trim_start_matches('+').parse().unwrap_or(0);
            }
            rest = rest[end + 1..].trim_start();
        }
        if stamps.is_empty() {
            continue; // a plain line, or a metadata-only line
        }
        for at in stamps {
            out.push(LyricLine { at_ms: at, text: rest.to_string() });
        }
    }
    // The offset tag can appear after the lines it affects, so it is applied at the end.
    for l in &mut out {
        l.at_ms = (l.at_ms - offset).max(0);
    }
    out.sort_by_key(|l| l.at_ms);
    out
}

/// Characters Windows won't take in a file name, plus the ones that would escape the folder.
fn safe(s: &str) -> String {
    s.chars().map(|c| if r#"<>:"/\|?*"#.contains(c) || c.is_control() { '_' } else { c }).collect::<String>().trim().to_string()
}

pub fn local_path(artist: &str, title: &str) -> PathBuf {
    crate::data_dir().join("lyrics").join(format!("{} - {}.lrc", safe(artist), safe(title)))
}

/// A .lrc the user put there. Read lossily: these files are often not UTF-8.
fn local(artist: &str, title: &str) -> Option<Vec<LyricLine>> {
    let bytes = std::fs::read(local_path(artist, title)).ok()?;
    let lines = parse_lrc(&String::from_utf8_lossy(&bytes));
    (!lines.is_empty()).then_some(lines)
}

/// Keyed on artist and title only: the same song has the same lyrics whichever album or
/// release it came from, and the reported duration can differ by a second between players.
pub fn cache_key(artist: &str, title: &str) -> String {
    format!("lyrics:{}|{}", artist.trim().to_lowercase(), title.trim().to_lowercase())
}

fn synced_of(v: &Value) -> Option<Vec<LyricLine>> {
    let s = v.get("syncedLyrics")?.as_str()?;
    let lines = parse_lrc(s);
    (!lines.is_empty()).then_some(lines)
}

/// Ok(Some) found · Ok(None) LRCLIB says this track has no synced lyrics · Err couldn't ask.
/// The distinction matters: only Ok(None) may be cached as a miss.
fn fetch(artist: &str, title: &str, album: &str, dur_s: i64) -> Result<Option<Vec<LyricLine>>, String> {
    let exact = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
        enc(artist), enc(title), enc(album), dur_s
    );
    match get_json(&exact) {
        Ok(v) => {
            if let Some(l) = synced_of(&v) {
                return Ok(Some(l));
            }
        }
        Err(e) if e.starts_with("network") => return Err(e),
        Err(_) => {} // 404 and friends: no exact match, try the search
    }
    let search = format!("https://lrclib.net/api/search?artist_name={}&track_name={}", enc(artist), enc(title));
    match get_json(&search) {
        Ok(v) => Ok(v.as_array().and_then(|a| a.iter().find_map(synced_of))),
        Err(e) if e.starts_with("network") => Err(e),
        Err(_) => Ok(None),
    }
}

fn get(app: &AppHandle, title: &str, artist: &str, album: &str, duration_ms: i64, lookup: bool) -> Lyrics {
    if title.trim().is_empty() {
        return none();
    }
    // A local file wins outright, and is the only source when the lookup is turned off.
    if let Some(lines) = local(artist, title) {
        return Lyrics { lines, source: "local".into(), fetched_at: now_ms() };
    }

    let key = cache_key(artist, title);
    let cached: Option<Lyrics> = {
        let db = app.state::<Db>();
        let raw = kv_get(&db.0.lock().unwrap(), &key);
        raw.and_then(|s| serde_json::from_str(&s).ok())
    };
    if let Some(c) = &cached {
        let ttl = if c.lines.is_empty() { MISS_TTL_MS } else { TTL_MS };
        if now_ms().saturating_sub(c.fetched_at) < ttl {
            return c.clone();
        }
    }
    if !lookup {
        return cached.unwrap_or_else(none);
    }

    match fetch(artist, title, album, duration_ms / 1000) {
        Ok(found) => {
            let r = match found {
                Some(lines) => Lyrics { lines, source: "lrclib".into(), fetched_at: now_ms() },
                None => none(),
            };
            let db = app.state::<Db>();
            kv_set(&db.0.lock().unwrap(), &key, &serde_json::to_string(&r).unwrap());
            r
        }
        // Offline: show whatever was cached and don't remember the failure.
        Err(_) => cached.unwrap_or_else(none),
    }
}

/// `lookup` false never touches the network: only a local .lrc or the cache is used.
#[tauri::command]
pub async fn lyrics_get(app: AppHandle, title: String, artist: String, album: String, duration_ms: i64, lookup: bool) -> Result<Lyrics, String> {
    crate::widgets::blocking(move || get(&app, &title, &artist, &album, duration_ms, lookup)).await
}

/// The folder a user drops .lrc files into, created on demand so "Open folder" always works.
#[tauri::command]
pub fn lyrics_folder() -> Result<String, String> {
    let d = crate::data_dir().join("lyrics");
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    Ok(d.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(l: &[LyricLine]) -> Vec<(i64, &str)> {
        l.iter().map(|x| (x.at_ms, x.text.as_str())).collect()
    }

    #[test]
    fn reads_timestamps_in_every_shape_lrc_files_use() {
        assert_eq!(stamp_ms("00:00"), Some(0));
        assert_eq!(stamp_ms("01:30"), Some(90_000));
        assert_eq!(stamp_ms("01:30.5"), Some(90_500));
        assert_eq!(stamp_ms("01:30.25"), Some(90_250));
        assert_eq!(stamp_ms("01:30.250"), Some(90_250));
        assert_eq!(stamp_ms("01:30:25"), Some(90_250), "some files use a colon for the fraction");
        assert_eq!(stamp_ms("100:00"), Some(6_000_000), "a long track is not an error");
    }

    #[test]
    fn is_not_fooled_by_metadata_tags() {
        assert_eq!(stamp_ms("ar:Dave"), None);
        assert_eq!(stamp_ms("ti:Sprinter"), None);
        assert_eq!(stamp_ms("offset:+500"), None);
        assert_eq!(stamp_ms("length:03:20"), None);
        assert_eq!(stamp_ms("00:60"), None, "60 seconds is not a second value");
        assert_eq!(stamp_ms("00:99.00"), None);
        assert_eq!(stamp_ms("nonsense"), None);
        assert_eq!(stamp_ms("-1:00"), None);
    }

    #[test]
    fn parses_a_normal_lrc_file() {
        let lines = parse_lrc("[ar:Dave]\n[ti:Sprinter]\n[00:12.00]First line\n[00:15.50]Second line\n");
        assert_eq!(at(&lines), vec![(12_000, "First line"), (15_500, "Second line")]);
    }

    #[test]
    fn one_line_can_carry_several_timestamps() {
        let lines = parse_lrc("[00:10.00][01:10.00]Chorus\n");
        assert_eq!(at(&lines), vec![(10_000, "Chorus"), (70_000, "Chorus")]);
    }

    #[test]
    fn sorts_lines_that_arrive_out_of_order() {
        let lines = parse_lrc("[00:30.00]Later\n[00:10.00]Earlier\n");
        assert_eq!(at(&lines), vec![(10_000, "Earlier"), (30_000, "Later")]);
    }

    #[test]
    fn keeps_blank_lines_as_gaps_and_drops_untimed_text() {
        let lines = parse_lrc("Some header\n[00:05.00]\n[00:08.00]Sings\n\n");
        assert_eq!(at(&lines), vec![(5_000, ""), (8_000, "Sings")]);
    }

    #[test]
    fn applies_an_offset_tag_wherever_it_sits() {
        // A positive offset means the lyrics should appear earlier.
        assert_eq!(at(&parse_lrc("[offset:+500]\n[00:10.00]Line\n")), vec![(9_500, "Line")]);
        assert_eq!(at(&parse_lrc("[00:10.00]Line\n[offset:+500]\n")), vec![(9_500, "Line")]);
        assert_eq!(at(&parse_lrc("[offset:-500]\n[00:10.00]Line\n")), vec![(10_500, "Line")]);
        assert_eq!(at(&parse_lrc("[offset:+99000]\n[00:10.00]Line\n")), vec![(0, "Line")], "never before the start");
    }

    #[test]
    fn empty_and_plain_text_yield_nothing() {
        assert!(parse_lrc("").is_empty());
        assert!(parse_lrc("just\nplain\nlyrics\n").is_empty());
        assert!(parse_lrc("[ar:Dave]\n[al:Split Decision]\n").is_empty());
    }

    #[test]
    fn takes_synced_lyrics_from_an_lrclib_reply_but_not_plain_ones() {
        let v = serde_json::json!({ "syncedLyrics": "[00:01.00]Hi", "plainLyrics": "Hi" });
        assert_eq!(at(&synced_of(&v).unwrap()), vec![(1_000, "Hi")]);
        assert!(synced_of(&serde_json::json!({ "plainLyrics": "Hi", "syncedLyrics": null })).is_none());
        assert!(synced_of(&serde_json::json!({ "instrumental": true, "syncedLyrics": "" })).is_none());
        assert!(synced_of(&serde_json::json!({})).is_none());
    }

    #[test]
    fn the_cache_key_ignores_case_and_stray_spaces() {
        assert_eq!(cache_key("Dave", "Sprinter"), cache_key("  dave ", "SPRINTER"));
        assert_ne!(cache_key("Dave", "Sprinter"), cache_key("Dave", "Starlight"));
    }

    #[test]
    fn a_track_name_cannot_escape_the_lyrics_folder() {
        let p = local_path("AC/DC", "../../evil");
        assert_eq!(p.parent().unwrap(), crate::data_dir().join("lyrics"));
        let name = p.file_name().unwrap().to_string_lossy();
        assert!(!name.contains('/') && !name.contains('\\'), "got {name}");
        assert!(name.starts_with("AC_DC - "), "got {name}");
    }
}
