//! Auto performance mode: drop to Low while the PC is short on memory or CPU (or battery saver
//! is on), and come back to Normal only after it has been calm for a while. Never picks High.
use crate::config::PerfMode;

pub const LOW_MEM: f32 = 85.0;
pub const LOW_CPU: f32 = 80.0;
pub const HOT_MS: u32 = 15_000;
pub const CALM_MEM: f32 = 75.0;
pub const CALM_CPU: f32 = 60.0;
pub const CALM_MS: u32 = 60_000;

/// System load (not just GlassBar's): memory used %, CPU %, Windows battery saver.
#[derive(Clone, Copy, Debug)]
pub struct Load { pub mem_pct: f32, pub cpu_pct: f32, pub saver: bool }

#[derive(Default)]
pub struct Auto { low: bool, hot_ms: u32, calm_ms: u32, reason: String }

impl Auto {
    /// Feed one sample, `dt_ms` after the last. Returns the mode and why, in words.
    pub fn step(&mut self, l: &Load, dt_ms: u32) -> (PerfMode, String) {
        if self.low {
            let calm = l.mem_pct < CALM_MEM && l.cpu_pct < CALM_CPU && !l.saver;
            self.calm_ms = if calm { self.calm_ms + dt_ms } else { 0 };
            if self.calm_ms >= CALM_MS { *self = Auto::default(); }
        } else {
            let cause = if l.mem_pct >= LOW_MEM { Some(format!("system memory {:.0}% used", l.mem_pct)) }
                else if l.cpu_pct >= LOW_CPU { Some(format!("CPU {:.0}% busy", l.cpu_pct)) }
                else if l.saver { Some("battery saver on".to_string()) }
                else { None };
            match cause {
                Some(c) => {
                    self.hot_ms += dt_ms;
                    if self.hot_ms >= HOT_MS { *self = Auto { low: true, reason: c, ..Auto::default() }; }
                }
                None => self.hot_ms = 0,
            }
        }
        if self.low { (PerfMode::Low, self.reason.clone()) } else { (PerfMode::Normal, "your PC has room".into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PerfMode;

    fn run(a: &mut Auto, l: Load, secs: u32) -> (PerfMode, String) { let mut r = a.step(&l, 0); for _ in 0..secs { r = a.step(&l, 1000); } r }
    const CALM: Load = Load { mem_pct: 50.0, cpu_pct: 10.0, saver: false };

    #[test] fn memory_at_85_for_15s_goes_low() {
        let mut a = Auto::default();
        assert_eq!(run(&mut a, Load { mem_pct: 85.0, ..CALM }, 14).0, PerfMode::Normal);
        assert_eq!(a.step(&Load { mem_pct: 85.0, ..CALM }, 1000), (PerfMode::Low, "system memory 85% used".into()));
    }
    #[test] fn cpu_just_under_80_never_goes_low() {
        assert_eq!(run(&mut Auto::default(), Load { cpu_pct: 79.9, ..CALM }, 120).0, PerfMode::Normal);
    }
    #[test] fn a_dip_restarts_the_15s() {
        let mut a = Auto::default();
        run(&mut a, Load { cpu_pct: 90.0, ..CALM }, 10); run(&mut a, CALM, 1);
        assert_eq!(run(&mut a, Load { cpu_pct: 90.0, ..CALM }, 14).0, PerfMode::Normal);
    }
    #[test] fn battery_saver_goes_low_with_its_reason() {
        assert_eq!(run(&mut Auto::default(), Load { saver: true, ..CALM }, 15), (PerfMode::Low, "battery saver on".into()));
    }
    #[test] fn back_to_normal_only_after_60s_calm() {
        let mut a = Auto::default();
        run(&mut a, Load { mem_pct: 90.0, ..CALM }, 15);
        assert_eq!(run(&mut a, Load { mem_pct: 74.9, cpu_pct: 59.9, saver: false }, 59).0, PerfMode::Low);
        assert_eq!(a.step(&Load { mem_pct: 74.9, cpu_pct: 59.9, saver: false }, 1000), (PerfMode::Normal, "your PC has room".into()));
    }
    #[test] fn memory_at_75_is_not_calm() {
        let mut a = Auto::default();
        run(&mut a, Load { mem_pct: 90.0, ..CALM }, 15);
        assert_eq!(run(&mut a, Load { mem_pct: 75.0, ..CALM }, 120).0, PerfMode::Low);
    }
    #[test] fn low_keeps_the_reason_it_switched_for() {
        let mut a = Auto::default();
        run(&mut a, Load { mem_pct: 88.0, ..CALM }, 15);
        assert_eq!(run(&mut a, Load { mem_pct: 80.0, ..CALM }, 5).1, "system memory 88% used");
    }
}
