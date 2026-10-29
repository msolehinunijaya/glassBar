//! Core Audio: master volume, output devices, per-app sessions.
use super::super::{AudioDevice, AudioSession, AudioState, Res};
use super::{com_init, err, wide, windows_list::exe_path};
use std::ffi::c_void;
use std::time::Duration;
use windows::core::{Interface, GUID, HRESULT, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl};
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
    com_init();
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

fn default_device() -> windows::core::Result<IMMDevice> {
    unsafe { enumerator()?.GetDefaultAudioEndpoint(eRender, eConsole) }
}

fn endpoint() -> windows::core::Result<IAudioEndpointVolume> {
    unsafe { default_device()?.Activate(CLSCTX_ALL, None) }
}

fn id_of(d: &IMMDevice) -> String {
    unsafe {
        let Ok(p) = d.GetId() else { return String::new() };
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as *const c_void));
        s
    }
}

fn name_of(d: &IMMDevice) -> String {
    unsafe {
        d.OpenPropertyStore(STGM_READ)
            .and_then(|s| s.GetValue(&PKEY_Device_FriendlyName))
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "Unknown device".into())
    }
}

pub fn state() -> Res<AudioState> {
    let ep = endpoint().map_err(err)?;
    let (volume, muted) = unsafe {
        (ep.GetMasterVolumeLevelScalar().map_err(err)?, ep.GetMute().map_err(err)?.as_bool())
    };
    let default_id = default_device().map(|d| id_of(&d)).unwrap_or_default();
    let mut devices = Vec::new();
    unsafe {
        let all = enumerator().and_then(|e| e.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)).map_err(err)?;
        for i in 0..all.GetCount().map_err(err)? {
            let d = all.Item(i).map_err(err)?;
            let id = id_of(&d);
            devices.push(AudioDevice { is_default: id == default_id, name: name_of(&d), id });
        }
    }
    Ok(AudioState { volume, muted, devices })
}

/// Tags GlassBar's own master volume/mute changes, so the change notification can tell them
/// from the keyboard keys, Windows' flyout or another app (the volume dock alert skips ours).
const OURS: GUID = GUID::from_u128(0x5b7c2e1a_4d3f_4a8e_9c61_2f0d8e7b4a13);

pub fn set_volume(v: f32) -> Res<()> {
    unsafe { endpoint().and_then(|e| e.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0), &OURS)).map_err(err) }
}

pub fn set_mute(m: bool) -> Res<()> {
    unsafe { endpoint().and_then(|e| e.SetMute(m, &OURS)).map_err(err) }
}

/// One master volume change, straight from Core Audio: (volume, muted, made by GlassBar).
type Level = (f32, bool, bool);

#[windows_core::implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback(std::sync::mpsc::Sender<Level>);

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, d: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        // Runs on a Core Audio thread: hand it over and return at once.
        if let Some(d) = unsafe { d.as_ref() } { let _ = self.0.send((d.fMasterVolume, d.bMuted.as_bool(), d.guidEventContext == OURS)); }
        Ok(())
    }
}

/// After one of our own changes, notifications with another tag this soon are its echoes:
/// on some PCs another component re-applies the level right away (0.667 -> 0.669), tagged
/// as its own. Seen arriving in the same millisecond.
const ECHO_MS: u64 = 500;

/// Pure: a burst of notifications -> the level to report, if it came from outside GlassBar.
/// A burst holding one of ours, or arriving just after one (`ours_recently`), is ours.
fn outside_level(burst: &[Level], ours_recently: bool) -> Option<(f32, bool)> {
    if ours_recently || burst.iter().any(|l| l.2) { None } else { burst.last().map(|l| (l.0, l.1)) }
}

