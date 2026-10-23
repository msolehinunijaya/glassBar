//! Temperature from Windows' "Thermal Zone Information" performance counters (no admin
//! needed). These are the ACPI thermal zones the firmware exposes; on most PCs the hottest
//! zone tracks the CPU package closely, but it is not a per-core sensor. If the PC exposes
//! no zone, this returns None and the UI shows N/A; nothing is estimated.
use std::sync::Mutex;
use windows::core::w;
use windows::Win32::System::Performance::*;

struct Query { q: PDH_HQUERY, c: PDH_HCOUNTER }
unsafe impl Send for Query {}
static QUERY: Mutex<Option<Option<Query>>> = Mutex::new(None);

fn open() -> Option<Query> {
    unsafe {
        let mut q = PDH_HQUERY::default();
        if PdhOpenQueryW(None, 0, &mut q) != 0 { return None; }
        let mut c = PDH_HCOUNTER::default();
        if PdhAddEnglishCounterW(q, w!("\\Thermal Zone Information(*)\\Temperature"), 0, &mut c) != 0 {
            let _ = PdhCloseQuery(q);
            return None;
        }
        Some(Query { q, c })
    }
}

/// Hottest thermal zone in °C, or None when the PC exposes no thermal zones.
pub fn cpu_temp_c() -> Option<f32> {
    let mut guard = QUERY.lock().unwrap();
    let Query { q, c } = guard.get_or_insert_with(open).as_ref()?;
    unsafe {
        if PdhCollectQueryData(*q) != 0 { return None; }
        let (mut size, mut count) = (0u32, 0u32);
        PdhGetFormattedCounterArrayW(*c, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if size == 0 { return None; }
        let mut buf = vec![0u8; size as usize];
        let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        if PdhGetFormattedCounterArrayW(*c, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 { return None; }
        let max_k = std::slice::from_raw_parts(items, count as usize).iter()
            .map(|i| i.FmtValue.Anonymous.doubleValue)
            .filter(|k| *k > 200.0 && *k < 400.0) // Kelvin; ignore empty/garbage zones
            .fold(f64::NAN, f64::max);
        (!max_k.is_nan()).then(|| (max_k - 273.15) as f32)
    }
}
