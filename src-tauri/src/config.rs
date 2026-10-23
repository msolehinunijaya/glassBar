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
            w("start", "small"), w("apps", "large"), w("music", "medium"),
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

/// The one shared location (Weather, sunrise/sunset, Waktu Solat, IPU, world clock).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LocationCfg {
    /// "auto" (Windows Location, then IP) | "windows" | "ip" | "manual".
    pub mode: String,
    /// Manual location (mode "manual").
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub name: String,
    /// IANA time zone of the manual location, when the city search gave one.
    pub tz: Option<String>,
    /// Before v9: auto + useDevice. Read once for the migration, never written.
    #[serde(skip_serializing)]
    pub auto: Option<bool>,
    #[serde(skip_serializing)]
    pub use_device: Option<bool>,
}
impl Default for LocationCfg {
    fn default() -> Self { Self { mode: "auto".into(), lat: None, lon: None, name: String::new(), tz: None, auto: None, use_device: None } }
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
    /// Pop-up banners next to the dock for GlassBar's own notifications.
    pub banners: Banners,
}

/// show: banners on (else a Windows toast) · seconds: 3/5/8/10 · position: "auto" (next
/// to the dock) or "corner" · on_click: "highlight" or "open" (also open the panel) ·
/// muted: sources with no banner (still in the list) · sound: play the alert sound.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Banners { pub show: bool, pub seconds: u32, pub position: String, pub on_click: String, pub muted: Vec<String>, pub sound: bool }
impl Default for Banners {
    fn default() -> Self { Self { show: true, seconds: 5, position: "auto".into(), on_click: "highlight".into(), muted: vec![], sound: true } }
}

/// Sounds when a USB device is plugged in / removed. Off by default: Windows plays its own.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceSounds { pub connect: bool, pub disconnect: bool,
    /// Charger plugged in / unplugged (two different sounds).
    pub charger: bool,
    /// 0..1
    pub volume: f32 }
impl Default for DeviceSounds {
    fn default() -> Self { Self { connect: false, disconnect: false, charger: true, volume: 0.4 } }
}

impl Default for Config {
    fn default() -> Self {
        Self { version: CURRENT_VERSION, active_profile: "Default".into(),
               profiles: BTreeMap::from([("Default".into(), Profile::default())]),
               theme: Theme::default(), startup: Startup::default(),
               dnd: false, location: LocationCfg::default(), device_sounds: DeviceSounds::default(), banners: Banners::default() }
    }
}

/// v2 added the Start/Search button widget.
/// v3 folded the Disk widget into the System Monitor; v4 the Tray widget into Quick Settings.
pub const CURRENT_VERSION: u32 = 9;

