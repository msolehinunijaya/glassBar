//! Jump lists (approximation): Windows has no public API to read another app's
//! jump list, so we show recent files whose file type opens with that app.
use crate::platform::RecentItem;

fn file_name(p: &str) -> String { p.rsplit(['\\', '/']).next().unwrap_or(p).to_lowercase() }

/// UNC paths can block for seconds (SMB timeouts) if the server is offline.
pub fn is_network_path(p: &str) -> bool { p.starts_with(r"\\") || p.starts_with("//") }

/// Does the program registered for a file type (`assoc_exe`) match `exe`?
pub fn matches_exe(assoc_exe: &str, exe: &str) -> bool {
    !assoc_exe.is_empty() && file_name(assoc_exe) == file_name(exe)
}

#[tauri::command]
pub async fn jumplist(exe: String) -> Vec<RecentItem> {
    crate::widgets::blocking(move || crate::platform::shell().recent_for(&exe, 8)).await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_paths_are_skipped() {
        assert!(is_network_path(r"\\server\share\a.docx"));
        assert!(is_network_path("//server/share/a.docx"));
        assert!(!is_network_path(r"C:\Users\me\a.docx"));
    }

    #[test]
    fn matches_exe_by_file_name_case_insensitive() {
        assert!(matches_exe(r"C:\Windows\system32\NOTEPAD.EXE", r"C:\Windows\notepad.exe"));
        assert!(matches_exe("notepad.exe", r"C:\Windows\System32\notepad.exe"));
        assert!(!matches_exe(r"C:\Program Files\Code\Code.exe", r"C:\Windows\notepad.exe"));
        assert!(!matches_exe("", "notepad.exe"));
    }
}
