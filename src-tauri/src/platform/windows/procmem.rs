//! Private memory of a process: bytes only it owns (committed), so summing a process tree
//! doesn't count shared pages twice the way working set would.
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

pub fn private_bytes(pid: u32) -> Option<u64> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut m = PROCESS_MEMORY_COUNTERS_EX::default();
        let ok = K32GetProcessMemoryInfo(h, &mut m as *mut _ as *mut PROCESS_MEMORY_COUNTERS, size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32).as_bool();
        let _ = CloseHandle(h);
        ok.then_some(m.PrivateUsage as u64)
    }
}