/// Undocumented but stable since Windows 7; used by EarTrumpet, SoundSwitch, etc.
#[windows_core::interface("f8679f50-850a-41cf-9c72-430f290290c8")]
unsafe trait IPolicyConfig: windows_core::IUnknown {
    fn GetMixFormat(&self, a: PCWSTR, b: *mut *mut c_void) -> HRESULT;
    fn GetDeviceFormat(&self, a: PCWSTR, b: i32, c: *mut *mut c_void) -> HRESULT;
    fn ResetDeviceFormat(&self, a: PCWSTR) -> HRESULT;
    fn SetDeviceFormat(&self, a: PCWSTR, b: *mut c_void, c: *mut c_void) -> HRESULT;
    fn GetProcessingPeriod(&self, a: PCWSTR, b: i32, c: *mut i64, d: *mut i64) -> HRESULT;
    fn SetProcessingPeriod(&self, a: PCWSTR, b: *mut i64) -> HRESULT;
    fn GetShareMode(&self, a: PCWSTR, b: *mut c_void) -> HRESULT;
    fn SetShareMode(&self, a: PCWSTR, b: *mut c_void) -> HRESULT;
    fn GetPropertyValue(&self, a: PCWSTR, b: *const c_void, c: *mut c_void) -> HRESULT;
    fn SetPropertyValue(&self, a: PCWSTR, b: *const c_void, c: *const c_void) -> HRESULT;
    fn SetDefaultEndpoint(&self, id: PCWSTR, role: ERole) -> HRESULT;
    fn SetEndpointVisibility(&self, a: PCWSTR, b: i32) -> HRESULT;
}
const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

pub fn set_default(id: &str) -> Res<()> {
    com_init();
    let w = wide(id);
    unsafe {
        let pc: IPolicyConfig = CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL).map_err(err)?;
        for role in [eConsole, eMultimedia, eCommunications] {
            pc.SetDefaultEndpoint(PCWSTR(w.as_ptr()), role).ok().map_err(err)?;
        }
    }
    Ok(())
}

/// All live sessions on the default device, with their volume control.
fn session_controls() -> Res<Vec<(u32, ISimpleAudioVolume)>> {
    let mut out = Vec::new();
    unsafe {
        let mgr: IAudioSessionManager2 = default_device().and_then(|d| d.Activate(CLSCTX_ALL, None)).map_err(err)?;
        let list = mgr.GetSessionEnumerator().map_err(err)?;
        for i in 0..list.GetCount().map_err(err)? {
            let Ok(c) = list.GetSession(i) else { continue };
            if c.GetState().map(|s| s == AudioSessionStateExpired).unwrap_or(true) { continue; }
            let Ok(c2) = c.cast::<IAudioSessionControl2>() else { continue };
            let pid = c2.GetProcessId().unwrap_or(0);
            if let Ok(v) = c.cast::<ISimpleAudioVolume>() { out.push((pid, v)); }
        }
    }
    Ok(out)
}

pub fn sessions() -> Res<Vec<AudioSession>> {
    let mut out: Vec<AudioSession> = Vec::new();
    for (pid, v) in session_controls()? {
        if out.iter().any(|s| s.pid == pid) { continue; } // one row per app (browsers open several)
        let exe = if pid == 0 { String::new() } else { exe_path(pid) };
        let name = if pid == 0 { "System sounds".to_string() } else {
            std::path::Path::new(&exe).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
        };
        let (volume, muted) = unsafe { (v.GetMasterVolume().unwrap_or(1.0), v.GetMute().map(|m| m.as_bool()).unwrap_or(false)) };
        out.push(AudioSession { pid, name, exe, volume, muted });
    }
    Ok(out)
}

pub fn set_session_volume(pid: u32, vol: f32) -> Res<()> {
    for (p, v) in session_controls()? {
        if p == pid { unsafe { v.SetMasterVolume(vol.clamp(0.0, 1.0), std::ptr::null()).map_err(err)? } }
    }
    Ok(())
}

pub fn set_session_mute(pid: u32, m: bool) -> Res<()> {
    for (p, v) in session_controls()? {
        if p == pid { unsafe { v.SetMute(m, std::ptr::null()).map_err(err)? } }
    }
    Ok(())
}

/// ponytail: 1 s poll instead of COM callbacks; no re-registration when the
/// default device changes, and the cost is a few COM calls per second.
/// Volume changes arrive at once through Core Audio's change notification on the default
/// speakers (re-registered when the default device changes); the device list is still
/// checked once a second. A notification also wakes the loop, so widgets update at once too.
pub fn watch(on_change: Box<dyn Fn(AudioState) + Send>, on_level: Box<dyn Fn(f32, bool) + Send>) {
    std::thread::spawn(move || {
        com_init();
        let (tx, rx) = std::sync::mpsc::channel::<Level>();
        let mut reg: Option<(String, IAudioEndpointVolume, IAudioEndpointVolumeCallback)> = None;
        let mut last: Option<AudioState> = None;
        let mut last_ours: Option<std::time::Instant> = None;
        loop {
            let id = default_device().map(|d| id_of(&d)).unwrap_or_default();
            if reg.as_ref().map(|r| r.0.as_str()) != Some(id.as_str()) {
                if let Some((_, ep, cb)) = reg.take() { unsafe { let _ = ep.UnregisterControlChangeNotify(&cb); } }
                if let Ok(ep) = endpoint() {
                    let cb: IAudioEndpointVolumeCallback = VolumeCallback(tx.clone()).into();
                    if unsafe { ep.RegisterControlChangeNotify(&cb) }.is_ok() { reg = Some((id, ep, cb)); }
                }
            }
            if let Ok(first) = rx.recv_timeout(Duration::from_secs(1)) {
                let burst: Vec<Level> = std::iter::once(first).chain(rx.try_iter()).collect();
                let recent = last_ours.is_some_and(|t| t.elapsed() < Duration::from_millis(ECHO_MS));
                if burst.iter().any(|l| l.2) { last_ours = Some(std::time::Instant::now()); }
                if let Some((v, m)) = outside_level(&burst, recent) { on_level(v, m); }
            }
            if let Ok(s) = state() {
                if last.as_ref() != Some(&s) {
                    on_change(s.clone());
                    last = Some(s);
                }
            }
        }
    });
}

