//! Platform abstraction: every OS call goes through these traits.
//! Windows lives in `platform/windows/`; a macOS module can be added later
//! behind `#[cfg(target_os = "macos")]` without touching widget code.
use crate::config::{Edge, TaskbarMode};
use serde::{Deserialize, Serialize};

pub type Res<T> = Result<T, String>;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppWindow { pub id: isize, pub title: String, pub exe: String, pub active: bool, pub minimized: bool }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice { pub id: String, pub name: String, pub is_default: bool }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioState { pub volume: f32, pub muted: bool, pub devices: Vec<AudioDevice> }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSession { pub pid: u32, pub name: String, pub exe: String, pub volume: f32, pub muted: bool }

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub app: String, pub title: String, pub artist: String, pub album: String,
    /// data: URL of the album art, if the player provides one.
    pub art: Option<String>,
    pub playing: bool,
    pub position_ms: i64, pub duration_ms: i64,
    /// Unix ms at which `position_ms` was true (frontend interpolates from here).
    pub updated_at_ms: i64,
    pub can_next: bool, pub can_prev: bool,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MediaAction { Toggle, Next, Prev }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RadioKind { Wifi, Bluetooth, Other }

#[derive(Serialize, Clone, Debug)]
pub struct RadioInfo { pub kind: RadioKind, pub name: String, pub on: bool }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode { Internal, Clone, Extend, External }

/// OS events the bar must react to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SystemEvent {
    /// Resolution, monitor added/removed, projection mode changed.
    DisplayChanged,
    /// Explorer restarted: AppBar registration and taskbar state must be re-applied.
    ShellRestarted,
    /// A fullscreen app appeared (true) or left (false) on the bar's monitor.
    Fullscreen(bool),
    /// Plugged in / unplugged / battery level changed (instant, from Windows).
    PowerChanged,
}

#[derive(Serialize, Clone, Debug)]
pub struct RecentItem { pub name: String, pub path: String }

/// How a window's glass is drawn.
pub struct Glass { pub native: bool, pub dark: bool, pub opacity: f64 }

pub trait Shell: Send + Sync {
    /// Hide a window at the OS level as well as through Tauri (belt and braces: a preview
    /// must never stay on screen as an empty box).
    fn hide_window(&self, win: &tauri::WebviewWindow);
    /// Native blur + rounded corners (or plain transparent window for CSS glass).
    fn style_window(&self, win: &tauri::WebviewWindow, glass: &Glass);
    /// Reserve `strip` on `edge` so maximized windows stop short of the bar.
    /// Start the system event source and set where to keep state that must survive a crash.
    /// Call once at startup, before any other Shell method.
    fn init(&self, data_dir: &std::path::Path, on_event: Box<dyn Fn(SystemEvent) + Send>);
    /// Returns the strip Windows actually granted (it may shift to avoid other bars).
    fn appbar_set(&self, edge: Edge, strip: Rect) -> Rect;
    fn appbar_remove(&self);
    /// No-op when the mode is unchanged. Restores the user's original state on `Show`.
    fn set_taskbar(&self, mode: TaskbarMode);
    /// Live thumbnails of windows drawn into `dest` (client pixels). Returns the windows
    /// that got one; the caller shows a fallback for the rest.
    fn thumbs_show(&self, dest: isize, items: &[(isize, Rect)]) -> Vec<isize>;
    /// Remove every thumbnail (must run whenever the preview hides).
    fn thumbs_clear(&self);
    /// Press keys together (in order) and release them: e.g. [LWIN] opens Start.
    fn press_keys(&self, keys: &[u16]);
    /// Installed input languages; `active` = the one in the last app you used.
    fn keyboard_layouts(&self) -> Vec<KeyboardLayout>;
    /// Switch the last-used app to this layout; false if that wasn't possible.
    fn keyboard_layout_set(&self, id: isize) -> bool;
    /// Is the left mouse button down right now (used to detect the end of a window drag)?
    fn mouse_down(&self) -> bool;
    /// Milliseconds since the last keyboard/mouse input.
    fn idle_ms(&self) -> u64;
    /// Full path of the foreground window's program, if any.
    fn foreground_exe(&self) -> Option<String>;
    /// Recently used files that open with `exe`, newest first.
    fn recent_for(&self, exe: &str, limit: usize) -> Vec<RecentItem>;
    fn list_windows(&self) -> Vec<AppWindow>;
    /// Calls `on_change` (debounced) whenever top-level windows change.
    fn watch_windows(&self, on_change: Box<dyn Fn() + Send>);
    /// Focus, restore, or minimize if it is already the active window.
    fn activate(&self, id: isize);
    fn close(&self, id: isize);
    fn launch(&self, path: &str, args: &str) -> Res<()>;
    /// PNG data: URL for an executable's icon.
    fn exe_icon(&self, exe: &str) -> Option<String>;
    /// Icon for a window (Store apps, or programs whose file has no icon).
    fn window_icon(&self, id: isize) -> Option<String>;
}

