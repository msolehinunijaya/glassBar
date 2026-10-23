# GlassBar

GlassBar is a lightweight, customizable glass taskbar for Windows 10/11, built with Tauri 2, Rust, and Svelte 5.

The bar can dock to any edge or float. You can drag it, resize it, and edit it iPhone-style. It supports profiles and light/dark/auto themes with Windows acrylic blur.

## Widgets

**Desktop**
- **App Launcher:** pinned and running apps grouped by program. Right-click shows recent files.
- **Music:** Spotify, browsers, and any media app.
- **Sound:** volume, output device, and per-app volume.
- **Quick Settings:** Wi-Fi, Bluetooth, hotspot, airplane mode, display mode, brightness, keyboard language and tray buttons (each section can be hidden).
- **Clock & Date:** with a calendar.
- **Battery:** charge, a plug icon whenever it's on mains, time left, and Windows power mode. Updates the moment you plug in or unplug.
- **System Monitor:** CPU (with temperature), RAM, GPU, and network with live graphs.
- **Disk Storage:** used/free space per drive, with orange/red thresholds.
- **Keyboard Language:** the input language (EN, MS…) of the app you're using; click to switch.
- **System Tray:** opens Windows' hidden tray icons.

**Productivity**
- **To-Do:** quick tasks with due-date reminders.
- **Quick Note:** sticky notes with pasted text and images. You can drag images out into other apps.
- **Pomodoro:** focus sessions with short and long breaks.
- **Timer & Stopwatch:** presets, several timers at once, and laps.
- **Screen Time:** today's usage, top apps, and a 7-day graph. It records only while the widget is on your bar.
- **Notifications:** GlassBar's alert history and Do Not Disturb, plus a Windows tab that opens Windows Notification Center.
- **Time Progress:** how much of the day, month or year has passed.

**Location**
- **Weather:** colourful pill; dashboard with feels-like, humidity, wind, rain chance, a sunrise/sunset arc, hourly and 3-day forecast (Open-Meteo).
- **Currency Converter:** live rates (ExchangeRate-API, Frankfurter as fallback), cached for an hour, with favourite pairs.
- **Waktu Solat:** prayer times by JAKIM zone (auto-detected), with Aladhan outside Malaysia. Shows a countdown and can alert you, with an optional azan sound.
- **IPU (Air Quality):** live Air Pollutant Index from the nearest JAS/DOE station, with haze alerts.

### Known limitations
- **CPU temperature** comes from Windows' ACPI thermal zones. It's close to the CPU package on most laptops but isn't a per-core sensor. PCs that expose no thermal zone show **N/A**; nothing is estimated.
- **Tray icons:** Windows 11 doesn't let other programs draw its tray icons. GlassBar opens Windows' own hidden-icons flyout, which only works while the Windows taskbar is shown; otherwise it opens tray settings.
- **Windows notifications:** only packaged (MSIX/Store) apps may read other apps' notifications, so the Windows tab opens Notification Center instead of listing them. A combined Dock + Windows list isn't possible for the same reason.
- **Currency rates** are indicative mid-market rates, not bank or card rates. ExchangeRate-API updates daily; Frankfurter uses ECB reference rates (working days).
- **Keyboard language** switches the app you were last using; a few elevated (admin) apps ignore the request from a normal-privilege program.

To-dos, notes, timers, screen time, and alert history are stored on this PC in `%APPDATA%\GlassBar\glassbar.db`. Settings live in `config.json` in the same folder.

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
- **Use the tray icon** to open Settings, switch profiles, check for updates, restore the Windows taskbar, or quit.

> If you stop `tauri dev` with Ctrl+C while the Windows taskbar is set to "hidden", the taskbar stays hidden: the process is killed before it can restore it. Start GlassBar again and choose tray → **Restore Windows taskbar**.

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

Installers are signed for the auto-updater, so the build needs the private signing key. It lives at `%USERPROFILE%\.tauri\glassbar.key` and must never be committed.

In Git Bash, set the key first:

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/glassbar.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```

Then build:

```bash
npm run tauri build
```

The MSI and NSIS installers, plus the `.sig` signature files the updater checks, are written to `src-tauri/target/release/bundle/`.

## Releasing (auto-update)

Installed copies check `https://github.com/msolehinunijaya/glassBar/releases/latest/download/latest.json` on start and once a day.

1. **One-time setup:** on GitHub, go to **Settings → Secrets and variables → Actions**. Add a secret named `TAURI_SIGNING_PRIVATE_KEY` containing the full text of `%USERPROFILE%\.tauri\glassbar.key`.
2. **Bump the version** in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (for example `0.2.0`).
3. **Commit, then tag and push:**

   ```bash
   git tag v0.2.0
   ```

   ```bash
   git push origin v0.2.0
   ```

The `Release` workflow then builds, signs, and publishes the installers and `latest.json`. Running copies show an "Update available" alert, and users install it from **Settings → Updates**.

Keep a backup of `glassbar.key`. If it is lost, existing installs can't verify future updates.

## Docs

- [Architecture](docs/ARCHITECTURE.md): folder structure, platform layer, data flow
- [Widget SDK](docs/WIDGET_SDK.md): how to add a widget

## Roadmap

- **Phase 3:** OCR Snap, AI Usage, hotkeys, secure API keys
- **Phase 4:** macOS
