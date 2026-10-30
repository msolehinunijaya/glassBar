//! Wi-Fi networks (Windows.Devices.WiFi) and Bluetooth devices (Windows.Devices.Enumeration).
//! Everything goes through Windows' own APIs and dialogs; passwords are handed straight to
//! Windows and never kept by GlassBar.
use super::super::{BtDevice, Res, WifiNetwork};
use super::{com_init, err};
use std::collections::HashSet;
use windows::core::HSTRING;
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::{DeviceInformation, DevicePairingResultStatus};
use windows::Devices::WiFi::{WiFiAccessStatus, WiFiAdapter, WiFiAvailableNetwork, WiFiConnectionStatus, WiFiReconnectionKind};
use windows::Networking::Connectivity::{NetworkAuthenticationType, NetworkInformation};
use windows::Security::Credentials::PasswordCredential;

fn adapter() -> Res<WiFiAdapter> {
    com_init();
    let access = WiFiAdapter::RequestAccessAsync().and_then(|o| o.join()).map_err(err)?;
    if access != WiFiAccessStatus::Allowed {
        return Err("Windows blocked Wi-Fi access for desktop apps. Turn on Settings › Privacy & security › Location › \"Let desktop apps access your location\" (Windows requires it to list networks).".into());
    }
    let list = WiFiAdapter::FindAllAdaptersAsync().and_then(|o| o.join()).map_err(err)?;
    list.into_iter().next().ok_or_else(|| "No Wi-Fi adapter found".to_string())
}

fn connected_ssid(a: &WiFiAdapter) -> Option<String> {
    let p = a.NetworkAdapter().ok()?.GetConnectedProfileAsync().ok()?.join().ok()?;
    let ssid = p.WlanConnectionProfileDetails().ok()?.GetConnectedSsid().ok()?.to_string();
    (!ssid.is_empty()).then_some(ssid)
}

fn saved_profiles() -> HashSet<String> {
    NetworkInformation::GetConnectionProfiles().map(|v| v.into_iter().filter(|p| p.IsWlanConnectionProfile().unwrap_or(false))
        .filter_map(|p| p.ProfileName().ok().map(|n| n.to_string())).collect()).unwrap_or_default()
}

fn security(n: &WiFiAvailableNetwork) -> (bool, String) {
    let t = n.SecuritySettings().and_then(|s| s.NetworkAuthenticationType()).unwrap_or(NetworkAuthenticationType::Unknown);
    let label = match t {
        NetworkAuthenticationType::None | NetworkAuthenticationType::Open80211 => "Open",
        NetworkAuthenticationType::Wpa3 | NetworkAuthenticationType::Wpa3Sae | NetworkAuthenticationType::Wpa3Enterprise => "WPA3",
        NetworkAuthenticationType::RsnaPsk | NetworkAuthenticationType::Rsna => "WPA2",
        NetworkAuthenticationType::WpaPsk | NetworkAuthenticationType::Wpa => "WPA",
        NetworkAuthenticationType::SharedKey80211 => "WEP",
        NetworkAuthenticationType::Owe => "Enhanced Open",
        _ => "Secured",
    };
    (!matches!(label, "Open"), label.to_string())
}

/// Scan (when `scan`) and list networks, strongest first, one entry per SSID.
pub fn wifi_list(scan: bool) -> Res<(Option<String>, Vec<WifiNetwork>)> {
    let a = adapter()?;
    if scan { a.ScanAsync().and_then(|o| o.join()).map_err(err)?; }
    let current = connected_ssid(&a);
    let saved = saved_profiles();
    let mut seen = HashSet::new();
    let mut nets: Vec<(f64, WifiNetwork)> = a.NetworkReport().and_then(|r| r.AvailableNetworks()).map_err(err)?.into_iter()
        .filter_map(|n| {
            let ssid = n.Ssid().ok()?.to_string();
            if ssid.is_empty() { return None; } // hidden networks: use Windows settings
            let rssi = n.NetworkRssiInDecibelMilliwatts().unwrap_or(-100.0);
            let (secured, sec) = security(&n);
            Some((rssi, WifiNetwork {
                connected: current.as_deref() == Some(ssid.as_str()),
                saved: saved.contains(&ssid),
                bars: n.SignalBars().unwrap_or(0),
                secured, security: sec, ssid,
            }))
        })
        .collect();
    nets.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let nets = nets.into_iter().map(|(_, n)| n).filter(|n| seen.insert(n.ssid.clone())).collect();
    Ok((current, nets))
}

/// Connect to `ssid` (strongest access point). `password` only for a new secured network.
pub fn wifi_connect(ssid: &str, password: Option<&str>) -> Res<()> {
    let a = adapter()?;
    let mut best: Option<(f64, WiFiAvailableNetwork)> = None;
    for n in a.NetworkReport().and_then(|r| r.AvailableNetworks()).map_err(err)? {
        if n.Ssid().map(|s| s.to_string()).as_deref() != Ok(ssid) { continue; }
        let r = n.NetworkRssiInDecibelMilliwatts().unwrap_or(-100.0);
        if best.as_ref().is_none_or(|(b, _)| r > *b) { best = Some((r, n)); }
    }
    let (_, net) = best.ok_or("That network is out of range now. Refresh and try again.")?;
    let op = match password.filter(|p| !p.is_empty()) {
        Some(p) => {
            let c = PasswordCredential::new().map_err(err)?;
            c.SetPassword(&HSTRING::from(p)).map_err(err)?;
            a.ConnectWithPasswordCredentialAsync(&net, WiFiReconnectionKind::Automatic, &c)
        }
        None => a.ConnectAsync(&net, WiFiReconnectionKind::Automatic),
    };
    let status = op.and_then(|o| o.join()).and_then(|r| r.ConnectionStatus()).map_err(err)?;
    match status {
        WiFiConnectionStatus::Success => Ok(()),
        WiFiConnectionStatus::InvalidCredential => Err("Wrong password".into()),
        WiFiConnectionStatus::NetworkNotAvailable => Err("Network not available".into()),
        WiFiConnectionStatus::Timeout => Err("Timed out connecting".into()),
        WiFiConnectionStatus::UnsupportedAuthenticationProtocol => Err("This network's sign-in type isn't supported here: use Windows settings".into()),
        WiFiConnectionStatus::AccessRevoked => Err("Windows revoked Wi-Fi access".into()),
        _ => Err("Couldn't connect".into()),
    }
}

