//! Windows location service (works when "Let desktop apps access your location" is on).
use super::super::Res;
use super::{com_init, err};
use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator};
use windows::Foundation::TimeSpan;

pub fn locate() -> Res<(f64, f64)> {
    com_init();
    let access = Geolocator::RequestAccessAsync().and_then(|op| op.join()).map_err(err)?;
    if access != GeolocationAccessStatus::Allowed { return Err("location access is off".into()); }
    let g = Geolocator::new().map_err(err)?;
    // Accept a fix up to 10 minutes old; give up after 10 seconds.
    let max_age = TimeSpan { Duration: 10 * 60 * 10_000_000 };
    let timeout = TimeSpan { Duration: 10 * 10_000_000 };
    let pos = g.GetGeopositionAsyncWithAgeAndTimeout(max_age, timeout).and_then(|op| op.join()).map_err(err)?;
    let p = pos.Coordinate().and_then(|c| c.Point()).and_then(|p| p.Position()).map_err(err)?;
    Ok((p.Latitude, p.Longitude))
}
