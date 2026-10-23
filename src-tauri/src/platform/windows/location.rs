//! Windows location service (works when "Let desktop apps access your location" is on).
use super::super::{Fix, Res};
use super::{com_init, err};
use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionSource};
use windows::Foundation::TimeSpan;

pub fn locate() -> Res<Fix> {
    com_init();
    let access = Geolocator::RequestAccessAsync().and_then(|op| op.join()).map_err(friendly)?;
    if access != GeolocationAccessStatus::Allowed {
        return Err("Windows Location is turned off for desktop apps".into());
    }
    let g = Geolocator::new().map_err(err)?;
    // Accept a fix up to 10 minutes old; give up after 10 seconds.
    let max_age = TimeSpan { Duration: 10 * 60 * 10_000_000 };
    let timeout = TimeSpan { Duration: 10 * 10_000_000 };
    let pos = g.GetGeopositionAsyncWithAgeAndTimeout(max_age, timeout).and_then(|op| op.join()).map_err(friendly)?;
    let c = pos.Coordinate().map_err(err)?;
    let p = c.Point().and_then(|p| p.Position()).map_err(err)?;
    let via = match c.PositionSource() {
        Ok(PositionSource::Satellite) => "GPS",
        Ok(PositionSource::WiFi) => "Wi-Fi",
        Ok(PositionSource::Cellular) => "cell network",
        Ok(PositionSource::IPAddress) => "IP address",
        _ => "Windows",
    };
    Ok(Fix { lat: p.Latitude, lon: p.Longitude, accuracy_m: c.Accuracy().ok(), via })
}

/// Plain words for the usual failures; the raw message otherwise.
fn friendly(e: windows::core::Error) -> String {
    match e.code().0 as u32 {
        0x8007_0422 => "Windows Geolocation Service (lfsvc) is disabled".into(),
        0x8007_0005 => "Windows Location is turned off for desktop apps".into(),
        0x8000_4004 | 0x8007_05B4 => "Windows Location didn't find a position in time".into(),
        _ => format!("Windows Location failed ({})", e.message()),
    }
}
