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
use crate::net::enc;
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
    // Checked: `min` is whatever parsed, and a hand-written or garbled .lrc can carry a
    // number big enough to wrap. Release builds have overflow checks off, so it would wrap
    // silently and `.max(0)` below would hide it.
    min.checked_mul(60_000)?.checked_add(sec * 1000)?.checked_add(frac_ms)
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
        l.at_ms = l.at_ms.saturating_sub(offset).max(0);
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
///
/// "lyrics2": misses saved under the old, 2-second length rule are not trusted any more.
pub fn cache_key(artist: &str, title: &str) -> String {
    format!("lyrics2:{}|{}", artist.trim().to_lowercase(), title.trim().to_lowercase())
}

/// Browser players report a video's title and its channel: "Song (Official Video)" by
/// "ArtistVEVO", or "Artist - Song" by "Artist - Topic". The song and the singer, as LRCLIB
/// knows them; unchanged when there is nothing to clean.
pub fn clean(artist: &str, title: &str) -> (String, String) {
    const NOISE: [&str; 9] = ["official", "lyric", "audio", "video", "visuali", " mv", "hd", "4k", "music"];
    let mut a = artist.trim().to_string();
    for suffix in [" - Topic", "VEVO", " Official"] {
        if a.len() > suffix.len() && a.to_lowercase().ends_with(&suffix.to_lowercase()) { a.truncate(a.len() - suffix.len()); }
    }
    let a = a.trim().to_string();
    // Drop bracketed parts that only describe the upload: (Official Video), [Lyrics], (HD).
    let mut t = String::new();
    let mut rest = title;
    while let Some(i) = rest.find(['(', '[']) {
        let close = if rest[i..].starts_with('(') { ')' } else { ']' };
        let Some(j) = rest[i..].find(close) else { break };
        let inside = rest[i + 1..i + j].to_lowercase();
        t.push_str(&rest[..i]);
        if !NOISE.iter().any(|n| inside.contains(n)) { t.push_str(&rest[i..=i + j]); }
        rest = &rest[i + j + 1..];
    }
    t.push_str(rest);
    let mut t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    // "Artist - Song" with the artist in the title too.
    if let Some((left, right)) = t.split_once(" - ") {
        if left.trim().eq_ignore_ascii_case(&a) || a.is_empty() {
            let (l, r) = (left.trim().to_string(), right.trim().to_string());
            return (if a.is_empty() { l } else { a }, r);
        }
    }
    if t.is_empty() { t = title.trim().to_string(); }
    (a, t)
}

fn synced_of(v: &Value) -> Option<Vec<LyricLine>> {
    let s = v.get("syncedLyrics")?.as_str()?;
    let lines = parse_lrc(s);
    (!lines.is_empty()).then_some(lines)
}

/// LRCLIB answering "I don't have that track", as opposed to not answering at all.
///
/// Only a 404 means absent. A 429, a 5xx, or a captive portal replying with HTML all mean we
/// couldn't ask -- and must never be cached as a miss, or one bad minute would hide a song's
/// lyrics until tomorrow.
/// Worth asking again in a moment: LRCLIB overloaded (5xx), rate-limited (429), or the
/// network hiccuped. Not a 404 (the song isn't there) and not a malformed reply.
fn retryable(err: &str) -> bool {
    err.starts_with("HTTP 5") || err.starts_with("HTTP 429") || err.starts_with("network:")
}

/// LRCLIB answers 503 to a good share of requests at busy times (2 of 3 identical requests
/// while testing): try three times, 0.4 s and 1.2 s apart, before giving up on this ask.
fn get_json(url: &str) -> Result<Value, String> {
    let mut last = String::new();
    for wait_ms in [0, 400, 1200] {
        if wait_ms > 0 { std::thread::sleep(std::time::Duration::from_millis(wait_ms)); }
        match crate::net::get_json(url) {
            Err(e) if retryable(&e) => last = e,
            r => return r,
        }
    }
    Err(last)
}

fn is_absent(err: &str) -> bool {
    err.starts_with("HTTP 404")
}

/// How far a candidate's length may be from the track's: 6 % of it, at least 3 s. Players and
/// releases disagree by seconds (Spotify's "back to friends" is 189 s, LRCLIB's 199 s); a live
/// version or a remix is usually much further off.
fn tolerance(dur_s: i64) -> f64 { (dur_s as f64 * 0.06).max(3.0) }

