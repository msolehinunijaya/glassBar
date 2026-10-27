//! Now playing via Global System Media Transport Controls (Spotify, browsers, ...).
use super::super::{MediaAction, MediaInfo, Res};
use super::{com_init, err};
use base64::Engine;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::*;
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};

type Manager = GlobalSystemMediaTransportControlsSessionManager;
type Session = GlobalSystemMediaTransportControlsSession;

/// (track key, art data URL): album art is only re-read when the track changes.
static ART: Mutex<(String, Option<String>)> = Mutex::new((String::new(), None));

/// The last seek GlassBar asked for: (track, target ms, when, unix ms). Players report the new
/// position late -- Spotify sometimes first repeats the old one -- so for 3 s `info()` trusts
/// the seek over a report that disagrees with it. Without this, clicking a lyric line made the
/// lyrics jump back and forth.
static SEEK: Mutex<Option<(String, i64, i64)>> = Mutex::new(None);
/// Wakes the watcher, so every window hears about a seek at once instead of when the player
/// gets round to reporting it.
static WAKE: OnceLock<Sender<bool>> = OnceLock::new();

fn now_unix_ms() -> i64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0) }

/// Pure: (position, updated-at) to report, given a recent seek. The reported position,
/// moved on to `now` while playing, is kept when it agrees with the seek (within 1.5 s);
/// otherwise the seek's own position is.
fn trusted(seek: Option<&(String, i64, i64)>, track: &str, pos: i64, upd: i64, playing: bool, now: i64) -> (i64, i64) {
    let Some((t, target, at)) = seek else { return (pos, upd) };
    if t != track || now - at > 3000 { return (pos, upd); }
    let expected = target + if playing { now - at } else { 0 };
    let reported = pos + if playing && upd > 0 { (now - upd).max(0) } else { 0 };
    if (reported - expected).abs() > 1500 { (expected, now) } else { (pos, upd) }
}

fn manager() -> windows::core::Result<Manager> {
    com_init();
    Manager::RequestAsync()?.join()
}

fn session() -> Option<Session> { manager().ok()?.GetCurrentSession().ok() }

/// Windows FILETIME-style ticks (100 ns since 1601) -> Unix ms.
fn unix_ms(universal_time: i64) -> i64 { (universal_time - 116_444_736_000_000_000) / 10_000 }

fn read_art(r: &IRandomAccessStreamReference) -> windows::core::Result<String> {
    let s = r.OpenReadAsync()?.join()?;
    let size = s.Size()? as u32;
    let mime = s.ContentType()?.to_string();
    let reader = DataReader::CreateDataReader(&s)?;
    reader.LoadAsync(size)?.join()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf)?;
    let mime = if mime.is_empty() { "image/png".into() } else { mime };
    Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(buf)))
}

fn app_name(id: &str) -> String {
    // "Spotify.exe" -> "Spotify"; "Microsoft.ZuneMusic_8wekyb!Microsoft.ZuneMusic" -> "Microsoft.ZuneMusic"
    let id = id.rsplit('!').next().unwrap_or(id);
    id.strip_suffix(".exe").unwrap_or(id).to_string()
}

pub fn source_id() -> Option<String> {
    session()?.SourceAppUserModelId().ok().map(|t| t.to_string()).filter(|s| !s.is_empty())
}

pub fn info() -> Res<Option<MediaInfo>> {
    let Some(s) = session() else { return Ok(None) };
    let props = s.TryGetMediaPropertiesAsync().and_then(|op| op.join()).map_err(err)?;
    let title = props.Title().map(|t| t.to_string()).unwrap_or_default();
    let artist = props.Artist().map(|t| t.to_string()).unwrap_or_default();
    let album = props.AlbumTitle().map(|t| t.to_string()).unwrap_or_default();
    let app = app_name(&s.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default());

    let key = format!("{app}\u{1}{title}\u{1}{artist}");
    let track = key.clone();
    let art = {
        let mut cache = ART.lock().unwrap();
        if cache.0 != key || cache.1.is_none() {
            let art = props.Thumbnail().ok().and_then(|r| read_art(&r).ok());
            *cache = (key, art);
        }
        cache.1.clone()
    };

    let pb = s.GetPlaybackInfo().map_err(err)?;
    let playing = pb.PlaybackStatus().map(|p| p == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing).unwrap_or(false);
    let (can_next, can_prev) = pb.Controls()
        .map(|c| (c.IsNextEnabled().unwrap_or(false), c.IsPreviousEnabled().unwrap_or(false)))
        .unwrap_or((false, false));

    let tl = s.GetTimelineProperties().map_err(err)?;
    let ms = |t: windows::core::Result<windows::Foundation::TimeSpan>| t.map(|t| t.Duration / 10_000).unwrap_or(0);
    let start = ms(tl.StartTime());
    let position_ms = ms(tl.Position()) - start;
    let duration_ms = ms(tl.EndTime()) - start;
    let updated_at_ms = tl.LastUpdatedTime().map(|d| unix_ms(d.UniversalTime)).unwrap_or(0);
    let (position_ms, updated_at_ms) = trusted(SEEK.lock().unwrap().as_ref(), &track, position_ms, updated_at_ms, playing, now_unix_ms());

    Ok(Some(MediaInfo { app, title, artist, album, art, playing, position_ms, duration_ms, updated_at_ms, can_next, can_prev }))
}

pub fn control(a: MediaAction) -> Res<()> {
    let s = session().ok_or("nothing is playing")?;
    let op = match a {
        MediaAction::Toggle => s.TryTogglePlayPauseAsync(),
        MediaAction::Next => s.TrySkipNextAsync(),
        MediaAction::Prev => s.TrySkipPreviousAsync(),
    };
    op.and_then(|op| op.join()).map(|_| ()).map_err(err)
}

