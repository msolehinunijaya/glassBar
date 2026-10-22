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
  bar.rs                     bar placement + AppBar, popup placement, settings window, tray
  widgets/*.rs               thin Tauri commands per widget, + start_watchers()
  platform/mod.rs            traits: Shell, Audio, Media, Radios, Display + shared data types
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

## Data flow

- **Request/response:** Svelte calls `ipc.*`, which invokes a `#[tauri::command]`, which calls a platform trait.
- **Live updates:** platform watchers emit Tauri events (`apps:changed`, `audio:changed`, `media:changed`). Widgets subscribe through `events.*`.
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

The config lives at `%APPDATA%\GlassBar\config.json`. It holds profiles (each with bar settings and an ordered widget list), the theme, and startup options. A bad file is copied to `config.bad.json` and replaced with defaults.

## Safety

The native taskbar is restored on Quit, on a panic (panic hook), and from the tray item "Restore Windows taskbar". A force-kill (Task Manager, or Ctrl+C on `tauri dev`) skips restore. In that case, restart Explorer or start GlassBar and choose the tray item.
