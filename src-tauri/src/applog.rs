//! A small diagnostics log (%APPDATA%\GlassBar\glassbar.log): start-up, wake-ups, monitor
//! changes, where the dock went, self-repairs. Capped at 256 KB (the previous one is kept as
//! glassbar.log.old). Never holds personal data.
use std::io::Write;
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());
const MAX: u64 = 256 * 1024;

pub fn log(msg: impl AsRef<str>) {
    if cfg!(test) { return; } // tests must not write to the real log
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = crate::data_dir().join("glassbar.log");
    if std::fs::metadata(&path).map(|m| m.len() > MAX).unwrap_or(false) {
        let _ = std::fs::rename(&path, path.with_extension("log.old"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), msg.as_ref());
    }
}
