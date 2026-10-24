use super::blocking;
use crate::platform::{display, radios, shell, DisplayMode, Hotspot, KeyboardLayout, RadioInfo, RadioKind, Res};

#[derive(serde::Serialize)]
pub struct WifiList { current: Option<String>, networks: Vec<crate::platform::WifiNetwork> }

#[tauri::command]
pub async fn wifi_list(scan: bool) -> Res<WifiList> {
    blocking(move || crate::platform::wireless().wifi_list(scan).map(|(current, networks)| WifiList { current, networks })).await?
}
/// The password goes straight to Windows; GlassBar doesn't store it.
#[tauri::command]
pub async fn wifi_connect(ssid: String, password: Option<String>) -> Res<()> {
    blocking(move || crate::platform::wireless().wifi_connect(&ssid, password.as_deref())).await?
}
#[tauri::command]
pub async fn wifi_disconnect() -> Res<()> { blocking(|| crate::platform::wireless().wifi_disconnect()).await? }
#[tauri::command]
pub async fn bt_list(discover: bool) -> Res<Vec<crate::platform::BtDevice>> { blocking(move || crate::platform::wireless().bt_list(discover)).await? }
#[tauri::command]
pub async fn bt_pair(id: String) -> Res<()> { blocking(move || crate::platform::wireless().bt_pair(&id)).await? }
#[tauri::command]
pub async fn bt_unpair(id: String) -> Res<()> { blocking(move || crate::platform::wireless().bt_unpair(&id)).await? }
/// Native fallbacks: the Windows network flyout / Bluetooth settings.
#[tauri::command]
pub fn open_settings_uri(uri: String) -> Res<()> {
    if !uri.starts_with("ms-settings:") && !uri.starts_with("ms-availablenetworks:") { return Err("not a settings link".into()); }
    shell().launch(&uri, "")
}

/// The clock's "follow Windows" 12/24-hour default.
#[tauri::command]
pub fn clock_system_12h() -> Option<bool> { shell().time_12h() }

#[tauri::command]
pub fn lang_list() -> Vec<KeyboardLayout> { shell().keyboard_layouts() }
/// Falls back to Win+Space (Windows' own switcher) when there's no app window to switch.
#[tauri::command]
pub fn lang_set(id: isize) { if !shell().keyboard_layout_set(id) { shell().press_keys(&[0x5B, 0x20]) } }
/// Minimise everything to the desktop, or bring it all back (Win+D).
#[tauri::command]
pub fn show_desktop() { shell().press_keys(&[0x5B, b'D' as u16]) }
/// Windows' own notification center (Win+N).
#[tauri::command]
pub fn windows_notifications() { shell().press_keys(&[0x5B, b'N' as u16]) }


#[tauri::command]
pub async fn radios_list() -> Res<Vec<RadioInfo>> { blocking(move || radios().list()).await? }
#[tauri::command]
pub async fn radio_set(kind: RadioKind, on: bool) -> Res<()> { blocking(move || radios().set(Some(kind), on)).await? }
/// Windows has no public airplane-mode API: "on" turns every radio off.
#[tauri::command]
pub async fn airplane_set(on: bool) -> Res<()> { blocking(move || radios().set(None, !on)).await? }
#[tauri::command]
pub async fn display_mode() -> Option<DisplayMode> { blocking(move || display().mode()).await.ok().flatten() }
#[tauri::command]
pub async fn display_set_mode(mode: DisplayMode) -> Res<()> { blocking(move || display().set_mode(mode)).await? }
#[tauri::command]
pub async fn brightness_get() -> Option<u32> { blocking(move || display().brightness()).await.ok().flatten() }
#[tauri::command]
pub async fn brightness_set(value: u32) -> Res<()> { blocking(move || display().set_brightness(value)).await? }

#[tauri::command]
pub async fn hotspot_get() -> Res<Hotspot> { blocking(move || radios().hotspot()).await? }
#[tauri::command]
pub async fn hotspot_set(on: bool) -> Res<()> { blocking(move || radios().set_hotspot(on)).await? }

// ---- Tray icons on the dock (opt-in) ----

/// Host tray icons while Quick Settings shows them (its panel section or on the dock).
pub fn sync_tray(app: &tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    let want = {
        let st = app.state::<crate::AppState>();
        let c = st.config.lock().unwrap();
        let q = c.widget_settings("quicksettings");
        c.uses("quicksettings") && (q["showTray"] != false || q["trayOnDock"] == true)
    };
    let t = crate::platform::tray();
    if want && !t.running() {
        let a = app.clone();
        t.start(Box::new(move || { let _ = a.emit("tray:changed", ()); }));
    } else if !want && t.running() {
        t.stop();
    }
}

#[tauri::command]
pub fn tray_icons() -> Vec<crate::platform::TrayIcon> { crate::platform::tray().icons() }
/// Screen recording with Windows' own tools: "area" = Snipping Tool (Win+Shift+R),
/// "app" = Xbox Game Bar (Win+Alt+R, records the active app).
#[tauri::command]
pub fn record_screen(kind: String) -> Res<()> {
    match kind.as_str() {
        "area" => { shell().press_keys(&[0x5B, 0x10, b'R' as u16]); Ok(()) }
        "app" => { shell().press_keys(&[0x5B, 0x12, b'R' as u16]); Ok(()) }
        _ => Err("unknown recording type".into()),
    }
}

#[tauri::command]
pub fn tray_refresh() { crate::platform::tray().refresh() }
#[tauri::command]
pub fn tray_click(id: String, button: String) -> bool { crate::platform::tray().click(&id, &button) }
