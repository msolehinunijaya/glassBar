//! Quick Settings toggles Windows has no public API for: Night light, Nearby sharing and Cast.
//! Each one reads the real state back, and anything unexpected is an error -- the panel then
//! opens the Windows page instead. Nothing here ever shows a state it didn't read.
use super::super::{CastDisplay, CastState, Res};
use super::err;
use windows::core::HSTRING;
use windows::Win32::System::Registry::*;

// ---- HKCU registry ----

fn read(key: &str, name: &str, flags: REG_ROUTINE_FLAGS) -> Option<Vec<u8>> {
    let (k, n) = (HSTRING::from(key), HSTRING::from(name));
    let mut size = 0u32;
    unsafe { if RegGetValueW(HKEY_CURRENT_USER, &k, &n, flags, None, None, Some(&mut size)).is_err() { return None; } }
    let mut buf = vec![0u8; size as usize];
    unsafe { if RegGetValueW(HKEY_CURRENT_USER, &k, &n, flags, None, Some(buf.as_mut_ptr() as _), Some(&mut size)).is_err() { return None; } }
    buf.truncate(size as usize);
    Some(buf)
}

fn write(key: &str, name: &str, ty: REG_VALUE_TYPE, data: &[u8]) -> Res<()> {
    let r = unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, &HSTRING::from(key), &HSTRING::from(name), ty.0, Some(data.as_ptr() as _), data.len() as u32) };
    if r.is_err() { Err(format!("Couldn't save the setting ({})", r.0)) } else { Ok(()) }
}

fn read_dword(key: &str, name: &str) -> Option<u32> {
    read(key, name, RRF_RT_REG_DWORD).and_then(|b| Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?)))
}
fn write_dword(key: &str, name: &str, v: u32) -> Res<()> { write(key, name, REG_DWORD, &v.to_le_bytes()) }

// ---- Night light ----
//
// Its state is a Bond CompactBinary v1 record that explorer.exe watches and applies live:
//   envelope  43 42 01 00 | 0A 02 01 00 | 2A 06 <varint unix-seconds> 2A 2B 0E <len> <inner> 00 00 00
//   inner     43 42 01 00 [10 00] D0 0A 02 C6 14 <varint FILETIME> [...] 00
// Field 0 (int32) is there only while Night light is on; 10 = what changed it (1 = manual);
// 20 = when; 30 (bool) only when false = not supported on this PC. Unknown fields are kept,
// in id order. A write stamped older than the cloud copy's is reverted by Windows, so the
// stamp always moves forward. Format: github.com/kvnxiao/win-nightlight-cli (docs/).

const NL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\DefaultAccount\Current\default$windows.data.bluelightreduction.bluelightreductionstate\windows.data.bluelightreduction.bluelightreductionstate";
const HEAD: [u8; 4] = [0x43, 0x42, 0x01, 0x00];
const ENV_A: [u8; 10] = [0x43, 0x42, 0x01, 0x00, 0x0A, 0x02, 0x01, 0x00, 0x2A, 0x06];
const ENV_B: [u8; 3] = [0x2A, 0x2B, 0x0E];

fn varint(b: &[u8], i: &mut usize) -> Option<u64> {
    let (mut v, mut shift) = (0u64, 0u32);
    loop {
        let x = *b.get(*i)?;
        *i += 1;
        v |= ((x & 0x7f) as u64) << shift;
        if x & 0x80 == 0 { return Some(v); }
        shift += 7;
        if shift > 63 { return None; }
    }
}
fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let x = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 { out.push(x); return; }
        out.push(x | 0x80);
    }
}

/// One field of the inner record: id, Bond type, and its value exactly as encoded.
#[derive(Debug, PartialEq)]
struct Field { id: u16, ty: u8, val: Vec<u8> }