/// Live output level 0..1 of the default speakers (Windows' own peak meter, the one the
/// volume mixer shows). Cheap: no audio is captured.
pub fn peak() -> Option<f32> {
    com_init();
    unsafe {
        let m: windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation = default_device().ok()?.Activate(CLSCTX_ALL, None).ok()?;
        m.GetPeakValue().ok()
    }
}

/// What the speakers are playing (WASAPI loopback), as mono f32 chunks: `on_chunk(samples, rate)`
/// runs every ~10 ms (empty slice = nothing playing). Returns when `on_chunk` says false, the
/// default output device changes (the caller just starts again), or on error.
#[windows_core::implement(IActivateAudioInterfaceCompletionHandler)]
struct Activated(std::sync::mpsc::SyncSender<()>);

impl IActivateAudioInterfaceCompletionHandler_Impl for Activated_Impl {
    fn ActivateCompleted(&self, _op: windows_core::Ref<IActivateAudioInterfaceAsyncOperation>) -> windows::core::Result<()> {
        let _ = self.0.send(());
        Ok(())
    }
}

/// A capture client for everything playing except one process tree, Windows 10 2004 and
/// later. 16-bit stereo 48 kHz.
/// Is `pid` a descendant of `root`? `parent` maps a process to its parent (one snapshot).
fn descends(mut pid: u32, root: u32, parent: &std::collections::HashMap<u32, u32>) -> bool {
    for _ in 0..8 {
        match parent.get(&pid) {
            Some(&p) if p == root => return true,
            Some(&p) if p != 0 && p != pid => pid = p,
            _ => return false,
        }
    }
    false
}

fn parents() -> std::collections::HashMap<u32, u32> {
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    let mut out = std::collections::HashMap::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok { out.insert(e.th32ProcessID, e.th32ParentProcessID); ok = Process32NextW(snap, &mut e).is_ok(); }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
    }
    out
}

/// The process GlassBar's sounds actually come out of: a WebView2 helper (Chromium's audio
/// service) that Windows doesn't count in GlassBar's process tree. Found among the audio
/// sessions; GlassBar itself until it has played anything.
fn own_audio_pid() -> u32 {
    let me = std::process::id();
    let parent = parents();
    session_controls().ok().and_then(|s| s.into_iter().map(|(p, _)| p).find(|&p| p != me && descends(p, me, &parent))).unwrap_or(me)
}

unsafe fn others_loopback(own: u32) -> windows::core::Result<(IAudioClient, usize, u32, u16)> { unsafe { process_loopback(own, PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE) } }

