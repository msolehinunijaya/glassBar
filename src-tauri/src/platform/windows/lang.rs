//! Keyboard input languages (the ones in Settings > Time & language). Reads the installed
//! layouts and the one active in the last app you used (GlassBar never has focus for
//! long, so its own layout is meaningless). Switching asks that app's window to change,
//! the same message the language bar sends.
use super::super::KeyboardLayout;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Globalization::{GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDDISPLAYNAME};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, PostMessageW, WM_INPUTLANGCHANGEREQUEST};

fn locale_of(hkl: HKL) -> (String, String) {
    let lcid = (hkl.0 as usize & 0xFFFF) as u32;
    let mut name = [0u16; 85];
    let n = unsafe { LCIDToLocaleName(lcid, Some(&mut name), 0) };
    if n <= 1 { return (format!("{lcid:04x}"), format!("Language {lcid:04x}")); }
    let mut disp = [0u16; 128];
    let m = unsafe { GetLocaleInfoEx(PCWSTR(name.as_ptr()), LOCALE_SLOCALIZEDDISPLAYNAME, Some(&mut disp)) };
    let loc = String::from_utf16_lossy(&name[..n as usize - 1]);
    let display = if m > 1 { String::from_utf16_lossy(&disp[..m as usize - 1]) } else { loc.clone() };
    (loc, display)
}

/// "en-US" -> "EN" (ISO 639-1, as Windows shows it).
pub fn code_of(locale: &str) -> String { locale.split('-').next().unwrap_or(locale).to_uppercase() }

fn active_hkl() -> HKL {
    let hwnd = HWND(super::windows_list::last_foreground() as *mut _);
    let tid = unsafe { GetWindowThreadProcessId(hwnd, None) };
    unsafe { GetKeyboardLayout(tid) }
}

pub fn list() -> Vec<KeyboardLayout> {
    let n = unsafe { GetKeyboardLayoutList(None) };
    let mut buf = vec![HKL::default(); n.max(0) as usize];
    let got = unsafe { GetKeyboardLayoutList(Some(&mut buf)) } as usize;
    let active = active_hkl().0 as isize;
    buf[..got].iter().map(|h| {
        let (locale, name) = locale_of(*h);
        KeyboardLayout { id: h.0 as isize, code: code_of(&locale), name, locale, active: h.0 as isize == active }
    }).collect()
}

/// Ask the last-used app to switch to `id`. Returns false if there is no such window.
pub fn set(id: isize) -> bool {
    let hwnd = HWND(super::windows_list::last_foreground() as *mut _);
    if hwnd.0.is_null() { return false; }
    unsafe { PostMessageW(Some(hwnd), WM_INPUTLANGCHANGEREQUEST, WPARAM(0), LPARAM(id)).is_ok() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn iso_code() {
        assert_eq!(super::code_of("en-US"), "EN");
        assert_eq!(super::code_of("ms-MY"), "MS");
        assert_eq!(super::code_of("zh-Hans-CN"), "ZH");
    }
}
