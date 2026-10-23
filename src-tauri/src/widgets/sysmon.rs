//! System Monitor (CPU, RAM, network, optional GPU) and Battery.
//! Sampling runs only while the widget is placed in a profile.
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const HISTORY: usize = 60;

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sample { pub cpu: f32, pub mem: f32, pub gpu: Option<f32>, pub rx_bps: u64, pub tx_bps: u64, pub at: i64,
    /// °C from the thermal-zone sensor; None = not available on this PC (shown as N/A).
    pub cpu_temp: Option<f32> }

static HIST: Mutex<VecDeque<Sample>> = Mutex::new(VecDeque::new());

#[tauri::command]
pub fn sysmon_history() -> Vec<Sample> { HIST.lock().unwrap().iter().cloned().collect() }

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    /// e.g. "C:\\"
    pub mount: String,
    /// Volume label ("Local Disk" when empty).
    pub label: String,
    pub total: u64,
    pub available: u64,
    pub removable: bool,
    pub fs: String,
}

/// Real drives from Windows (fixed and removable). Drives that can't be read are skipped.
#[tauri::command]
pub async fn disks_list() -> Vec<DiskInfo> {
    crate::widgets::blocking(|| {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut v: Vec<DiskInfo> = disks.list().iter()
            .filter(|d| d.total_space() > 0)
            .map(|d| DiskInfo {
                mount: d.mount_point().to_string_lossy().into_owned(),
                label: { let n = d.name().to_string_lossy().into_owned(); if n.is_empty() { "Local Disk".into() } else { n } },
                total: d.total_space(),
                available: d.available_space(),
                removable: d.is_removable(),
                fs: d.file_system().to_string_lossy().into_owned(),
            })
            .collect();
        v.sort_by(|a, b| a.mount.cmp(&b.mount));
        v
    }).await.unwrap_or_default()
}

#[tauri::command]
pub async fn battery_get() -> Option<crate::platform::Battery> {
    crate::widgets::blocking(|| crate::platform::power().battery()).await.ok().flatten()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerInfo { pub mode: Option<crate::platform::PowerMode>, pub saver_on: Option<bool>, pub other_plan: Option<String> }

#[tauri::command]
pub async fn power_get() -> PowerInfo {
    crate::widgets::blocking(|| {
        let p = crate::platform::power();
        PowerInfo { mode: p.power_mode(), saver_on: p.saver_on(), other_plan: p.other_plan() }
    }).await.unwrap_or(PowerInfo { mode: None, saver_on: None, other_plan: None })
}

#[tauri::command]
pub async fn power_set_mode(mode: crate::platform::PowerMode) -> crate::platform::Res<()> {
    crate::widgets::blocking(move || crate::platform::power().set_power_mode(mode)).await?
}

/// The user asked for Windows' Balanced plan so power modes work.
#[tauri::command]
pub async fn power_use_balanced() -> crate::platform::Res<()> {
    crate::widgets::blocking(|| crate::platform::power().use_balanced_plan()).await?
}

pub fn start(app: AppHandle) {
    // System monitor: every 2 s while placed.
    let a = app.clone();
    std::thread::spawn(move || {
        use sysinfo::{MemoryRefreshKind, Networks, System};
        let mut sys = System::new();
        let mut nets = Networks::new_with_refreshed_list();
        let period = Duration::from_secs(2);
        loop {
            let (on, gpu_on) = {
                let cfg = a.state::<crate::AppState>().config.lock().unwrap().clone();
                (cfg.uses("sysmon"), cfg.widget_settings("sysmon")["gpu"] == true)
                // (temperature is cheap: one counter read per sample)
            };
            if !on {
                HIST.lock().unwrap().clear();
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            sys.refresh_cpu_usage();
            sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
            nets.refresh(true);
            let (rx, tx) = nets.iter().fold((0, 0), |(r, t), (_, n)| (r + n.received(), t + n.transmitted()));
            let s = Sample {
                cpu: sys.global_cpu_usage(),
                mem: if sys.total_memory() > 0 { sys.used_memory() as f32 * 100.0 / sys.total_memory() as f32 } else { 0.0 },
                gpu: if gpu_on { crate::platform::gpu().usage() } else { None },
                rx_bps: rx / period.as_secs(),
                tx_bps: tx / period.as_secs(),
                at: crate::db::now_ms(),
                cpu_temp: crate::platform::gpu().cpu_temp_c(),
            };
            {
                let mut h = HIST.lock().unwrap();
                h.push_back(s.clone());
                while h.len() > HISTORY { h.pop_front(); }
            }
            let _ = a.emit("sysmon:sample", s);
            std::thread::sleep(period);
        }
    });

    // Battery: every 30 s while placed, emit on change.
    std::thread::spawn(move || {
        let mut last = None;
        loop {
            if app.state::<crate::AppState>().config.lock().unwrap().uses("quicksettings") { // battery lives there
                let b = crate::platform::power().battery();
                if b != last {
                    let _ = app.emit("battery:changed", ());
                    last = b;
                }
            }
            std::thread::sleep(Duration::from_secs(30));
        }
    });
}
