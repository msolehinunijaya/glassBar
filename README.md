# GlassBar

GlassBar is a lightweight, customizable glass taskbar for Windows 10/11, built with Tauri 2, Rust, and Svelte 5.

The bar can dock to any edge or float. You can drag it, resize it, and edit it iPhone-style. It supports profiles and light/dark/auto themes with Windows acrylic blur.

## Widgets

**Desktop**
- **App Launcher:** pinned and running apps grouped by program. Drag icons to reorder (saved); right-click to pin/unpin, move, or pin another app; pin from installed apps or by browsing in its settings. Missing programs show greyed out. Right-click also shows recent files.
- **Music:** Spotify, browsers, and any media app. The Player design shows art, title and artist, Previous / Play-Pause / Next, progress with times, volume, and a live sound visualizer (length adjustable) driven by Windows' output level meter. Classic and Mini (Play/Pause + Next) designs remain.
- **Sound:** volume, output device, and per-app volume.
- **Quick Settings:** Wi-Fi (list networks, connect with password, disconnect), Bluetooth (paired and nearby devices, pair, remove), hotspot, airplane mode, display mode, brightness, keyboard language and tray buttons (each section can be hidden).
- **Clock & Date:** with a calendar.
- **Battery:** charge, a plug icon whenever it's on mains, time left, and Windows power mode. Updates the moment you plug in or unplug.
- **System Monitor:** CPU (with temperature), RAM, network and disk storage (used of total, %, per drive, orange/red thresholds) with live graphs. The old separate Disk widget was merged in automatically.
- **Keyboard Language:** the input language (EN, MS…) of the app you're using; click to switch.
- **System Tray:** click it for a panel of every real tray icon with its app name; real tray icons of background apps on the dock (opt-in): always-shown ones inline, the rest in an overflow panel; click, double-click and right-click reach the app as in Windows. Off: opens Windows' hidden-icons flyout.

**Productivity**
- **To-Do:** quick tasks with due-date reminders; scroll through every task right on the dock (green 3D icon).
- **Quick Note:** (yellow 3D icon) sticky notes with pasted text and images. You can drag images out into other apps.
- **Pomodoro:** focus sessions with short and long breaks.
- **Timer & Stopwatch:** presets, several timers at once, and laps.
- **Screen Time:** today's usage, top apps, and a 7-day graph. It records only while the widget is on your bar.
- **Notifications:** GlassBar's alert history and Do Not Disturb, plus a Windows tab that opens Windows Notification Center.
- **Time Progress:** dot-grid cards for the month, day and year (52 week-dots for the year), with the percentage passed.

**Location**
- **Weather:** colourful pill; dashboard with feels-like, humidity, wind, rain chance, a sunrise/sunset arc, hourly and 3-day forecast (Open-Meteo).
- **Currency Converter:** live rates (ExchangeRate-API, Frankfurter as fallback), cached for an hour, with favourite pairs.
- **Waktu Solat:** prayer times by JAKIM zone (auto-detected), with Aladhan outside Malaysia. Shows a countdown and can alert you, with an optional azan sound.
- **IPU (Air Quality):** live Air Pollutant Index from the nearest JAS/DOE station, with haze alerts.

**OCR & AI**
- **Capture** (Screenshot | Recording | OCR Snap): screenshots of the full screen, an area, a window, or a whole scrolling page (it scrolls the window and stitches the page), copied to the clipboard automatically and kept in a history (copy, open, discard) that cleans itself after 1–30 days. Recording starts Windows' own recorders (Snipping Tool for an area, Game Bar for an app).
- **OCR Snap:** press **Win+Shift+T** (changeable), drag a box over anything on screen, and the text is on your clipboard. The screen is frozen while you select, so video text can't move. Modes: Text, Code (keeps indentation) and Terminal (drops shell prompts). Engine: Windows OCR (offline, default) or Claude (sends the selection to Anthropic). Keeps the last 200 captures.
- **AI Usage:** ask Claude a quick question from the bar, and see tokens used today and this month (as reported by Anthropic). Your API key is stored in Windows Credential Manager, never in GlassBar's files.

**Dock and sounds**
- Works on all four screen edges; on the left/right every widget stacks in a column, text wraps instead of rotating.
- Optional sounds when a USB device is connected or removed, and distinct sounds when the charger is plugged in or unplugged (Settings › Sounds).
- One shared location for every location-based widget: Windows location (with its accuracy), approximate IP location, or a city you pick; refresh and reset in Settings.

### Known limitations
- **Scrolling screenshots** work in windows that scroll with the mouse wheel (browsers, documents, lists). The pointer briefly sits over the window while it scrolls; content that sticks while scrolling (e.g. a site's floating header) can appear more than once.
- **Store apps** (WhatsApp, Spotify, Settings…) are started through their app identity, as the Start menu does; Windows doesn't allow starting them from their .exe.
- **Sound visualizer** shows the real output level (Windows' peak meter, the one in the volume mixer); it is not a per-frequency spectrum, which would need capturing the audio stream.
- **CPU temperature** comes from Windows' ACPI thermal zones. It's close to the CPU package on most laptops but isn't a per-core sensor. PCs that expose no thermal zone show **N/A**; nothing is estimated.
- **Tray icons:** Windows 11 doesn't let other programs draw its tray icons. GlassBar opens Windows' own hidden-icons flyout, which only works while the Windows taskbar is shown; otherwise it opens tray settings.
- **Windows notifications:** only packaged (MSIX/Store) apps may read other apps' notifications, so the Windows tab opens Notification Center instead of listing them. A combined Dock + Windows list isn't possible for the same reason.
- **Currency rates** are indicative mid-market rates, not bank or card rates. ExchangeRate-API updates daily; Frankfurter uses ECB reference rates (working days).
- **OCR quality** depends on the Windows language packs installed (Settings › Time & language). Tesseract isn't bundled; Windows OCR covers the same languages without a 30 MB+ install.
- **AI token counts** only cover GlassBar's own requests; no cost is shown because prices change. Check console.anthropic.com for billing.
- **Hotkeys** that Windows reserves can't be used; GlassBar shows when registering fails.
- **Tray icons on the dock** work by receiving the notification-icon messages apps send to the taskbar (as RetroBar and Cairo do) and forwarding every one to Explorer, so Windows' own tray is unaffected. Icons appear as apps add or update them (GlassBar asks them all to re-add when it starts). Rich pop-up tooltips and balloon notifications are not shown on the dock. "Always show" grouping follows Windows 11's per-app setting.
- **Bluetooth connect/disconnect** of an already paired device has no public Windows API for apps, so those buttons open Windows Bluetooth settings. Pairing (with Windows' own PIN/confirm dialog) and removing work in the dock.
- **Wi-Fi** listing needs "Let desktop apps access your location" on Windows 11 (Windows' rule for network scans). Hidden networks and enterprise (802.1X) sign-in use Windows settings.
- **Location:** when the Windows location service is off, location comes from the IP address and is approximate (often several km off); the panel says so. Positions are rounded to about 100 m.
- **Device sounds** cover USB devices (Windows' USB device notifications). Windows plays its own device sounds too unless you turn them off in the Sound control panel.
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

- **Phase 3:** OCR Snap, AI Usage, hotkeys, secure API keys (done)
- **Phase 4:** macOS
