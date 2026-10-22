//! Now playing via Global System Media Transport Controls (Spotify, browsers, ...).
use super::super::{MediaAction, MediaInfo, Res};
use super::{com_init, err};
use base64::Engine;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::time::Duration;
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::*;
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};

type Manager = GlobalSystemMediaTransportControlsSessionManager;
type Session = GlobalSystemMediaTransportControlsSession;

/// (track key, art data URL): album art is only re-read when the track changes.
static ART: Mutex<(String, Option<String>)> = Mutex::new((String::new(), None));

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

pub fn info() -> Res<Option<MediaInfo>> {
    let Some(s) = session() else { return Ok(None) };
    let props = s.TryGetMediaPropertiesAsync().and_then(|op| op.join()).map_err(err)?;
    let title = props.Title().map(|t| t.to_string()).unwrap_or_default();
    let artist = props.Artist().map(|t| t.to_string()).unwrap_or_default();
    let album = props.AlbumTitle().map(|t| t.to_string()).unwrap_or_default();
    let app = app_name(&s.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default());

    let key = format!("{app}\u{1}{title}\u{1}{artist}");
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

pub fn seek(position_ms: i64) -> Res<()> {
    let s = session().ok_or("nothing is playing")?;
    s.TryChangePlaybackPositionAsync(position_ms * 10_000).and_then(|op| op.join()).map(|_| ()).map_err(err)
}

pub fn watch(on_change: Box<dyn Fn(Option<MediaInfo>) + Send>) {
    std::thread::spawn(move || {
        let Ok(mgr) = manager() else { return };
        // true = the current session changed (re-subscribe), false = same session, new data.
        let (tx, rx) = channel::<bool>();
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
