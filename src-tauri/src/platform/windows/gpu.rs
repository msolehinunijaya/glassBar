//! GPU 3D-engine utilization from Windows performance counters (what Task Manager shows).
use std::sync::Mutex;
use windows::core::w;
use windows::Win32::System::Performance::*;

struct Query { q: PDH_HQUERY, c: PDH_HCOUNTER }
// PDH handles are plain process-wide handles; we only use them under the mutex.
unsafe impl Send for Query {}

static QUERY: Mutex<Option<Query>> = Mutex::new(None);

fn open() -> Option<Query> {
    unsafe {
        let mut q = PDH_HQUERY::default();
        if PdhOpenQueryW(None, 0, &mut q) != 0 { return None; }
        let mut c = PDH_HCOUNTER::default();
        if PdhAddEnglishCounterW(q, w!("\\GPU Engine(*engtype_3D)\\Utilization Percentage"), 0, &mut c) != 0 {
            let _ = PdhCloseQuery(q);
            return None;
        }
        PdhCollectQueryData(q); // first sample primes the rate counter
        Some(Query { q, c })
    }
}

/// Sum of 3D engine utilization across processes, 0..100. First call returns ~0.
pub fn usage() -> Option<f32> {
    let mut guard = QUERY.lock().unwrap();
    if guard.is_none() { *guard = open(); }
    let Query { q, c } = guard.as_ref()?;
    unsafe {
        if PdhCollectQueryData(*q) != 0 { return None; }
        let (mut size, mut count) = (0u32, 0u32);
        // First call asks for the buffer size (PDH_MORE_DATA).
        PdhGetFormattedCounterArrayW(*c, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if size == 0 { return Some(0.0); }
        let mut buf = vec![0u8; size as usize];
        let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        if PdhGetFormattedCounterArrayW(*c, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 { return None; }
        let total: f64 = std::slice::from_raw_parts(items, count as usize).iter()
            .map(|i| i.FmtValue.Anonymous.doubleValue)
            .sum();
        Some(total.clamp(0.0, 100.0) as f32)
    }
}