/// The synced candidate closest in length, within the tolerance. Without a length check the
/// first hit wins, so a live version or a remix takes the studio track's place, with timings
/// wrong for the whole song. A track whose own length is unknown can't be checked: the first.
fn pick(v: &Value, dur_s: i64) -> Option<Vec<LyricLine>> {
    let all = v.as_array()?;
    if dur_s <= 0 { return all.iter().find_map(synced_of); }
    let mut near: Vec<(f64, &Value)> = all.iter()
        .filter_map(|c| Some(((c.get("duration")?.as_f64()? - dur_s as f64).abs(), c)))
        .filter(|(d, _)| *d <= tolerance(dur_s))
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.into_iter().find_map(|(_, c)| synced_of(c))
}

/// What LRCLIB said. Only `Absent` may be remembered as "this song has no lyrics": `NoMatch`
/// can be a player that briefly reported the previous song's length at a track change.
enum Found { Lines(Vec<LyricLine>), Absent, NoMatch }

fn search(url: &str, dur_s: i64) -> Result<Found, String> {
    match get_json(url) {
        Ok(v) => Ok(match pick(&v, dur_s) {
            Some(l) => Found::Lines(l),
            None if v.as_array().is_some_and(|a| a.iter().any(|c| synced_of(c).is_some())) => Found::NoMatch,
            None => Found::Absent,
        }),
        Err(e) if is_absent(&e) => Ok(Found::Absent),
        Err(e) => Err(e),
    }
}

