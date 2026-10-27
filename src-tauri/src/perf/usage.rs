//! What the GlassBar process tree uses: glassbar.exe plus every WebView2 helper it started.

/// `root` and every process below it, from `(pid, parent pid)` pairs. A visited set stops
/// cycles (PID reuse can make a process look like its own ancestor).
pub fn tree(root: u32, procs: &[(u32, u32)]) -> Vec<u32> {
    let mut out = vec![root];
    let mut i = 0;
    while i < out.len() {
        let p = out[i];
        for &(pid, parent) in procs {
            if parent == p && !out.contains(&pid) { out.push(pid); }
        }
        i += 1;
    }
    out
}

/// One reading of the GlassBar tree, plus the system load Auto decides from.
pub struct Snapshot {
    /// Private memory of the whole tree, MB.
    pub ram_mb: f32,
    /// % of the whole CPU (Task Manager's scale), summed over the tree.
    pub cpu: f32,
    /// 3D engine %, summed over the tree; None when GPU counters are unavailable.
    pub gpu: Option<f32>,
    pub sys_mem_pct: f32,
    pub sys_cpu_pct: f32,
}

pub struct Meter { sys: sysinfo::System }

impl Meter {
    pub fn new() -> Self { Self { sys: sysinfo::System::new() } }

    /// CPU is measured since the previous call, so the first reading is ~0.
    pub fn sample(&mut self) -> Snapshot {
        use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate};
        let s = &mut self.sys;
        s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_cpu());
        s.refresh_cpu_usage();
        s.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let procs: Vec<(u32, u32)> = s.processes().iter()
            .filter_map(|(pid, p)| Some((pid.as_u32(), p.parent()?.as_u32()))).collect();
        let pids = tree(std::process::id(), &procs);
        let hw = crate::platform::gpu();
        let cores = s.cpus().len().max(1) as f32;
        Snapshot {
            ram_mb: pids.iter().filter_map(|&p| hw.private_bytes(p)).sum::<u64>() as f32 / 1_048_576.0,
            cpu: pids.iter().filter_map(|&p| s.process(Pid::from_u32(p))).map(|p| p.cpu_usage()).sum::<f32>() / cores,
            gpu: hw.usage_for(&pids),
            sys_mem_pct: if s.total_memory() > 0 { s.used_memory() as f32 * 100.0 / s.total_memory() as f32 } else { 0.0 },
            sys_cpu_pct: s.global_cpu_usage(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_has_root_and_all_descendants_only() {
        let procs = [(10, 1), (20, 10), (21, 10), (30, 20), (99, 1), (98, 99)];
        let mut t = tree(10, &procs); t.sort();
        assert_eq!(t, vec![10, 20, 21, 30]);
    }

    #[test]
    fn tree_survives_a_parent_cycle() {
        // PID reuse can make a process look like its own ancestor.
        let mut t = tree(10, &[(10, 30), (20, 10), (30, 20)]); t.sort();
        assert_eq!(t, vec![10, 20, 30]);
    }
}
