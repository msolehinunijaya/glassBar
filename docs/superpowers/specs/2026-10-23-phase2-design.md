# GlassBar Phase 2: Productivity Widgets (Design)

Status: ready for approval · Date: 2026-10-23 · Builds on the MVP (branch `mvp`, see `docs/ARCHITECTURE.md`)

## Intent

The MVP made GlassBar a working taskbar. Phase 2 turns it into the place where the user's day runs: tasks, notes, focus sessions, timers, prayer times, weather, and an honest view of where their time and machine resources go. Everything has to stay glanceable from the bar, and work offline wherever the data allows.

## Goals

1. **Storage.** Add SQLite (one DB file) for to-dos, notes, screen time and alert history. Settings stay in `config.json`.
2. **New widgets.** Each follows the existing widget SDK (Pill + optional Popup + optional SettingsForm), so it can be added, removed, resized and configured like the MVP widgets:
   - **To-Do**
   - **Quick Note**
   - **Pomodoro / Focus**
   - **Timer & Stopwatch**
   - **Battery**
   - **Weather**
   - **Waktu Solat**
   - **IPU (Air Pollutant Index)**
   - **System Monitor**
   - **Screen Time**
   - **Notifications**
3. **App Launcher.** Add right-click jump lists.
4. **Auto-update.** Updates come from GitHub Releases on `msolehinunijaya/glassBar`.

## Non-goals

- Reading other apps' Windows notifications. This needs MSIX packaging. **User decision:** the widget is GlassBar's own alert center instead.
- OCR, AI and hotkeys (Phase 3). macOS (Phase 4).
- Cloud sync of to-dos or notes.

## Anti-goals (a failure even if it "works")

- **A busy bar.** Timers or polling must not push idle CPU above the MVP baseline (~0.1% of all cores). There must be no per-second work while nothing is visible or running.
- **Lost data.** To-dos and notes must never be lost: writes are transactional, and the DB is never recreated over an existing file.
- **Silent alerts.** Alerts that fire twice (once per window), or not at all because a popup was closed. Timers must live in Rust, not in a webview.
- **Silent network calls.** Network calls on every redraw, or location sent anywhere the user didn't agree to.

## Decisions (user)

| Topic | Decision |
|---|---|
| Notifications | GlassBar's own alert center: alerts from Pomodoro, Timer, Waktu Solat and To-Do due dates, a badge count, history, and a Do Not Disturb toggle. Each alert also raises a Windows toast unless DND is on. |
| Updates | GitHub Releases. Installers are signed with a Tauri updater key. The app checks `latest.json` from the latest release. |
| Prayer zone | Auto from location, with a manual JAKIM zone picker as override. Outside Malaysia, use Aladhan by coordinates. |
| Screen Time | Records only while a Screen Time widget is placed in at least one profile. History is kept 30 days. |
| Note images | Plain text area with pasted images as a thumbnail strip below it (not inline). |
| Updater repo | `msolehinunijaya/glassBar` is public, so releases are fetched without a token. |

## Widget behavior

- **To-Do**
  - Quick add from the popup (Enter).
  - Checkbox, optional due date/time (native `<input type="datetime-local">`), delete, and "clear completed".
  - The pill shows the open count and, at medium/large size, the next due item.
  - A due item fires an alert once.
- **Quick Note**
  - Multiple notes, each with text plus pasted images (Ctrl+V of image data or text), a color tag, and a pin. Pinned notes sort first.
  - Auto-saves while typing (debounced).
  - "Temporary" mode: notes not edited for X days (setting, default off) are deleted on startup and daily.
  - Images can be dragged out of a note into other apps as real files.
- **Pomodoro**
  - Work/short-break/long-break cycle (defaults 25/5/15, long break every 4).
  - Shows a session counter ("2/4"), start/pause/skip/reset, and a sound + alert at each phase end.
- **Timer & Stopwatch**
  - Countdown timer with presets (1, 5, 10, 15, 30 min, plus custom) and an alert at zero.
  - Stopwatch with laps.
  - Several timers may run at once.
- **Battery**
  - Percentage, charging state, and estimated time left.
  - Hidden pill content on PCs with no battery: it shows "AC power".
- **Weather**
  - Current temperature and icon on the pill. The popup shows hourly (next 12 h) and a 3-day forecast.
  - Location is auto or a manual city search.
  - Units °C/°F.
- **Waktu Solat**
  - Next prayer name and countdown on the pill. The popup shows today's six times (Imsak optional, Subuh, Syuruk, Zohor, Asar, Maghrib, Isyak).
  - Alert at each prayer time, with an optional azan sound: a user-chosen audio file, nothing bundled.
- **IPU (Air Pollutant Index)** *(added at user request)*
  - The pill shows the current IPU number with its status color, from the DOE station nearest the user's location (Weather and Waktu Solat use the same location):
    - Good (0–50): blue
    - Moderate (51–100): green
    - Unhealthy (101–200): yellow
    - Very Unhealthy (201–300): orange
    - Hazardous (>300): red
  - The popup shows the station name and distance, the reading time, the dominant pollutant (e.g. PM2.5), and the three next-nearest stations.
  - A manual station picker in settings overrides auto.
  - **Source:** live readings from DOE Malaysia's public APIMS map service (`eqms.doe.gov.my/api3/publicmapproxy/PUBLIC_DISPLAY/CAQM_MCAQM_Current_Reading/MapServer/0/query`). It covers 68 stations, updated hourly, so "realtime" means the latest hourly reading. Refresh every 15 min (cheap: one small JSON request).
  - **Outside Malaysia,** or if DOE is unreachable: Open-Meteo Air Quality (`us_aqi`, no key), clearly labelled "AQI (US)" rather than IPU, because the two scales differ.
  - An alert fires once when the reading crosses into Unhealthy (>100) and again if it later reaches Very Unhealthy (>200). This can be switched off in settings.