unsafe fn process_loopback(pid: u32, mode: PROCESS_LOOPBACK_MODE) -> windows::core::Result<(IAudioClient, usize, u32, u16)> {
    use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
    use windows::Win32::System::Variant::VT_BLOB;
    let params = AUDIOCLIENT_ACTIVATION_PARAMS {
        ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
        Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 { ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
            TargetProcessId: pid, ProcessLoopbackMode: mode } },
    };
    // Never dropped: PROPVARIANT's Drop calls PropVariantClear, which would free the blob
    // (it points at `params` on the stack) and corrupt the heap.
    let mut pv = std::mem::ManuallyDrop::new(PROPVARIANT::default());
    unsafe {
        let v = &mut *pv.Anonymous.Anonymous;
        v.vt = VT_BLOB;
        v.Anonymous.blob = BLOB { cbSize: std::mem::size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32, pBlobData: &params as *const _ as *mut u8 };
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let handler: IActivateAudioInterfaceCompletionHandler = Activated(tx).into();
        let op = ActivateAudioInterfaceAsync(VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, &IAudioClient::IID, Some(&*pv), &handler)?;
        rx.recv_timeout(Duration::from_secs(5)).map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_FAIL))?;
        let (mut hr, mut unk) = (HRESULT(0), None);
        op.GetActivateResult(&mut hr, &mut unk)?;
        hr.ok()?;
        let client: IAudioClient = unk.ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_FAIL))?.cast()?;
        let (ch, rate, bits) = (2u16, 48_000u32, 16u16);
        let fmt = WAVEFORMATEX { wFormatTag: WAVE_FORMAT_PCM as u16, nChannels: ch, nSamplesPerSec: rate, wBitsPerSample: bits,
                                 nBlockAlign: ch * bits / 8, nAvgBytesPerSec: rate * (ch * bits / 8) as u32, cbSize: 0 };
        client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM, 2_000_000, 0, &fmt, None)?;
        Ok((client, ch as usize, rate, bits))
    }
}

/// The classic capture of the whole output mix (GlassBar's own sounds included).
unsafe fn device_loopback(dev: &IMMDevice) -> Res<(IAudioClient, usize, u32, u16)> {
    unsafe {
        let client: IAudioClient = dev.Activate(CLSCTX_ALL, None).map_err(err)?;
        let fmt = client.GetMixFormat().map_err(err)?;
        let (ch, rate, bits) = ((*fmt).nChannels as usize, (*fmt).nSamplesPerSec, (*fmt).wBitsPerSample);
        // 200 ms buffer; shared-mode mix format is float32 on every modern Windows, 16-bit kept as a fallback.
        let init = client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 2_000_000, 0, fmt, None);
        CoTaskMemFree(Some(fmt as *const c_void));
        init.map_err(err)?;
        Ok((client, ch, rate, bits))
    }
}

/// What the speakers play, as mono chunks, until `on_chunk` returns false or the default
/// output device changes. GlassBar's own sounds are left out where Windows allows it.
pub fn loopback(on_chunk: &mut dyn FnMut(&[f32], u32) -> bool) -> Res<()> {
    com_init();
    unsafe {
        let dev = default_device().map_err(err)?;
        let dev_id = id_of(&dev);
        // Everything but GlassBar's own sounds (the adzan, chimes, the volume tick).
        let own = own_audio_pid();
        let (mut per_app, mut ownchk) = (true, std::time::Instant::now());
        let (client, ch, rate, bits) = match others_loopback(own) {
            Ok(c) => c,
            Err(e) => {
                static ONCE: std::sync::Once = std::sync::Once::new();
                ONCE.call_once(|| crate::applog::log(format!("equalizer: per-app capture unavailable ({e}); using the whole mix (GlassBar's own sounds and the volume level show)")));
                per_app = false;
                device_loopback(&dev)?
            }
        };
        if ch == 0 || !(bits == 32 || bits == 16) { return Err(format!("unsupported mix format ({bits}-bit)")); }
        let cap: IAudioCaptureClient = client.GetService().map_err(err)?;
        client.Start().map_err(err)?;

        let mut mono: Vec<f32> = Vec::new();
        let mut checked = std::time::Instant::now();
        let drain = |mono: &mut Vec<f32>| -> windows::core::Result<()> {
            while cap.GetNextPacketSize()? > 0 {
                let (mut data, mut frames, mut flags) = (std::ptr::null_mut(), 0u32, 0u32);
                cap.GetBuffer(&mut data, &mut frames, &mut flags, None, None)?;
                let n = frames as usize;
                if data.is_null() || flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
                    mono.extend(std::iter::repeat_n(0.0, n));
                } else if bits == 32 {
                    let s = std::slice::from_raw_parts(data as *const f32, n * ch);
                    mono.extend(s.chunks_exact(ch).map(|f| f.iter().sum::<f32>() / ch as f32));
                } else {
                    let s = std::slice::from_raw_parts(data as *const i16, n * ch);
                    mono.extend(s.chunks_exact(ch).map(|f| f.iter().map(|&v| v as f32).sum::<f32>() / (ch as f32 * 32768.0)));
                }
                cap.ReleaseBuffer(frames)?;
            }
            Ok(())
        };
        let res = loop {
            std::thread::sleep(Duration::from_millis(10));
            mono.clear();
            if let Err(e) = drain(&mut mono) { break Err(err(e)); }
            if !on_chunk(&mono, rate) { break Ok(()); }
            if checked.elapsed() > Duration::from_secs(1) {
                checked = std::time::Instant::now();
                if default_device().map(|d| id_of(&d)).ok().as_deref() != Some(dev_id.as_str()) { break Ok(()); }
            }
            // GlassBar's sound helper started (first sound) or restarted: capture again without it.
            if per_app && ownchk.elapsed() > Duration::from_secs(2) {
                ownchk = std::time::Instant::now();
                if own_audio_pid() != own { break Ok(()); }
            }
        };
        let _ = client.Stop();
        res
    }
}

