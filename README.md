# GlassBar

GlassBar is a lightweight, customizable glass taskbar for Windows 10/11, built with Tauri 2, Rust, and Svelte 5.

**MVP widgets:**

- **App Launcher:** pinned and running apps, grouped by program
- **Music:** Spotify, browsers, and any media app
- **Sound:** volume, output device, and per-app volume
- **Quick Settings:** Wi-Fi, Bluetooth, airplane mode, display mode, and brightness
- **Clock & Date:** includes a calendar

The bar can dock to any edge or float. You can drag it, resize it, and edit it iPhone-style. It supports profiles and light/dark/auto themes with Windows acrylic blur.

## Prerequisites

- **Windows 10 1809+ or Windows 11.** WebView2 is preinstalled on Windows 11.
- **Rust (stable)** with the MSVC toolchain, plus **Visual Studio Build Tools** ("Desktop development with C++").
- **Node.js 20+.**

## Run

```bash
npm install
```

```bash
npm run tauri dev
```

The bar first appears at the top of the screen. To change where it sits:

- **Drag the bar** to dock it to an edge or let it float.
- **Right-click the bar** to edit widgets or open Settings.
- **Use the tray icon** to open Settings, switch profiles, restore the Windows taskbar, or quit.

> If you stop `tauri dev` with Ctrl+C while the Windows taskbar is set to "hidden", the taskbar stays hidden: the process is killed before it can restore it. Start GlassBar again and choose tray → **Restore Windows taskbar**, or restart Explorer.

## Test

```bash
npm test
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

```bash
npm run check
```

## Build the installer

```bash
npm run tauri build
```

The MSI and NSIS installers are written to `src-tauri/target/release/bundle/`.

## Docs

- [Architecture](docs/ARCHITECTURE.md): folder structure, platform layer, data flow
- [Widget SDK](docs/WIDGET_SDK.md): how to add a widget

## Roadmap

- **Phase 2:** SQLite, To-Do, Quick Note, Pomodoro, Timer, Battery, Weather, Waktu Solat, System Monitor, Screen Time, Notifications, jump lists, auto-update
- **Phase 3:** OCR Snap, AI Usage, hotkeys, secure API keys
- **Phase 4:** macOS
