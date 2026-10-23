//! Windows location service (works when "Let desktop apps access your location" is on),
//! and Windows' own map service to name a point.
use super::super::{Fix, Res};
use super::{com_init, err};
use windows::Devices::Geolocation::{BasicGeoposition, Geopoint, GeolocationAccessStatus, Geolocator, PositionSource};
use windows::Foundation::TimeSpan;
use windows::Services::Maps::{MapLocationFinder, MapLocationFinderStatus};

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

pub fn place_name(lat: f64, lon: f64) -> Res<String> {
    com_init();
    let point = Geopoint::Create(BasicGeoposition { Latitude: lat, Longitude: lon, Altitude: 0.0 }).map_err(err)?;
    let r = MapLocationFinder::FindLocationsAtAsync(&point).and_then(|op| op.join()).map_err(err)?;
    let st = r.Status().map_err(err)?;
    if st != MapLocationFinderStatus::Success { return Err(format!("Windows map lookup failed (status {})", st.0)); }
    let a = r.Locations().and_then(|l| l.GetAt(0)).and_then(|l| l.Address()).map_err(err)?;
    let parts = [a.Town(), a.Region(), a.Country()].into_iter().filter_map(|s| s.ok()).map(|s| s.to_string());
    let mut out: Vec<String> = Vec::new();
    for s in parts { if !s.is_empty() && !out.contains(&s) { out.push(s); } }
    if out.is_empty() { Err("no place name".into()) } else { Ok(out.join(", ")) }
}