pub fn wifi_disconnect() -> Res<()> { adapter()?.Disconnect().map_err(err) }

// ---- Bluetooth ----

fn find(selector: &HSTRING) -> Res<Vec<DeviceInformation>> {
    let list = DeviceInformation::FindAllAsyncAqsFilter(selector).and_then(|o| o.join()).map_err(err)?;
    Ok(list.into_iter().collect())
}

/// (type, connected) for a classic or LE device.
fn details(id: &HSTRING, le: bool) -> (String, bool) {
    if le {
        let c = BluetoothLEDevice::FromIdAsync(id).and_then(|o| o.join())
            .and_then(|d| d.ConnectionStatus()).map(|s| s == BluetoothConnectionStatus::Connected).unwrap_or(false);
        return ("Bluetooth LE".into(), c);
    }
    let Ok(d) = BluetoothDevice::FromIdAsync(id).and_then(|o| o.join()) else { return ("Device".into(), false) };
    let major = d.ClassOfDevice().and_then(|c| c.MajorClass()).map(|m| m.0).unwrap_or(0);
    let kind = match major { 1 => "Computer", 2 => "Phone", 4 => "Audio", 5 => "Input", 6 => "Imaging", 7 => "Wearable", 8 => "Toy", _ => "Device" };
    (kind.into(), d.ConnectionStatus().map(|s| s == BluetoothConnectionStatus::Connected).unwrap_or(false))
}

/// Paired devices, plus nearby discoverable ones (a short Windows-run inquiry).
pub fn bt_list(discover: bool) -> Res<Vec<BtDevice>> {
    com_init();
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut add = |sel: HSTRING, paired: bool, le: bool| -> Res<()> {
        for i in find(&sel)? {
            let name = i.Name().map(|n| n.to_string()).unwrap_or_default();
            if name.is_empty() { continue; }
            let id = i.Id().map_err(err)?;
            if !seen.insert(id.to_string()) { continue; }
            let (kind, connected) = details(&id, le);
            out.push(BtDevice { kind, id: id.to_string(), name, paired, connected: paired && connected, battery: None });
        }
        Ok(())
    };
    add(BluetoothDevice::GetDeviceSelectorFromPairingState(true).map_err(err)?, true, false)?;
    add(BluetoothLEDevice::GetDeviceSelectorFromPairingState(true).map_err(err)?, true, true)?;
    if discover {
        add(BluetoothDevice::GetDeviceSelectorFromPairingState(false).map_err(err)?, false, false)?;
        add(BluetoothLEDevice::GetDeviceSelectorFromPairingState(false).map_err(err)?, false, true)?;
    }
    // Battery: only for connected devices (a disconnected one's last value is stale).
    if out.iter().any(|d| d.connected) {
        let levels = super::btbattery::levels();
        for d in out.iter_mut().filter(|d| d.connected) {
            let mac = super::btbattery::mac_from_winrt_id(&d.id);
            d.battery = levels.iter().find(|(m, _)| Some(m) == mac.as_ref()).map(|(_, p)| *p);
        }
    }
    Ok(out)
}

/// Pair using Windows' own pairing flow (it shows the PIN/confirm dialog when needed).
pub fn bt_pair(id: &str) -> Res<()> {
    com_init();
    let info = DeviceInformation::CreateFromIdAsync(&HSTRING::from(id)).and_then(|o| o.join()).map_err(err)?;
    let st = info.Pairing().and_then(|p| p.PairAsync()).and_then(|o| o.join()).and_then(|r| r.Status()).map_err(err)?;
    match st {
        DevicePairingResultStatus::Paired | DevicePairingResultStatus::AlreadyPaired => Ok(()),
        DevicePairingResultStatus::NotReadyToPair => Err("The device isn't in pairing mode".into()),
        DevicePairingResultStatus::AuthenticationTimeout | DevicePairingResultStatus::ConnectionRejected => Err("The device didn't accept pairing".into()),
        DevicePairingResultStatus::PairingCanceled => Err("Pairing was cancelled".into()),
        DevicePairingResultStatus::RequiredHandlerNotRegistered => Err("This device needs a PIN: pair it in Windows Bluetooth settings".into()),
        s => Err(format!("Pairing failed ({})", s.0)),
    }
}

pub fn bt_unpair(id: &str) -> Res<()> {
    com_init();
    let info = DeviceInformation::CreateFromIdAsync(&HSTRING::from(id)).and_then(|o| o.join()).map_err(err)?;
    info.Pairing().and_then(|p| p.UnpairAsync()).and_then(|o| o.join()).map_err(err)?;
    Ok(())
}

/// Windows raises NetworkStatusChanged on connect/disconnect; registered once for the app's life.
pub fn watch_network(f: Box<dyn Fn() + Send + Sync>) {
    com_init();
    let h = windows::Networking::Connectivity::NetworkStatusChangedEventHandler::new(move |_| { f(); Ok(()) });
    let _ = NetworkInformation::NetworkStatusChanged(&h);
}