/// Exact match first, then a search by artist and title, then (for a browser's video title)
/// the cleaned-up song and singer, then a free-text search. Err = couldn't ask at all.
fn fetch(artist: &str, title: &str, album: &str, dur_s: i64) -> Result<Found, String> {
    let exact = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
        enc(artist), enc(title), enc(album), dur_s
    );
    match get_json(&exact) {
        Ok(v) => { if let Some(l) = synced_of(&v) { return Ok(Found::Lines(l)); } }
        Err(e) if is_absent(&e) => {} // no exact match: search
        Err(e) => return Err(e),
    }
    let mut best = search(&format!("https://lrclib.net/api/search?artist_name={}&track_name={}", enc(artist), enc(title)), dur_s)?;
    let (a, t) = clean(artist, title);
    if !matches!(best, Found::Lines(_)) && (a != artist.trim() || t != title.trim()) {
        if let Found::Lines(l) = search(&format!("https://lrclib.net/api/search?artist_name={}&track_name={}", enc(&a), enc(&t)), dur_s)? { return Ok(Found::Lines(l)); }
    }
    if !matches!(best, Found::Lines(_)) {
        match search(&format!("https://lrclib.net/api/search?q={}", enc(&format!("{a} {t}"))), dur_s)? {
            Found::Lines(l) => return Ok(Found::Lines(l)),
            Found::NoMatch => best = Found::NoMatch,
            Found::Absent => {}
        }
    }
    Ok(best)
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
        // Lyrics of another length only: maybe the player's length was wrong for a moment.
        // Not remembered, so the next ask (with the right length) tries again.
        Ok(Found::NoMatch) => cached.unwrap_or_else(none),
        Ok(found) => {
            let r = match found {
                Found::Lines(lines) => Lyrics { lines, source: "lrclib".into(), fetched_at: now_ms() },
                _ => none(),
            };
            let db = app.state::<Db>();
            kv_set(&db.0.lock().unwrap(), &key, &serde_json::to_string(&r).unwrap());
            r
        }
        // Offline: show whatever was cached and don't remember the failure.
        // "offline" (not "none"): the page asks again in a little while. LRCLIB answers 503 now
        // and then; without this a passing error lost the lyrics for the whole song.
        Err(_) => cached.unwrap_or_else(|| Lyrics { source: "offline".into(), ..none() }),
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
    fn busy_and_flaky_answers_are_asked_again_but_not_a_missing_song() {
        assert!(retryable("HTTP 503 Service Unavailable"));
        assert!(retryable("HTTP 429 Too Many Requests"));
        assert!(retryable("network: timed out"));
        assert!(!retryable("HTTP 404 Not Found"));
        assert!(!retryable("bad response: expected value"));
    }

    #[test]
    fn only_a_404_counts_as_the_track_being_absent() {
        // Anything else means we couldn't ask, and must not be remembered as "no lyrics".
        assert!(is_absent("HTTP 404 Not Found"));
        assert!(!is_absent("HTTP 429 Too Many Requests"));
        assert!(!is_absent("HTTP 503 Service Unavailable"));
        assert!(!is_absent("network: timed out"));
        assert!(!is_absent("bad response: expected value"), "a captive portal answering HTML");
    }

    #[test]
    fn the_search_takes_a_candidate_of_the_right_length() {
        let results = serde_json::json!([
            { "duration": 412.0, "syncedLyrics": "[00:01.00]live version" },
            { "duration": 296.0, "syncedLyrics": "[00:02.00]the studio one" },
        ]);
        assert_eq!(at(&pick(&results, 296).unwrap()), vec![(2_000, "the studio one")]);
        // Nothing close enough is better than the wrong song's timings.
        assert!(pick(&results, 120).is_none());
        // A few seconds out is the same recording; the closest one wins.
        assert_eq!(at(&pick(&results, 301).unwrap()), vec![(2_000, "the studio one")]);
        assert!(pick(&results, 330).is_none(), "34 s out: another version");
    }

    #[test]
    fn a_radio_edit_ten_seconds_shorter_still_finds_its_lyrics() {
        // Spotify's "back to friends" is 189 s; every LRCLIB copy is 199 s.
        let results = serde_json::json!([{ "duration": 82.0, "syncedLyrics": "[00:01.00]snippet" }, { "duration": 199.0, "syncedLyrics": "[00:02.00]song" }]);
        assert_eq!(at(&pick(&results, 189).unwrap()), vec![(2_000, "song")]);
    }

    #[test]
    fn cleans_a_browser_video_title_into_song_and_singer() {
        assert_eq!(clean("Taylor Swift", "The Archer"), ("Taylor Swift".into(), "The Archer".into()), "nothing to clean");
        assert_eq!(clean("LordHuronVEVO", "The Night We Met (Official Video)"), ("LordHuron".into(), "The Night We Met".into()));
        assert_eq!(clean("Lord Huron - Topic", "The Night We Met"), ("Lord Huron".into(), "The Night We Met".into()));
        assert_eq!(clean("sombr", "sombr - back to friends [Lyrics]"), ("sombr".into(), "back to friends".into()));
        assert_eq!(clean("Song (feat. X)", "Song (feat. X)").1, "Song (feat. X)", "a real part of the title stays");
    }

    #[test]
    fn the_search_cannot_check_a_track_with_no_duration() {
        let results = serde_json::json!([{ "syncedLyrics": "[00:01.00]something" }]);
        assert!(pick(&results, 0).is_some(), "a live stream: take what there is");
        assert!(pick(&results, 296).is_none(), "a real track: no duration to agree with");
        assert!(pick(&serde_json::json!([]), 0).is_none());
    }

    #[test]
    fn a_hostile_timestamp_cannot_overflow() {
        assert_eq!(stamp_ms("9223372036854775:00"), None, "minutes big enough to wrap");
        assert_eq!(stamp_ms("153722867280913:00"), None, "one minute past what an i64 holds");
        // Just inside the range is not an error: a nonsense timestamp that simply never
        // matches a playing position does no harm.
        assert_eq!(stamp_ms("153722867280912:00"), Some(9_223_372_036_854_720_000));
        // And an offset big enough to wrap the subtraction clamps instead.
        assert_eq!(at(&parse_lrc("[offset:-9223372036854775807]\n[00:10.00]Line\n")), vec![(i64::MAX, "Line")]);
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


/// Live check against LRCLIB (network, read only): `cargo test -- --ignored lrclib_live --nocapture`.
#[cfg(test)]
#[test]
#[ignore]
fn lrclib_live() {
    for (a, t, d) in [("sombr", "back to friends", 189), ("Taylor Swift", "The Archer", 211), ("LordHuronVEVO", "The Night We Met (Official Video)", 208)] {
        let n = match fetch(a, t, "", d) { Ok(Found::Lines(l)) => format!("{} lines", l.len()), Ok(Found::Absent) => "absent".into(), Ok(Found::NoMatch) => "no match".into(), Err(e) => e };
        println!("{a} - {t}: {n}");
    }
}

/// What is playing and what the lookup finds for it (network, read only):
/// `cargo test -- --ignored playing_now --nocapture`.
#[cfg(test)]
#[test]
#[ignore]
fn playing_now() {
    let Ok(Some(m)) = crate::platform::media().info() else { println!("NOTHING PLAYING"); return };
    let r = match fetch(&m.artist, &m.title, &m.album, m.duration_ms / 1000) { Ok(Found::Lines(l)) => format!("{} lines", l.len()), Ok(Found::Absent) => "absent".into(), Ok(Found::NoMatch) => "no match (other lengths only)".into(), Err(e) => e };
    println!("PLAYING app={} | artist={} | title={} | album={} | dur={}s | pos={} | playing={} => {r}", m.app, m.artist, m.title, m.album, m.duration_ms / 1000, m.position_ms, m.playing);
}
