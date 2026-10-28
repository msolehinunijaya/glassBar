# GlassBar Architecture

GlassBar is a glass-style taskbar replacement for Windows 10/11, built with Tauri 2 (Rust) and Svelte 5 (TypeScript). macOS is planned later; the code is laid out so it can be added without a rewrite.

## Layout

```
src/                         Svelte frontend (one bundle, three windows)
  main.ts                    picks Bar / Popup / Settings by window label
  app/Bar.svelte             bar shell: drag-to-move + snap, end handles, momentum scroll, edit mode
  app/Popup.svelte           shared popup window; shows the clicked widget's panel
  app/Gallery.svelte         "add widget" list (in the popup)
  app/Settings.svelte        settings window
  lib/ipc.ts                 the ONLY file that calls invoke()/listen(); typed
  lib/config.svelte.ts       reactive config mirror (Rust owns the truth)
  lib/theme.ts               theme -> CSS variables
  lib/geometry.ts            snap/reorder math (unit tested)
  styles/glass.css           design tokens + glass primitives (.pill, .card, .tile, .slider)
  widgets/registry.ts        auto-discovers widgets/*/index.ts
  widgets/types.ts           the widget contract (defineWidget)
  widgets/<id>/              one folder per widget (+ _template)

src-tauri/src/
  main.rs                    builder, plugins, AppState, commit(), command list
  config.rs                  Config types, load (corrupt -> config.bad.json), atomic save, import/export
  db.rs                      SQLite (glassbar.db): forward-only migrations, kv cache table, fired markers
  alerts.rs                  alert center: history, toast, sound, Do Not Disturb
  scheduler.rs               one thread that sleeps until the next due item and fires it exactly once
  net.rs                     shared HTTP client (10 s timeout)
  updates.rs                 update check (GitHub Releases), one alert per version
  bar.rs                     bar placement + AppBar, popup placement, settings window, tray
  widgets/*.rs               Tauri commands per widget (todo, notes, focus, weather, solat, ipu, sysmon, ...)
  platform/mod.rs            traits: Shell, Audio, Media, Radios, Display, Location, Power, Gpu + data types
  platform/windows/*.rs      Windows implementation (windows-rs 0.62)
```

## Platform abstraction

All OS calls go through traits in `platform/mod.rs`, reached through `platform::shell()`, `audio()`, `media()`, `radios()` and `display()`. Today those return the Windows implementation under `#[cfg(windows)]`. A macOS port adds `platform/macos/` and a `#[cfg(target_os = "macos")]` arm, and nothing else changes. Widget command modules never import `windows::` directly.

| Trait | Windows implementation |
|---|---|
| Shell | `SHAppBarMessage` (reserve space), taskbar auto-hide/hide via `Shell_TrayWnd`, `EnumWindows` + `SetWinEventHook` (live window list), `ShellExecuteW`, `SHDefExtractIconW` (icons), acrylic via `window-vibrancy` + DWM rounded corners |
| Audio | Core Audio (`IMMDeviceEnumerator`, `IAudioEndpointVolume`, `IAudioSessionManager2`), default-device switch via `IPolicyConfig` |
| Media | `GlobalSystemMediaTransportControlsSessionManager` (WinRT), with events |
| Radios | `Windows.Devices.Radios` (Wi-Fi, Bluetooth; airplane = all off) |
| Display | `SetDisplayConfig` topology (Win+P modes); brightness via DDC/CI (`dxva2`), WMI fallback for laptops |
| Location | `Windows.Devices.Geolocation` (falls back to IP lookup in `widgets/location.rs`) |
| Power | `GetSystemPowerStatus` |
| Gpu | PDH counter `\GPU Engine(*engtype_3D)\Utilization Percentage` |

## Data flow