/// The inner record's fields; None for anything this doesn't fully understand.
fn fields(b: &[u8]) -> Option<Vec<Field>> {
    if b.get(..4)? != HEAD { return None; }
    let (mut i, mut out) = (4usize, vec![]);
    loop {
        let h = *b.get(i)?;
        i += 1;
        if h == 0 { break; } // end of the struct
        let ty = h & 0x1f;
        let id = match h >> 5 {
            6 => { let x = *b.get(i)? as u16; i += 1; x }
            7 => { let x = u16::from_le_bytes([*b.get(i)?, *b.get(i + 1)?]); i += 2; x }
            n => n as u16,
        };
        let start = i;
        match ty {
            2 | 3 | 14 => i += 1,                         // bool, uint8, int8
            4 | 5 | 6 | 15 | 16 | 17 => { varint(b, &mut i)?; } // (u)int16/32/64: varints
            _ => return None,
        }
        out.push(Field { id, ty, val: b.get(start..i)?.to_vec() });
    }
    (i == b.len()).then_some(out)
}

fn encode(fs: &[Field]) -> Vec<u8> {
    let mut o = HEAD.to_vec();
    for f in fs {
        if f.id <= 5 { o.push(((f.id as u8) << 5) | f.ty); }
        else if f.id <= 0xff { o.push(0xC0 | f.ty); o.push(f.id as u8); }
        else { o.push(0xE0 | f.ty); o.extend(f.id.to_le_bytes()); }
        o.extend(&f.val);
    }
    o.push(0);
    o
}

/// (envelope timestamp, inner record); None for anything unexpected.
fn open(b: &[u8]) -> Option<(u64, Vec<u8>)> {
    if b.get(..10)? != ENV_A { return None; }
    let mut i = 10;
    let ts = varint(b, &mut i)?;
    if b.get(i..i + 3)? != ENV_B { return None; }
    i += 3;
    let len = varint(b, &mut i)? as usize;
    let inner = b.get(i..i + len)?.to_vec();
    (b.get(i + len..)? == [0, 0, 0]).then_some((ts, inner))
}

fn seal(ts: u64, inner: &[u8]) -> Vec<u8> {
    let mut o = ENV_A.to_vec();
    put_varint(&mut o, ts);
    o.extend(ENV_B);
    put_varint(&mut o, inner.len() as u64);
    o.extend(inner);
    o.extend([0, 0, 0]);
    o
}

/// Pure: on (Some(true)), off, or not supported here (None) from a stored record.
fn nl_state(data: &[u8]) -> Option<Result<bool, &'static str>> {
    let fs = fields(&open(data)?.1)?;
    if fs.iter().any(|f| f.id == 30 && f.val == [0]) { return Some(Err("Night light isn't available on this PC")); }
    Some(Ok(fs.iter().any(|f| f.id == 0)))
}

/// Pure: the record turned on/off at `unix`/`filetime`, stamped after `floor` (the newest
/// stamp Windows or we have used). None if the record isn't one this understands.
fn nl_toggle(data: &[u8], on: bool, unix: u64, filetime: u64, floor: u64) -> Option<Vec<u8>> {
    let (prev, inner) = open(data)?;
    let mut fs = fields(&inner)?;
    fs.retain(|f| !matches!(f.id, 0 | 10 | 20));
    if on { fs.push(Field { id: 0, ty: 16, val: vec![0] }); }
    fs.push(Field { id: 10, ty: 16, val: vec![2] }); // zigzag(1): changed by hand
    let mut ft = vec![];
    put_varint(&mut ft, filetime);
    fs.push(Field { id: 20, ty: 6, val: ft });
    fs.sort_by_key(|f| f.id);
    Some(seal(unix.max(prev + 2).max(floor + 2), &encode(&fs)))
}

static NL_LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn night_light() -> Res<bool> {
    let data = read(NL_KEY, "Data", RRF_RT_REG_BINARY).ok_or("Night light has never been set up here")?;
    nl_state(&data).ok_or("Unknown Night light format")?.map_err(String::from)
}