/// Migration helper: widget `from` became part of widget `into`. Its settings (mapped by
/// `map`) are merged into the existing `into` widget, or it turns into one in place.
fn fold(widgets: &mut Vec<WidgetInst>, from: &str, into: &str, map: impl Fn(&serde_json::Value) -> serde_json::Value) {
    let Some(i) = widgets.iter().position(|w| w.id == from) else { return };
    let add = map(&widgets[i].settings);
    if let Some(t) = widgets.iter_mut().find(|w| w.id == into) {
        if !t.settings.is_object() { t.settings = serde_json::json!({}); }
        for (k, v) in add.as_object().into_iter().flatten() { t.settings[k] = v.clone(); }
        widgets.remove(i);
    } else {
        let w = &mut widgets[i];
        w.id = into.into();
        w.instance = format!("{into}-{}", w.instance);
        w.settings = add;
    }
    widgets.retain(|w| w.id != from);
}

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
        if self.version < 3 {
            // Disk storage moved into the System Monitor.
            for p in self.profiles.values_mut() {
                let Some(i) = p.widgets.iter().position(|w| w.id == "disk") else { continue };
                if let Some(m) = p.widgets.iter_mut().find(|w| w.id == "sysmon") {
                    if !m.settings.is_object() { m.settings = serde_json::json!({}); }
                    m.settings["disk"] = true.into();
                    p.widgets.remove(i);
                } else {
                    let w = &mut p.widgets[i];
                    w.id = "sysmon".into();
                    w.instance = format!("sysmon-{}", w.instance);
                    w.settings = serde_json::json!({ "cpu": false, "ram": false, "network": false, "disk": true });
                }
                p.widgets.retain(|w| w.id != "disk");
            }
            // The music "Mini" (two buttons) was replaced by the full Player as the compact look.
            for p in self.profiles.values_mut() {
                for w in p.widgets.iter_mut().filter(|w| w.id == "music" && w.settings["style"] == "mini") {
                    w.settings["style"] = "player".into();
                }
            }
        }
        if self.version < 4 {
            // The System Tray widget became a section of Quick Settings.
            for p in self.profiles.values_mut() {
                let Some(i) = p.widgets.iter().position(|w| w.id == "tray") else { continue };
                if let Some(q) = p.widgets.iter_mut().find(|w| w.id == "quicksettings") {
                    if !q.settings.is_object() { q.settings = serde_json::json!({}); }
                    q.settings["trayOnDock"] = true.into();
                    p.widgets.remove(i);
                } else {
                    let w = &mut p.widgets[i];
                    w.id = "quicksettings".into();
                    w.instance = format!("quicksettings-{}", w.instance);
                    w.settings = serde_json::json!({ "trayOnDock": true });
                }
                p.widgets.retain(|w| w.id != "tray");
            }
        }
        if self.version < 5 {
            // The Keyboard Language widget became a Quick Settings option (code on the dock).
            for p in self.profiles.values_mut() {
                let Some(i) = p.widgets.iter().position(|w| w.id == "lang") else { continue };
                if let Some(q) = p.widgets.iter_mut().find(|w| w.id == "quicksettings") {
                    if !q.settings.is_object() { q.settings = serde_json::json!({}); }
                    q.settings["langOnDock"] = true.into();
                    p.widgets.remove(i);
                } else {
                    let w = &mut p.widgets[i];
                    w.id = "quicksettings".into();
                    w.instance = format!("quicksettings-{}", w.instance);
                    w.settings = serde_json::json!({ "langOnDock": true });
                }
                p.widgets.retain(|w| w.id != "lang");
            }
        }
        if self.version < 6 {
            // Battery became part of Quick Settings (icon after Bluetooth), Weather part of
            // the Clock (on its left). Each keeps showing on the dock, with its settings.
            for p in self.profiles.values_mut() {
                fold(&mut p.widgets, "battery", "quicksettings", |old| serde_json::json!({
                    "batteryOnDock": true,
                    "batteryStyle": old["style"].as_str().unwrap_or("classic"),
                }));
                fold(&mut p.widgets, "weather", "clock", |old| serde_json::json!({
                    "showWeather": true,
                    "weatherUnit": old["unit"].as_str().unwrap_or("c"),
                    "weatherColorful": old["colorful"].as_bool().unwrap_or(true),
                }));
            }
        }
        if self.version < 7 {
            // Volume became the middle segment of Quick Settings. Only bars that had the
            // Sound widget get it; the others keep it off.
            for p in self.profiles.values_mut() {
                if p.widgets.iter().any(|w| w.id == "sound") {
                    fold(&mut p.widgets, "sound", "quicksettings", |old| serde_json::json!({
                        "soundOnDock": true,
                        "sound": if old.is_object() { old.clone() } else { serde_json::json!({}) },
                    }));
                } else if let Some(q) = p.widgets.iter_mut().find(|w| w.id == "quicksettings") {
                    if !q.settings.is_object() { q.settings = serde_json::json!({}); }
                    q.settings["soundOnDock"] = false.into();
                }
            }
        }
        if self.version < 8 {
            // Pomodoro, To-Do and Notes became one widget ("productivity") with a segment
            // each. It takes the place of the first of them; only the ones that were on the
            // bar show; their settings carry over (flat).
            for p in self.profiles.values_mut() {
                const PARTS: [(&str, &str); 3] = [("pomodoro", "showPomo"), ("todo", "showTodo"), ("notes", "showNotes")];
                let Some(at) = p.widgets.iter().position(|w| PARTS.iter().any(|(id, _)| *id == w.id)) else { continue };
                let mut s = serde_json::json!({});
                for (id, flag) in PARTS {
                    let old = p.widgets.iter().find(|w| w.id == id);
                    s[flag] = old.is_some().into();
                    for (k, v) in old.and_then(|w| w.settings.as_object()).into_iter().flatten() { s[k] = v.clone(); }
                }
                let size = p.widgets[at].size.clone();
                p.widgets[at] = WidgetInst { id: "productivity".into(), instance: "productivity-1".into(), size, settings: s };
                p.widgets.retain(|w| !PARTS.iter().any(|(id, _)| *id == w.id));
            }
        }
        if self.version < 9 {
            // Location "detect automatically" + "use device location" became one mode.
            let l = &mut self.location;
            l.mode = match (l.auto, l.use_device) {
                (Some(false), _) => "manual",
                (_, Some(false)) => "ip",
                _ => "auto",
            }.into();
        }
        self.location.auto = None;
        self.location.use_device = None;
        if !matches!(self.location.mode.as_str(), "auto" | "windows" | "ip" | "manual") { self.location.mode = "auto".into(); }
        if !matches!((self.location.lat, self.location.lon), (Some(a), Some(o)) if crate::widgets::location::valid_lat_lon(a, o)) {
            (self.location.lat, self.location.lon) = (None, None);
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
    fn v9_turns_auto_and_use_device_into_a_mode() {
        let mode = |loc: serde_json::Value| {
            let c: Config = serde_json::from_value(serde_json::json!({ "version": 8, "location": loc })).unwrap();
            c.normalized().location
        };
        assert_eq!(mode(serde_json::json!({ "auto": true, "useDevice": true })).mode, "auto");
        assert_eq!(mode(serde_json::json!({ "auto": true, "useDevice": false })).mode, "ip");
        let m = mode(serde_json::json!({ "auto": false, "useDevice": true, "lat": 3.1, "lon": 101.7, "name": "KL" }));
        assert_eq!((m.mode.as_str(), m.lat, m.name.as_str()), ("manual", Some(3.1), "KL"));
        assert_eq!((m.auto, m.use_device), (None, None));
        assert!(!serde_json::to_string(&m).unwrap().contains("useDevice"));
        assert_eq!(mode(serde_json::json!({})).mode, "auto");
        let bad: Config = serde_json::from_value(serde_json::json!({ "location": { "mode": "x", "lat": 95.0, "lon": 0.0 } })).unwrap();
        let bad = bad.normalized().location;
        assert_eq!((bad.mode.as_str(), bad.lat), ("auto", None));
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

    #[test]
    fn v3_folds_the_disk_widget_into_the_system_monitor() {
        let w = |id: &str| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "small".into(), settings: serde_json::Value::Null };
        // With a system monitor already placed: disk goes, the monitor shows disk.
        let mut c = Config { version: 2, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("sysmon"), w("disk"), w("clock")];
        let c = c.normalized();
        let ws = &c.profiles["Default"].widgets;
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["sysmon", "clock"]);
        assert_eq!(ws[0].settings["disk"], true);
        // Without one: the disk widget becomes a system monitor showing disk only.
        let mut c = Config { version: 2, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("disk")];
        let c = c.normalized();
        let ws = &c.profiles["Default"].widgets;
        assert_eq!((ws[0].id.as_str(), &ws[0].settings["disk"], &ws[0].settings["cpu"]), ("sysmon", &serde_json::json!(true), &serde_json::json!(false)));
    }

    #[test]
    fn v4_moves_the_tray_widget_into_quick_settings() {
        let w = |id: &str| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "small".into(), settings: serde_json::Value::Null };
        let mut c = Config { version: 3, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("quicksettings"), w("tray"), w("clock")];
        let c = c.normalized();
        let ws = &c.profiles["Default"].widgets;
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["quicksettings", "clock"]);
        assert_eq!(ws[0].settings["trayOnDock"], true, "tray icons stay visible on the dock");
        // Tray widget but no Quick Settings: it becomes Quick Settings with tray icons on.
        let mut c = Config { version: 3, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("tray")];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!((ws[0].id.as_str(), &ws[0].settings["trayOnDock"]), ("quicksettings", &serde_json::json!(true)));
    }

    #[test]
    fn v8_combines_pomodoro_todo_and_notes() {
        let w = |id: &str, s: serde_json::Value| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "medium".into(), settings: s };
        let mut c = Config { version: 7, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![
            w("clock", serde_json::Value::Null), w("pomodoro", serde_json::json!({ "workMin": 50 })),
            w("todo", serde_json::Value::Null), w("music", serde_json::Value::Null),
        ];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["clock", "productivity", "music"], "takes the first one's place");
        let s = &ws[1].settings;
        assert_eq!((&s["showPomo"], &s["showTodo"], &s["showNotes"], &s["workMin"]),
            (&serde_json::json!(true), &serde_json::json!(true), &serde_json::json!(false), &serde_json::json!(50)), "notes wasn't on the bar");
        // None of them on the bar: nothing added.
        let mut c = Config { version: 7, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("clock", serde_json::Value::Null)];
        assert!(!c.normalized().uses("productivity"));
    }

    #[test]
    fn v7_moves_volume_into_quick_settings_only_where_it_was() {
        let w = |id: &str, s: serde_json::Value| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "small".into(), settings: s };
        let mut c = Config { version: 6, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("sound", serde_json::json!({ "layout": "icon", "wheel": true })), w("quicksettings", serde_json::Value::Null)];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["quicksettings"]);
        assert_eq!((&ws[0].settings["soundOnDock"], &ws[0].settings["sound"]["layout"], &ws[0].settings["sound"]["wheel"]),
            (&serde_json::json!(true), &serde_json::json!("icon"), &serde_json::json!(true)), "keeps the volume options");
        // No Sound widget: Quick Settings must not suddenly grow a volume control.
        let mut c = Config { version: 6, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("quicksettings", serde_json::Value::Null)];
        assert_eq!(c.normalized().profiles["Default"].widgets[0].settings["soundOnDock"], false);
    }

    #[test]
    fn v6_folds_battery_into_quick_settings_and_weather_into_the_clock() {
        let w = |id: &str, s: serde_json::Value| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "small".into(), settings: s };
        let mut c = Config { version: 5, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![
            w("quicksettings", serde_json::json!({ "trayOnDock": true })), w("battery", serde_json::json!({ "style": "liquid" })),
            w("clock", serde_json::Value::Null), w("weather", serde_json::json!({ "unit": "f", "colorful": false })),
        ];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["quicksettings", "clock"]);
        assert_eq!((&ws[0].settings["batteryOnDock"], &ws[0].settings["batteryStyle"], &ws[0].settings["trayOnDock"]),
            (&serde_json::json!(true), &serde_json::json!("liquid"), &serde_json::json!(true)), "keeps existing settings");
        assert_eq!((&ws[1].settings["showWeather"], &ws[1].settings["weatherUnit"], &ws[1].settings["weatherColorful"]),
            (&serde_json::json!(true), &serde_json::json!("f"), &serde_json::json!(false)));
        // No Quick Settings / Clock on the bar: the old widget becomes one, in place.
        let mut c = Config { version: 5, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("weather", serde_json::Value::Null), w("battery", serde_json::Value::Null)];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!(ws.iter().map(|x| (x.id.as_str(), x.settings["showWeather"].as_bool().or(x.settings["batteryOnDock"].as_bool()))).collect::<Vec<_>>(),
            [("clock", Some(true)), ("quicksettings", Some(true))]);
    }

    #[test]
    fn v5_moves_the_language_widget_into_quick_settings() {
        let w = |id: &str| WidgetInst { id: id.into(), instance: format!("{id}-1"), size: "small".into(), settings: serde_json::Value::Null };
        let mut c = Config { version: 4, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("lang"), w("quicksettings"), w("clock")];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!(ws.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["quicksettings", "clock"]);
        assert_eq!(ws[0].settings["langOnDock"], true);
        let mut c = Config { version: 4, ..Config::default() };
        c.profiles.get_mut("Default").unwrap().widgets = vec![w("lang")];
        let ws = c.normalized().profiles["Default"].widgets.clone();
        assert_eq!((ws[0].id.as_str(), &ws[0].settings["langOnDock"]), ("quicksettings", &serde_json::json!(true)));
    }
}
