//! Battery status via GetSystemPowerStatus.
use super::super::Battery;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

pub fn battery() -> Option<Battery> {
    let mut s = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut s).ok()? };
    // BatteryFlag 128 = no system battery; 255 = unknown.
    if s.BatteryFlag & 128 != 0 || s.BatteryFlag == 255 || s.BatteryLifePercent == 255 { return None; }
    Some(Battery {
        percent: s.BatteryLifePercent.min(100),
        charging: s.BatteryFlag & 8 != 0,
        on_ac: s.ACLineStatus == 1,
        secs_left: (s.BatteryLifeTime != u32::MAX).then_some(s.BatteryLifeTime),
    })
}
