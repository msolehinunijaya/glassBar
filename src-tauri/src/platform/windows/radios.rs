//! Wi-Fi / Bluetooth radios via Windows.Devices.Radios.
use super::super::{RadioInfo, RadioKind, Res};
use super::{com_init, err};
use windows::Devices::Radios::{Radio, RadioAccessStatus, RadioKind as RK, RadioState};

fn kind(r: &Radio) -> RadioKind {
    match r.Kind() {
        Ok(RK::WiFi) => RadioKind::Wifi,
        Ok(RK::Bluetooth) => RadioKind::Bluetooth,
        _ => RadioKind::Other,
    }
}

fn radios() -> Res<Vec<Radio>> {
    com_init();
    let access = Radio::RequestAccessAsync().and_then(|op| op.join()).map_err(err)?;
    if access != RadioAccessStatus::Allowed {
        return Err("Radio control is blocked in Windows privacy settings".into());
    }
    let list = Radio::GetRadiosAsync().and_then(|op| op.join()).map_err(err)?;
    Ok(list.into_iter().collect())
}

pub fn list() -> Res<Vec<RadioInfo>> {
    Ok(radios()?
        .iter()
        .map(|r| RadioInfo {
            kind: kind(r),
            name: r.Name().map(|n| n.to_string()).unwrap_or_default(),
            on: r.State().map(|s| s == RadioState::On).unwrap_or(false),
        })
        .collect())
}

/// `kind: None` = every Wi-Fi and Bluetooth radio (airplane mode).
pub fn set(want: Option<RadioKind>, on: bool) -> Res<()> {
    let targets: Vec<Radio> = radios()?
        .into_iter()
        .filter(|r| match want { Some(k) => kind(r) == k, None => kind(r) != RadioKind::Other })
        .collect();
    if targets.is_empty() { return Err("No matching radio on this PC".into()); }
    let state = if on { RadioState::On } else { RadioState::Off };
    for r in targets {
        let status = r.SetStateAsync(state).and_then(|op| op.join()).map_err(err)?;
        if status != RadioAccessStatus::Allowed { return Err("Windows refused the radio change".into()); }
    }
    Ok(())
}