- **Request/response:** Svelte calls `ipc.*`, which invokes a `#[tauri::command]`, which calls a platform trait.
- **Live updates:** platform watchers emit Tauri events (`apps:changed`, `audio:changed`, `media:changed`). Widgets subscribe through `events.*`.
- **Stored data:** to-dos, notes, timers, alerts, and screen time live in SQLite. Every write emits `<name>:changed` (for example `todos:changed`), and widgets reload through `live()` in `src/lib/live.svelte.ts`.
- **Time-based work:** Pomodoro phases, timers, to-do due dates, prayer times, and daily housekeeping are `Due` items. The scheduler thread fires each once (recorded in the `fired` table) and sleeps until the next one, capped at 60 s so it notices PC sleep. Anything more than 15 minutes late is recorded as missed, with no toast or sound. Webviews never count time themselves; they compute countdowns from timestamps.
- **Network data:** weather (30 min), prayer times (weekly per zone), IPU (15 min), and location (6 h) are fetched in Rust and cached in the `kv` table. When offline, the cache is returned, marked `offline`.
- **Background samplers:** screen time (5 s), system monitor (2 s), and battery (30 s) run only while their widget is placed in some profile (`Config::uses`).
- **Config:** Rust holds `Mutex<Config>`. `save_config` calls `commit()`, which normalizes, writes the file atomically, re-applies the bar and tray, syncs autostart, and broadcasts `config:changed` to every window.

## Windows

| Label | What it is |
|---|---|
| `bar` | frameless, transparent, always-on-top, not in the taskbar |
| `popup` | one reusable window. Rust places it beside the bar, next to the clicked widget, and hides it on blur |
| `settings` | normal window, created on first open |

## Widget plugin system

A widget is a folder `src/widgets/<id>/` whose `index.ts` default-exports `defineWidget({...})`. The manifest declares `Pill`, an optional `Popup` (with its size), an optional `SettingsForm`, the allowed sizes, and default settings. The registry finds widgets automatically. A widget that needs OS access adds `src-tauri/src/widgets/<id>.rs` and registers its commands once in `main.rs`. See [WIDGET_SDK.md](WIDGET_SDK.md).

## Config file

The config lives at `%APPDATA%\GlassBar\config.json`. It holds:
- profiles, each with bar settings and an ordered widget list;
- the theme;
- startup options;
- Do Not Disturb;
- the shared location used by Weather, Waktu Solat and IPU.

Rust reads widget settings with `Config::widget_settings(id)`. A bad file is copied to `config.bad.json` and replaced with defaults.

## Safety

The native taskbar is restored on Quit, on a panic (panic hook), and from the tray item "Restore Windows taskbar". A force-kill (Task Manager, or Ctrl+C on `tauri dev`) skips restore. In that case, restart Explorer or start GlassBar and choose the tray item.

## Dock effects

- **Magnification:** the enlarged App Launcher icons are drawn in their own small window, `magnify` (`src-tauri/src/magnify.rs`, `src/app/Magnify.svelte`). It is transparent, never focusable, and lets clicks through. This is needed because the dock's native glass covers its whole window. The dock itself only widens the icon slots, and gets longer (`bar_grow`) only when they no longer fit.
- **Minimize effects:** Genie, Scale and Suck are live DWM thumbnails of the window, drawn on a transparent, click-through overlay (`platform/windows/effects.rs`). Genie and Suck cut the window into 40 strips. The geometry is pure and unit tested (`src-tauri/src/effects.rs`). The first App Launcher tells Rust where each window flies to (`effects_targets`).
- **What we checked on Windows 11 26200:**
  - At `EVENT_SYSTEM_MINIMIZESTART` the window is already iconic, so its old place comes from `GetWindowPlacement`.
  - DWM doesn't draw thumbnails on such an overlay when it covers a whole monitor, so the overlay is sized to the effect's path and kept one pixel short of the screen.
  - `DWMWA_TRANSITIONS_FORCEDISABLED` is accepted on other apps' windows. GlassBar sets it while effects are on, and clears it on Quit.
- **Restore:** the reverse effect plays only when GlassBar itself restores the window, by a click on its icon or its spot.
