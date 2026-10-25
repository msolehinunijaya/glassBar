# Changelog

## 0.2.3 — 2026-10-25

### New
- **Settings › Bar › Space from the edge:** set the gap between the dock and its screen edge (0–48 px), in any position.
- **App Launcher:** put Start & Search on the left or the right of your apps. Choose how many icons show; the rest collapse into a "+N" chip, and hovering the launcher shows them all. The app you're using always stays visible.
- **Waktu Solat:** adzan volume.
- The arrows that page through more widgets show only while the pointer is on the dock.
- Clock shows the weather by default.
- System Monitor: the Recycle Bin icon sits above its size.

### Fixed
- Screenshot and OCR Snap no longer flash a black screen before the overlay appears.
- After a restart the Windows taskbar stays hidden, and GlassBar starts with Windows from the installed app (it could point at an old copy, making the dock slow to appear).

## 0.2.2 — 2026-10-24

### New
- **AI Usage:** a "Claude plan (claude.ai)" source. Sign in to claude.ai once and see Current session and This week, like claude.ai › Settings › Usage. Works for people who use the Claude app rather than Claude Code.
- **Music (Classic):** Previous and Next buttons at every size, not only Large.

### Fixed
- Scrolling screenshots capture the whole page in browsers: they wait for smooth scrolling to finish, ignore side panels and vertical tabs that don't scroll, cope with moving pictures and videos, leave out the taskbar or a dock covering the window, and no longer show the scrollbar or corner marks at each join.

## 0.2.1 — 2026-10-24

### New
- Right-click the dock › **Show the desktop** (after Task Manager).
- New installs start with Windows, hide the Windows taskbar, and show tray icons on the dock. Change them in Settings › Startup, Settings › Bar and Quick Settings.

### Fixed
- Auto-hide: resting the pointer on the screen edge below the dock no longer makes it hide and reappear over and over.
- The dock stays above other apps: it comes to the front every time it slides in, and clicking the desktop no longer drops it behind windows.

## 0.2.0 — 2026-10-24

### New
- **Settings › Appearance:** themes (Light, Dark, Glass, System, Custom) and design styles inspired by macOS, Windows, GNOME, KDE Plasma and Cinnamon, with a live preview in all four dock positions. Choose the Windows material (Acrylic, Mica or classic blur) or turn blur off; set separate opacity for the dock, panels and widget cards; pick accent and background colours, contrast, corner radius, shadows, borders, icon size, spacing and animations. Save your own presets, revert single settings, or reset. Effects this Windows can't draw say so and fall back.
- **Liquid Glass theme:** Apple-style glass for the dock, flyouts and widgets: a soft sheen that follows the pointer, bright rims, white toggles when on, thin sliders, and a faint colour rim on open panels. The tint rises automatically where your wallpaper would make text hard to read.
- **Settings › Location:** one shared location for Weather, sunrise/sunset, Waktu Solat, IPU and the world clock. Choose Automatic, Windows Location, IP or Manual; see the source, accuracy, time zone and when it was last checked. Widgets show where their location comes from and link here.
- **Settings › Widgets:** a visual widget manager with a live preview of your dock, categories, search, on/off switches that keep a widget's settings, sizes, drag-to-reorder and layout presets.
- **Settings › About:** version and this changelog.
- **Left/right dock:** a side width option (Slim, Regular, Wide) and designs made for a side dock for System Monitor, Clock & Weather, IPU, Pomodoro / To-Do / Notes and Quick Settings (connectivity, volume, battery).
- Notification banners next to the dock that jump to their widget, per-widget banner switches and Do Not Disturb.
- Downloads widget; Recycle Bin in the System Monitor.
- Capture widget: screenshots (full screen, area, window, scrolling page), recording, OCR Snap and clipboard history, with icon styles.
- Pomodoro, To-Do and Notes in one Productivity widget.
- **Pomodoro redesign:** one colour per phase (focus, short break, long break) across the whole card; a ring that drains as time runs out; Focus/Short/Long switch; session dots that say what comes next; keyboard shortcuts (Space, R, S, 1–3); today's focus with a daily goal, a 7-day chart, your streak and longest session; optional auto-start, sound volume and dimming the rest of the dock while you focus. On the dock: a mini ring (S), with the time (M) and the session (L); hover for quick pause/resume.
- Quick Settings gathers volume, battery, keyboard language, Wi-Fi, Bluetooth, mobile hotspot and real tray icons.
- Weather joins the clock: forecast, feels-like, humidity, wind, rain chance and a sunrise/sunset arc.
- Clock formats (12/24 h, date presets, custom pattern, layouts) and world clocks with day/night.
- Waktu Solat with adzan at prayer time (short, full or normal alert) and a prayer pill.
- IPU air quality with a Malaysia state/station browser, other-country AQI and favourites.
- **AI Usage:** how much of your Claude and ChatGPT limits you have used. Connect an Anthropic or OpenAI Admin key (kept in Windows Credential Manager, checked on save), Claude Code (read from its local logs, no sign-in), your Claude plan limits (from Claude Code's login, no sign-in, unofficial) or your ChatGPT plan (sign-in window, unofficial). On the dock: rings (S), the most urgent number (M) or a row per provider with time to reset (L), coloured by how full. The panel shows the limit, session / today / week / month, token split, a 30-day chart, models, projects and a cost estimate in USD and RM. Banners at 80% and 95% and when a limit resets. Only limits a source publishes are shown.
- Currency converter, Ask Claude (quick questions), Time Progress, Screen Time with a mini graph, Timer & Stopwatch.
- System Monitor: CPU, RAM, network, disk and CPU temperature.
- App launcher: pinning, drag-reorder, live window previews, recent-file jump lists, Start & Search buttons.
- Music player with album art and a live level visualizer.
- Left and right dock layouts, auto-hide, reserved screen space, and a right-click menu for position.
- Charger and USB device sounds.
- Automatic updates from GitHub releases.

### Fixed
- The dock, panels and notification banners no longer show a black background behind their glass; Acrylic and Blur keep blurring while the dock isn't the active window.
- The bar keeps its size and position after restarts, DPI changes and auto-hide changes.
- Panels fit their content and follow the light theme when Windows transparency effects are off.
- Store apps launch with their real icons and survive app updates.
- Tray icons appear reliably after start-up.
- Battery reacts instantly to plugging in and unplugging.
- Menus, hover previews and drag-to-reorder work every time.

## 0.1.0 — 2026-10-22

### New
- First version: a glass bar that replaces the Windows taskbar, with placement on any edge, a settings window, a popup panel per widget and five widgets (Start, Apps, Music, Quick Settings, Clock).