- **System Monitor**
  - CPU %, RAM %, optional GPU % and network up/down, each with a mini sparkline (last 60 samples).
  - Sampling every 2 s, only while the widget is placed.
- **Screen Time**
  - Tracks foreground app time per day. It pauses after 5 min of no input and while the screen is locked.
  - The pill shows today's total. The popup shows the top apps today and a 7-day mini bar graph.
- **Notifications**
  - Bell with a badge count. The popup lists alert history (newest first), with dismiss / clear all and a DND toggle.
- **Jump lists**
  - Right-click on an app shows its recent files and the existing items.

## Constraints

- Follow the MVP architecture:
  - OS calls go behind `platform` traits.
  - Widgets are self-contained folders.
  - Only `ipc.ts` calls `invoke`.
  - Blocking calls use the blocking pool.
- **Timers and alert scheduling live in Rust.** One source of truth shared by the bar and the popup window. Webviews compute display time from timestamps; they never count seconds themselves.
- **Network fetches live in Rust** and are cached:
  - Weather: refresh at most every 30 min.
  - Prayer times: once per day per zone.
  - IPU: every 15 min (the data changes hourly).
  - All fetches work again after the network comes back.
- **Installer.** It stays well under the 15 MB target; the MVP is ~2 MB.
- **Network services.** All must be free and need no API key:
  - Weather: Open-Meteo forecast and geocoding.
  - Prayer times: JAKIM e-Solat `takwimsolat`, plus the waktusolat.app zone-by-coordinates lookup, plus Aladhan.
  - Location fallback: an IP lookup over HTTPS.

## Builder's calls (not decided by the user; can be changed)

- **Shared data.** Data is shared, not per widget instance. There is one to-do list, one set of notes, one Pomodoro and one set of timers. Two pills of the same widget show the same data; instance settings only change how it looks.
- **Background engines.** To-do due alerts, running timers and Pomodoro fire whether or not the widget is on the active profile, because the user started them. Prayer alerts run while a Waktu Solat widget is placed in any profile.
- **Location consent.**
  - Adding Weather or Waktu Solat turns on auto-location (the user chose "auto").
  - The popup shows "Approximate location from your IP" when the IP fallback is used.
  - The settings show the place and offer a manual city/zone that switches auto off.
- **Missed alerts and restarts.**
  - Timer and Pomodoro state (end timestamps) is stored in SQLite, so it survives a quit or reboot.
  - An alert missed while the app was closed or the PC asleep fires on the next launch or wake if it is less than 15 minutes late. Otherwise it is recorded in history as missed, with no toast or sound.
  - A "fired" marker in SQLite keeps every alert to one firing.
- **Updater installer.** The updater ships the NSIS installer (Tauri's recommended updater target). The MSI is still built for manual installs.
- **Note auto-delete.**
  - Temporary mode never deletes pinned notes.
  - Deleted notes (by the user or by expiry) go to a trash that keeps them 7 days, then they and their image files are removed.

- **Database.** `rusqlite` with the bundled SQLite, at `%APPDATA%\GlassBar\glassbar.db`. Migrations use `PRAGMA user_version`. Note images are saved as PNG files in `%APPDATA%\GlassBar\notes\`; the DB stores file names.
- **Location order.** Try Windows location (`Windows.Devices.Geolocation`) first. If it's off or denied, fall back to an IP lookup. The detected place name and coordinates are shown in the widget settings, and the user can turn auto-location off.
- **Toasts.** Use `tauri-plugin-notification`. Do Not Disturb is GlassBar's own flag; it does not change Windows Focus settings.
- **Sounds.** The Pomodoro/timer chime is synthesized with WebAudio in the bar window, so no audio asset is bundled.
- **Jump lists.** Windows has no public API to read another app's jump list. GlassBar shows recent files from `%APPDATA%\Microsoft\Windows\Recent` whose file type opens with that app. This is an approximation; pinned jump-list tasks are not shown.
- **GPU %.** Read from Windows performance counters (`GPU Engine` utilization). It is optional and off by default.
- **Image drag-out.** Uses the `drag` crate (native OLE drag source) started from a Tauri command.
- **Updater release flow.**
  - A GitHub Actions workflow builds and signs on tag push and uploads the installers plus `latest.json` (via `tauri-action`).
  - The private signing key goes in a GitHub secret. **The user does this once.**
  - The app checks for updates on startup and every 24 h, and offers "Install & restart" in Settings and the tray.

## Done looks like

- Every widget above can be added from the gallery, works at its sizes, and has its settings.
- Alerts fire exactly once, even with the popup closed. DND silences toasts and sounds but still records history.
- With no network:
  - Weather shows its last cached data and when it was fetched.
  - Waktu Solat uses cached times for today.
- The updater finds a newer test release when one is published.
- Idle CPU and the installer stay within the targets above.
- Pure logic has unit tests: Pomodoro phase machine, prayer "next prayer" selection, screen-time accounting with idle gaps, note expiry, DB migrations, and IPU nearest-station selection plus status bands.

## Known risk

The DOE map service is public but undocumented, so DOE could change it without notice. If it breaks, the IPU widget falls back to Open-Meteo AQI (labelled as such) until the endpoint is updated.
