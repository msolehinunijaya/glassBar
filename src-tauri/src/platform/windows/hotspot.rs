//! Mobile hotspot via Windows' tethering API (the same one Settings → Mobile hotspot uses).
//! It shares the current internet connection; if there is none, or Windows reports the
//! adapter/policy can't share, the reason is returned instead of a working toggle.
use super::super::{Hotspot, Res};
use super::{com_init, err};
use windows::Networking::Connectivity::NetworkInformation;
use windows::Networking::NetworkOperators::{
    NetworkOperatorTetheringManager, TetheringCapability, TetheringOperationStatus, TetheringOperationalState,
};

fn manager() -> Res<NetworkOperatorTetheringManager> {
    com_init();
    let profile = NetworkInformation::GetInternetConnectionProfile()
        .map_err(|_| "No internet connection to share".to_string())?;
    let cap = NetworkOperatorTetheringManager::GetTetheringCapabilityFromConnectionProfile(&profile).map_err(err)?;
    if cap != TetheringCapability::Enabled {
        return Err(match cap {
            TetheringCapability::DisabledByGroupPolicy => "Turned off by your organisation's policy",
            TetheringCapability::DisabledByHardwareLimitation => "This PC's Wi-Fi adapter can't host a hotspot",
            TetheringCapability::DisabledByOperator => "Blocked by your mobile operator",
            TetheringCapability::DisabledBySku => "Not available in this edition of Windows",
            TetheringCapability::DisabledBySystemCapability => "Not available on this PC",
            _ => "Windows says the hotspot isn't available right now",
        }.into());
    }
    NetworkOperatorTetheringManager::CreateFromConnectionProfile(&profile).map_err(err)
}

pub fn state() -> Res<Hotspot> {
    let m = manager()?;
    let on = m.TetheringOperationalState().map_err(err)? == TetheringOperationalState::On;
    let ssid = m.GetCurrentAccessPointConfiguration().and_then(|c| c.Ssid()).map(|s| s.to_string()).unwrap_or_default();
    Ok(Hotspot { on, ssid, clients: m.ClientCount().unwrap_or(0), max_clients: m.MaxClientCount().unwrap_or(0) })
}

pub fn set(on: bool) -> Res<()> {
    let m = manager()?;
    let op = if on { m.StartTetheringAsync() } else { m.StopTetheringAsync() };
    let result = op.and_then(|o| o.join()).map_err(err)?;
    match result.Status().map_err(err)? {
        TetheringOperationStatus::Success => Ok(()),
        _ => {
            let msg = result.AdditionalErrorMessage().map(|s| s.to_string()).unwrap_or_default();
            Err(if msg.is_empty() { "Windows couldn't change the hotspot".into() } else { msg })
        }
    }
}
