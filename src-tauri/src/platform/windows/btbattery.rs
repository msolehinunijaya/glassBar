//! Bluetooth battery levels: the value Windows' Bluetooth stack publishes on a device's node
//! (the Hands-Free profile for headsets and phones, and LE devices that report it), read with
//! the Configuration Manager API. Each is matched to a paired device by its Bluetooth address.
use super::wide;
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::*;
use windows::Win32::Devices::Properties::DEVPROPTYPE;
use windows::Win32::Foundation::DEVPROPKEY;

/// {104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2: battery level, a byte 0..100.
const BATTERY: DEVPROPKEY = DEVPROPKEY { fmtid: windows::core::GUID::from_u128(0x104ea319_6ee2_4701_bd47_8ddbf425bbe5), pid: 2 };

/// Pure: the 12-hex Bluetooth address in a device instance id, lower case.
/// "BTHENUM\{0000111F-...}_VID&...\7&169FE51&0&C4776487D293_C00000000" -> "c4776487d293";
/// "BTHLE\DEV_C4776487D293\7&..." -> "c4776487d293".
pub fn mac_from_instance(id: &str) -> Option<String> {
    let up = id.to_uppercase();
    let tail = up.rsplit('\\').next()?;
    let candidates = tail.split(['&', '_']).chain(up.split('\\').flat_map(|p| p.split('_')));
    candidates.map(|s| s.trim()).find(|s| s.len() == 12 && s.chars().all(|c| c.is_ascii_hexdigit()) && *s != "000000000000")
        .map(|s| s.to_lowercase())
}

/// Pure: the address in a WinRT Bluetooth device id ("Bluetooth#Bluetooth9c:b6:..-c4:77:64:87:d2:93",
/// "BluetoothLE#BluetoothLE...-c4:77:..."): the device's address comes last.
pub fn mac_from_winrt_id(id: &str) -> Option<String> {
    let last = id.rsplit('-').next()?;
    let hex: String = last.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    (hex.len() == 12 && last.matches(':').count() == 5).then(|| hex.to_lowercase())
}

fn ids(enumerator: &str) -> Vec<String> {
    let f = wide(enumerator);
    let flags = CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT;
    unsafe {
        let mut len = 0u32;
        if CM_Get_Device_ID_List_SizeW(&mut len, PCWSTR(f.as_ptr()), flags) != CR_SUCCESS || len == 0 { return vec![]; }
        let mut buf = vec![0u16; len as usize];
        if CM_Get_Device_ID_ListW(PCWSTR(f.as_ptr()), &mut buf, flags) != CR_SUCCESS { return vec![]; }
        buf.split(|c| *c == 0).filter(|s| !s.is_empty()).map(String::from_utf16_lossy).collect()
    }
}

fn level(instance: &str) -> Option<u8> {
    let w = wide(instance);
    unsafe {
        let mut node = 0u32;
        if CM_Locate_DevNodeW(&mut node, PCWSTR(w.as_ptr()), CM_LOCATE_DEVNODE_NORMAL) != CR_SUCCESS { return None; }
        let mut ty = DEVPROPTYPE::default();
        let mut v = 0u8;
        let mut size = 1u32;
        (CM_Get_DevNode_PropertyW(node, &BATTERY, &mut ty, Some(&mut v), &mut size, 0) == CR_SUCCESS && size == 1 && v <= 100).then_some(v)
    }
}

/// (address, battery %) for every present Bluetooth node that reports one. A device with
/// several nodes (Hands-Free and Audio Gateway) is listed once.
pub fn levels() -> Vec<(String, u8)> {
    let mut out: Vec<(String, u8)> = vec![];
    for e in ["BTHENUM", "BTHLE", "BTHLEDEVICE"] {
        for id in ids(e) {
            let (Some(mac), Some(p)) = (mac_from_instance(&id), level(&id)) else { continue };
            if !out.iter().any(|(m, _)| *m == mac) { out.push((mac, p)); }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_addresses() {
        assert_eq!(mac_from_instance(r"BTHENUM\{0000111F-0000-1000-8000-00805F9B34FB}_HCIBYPASS_VID&00010075_PID&0100\7&169FE51&0&C4776487D293_C00000000").as_deref(), Some("c4776487d293"));
        assert_eq!(mac_from_instance(r"BTHLE\DEV_C4776487D293\7&2B3A&0&C4776487D293").as_deref(), Some("c4776487d293"));
        assert_eq!(mac_from_instance(r"BTHENUM\{0000111F-0000-1000-8000-00805F9B34FB}_LOCALMFG&0000\7&1&0&000000000000_00000000"), None);
        assert_eq!(mac_from_winrt_id("Bluetooth#Bluetooth9c:b6:d0:11:22:33-c4:77:64:87:d2:93").as_deref(), Some("c4776487d293"));
        assert_eq!(mac_from_winrt_id("BluetoothLE#BluetoothLE9c:b6:d0:11:22:33-c4:77:64:87:d2:93").as_deref(), Some("c4776487d293"));
    }

    /// Read-only. `cargo test btbattery::tests::live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live() { println!("{:?}", levels()); }
}

#[cfg(test)]
mod live_match {
    #[test]
    #[ignore]
    fn paired_with_battery() {
        for d in super::super::wireless::bt_list(false).unwrap() { println!("{} | {} | conn {} | {:?} | {}", d.name, d.kind, d.connected, d.battery, d.id); }
    }
}
