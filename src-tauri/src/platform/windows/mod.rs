//! Windows implementation of the platform traits.
mod appbar;
mod audio;
mod display;
mod location;
mod media;
mod radios;
mod system;
mod windows_list;

use super::*;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

pub struct WinShell;
pub struct WinAudio;
pub struct WinMedia;
pub struct WinRadios;
pub struct WinDisplay;
pub struct WinLocation;

/// Join this thread to the COM multithreaded apartment (no-op if already joined).
pub(crate) fn com_init() {
    unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
}

pub(crate) fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }

pub(crate) fn err(e: impl std::fmt::Display) -> String { e.to_string() }

impl Shell for WinShell {
    fn style_window(&self, win: &tauri::WebviewWindow, g: &Glass) { appbar::style_window(win, g) }
    fn init(&self, dir: &std::path::Path, f: Box<dyn Fn(SystemEvent) + Send>) {
        appbar::set_state_dir(dir);
        system::start(f);
    }
    fn appbar_set(&self, edge: Edge, strip: Rect) -> Rect { appbar::set(edge, strip) }
    fn appbar_remove(&self) { appbar::remove() }
    fn mouse_down(&self) -> bool {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
        unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000 != 0 }
    }
    fn set_taskbar(&self, mode: TaskbarMode) { appbar::set_taskbar(mode) }
    fn list_windows(&self) -> Vec<AppWindow> { windows_list::list() }
    fn watch_windows(&self, f: Box<dyn Fn() + Send>) { windows_list::watch(f) }
    fn activate(&self, id: isize) { windows_list::activate(id) }
    fn close(&self, id: isize) { windows_list::close(id) }
    fn launch(&self, path: &str, args: &str) -> Res<()> { windows_list::launch(path, args) }
    fn exe_icon(&self, exe: &str) -> Option<String> { windows_list::exe_icon(exe) }
}

impl Audio for WinAudio {
    fn state(&self) -> Res<AudioState> { audio::state() }
    fn set_volume(&self, v: f32) -> Res<()> { audio::set_volume(v) }
    fn set_mute(&self, m: bool) -> Res<()> { audio::set_mute(m) }
    fn set_default(&self, id: &str) -> Res<()> { audio::set_default(id) }
    fn sessions(&self) -> Res<Vec<AudioSession>> { audio::sessions() }
    fn set_session_volume(&self, pid: u32, v: f32) -> Res<()> { audio::set_session_volume(pid, v) }
    fn set_session_mute(&self, pid: u32, m: bool) -> Res<()> { audio::set_session_mute(pid, m) }
    fn watch(&self, f: Box<dyn Fn(AudioState) + Send>) { audio::watch(f) }
}

impl Media for WinMedia {
    fn info(&self) -> Res<Option<MediaInfo>> { media::info() }
    fn control(&self, a: MediaAction) -> Res<()> { media::control(a) }
    fn seek(&self, ms: i64) -> Res<()> { media::seek(ms) }
    fn watch(&self, f: Box<dyn Fn(Option<MediaInfo>) + Send>) { media::watch(f) }
}

impl Radios for WinRadios {
    fn list(&self) -> Res<Vec<RadioInfo>> { radios::list() }
    fn set(&self, kind: Option<RadioKind>, on: bool) -> Res<()> { radios::set(kind, on) }
}

impl Display for WinDisplay {
    fn mode(&self) -> Option<DisplayMode> { display::mode() }
    fn set_mode(&self, m: DisplayMode) -> Res<()> { display::set_mode(m) }
    fn brightness(&self) -> Option<u32> { display::brightness() }
    fn set_brightness(&self, v: u32) -> Res<()> { display::set_brightness(v) }
}

impl Location for WinLocation {
    fn locate(&self) -> Res<(f64, f64)> { location::locate() }
}
