//! Core Audio: master volume, output devices, per-app sessions.
use super::super::{AudioDevice, AudioSession, AudioState, Res};
use super::{com_init, err, wide, windows_list::exe_path};
use std::ffi::c_void;
use std::time::Duration;
use windows::core::{Interface, GUID, HRESULT, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
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

pub fn set_volume(v: f32) -> Res<()> {
    unsafe { endpoint().and_then(|e| e.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0), std::ptr::null())).map_err(err) }
}

pub fn set_mute(m: bool) -> Res<()> {
    unsafe { endpoint().and_then(|e| e.SetMute(m, std::ptr::null())).map_err(err) }
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
pub fn watch(on_change: Box<dyn Fn(AudioState) + Send>) {
    std::thread::spawn(move || {
        com_init();
        let mut last: Option<AudioState> = None;
        loop {
            if let Ok(s) = state() {
                if last.as_ref() != Some(&s) {
                    on_change(s.clone());
                    last = Some(s);
                }
            }
            std::thread::sleep(Duration::from_secs(1));
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
pub fn loopback(on_chunk: &mut dyn FnMut(&[f32], u32) -> bool) -> Res<()> {
    com_init();
    unsafe {
        let dev = default_device().map_err(err)?;
        let dev_id = id_of(&dev);
        let client: IAudioClient = dev.Activate(CLSCTX_ALL, None).map_err(err)?;
        let fmt = client.GetMixFormat().map_err(err)?;
        let (ch, rate, bits) = ((*fmt).nChannels as usize, (*fmt).nSamplesPerSec, (*fmt).wBitsPerSample);
        // 200 ms buffer; shared-mode mix format is float32 on every modern Windows, 16-bit kept as a fallback.
        let init = client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 2_000_000, 0, fmt, None);
        CoTaskMemFree(Some(fmt as *const c_void));
        init.map_err(err)?;
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
        };
        let _ = client.Stop();
        res
    }
}
