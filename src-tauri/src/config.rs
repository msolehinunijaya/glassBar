//! Settings + layout config, stored as JSON in the app config dir.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Edge { Top, Bottom, Left, Right }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BarSize { Compact, Normal, Large }

impl BarSize {
    /// Bar thickness in logical px.
    pub fn px(self) -> f64 {
        match self { BarSize::Compact => 36.0, BarSize::Normal => 48.0, BarSize::Large => 60.0 }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskbarMode { Show, Autohide, Hidden }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode { Auto, Light, Dark }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Pos { pub x: i32, pub y: i32 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct BarCfg {
    pub edge: Edge,
    /// Index into the monitor list; None = the main (primary) display.
    pub monitor: Option<usize>,
    /// Fraction (0.2..=1.0) of the screen edge the bar spans.
    pub length: f64,
    pub size: BarSize,
    /// Physical top-left when floating; None = docked to `edge`.
    pub floating: Option<Pos>,
    pub reserve_space: bool,
    pub native_taskbar: TaskbarMode,
    /// Slide out of view, leaving a sliver at the edge; hovering the edge shows it.
    pub auto_hide: bool,
}

impl Default for BarCfg {
    fn default() -> Self {
        // Top + taskbar shown: safe first run, nothing hidden if the app is killed.
        Self { edge: Edge::Top, monitor: None, length: 0.7, size: BarSize::Normal, floating: None,
               reserve_space: true, native_taskbar: TaskbarMode::Show, auto_hide: false }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetInst {
    pub id: String,
    pub instance: String,
    #[serde(default = "medium")]
    pub size: String,
    #[serde(default)]
    pub settings: serde_json::Value,
}
fn medium() -> String { "medium".into() }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile { pub bar: BarCfg, pub widgets: Vec<WidgetInst> }

impl Default for Profile {
    fn default() -> Self {
        let w = |id: &str, size: &str| WidgetInst {
            id: id.into(), instance: format!("{id}-1"), size: size.into(), settings: serde_json::Value::Null,
        };
        Self { bar: BarCfg::default(), widgets: vec![
            w("start", "small"), w("apps", "large"), w("music", "medium"), w("sound", "small"),
            w("quicksettings", "small"), w("clock", "medium"),
        ] }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Theme {
    pub mode: ThemeMode,
    pub accent: String,
    /// CSS blur px, used when native blur is off.
    pub blur: u32,
    /// Tint opacity 0..1.
    pub opacity: f64,
    /// Windows acrylic behind the bar (true) or CSS-only glass (false).
    pub native_blur: bool,
}
impl Default for Theme {
    fn default() -> Self {
        Self { mode: ThemeMode::Auto, accent: "#0A84FF".into(), blur: 24, opacity: 0.45, native_blur: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Startup { pub launch_at_login: bool }
impl Default for Startup { fn default() -> Self { Self { launch_at_login: false } } }

/// Shared location for Weather, Waktu Solat and IPU.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LocationCfg {
    /// Detect automatically (Windows location, then IP). False = use lat/lon below.
    pub auto: bool,
    /// Ask Windows' location service (GPS/Wi-Fi). Off = approximate location from the IP only.
    pub use_device: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub name: String,
}
impl Default for LocationCfg {
    fn default() -> Self { Self { auto: true, use_device: true, lat: None, lon: None, name: String::new() } }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub version: u32,
    pub active_profile: String,
    pub profiles: BTreeMap<String, Profile>,
    pub theme: Theme,
    pub startup: Startup,
    /// Do Not Disturb: alerts are recorded but make no toast or sound.
    pub dnd: bool,
    pub location: LocationCfg,
    pub device_sounds: DeviceSounds,
}

/// Sounds when a USB device is plugged in / removed. Off by default: Windows plays its own.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceSounds { pub connect: bool, pub disconnect: bool, /// 0..1
    pub volume: f32 }
impl Default for DeviceSounds {
    fn default() -> Self { Self { connect: false, disconnect: false, volume: 0.4 } }
}

impl Default for Config {
    fn default() -> Self {
        Self { version: CURRENT_VERSION, active_profile: "Default".into(),
               profiles: BTreeMap::from([("Default".into(), Profile::default())]),
               theme: Theme::default(), startup: Startup::default(),
               dnd: false, location: LocationCfg::default(), device_sounds: DeviceSounds::default() }
    }
}

/// v2 added the Start/Search button widget.
pub const CURRENT_VERSION: u32 = 2;

impl Config {
    /// Repair values a hand-edited or imported file can get wrong, and upgrade old versions.
    pub fn normalized(mut self) -> Self {
        if self.version < 2 {
            // Put the new Start/Search button at the far left, once.
            for p in self.profiles.values_mut() {
                if !p.widgets.iter().any(|w| w.id == "start") {
                    p.widgets.insert(0, WidgetInst { id: "start".into(), instance: "start-1".into(), size: "small".into(), settings: serde_json::Value::Null });
                }
            }
        }
        self.version = CURRENT_VERSION;
        if self.profiles.is_empty() {
            self.profiles.insert("Default".into(), Profile::default());
        }
        if !self.profiles.contains_key(&self.active_profile) {
            self.active_profile = self.profiles.keys().next().unwrap().clone();
        }
        for p in self.profiles.values_mut() {
            p.bar.length = p.bar.length.clamp(0.2, 1.0);
            // Instance ids key the bar's widget list: they must be unique.
            let mut seen = std::collections::HashSet::new();
            for (i, w) in p.widgets.iter_mut().enumerate() {
                if !seen.insert(w.instance.clone()) {
                    w.instance = format!("{}-dup{i}", w.instance);
                    seen.insert(w.instance.clone());
                }
                if !matches!(w.size.as_str(), "small" | "medium" | "large") { w.size = medium(); }
            }
        }
        self.theme.opacity = self.theme.opacity.clamp(0.0, 1.0);
        self
    }
    pub fn profile(&self) -> &Profile { &self.profiles[&self.active_profile] }

    /// Is widget `id` placed in any profile?
    pub fn uses(&self, id: &str) -> bool {
        self.profiles.values().any(|p| p.widgets.iter().any(|w| w.id == id))
    }

    /// Settings of the first `id` instance, active profile first; Null if none.
    pub fn widget_settings(&self, id: &str) -> serde_json::Value {
        std::iter::once(self.profile())
            .chain(self.profiles.values())
            .flat_map(|p| p.widgets.iter())
            .find(|w| w.id == id)
            .map(|w| w.settings.clone())
            .unwrap_or(serde_json::Value::Null)
    }
    pub fn profile_mut(&mut self) -> &mut Profile {
        self.profiles.get_mut(&self.active_profile).expect("normalized config")
    }
}

/// Load config; a missing file gives defaults, a corrupt one is backed up first.
pub fn load(path: &Path) -> Config {
    let Ok(text) = std::fs::read_to_string(path) else { return Config::default() };
    // Notepad and PowerShell may add a UTF-8 byte-order mark; JSON parsers reject it.
    match serde_json::from_str::<Config>(text.trim_start_matches('\u{feff}')) {
        Ok(c) => c.normalized(),
        Err(e) => {
            eprintln!("config: {e}; backing up to config.bad.json");
            let _ = std::fs::write(path.with_file_name("config.bad.json"), &text);
            Config::default()
        }
    }
}

/// Atomic save: write a temp file, then rename over the old one.
pub fn save(path: &Path, c: &Config) -> std::io::Result<()> {
    if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(c)?)?;
    std::fs::rename(&tmp, path)
}

pub fn export(c: &Config) -> String { serde_json::to_string_pretty(c).unwrap() }

pub fn import(text: &str) -> Result<Config, String> {
    serde_json::from_str::<Config>(text).map(Config::normalized).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("glassbar-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn missing_file_gives_defaults() {
        let d = tmp("missing");
        let c = load(&d.join("config.json"));
        assert_eq!(c, Config::default());
        assert!(c.profiles.contains_key(&c.active_profile));
    }

    #[test]
    fn save_then_load_round_trips() {
        let d = tmp("roundtrip");
        let p = d.join("config.json");
        let mut c = Config::default();
        c.theme.accent = "#FF0000".into();
        c.profile_mut().bar.edge = Edge::Left;
        save(&p, &c).unwrap();
        assert_eq!(load(&p), c);
        assert!(!d.join("config.json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_backed_up_not_lost() {
        let d = tmp("corrupt");
        let p = d.join("config.json");
        std::fs::write(&p, "{ not json").unwrap();
        let c = load(&p);
        assert_eq!(c, Config::default());
        assert_eq!(std::fs::read_to_string(d.join("config.bad.json")).unwrap(), "{ not json");
    }

    #[test]
    fn partial_file_fills_missing_fields() {
        let d = tmp("partial");
        let p = d.join("config.json");
        std::fs::write(&p, r##"{"theme":{"accent":"#123456"}}"##).unwrap();
        let c = load(&p);
        assert_eq!(c.theme.accent, "#123456");
        assert_eq!(c.theme.mode, ThemeMode::Auto);
        assert!(c.profiles.contains_key(&c.active_profile));
    }

    #[test]
    fn unknown_active_profile_is_repaired() {
        let mut c = Config::default();
        c.active_profile = "nope".into();
        let c = c.normalized();
        assert!(c.profiles.contains_key(&c.active_profile));
    }

    #[test]
    fn import_repairs_duplicate_instances_and_bad_sizes() {
        let mut c = Config::default();
        let w = &mut c.profile_mut().widgets;
        w[1].instance = w[0].instance.clone();
        w[2].size = "gigantic".into();
        let c = import(&export(&c)).unwrap();
        let ids: std::collections::HashSet<_> = c.profile().widgets.iter().map(|w| w.instance.clone()).collect();
        assert_eq!(ids.len(), c.profile().widgets.len());
        assert_eq!(c.profile().widgets[2].size, "medium");
    }

    #[test]
    fn uses_finds_widget_in_any_profile() {
        let mut c = Config::default();
        assert!(c.uses("clock"));
        assert!(!c.uses("todo"));
        let mut other = Profile::default();
        other.widgets.push(WidgetInst { id: "todo".into(), instance: "todo-1".into(), size: "medium".into(), settings: serde_json::Value::Null });
        c.profiles.insert("Work".into(), other);
        assert!(c.uses("todo"));
    }

    #[test]
    fn widget_settings_prefers_active_profile() {
        let mut c = Config::default();
        let w = |v: i64| WidgetInst { id: "solat".into(), instance: format!("solat-{v}"), size: "small".into(), settings: serde_json::json!({ "v": v }) };
        let mut a = Profile::default();
        a.widgets.push(w(1));
        let mut b = Profile::default();
        b.widgets.push(w(2));
        c.profiles.insert("A".into(), a);
        c.profiles.insert("B".into(), b);
        c.active_profile = "B".into();
        assert_eq!(c.widget_settings("solat")["v"], 2);
        assert!(c.widget_settings("nope").is_null());
    }

    #[test]
    fn load_tolerates_utf8_bom() {
        let d = tmp("bom");
        let p = d.join("config.json");
        let mut c = Config::default();
        c.theme.accent = "#ABCDEF".into();
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(export(&c).into_bytes());
        std::fs::write(&p, bytes).unwrap();
        assert_eq!(load(&p).theme.accent, "#ABCDEF");
        assert!(!d.join("config.bad.json").exists());
    }

    #[test]
    fn default_bar_uses_the_primary_monitor() {
        assert_eq!(Config::default().profile().bar.monitor, None);
        let c = import(r#"{"profiles":{"P":{"bar":{"monitor":1}}},"activeProfile":"P"}"#).unwrap();
        assert_eq!(c.profile().bar.monitor, Some(1));
    }

    #[test]
    fn v1_configs_get_the_start_button_first_once() {
        let v1 = r#"{"version":1,"activeProfile":"A","profiles":{"A":{"widgets":[{"id":"apps","instance":"apps-1"}]},"B":{"widgets":[]}}}"#;
        let c = import(v1).unwrap();
        assert_eq!(c.version, CURRENT_VERSION);
        assert_eq!(c.profiles["A"].widgets[0].id, "start");
        assert_eq!(c.profiles["A"].widgets[1].id, "apps");
        assert_eq!(c.profiles["B"].widgets[0].id, "start");
        // Already migrated (user removed it on purpose): not added back.
        let mut again = c.clone();
        again.profiles.get_mut("A").unwrap().widgets.remove(0);
        let again = import(&export(&again)).unwrap();
        assert_eq!(again.profiles["A"].widgets[0].id, "apps");
    }

    #[test]
    fn default_profile_starts_with_the_start_button() {
        assert_eq!(Config::default().profile().widgets[0].id, "start");
    }

    #[test]
    fn import_rejects_garbage() {
        assert!(import("[1,2,3]").is_err());
        assert!(import(&export(&Config::default())).is_ok());
    }
}