pub trait Audio: Send + Sync {
    fn state(&self) -> Res<AudioState>;
    fn set_volume(&self, v: f32) -> Res<()>;
    fn set_mute(&self, m: bool) -> Res<()>;
    fn set_default(&self, id: &str) -> Res<()>;
    fn sessions(&self) -> Res<Vec<AudioSession>>;
    fn set_session_volume(&self, pid: u32, v: f32) -> Res<()>;
    fn set_session_mute(&self, pid: u32, m: bool) -> Res<()>;
    fn watch(&self, on_change: Box<dyn Fn(AudioState) + Send>);
}

pub trait Media: Send + Sync {
    fn info(&self) -> Res<Option<MediaInfo>>;
    fn control(&self, a: MediaAction) -> Res<()>;
    fn seek(&self, position_ms: i64) -> Res<()>;
    fn watch(&self, on_change: Box<dyn Fn(Option<MediaInfo>) + Send>);
}

pub trait Radios: Send + Sync {
    fn list(&self) -> Res<Vec<RadioInfo>>;
    fn set(&self, kind: Option<RadioKind>, on: bool) -> Res<()>;
    /// Mobile hotspot; Err(reason) when this PC/connection can't host one.
    fn hotspot(&self) -> Res<Hotspot>;
    fn set_hotspot(&self, on: bool) -> Res<()>;
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Hotspot { pub on: bool, pub ssid: String, pub clients: u32, pub max_clients: u32 }

/// An installed keyboard input language. `id` is the Windows HKL.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardLayout { pub id: isize, pub code: String, pub name: String, pub locale: String, pub active: bool }

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Battery { pub percent: u8, pub charging: bool, pub on_ac: bool, pub secs_left: Option<u32> }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PowerMode { Efficiency, Balanced, Performance }

pub trait Power: Send + Sync {
    /// None on PCs without a battery.
    fn battery(&self) -> Option<Battery>;
    fn power_mode(&self) -> Option<PowerMode>;
    fn set_power_mode(&self, m: PowerMode) -> Res<()>;
    /// Battery/energy saver on? None if unknown.
    fn saver_on(&self) -> Option<bool>;
}

pub trait Gpu: Send + Sync {
    /// GPU utilization 0..100, None if counters are unavailable.
    fn usage(&self) -> Option<f32>;
    /// CPU/thermal-zone temperature in °C; None when the PC exposes no sensor.
    fn cpu_temp_c(&self) -> Option<f32>;
}

pub trait Location: Send + Sync {
    /// Device position (lat, lon) from the OS location service.
    fn locate(&self) -> Res<(f64, f64)>;
}

pub trait Display: Send + Sync {
    fn mode(&self) -> Option<DisplayMode>;
    fn set_mode(&self, m: DisplayMode) -> Res<()>;
    fn brightness(&self) -> Option<u32>;
    fn set_brightness(&self, v: u32) -> Res<()>;
}

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod win;

#[cfg(windows)]
pub fn shell() -> &'static dyn Shell { &win::WinShell }
#[cfg(windows)]
pub fn audio() -> &'static dyn Audio { &win::WinAudio }
#[cfg(windows)]
pub fn media() -> &'static dyn Media { &win::WinMedia }
#[cfg(windows)]
pub fn radios() -> &'static dyn Radios { &win::WinRadios }
#[cfg(windows)]
pub fn display() -> &'static dyn Display { &win::WinDisplay }
#[cfg(windows)]
pub fn location() -> &'static dyn Location { &win::WinLocation }
#[cfg(windows)]
pub fn power() -> &'static dyn Power { &win::WinPower }
#[cfg(windows)]
pub fn gpu() -> &'static dyn Gpu { &win::WinGpu }