pub fn set_night_light(on: bool) -> Res<()> {
    let data = read(NL_KEY, "Data", RRF_RT_REG_BINARY).ok_or("Night light has never been set up here")?;
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(err)?;
    let filetime = since.as_nanos() as u64 / 100 + 116_444_736_000_000_000;
    // Windows reverts a write stamped older than the cloud copy of the same record.
    let cloud = read(&NL_KEY.replace(r"\Current\", r"\Cloud\"), "Data", RRF_RT_REG_BINARY).and_then(|d| open(&d)).map(|(ts, _)| ts).unwrap_or(0);
    let floor = cloud.max(NL_LAST.load(std::sync::atomic::Ordering::Relaxed));
    let next = nl_toggle(&data, on, since.as_secs(), filetime, floor).ok_or("Unknown Night light format")?;
    write(NL_KEY, "Data", REG_BINARY, &next)?;
    if let Some((ts, _)) = open(&next) { NL_LAST.store(ts, std::sync::atomic::Ordering::Relaxed); }
    Ok(())
}

// ---- Nearby sharing ----
// 0 off, 1 my devices only, 2 everyone nearby; the Settings page reads the same values.
// `RomeSdkChannelUserAuthzPolicy` is "Share across devices", a different setting: untouched.

const CDP: &str = r"Software\Microsoft\Windows\CurrentVersion\CDP";
const CDP_PAGE: &str = r"Software\Microsoft\Windows\CurrentVersion\CDP\SettingsPage";

pub fn nearby() -> Res<u32> { Ok(read_dword(CDP, "NearShareChannelUserAuthzPolicy").unwrap_or(0).min(2)) }

pub fn set_nearby(mode: u32) -> Res<()> {
    let mode = mode.min(2);
    write_dword(CDP, "NearShareChannelUserAuthzPolicy", mode)?;
    write_dword(CDP_PAGE, "NearShareChannelUserAuthzPolicy", mode)?;
    if mode > 0 {
        // Sharing needs the device-session channel open at least as wide.
        if read_dword(CDP, "CdpSessionUserAuthzPolicy").unwrap_or(0) < mode { write_dword(CDP, "CdpSessionUserAuthzPolicy", mode)?; }
    } else {
        // What the Settings page also clears when it turns sharing off.
        write_dword(CDP_PAGE, "BluetoothLastDisabledNearShare", 0)?;
        write_dword(CDP_PAGE, "WifiLastDisabledNearShare", 0)?;
    }
    Ok(())
}

// ---- Cast (Miracast) ----
// Finding displays and telling whether a cast is on work from a desktop app; starting or
// ending one does not (ProjectionManager needs a UWP view), so the panel opens Windows' own
// Cast panel for that.

const MIRACAST_AQS: &str = r#"System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:="{0407d24e-53de-4c9a-9ba1-9ced54641188}" AND System.Devices.WiFiDirect.Services:~="Miracast""#;

fn casting() -> bool {
    use windows::Win32::Devices::Display::*;
    unsafe {
        let (mut np, mut nm) = (0u32, 0u32);
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut np, &mut nm).is_err() { return false; }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); np as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); nm as usize];
        if QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, &mut np, paths.as_mut_ptr(), &mut nm, modes.as_mut_ptr(), None).is_err() { return false; }
        paths.iter().take(np as usize).any(|p| p.targetInfo.outputTechnology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST)
    }
}