/// Pause every session that is playing; returns their app ids so `resume` can restart just those.
pub fn pause_playing() -> Res<Vec<String>> {
    use GlobalSystemMediaTransportControlsSessionPlaybackStatus as St;
    let mut paused = vec![];
    for s in manager().and_then(|m| m.GetSessions()).map_err(err)? {
        let playing = s.GetPlaybackInfo().and_then(|p| p.PlaybackStatus()).map(|p| p == St::Playing).unwrap_or(false);
        if playing && s.TryPauseAsync().and_then(|op| op.join()).unwrap_or(false) {
            paused.push(s.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default());
        }
    }
    Ok(paused)
}

/// Play again the sessions `pause_playing` paused, if they are still paused (not closed or stopped).
pub fn resume(ids: &[String]) -> Res<()> {
    use GlobalSystemMediaTransportControlsSessionPlaybackStatus as St;
    for s in manager().and_then(|m| m.GetSessions()).map_err(err)? {
        let id = s.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default();
        let paused = s.GetPlaybackInfo().and_then(|p| p.PlaybackStatus()).map(|p| p == St::Paused).unwrap_or(false);
        if paused && ids.contains(&id) { let _ = s.TryPlayAsync().and_then(|op| op.join()); }
    }
    Ok(())
}

pub fn seek(position_ms: i64) -> Res<()> {
    let s = session().ok_or("nothing is playing")?;
    // Remember it first (see SEEK), keyed like info() keys the track.
    if let Ok(props) = s.TryGetMediaPropertiesAsync().and_then(|op| op.join()) {
        let app = app_name(&s.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default());
        let (title, artist) = (props.Title().map(|t| t.to_string()).unwrap_or_default(), props.Artist().map(|t| t.to_string()).unwrap_or_default());
        *SEEK.lock().unwrap() = Some((format!("{app}\u{1}{title}\u{1}{artist}"), position_ms, now_unix_ms()));
    }
    // Our positions are relative to StartTime; the API wants an absolute position.
    let start = s.GetTimelineProperties().and_then(|t| t.StartTime()).map(|t| t.Duration).unwrap_or(0);
    let r = s.TryChangePlaybackPositionAsync(start + position_ms * 10_000).and_then(|op| op.join()).map(|_| ()).map_err(err);
    if let Some(w) = WAKE.get() { let _ = w.send(false); } // every window, now
    r
}

pub fn watch(on_change: Box<dyn Fn(Option<MediaInfo>) + Send>) {
    std::thread::spawn(move || {
        let Ok(mgr) = manager() else { return };
        // true = the current session changed (re-subscribe), false = same session, new data.
        let (tx, rx) = channel::<bool>();
        let _ = WAKE.set(tx.clone());
        let t = tx.clone();
        let _ = mgr.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| { let _ = t.send(true); Ok(()) }));
        let t = tx.clone();
        let _ = mgr.SessionsChanged(&TypedEventHandler::new(move |_, _| { let _ = t.send(true); Ok(()) }));

        let mut hooked: Option<(Session, [i64; 3])> = None;
        let mut rehook = true;
        loop {
            if rehook {
                if let Some((s, tok)) = hooked.take() {
                    let _ = s.RemoveMediaPropertiesChanged(tok[0]);
                    let _ = s.RemovePlaybackInfoChanged(tok[1]);
                    let _ = s.RemoveTimelinePropertiesChanged(tok[2]);
                }
                if let Ok(s) = mgr.GetCurrentSession() {
                    macro_rules! h {
                        () => {{ let t = tx.clone(); TypedEventHandler::new(move |_, _| { let _ = t.send(false); Ok(()) }) }};
                    }
                    let tok = [
                        s.MediaPropertiesChanged(&h!()).unwrap_or_default(),
                        s.PlaybackInfoChanged(&h!()).unwrap_or_default(),
                        s.TimelinePropertiesChanged(&h!()).unwrap_or_default(),
                    ];
                    hooked = Some((s, tok));
                }
            }
            on_change(info().ok().flatten());
            let Ok(first) = rx.recv() else { return };
            std::thread::sleep(Duration::from_millis(150)); // let the burst settle
            rehook = first;
            while let Ok(r) = rx.try_recv() { rehook |= r; }
        }
    });
}

#[cfg(test)]
mod seek_tests {
    use super::trusted;
    const NOW: i64 = 1_000_000;
    fn seek(at: i64) -> Option<(String, i64, i64)> { Some(("t".into(), 60_000, at)) }

    #[test]
    fn a_stale_report_right_after_a_seek_gives_way_to_the_seek() {
        // Seeked to 1:00 half a second ago; the player still says 0:20.
        assert_eq!(trusted(seek(NOW - 500).as_ref(), "t", 20_000, NOW - 4000, true, NOW), (60_500, NOW));
    }
    #[test]
    fn a_report_that_agrees_is_kept() {
        assert_eq!(trusted(seek(NOW - 500).as_ref(), "t", 60_300, NOW - 100, true, NOW), (60_300, NOW - 100));
    }
    #[test]
    fn after_three_seconds_or_another_song_the_player_is_believed() {
        assert_eq!(trusted(seek(NOW - 3500).as_ref(), "t", 20_000, NOW, true, NOW), (20_000, NOW));
        assert_eq!(trusted(seek(NOW - 500).as_ref(), "other", 20_000, NOW, true, NOW), (20_000, NOW));
        assert_eq!(trusted(None, "t", 20_000, NOW, true, NOW), (20_000, NOW));
    }
    #[test]
    fn paused_the_seek_stays_where_it_was_put() {
        assert_eq!(trusted(seek(NOW - 800).as_ref(), "t", 10_000, NOW - 9000, false, NOW), (60_000, NOW));
    }
}
