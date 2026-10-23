//! Windows location service (works when "Let desktop apps access your location" is on).
use super::super::{Fix, Res};
use super::{com_init, err};
use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionSource};
use windows::Foundation::TimeSpan;

pub fn locate() -> Res<Fix> {
    com_init();
    let access = Geolocator::RequestAccessAsync().and_then(|op| op.join()).map_err(err)?;
    if access != GeolocationAccessStatus::Allowed {
        return Err("Windows location is off for desktop apps (Settings › Privacy & security › Location)".into());
    }
    let g = Geolocator::new().map_err(err)?;
    // Accept a fix up to 10 minutes old; give up after 10 seconds.
    let max_age = TimeSpan { Duration: 10 * 60 * 10_000_000 };
    let timeout = TimeSpan { Duration: 10 * 10_000_000 };
    let pos = g.GetGeopositionAsyncWithAgeAndTimeout(max_age, timeout).and_then(|op| op.join()).map_err(err)?;
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