pub fn cast() -> Res<CastState> {
    use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
    use windows_collections::IIterable;
    let props: IIterable<HSTRING> = Vec::<HSTRING>::new().into();
    let list = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(&HSTRING::from(MIRACAST_AQS), &props, DeviceInformationKind::AssociationEndpoint)
        .and_then(|o| o.join()).map_err(err)?;
    let mut displays = vec![];
    for d in list {
        let name = d.Name().map(|n| n.to_string()).unwrap_or_default();
        if !name.is_empty() { displays.push(CastDisplay { id: d.Id().map(|i| i.to_string()).unwrap_or_default(), name }); }
    }
    Ok(CastState { casting: casting(), displays })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record as Windows writes it with Night light off (this PC's stamp, a 2026 FILETIME).
    fn off() -> Vec<u8> {
        let mut inner = HEAD.to_vec();
        inner.extend([0xD0, 0x0A, 0x02, 0xC6, 0x14]);
        put_varint(&mut inner, 134_000_000_000_000_000);
        inner.push(0);
        let mut i = 0;
        let stamp = varint(&[0xAD, 0xC8, 0xC3, 0xCB, 0x06], &mut i).unwrap();
        seal(stamp, &inner)
    }

    #[test]
    fn reads_the_record_windows_writes() {
        let d = off();
        assert_eq!(d[18], 0x13, "19-byte inner record when off, as on a real PC");
        assert_eq!(nl_state(&d), Some(Ok(false)));
    }

    #[test]
    fn turning_on_inserts_field_zero_and_moves_the_stamp_forward() {
        let d = off();
        let (prev, _) = open(&d).unwrap();
        let on = nl_toggle(&d, true, 10, 134_000_000_100_000_000, 0).unwrap();
        assert_eq!(nl_state(&on), Some(Ok(true)));
        assert_eq!(on[18], 0x15, "21-byte inner record when on");
        assert_eq!(&on[19..25], &[0x43, 0x42, 0x01, 0x00, 0x10, 0x00], "field 0 right after the header");
        assert!(open(&on).unwrap().0 >= prev + 2);
        let back = nl_toggle(&on, false, 10, 134_000_000_200_000_000, 0).unwrap();
        assert_eq!(nl_state(&back), Some(Ok(false)));
    }

    #[test]
    fn the_stamp_never_goes_below_the_cloud_copy() {
        let cloud = 9_000_000_000;
        let on = nl_toggle(&off(), true, 10, 1, cloud).unwrap();
        assert!(open(&on).unwrap().0 > cloud);
    }

    #[test]
    fn unknown_fields_survive_in_order() {
        let (ts, mut inner) = open(&off()).unwrap();
        inner.pop(); // the end marker
        inner.extend([0xC2, 0x28, 0x01, 0x00]); // field 40, bool true
        let d = seal(ts, &inner);
        let on = nl_toggle(&d, true, 10, 1, 0).unwrap();
        let fs = fields(&open(&on).unwrap().1).unwrap();
        assert_eq!(fs.iter().map(|f| f.id).collect::<Vec<_>>(), vec![0, 10, 20, 40]);
    }

    #[test]
    fn anything_unexpected_is_refused_not_guessed() {
        let mut d = off();
        d[5] = 0x99;
        assert_eq!(nl_state(&d), None);
        assert!(nl_toggle(&d, true, 10, 1, 0).is_none());
        assert_eq!(nl_state(&[]), None);
        let mut cut = off();
        cut.truncate(cut.len() - 1);
        assert_eq!(nl_state(&cut), None);
    }

    #[test]
    fn says_when_this_pc_has_no_night_light() {
        let (ts, mut inner) = open(&off()).unwrap();
        inner.pop();
        inner.extend([0xC2, 0x1E, 0x00, 0x00]); // field 30 = false
        assert_eq!(nl_state(&seal(ts, &inner)), Some(Err("Night light isn't available on this PC")));
    }
}

/// Reads (never writes) the real state on this PC: `cargo test -- --ignored live_read --nocapture`.
#[cfg(test)]
#[test]
#[ignore]
fn live_read() {
    super::com_init();
    println!("night light: {:?}", night_light());
    println!("nearby: {:?}", nearby());
    println!("cast: {:?}", cast());
    // The real record, toggled in memory only.
    if let Some(d) = read(NL_KEY, "Data", RRF_RT_REG_BINARY) {
        let on = nl_toggle(&d, true, 1, 1, 0).expect("real record understood");
        let off = nl_toggle(&on, false, 1, 1, 0).unwrap();
        println!("toggle round trip: on={:?} off={:?} len {} -> {} -> {}", nl_state(&on), nl_state(&off), d.len(), on.len(), off.len());
    }
}