#[cfg(test)]
mod tests {

    /// Live: `cargo test loopback_levels_live -- --ignored --nocapture` while music plays.
    /// Prints the level each capture sees at the current volume and at half of it.
    #[test]
    #[ignore]
    fn loopback_levels_live() {
        use windows::Win32::Media::Audio::*;
        super::com_init();
        let rms = |make: &dyn Fn() -> (IAudioClient, usize, u32, u16)| -> f32 {
            unsafe {
                let (client, ch, _, bits) = make();
                let cap: IAudioCaptureClient = client.GetService().unwrap();
                client.Start().unwrap();
                let (mut sum, mut n) = (0f64, 0usize);
                let t = std::time::Instant::now();
                while t.elapsed() < std::time::Duration::from_millis(1500) {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    while cap.GetNextPacketSize().unwrap() > 0 {
                        let (mut d, mut f, mut fl) = (std::ptr::null_mut(), 0u32, 0u32);
                        cap.GetBuffer(&mut d, &mut f, &mut fl, None, None).unwrap();
                        let k = f as usize * ch;
                        if !d.is_null() && fl & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 == 0 {
                            if bits == 32 { for v in std::slice::from_raw_parts(d as *const f32, k) { sum += (*v as f64).powi(2); } }
                            else { for v in std::slice::from_raw_parts(d as *const i16, k) { sum += (*v as f64 / 32768.0).powi(2); } }
                        }
                        n += k;
                        cap.ReleaseBuffer(f).unwrap();
                    }
                }
                let _ = client.Stop();
                if n == 0 { 0.0 } else { (sum / n as f64).sqrt() as f32 }
            }
        };
        let v0 = super::endpoint().and_then(|e| unsafe { e.GetMasterVolumeLevelScalar() }).unwrap();
        for v in [v0, v0 * 0.5, v0] {
            super::set_volume(v).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(400));
            let others = rms(&|| unsafe { super::others_loopback(super::own_audio_pid()).unwrap() });
            let device = rms(&|| unsafe { super::device_loopback(&super::default_device().unwrap()).unwrap() });
            eprintln!("volume {v:.2}: without GlassBar rms={others:.4}  whole mix rms={device:.4}");
        }
        super::set_volume(v0).unwrap();
    }
    use super::outside_level;

    #[test]
    fn finds_glassbars_sound_helper_through_its_parents() {
        // GlassBar 100 -> WebView2 browser 200 -> audio service 300; Chrome 400 -> 500.
        let parent: std::collections::HashMap<u32, u32> = [(200, 100), (300, 200), (500, 400), (100, 1)].into_iter().collect();
        assert!(super::descends(300, 100, &parent));
        assert!(!super::descends(500, 100, &parent));
        assert!(!super::descends(100, 100, &parent), "itself is not its own helper");
        let looped: std::collections::HashMap<u32, u32> = [(7, 8), (8, 7)].into_iter().collect();
        assert!(!super::descends(7, 100, &looped), "a reused PID loop ends");
    }

    #[test]
    fn only_changes_from_outside_glassbar_raise_the_volume_alert() {
        assert_eq!(outside_level(&[(0.4, false, true)], false), None, "our own slider");
        assert_eq!(outside_level(&[(0.4, false, false)], false), Some((0.4, false)), "a volume key");
        assert_eq!(outside_level(&[], false), None);
    }

    #[test]
    fn echoes_of_our_own_change_are_ours_too() {
        // Seen on this PC: each change from GlassBar is followed in the same millisecond by
        // 2-4 notifications from another component that snaps the level (0.667 -> 0.669).
        let burst = [(0.667, false, true), (0.667, false, false), (0.669, false, false)];
        assert_eq!(outside_level(&burst, false), None, "same burst as ours");
        assert_eq!(outside_level(&[(0.669, false, false)], true), None, "just after ours");
    }
}
