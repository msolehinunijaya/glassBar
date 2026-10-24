# GlassBar Phase 2: Productivity Widgets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add SQLite storage, an alert center, and 11 widgets (To-Do, Quick Note, Pomodoro, Timer/Stopwatch, Battery, Weather, Waktu Solat, IPU, System Monitor, Screen Time, Notifications), plus app jump lists and auto-update.

**Architecture:**
- All state and timing live in Rust:
  - One SQLite DB for state.
  - One scheduler thread that sleeps until the next due item.
  - Network fetchers with caches in a `kv` table.
- Webviews only render. They compute countdowns from timestamps Rust gives them.
- New OS calls go behind `platform` traits: `Power`, `Location`, `Gpu`, `Recent`. Widgets follow `docs/WIDGET_SDK.md`.

**Tech Stack:**
- **Already in use:** Tauri 2, windows-rs 0.62, Svelte 5.
- **New crates:** `rusqlite` (bundled), `reqwest` (rustls, json, blocking), `sysinfo`, `drag`, `tauri-plugin-notification`, `tauri-plugin-updater`, `tauri-plugin-process`.

**Spec:** `docs/superpowers/specs/2026-10-23-phase2-design.md`

## Global Constraints

- **Idle CPU:** no per-second work while nothing is visible or running. Idle CPU stays ~0.1% of all cores.
- **Data safety:** to-dos and notes are never lost. Writes are transactional, and the DB is never recreated over an existing file.
- **Alerts:** fire exactly once, even with the popup closed. Every alert is recorded in history, including missed ones and ones silenced by DND.
- **Missed alerts:** if an alert is less than 15 min late, it fires on launch or wake. Otherwise it goes to history as missed, with no toast or sound.
- **Network:** fetches run in Rust with caching:
  - weather at most every 30 min;
  - prayer times once per day per zone;
  - IPU every 15 min.
- **Free services only, with no API keys:**
  - Open-Meteo (forecast, geocoding, air quality)
  - JAKIM e-Solat
  - waktusolat.app
  - Aladhan
  - DOE APIMS map service
  - ipapi.co
- **Installer:** well under 15 MB.
- **Screen Time:** records only while a `screentime` widget is placed in any profile. History is kept 30 days.
- **Notes:**
  - Pasted images show as a thumbnail strip under the text.
  - Temporary mode never deletes pinned notes.
  - Deleted notes stay in trash 7 days.
- **Shared data:** one to-do list, one set of notes, one Pomodoro, one set of timers.
- **IPU:**
  - Status bands are Good 0–50 (blue), Moderate 51–100 (green), Unhealthy 101–200 (yellow), Very Unhealthy 201–300 (orange), Hazardous >300 (red).
  - Alert once on crossing >100 and once on crossing >200.
  - The fallback reading is labelled "AQI (US)".
- **Updater:**
  - Uses the NSIS artifact.
  - Endpoint is `https://github.com/msolehinunijaya/glassBar/releases/latest/download/latest.json`.
  - Checks on startup and every 24 h.

## Review Focus

1. **App closed or PC asleep when a to-do, timer or prayer comes due:** it fires once on wake if under 15 min late; otherwise it shows as missed in history. *(Task 2 tests `due_actions` with late items.)*
2. **No network at startup:** Weather and Waktu Solat show their cached data with the fetch time, and IPU shows the last reading. Nothing panics or spins. *(Tasks 5–7: every fetcher returns the cache on error; tests use `parse_*` on saved fixtures.)*
3. **No location (Windows location off, IP lookup fails):** the widgets ask for a manual city or zone instead of erroring. *(Task 5 tests `resolve_location` order.)*
4. **The DB file exists from an older version, or is locked by a second process:** migrations run forward only, and the DB never truncates. *(Task 1 tests that the migration is idempotent and that existing rows survive.)*
5. **Pasted content is huge (a 20 MB image) or not an image:** the note rejects it with a message. Images larger than 10 MB are refused. *(Task 3 tests `note_add_image` size limit.)*

---

## File Structure

**Rust (`src-tauri/src/`):**

| File | Role |
|---|---|
| `db.rs` | open, migrations, `Db` handle, `kv_get`/`kv_set` |
| `alerts.rs` | `notify()`, alert history commands, DND, toast, sound event |
| `scheduler.rs` | due-item model (pure `due_actions`), the sleeper thread, `poke()` |
| `widgets/todo.rs`, `widgets/notes.rs` | CRUD commands, note image save/drag/expiry |
| `widgets/focus.rs` | Pomodoro state machine (pure) + timers + stopwatch commands |
| `net.rs` | shared blocking reqwest client with timeouts |
| `widgets/location.rs` | resolve + cache location (Windows → IP → manual) |
| `widgets/weather.rs`, `widgets/solat.rs`, `widgets/ipu.rs` | fetch, parse, cache |
| `widgets/sysmon.rs`, `widgets/screentime.rs`, `widgets/battery.rs` | stats and tracking |
| `widgets/jumplist.rs` | recent files per app |
| `platform/windows/{power,location,gpu,recent}.rs` | OS calls behind new traits in `platform/mod.rs` |

**Frontend:**
- One folder per widget in `src/widgets/`: `todo`, `notes`, `pomodoro`, `timer`, `battery`, `weather`, `solat`, `ipu`, `sysmon`, `screentime`, `notifications`.
- `src/lib/Sparkline.svelte` and `src/lib/sound.ts` (WebAudio chime plus optional azan file).
- `ipc.ts` gains typed wrappers for every command below.

**Config:**
- `Config` gains `dnd: bool` (default false) and `location: LocationCfg { auto: bool (default true), lat: Option<f64>, lon: Option<f64>, name: String }`.
- `Config::uses(&self, widget_id: &str) -> bool` returns whether any profile places it.
- `Config::widget_settings(&self, widget_id: &str) -> serde_json::Value` returns the first instance's settings, active profile first, `Null` if none.

---

### Task 1: SQLite storage + alert center + Notifications widget

**Files:**
- Create: `src-tauri/src/db.rs`, `src-tauri/src/alerts.rs`, `src/widgets/notifications/{index.ts,Pill.svelte,Popup.svelte}`, `src/lib/sound.ts`
- Modify: `Cargo.toml` (rusqlite `bundled`, tauri-plugin-notification), `main.rs` (manage `Db`, register commands, plugin), `config.rs` (`dnd`, `location`, `uses`, `widget_settings`), `capabilities/default.json` (`notification:default`), `src/lib/ipc.ts`, `src/app/Bar.svelte` (listen `sound:play`)

**Interfaces:**
- **DB handle:**
  - `pub struct Db(pub Mutex<rusqlite::Connection>)`
  - `db::open(path: &Path) -> rusqlite::Result<Db>`, which runs `migrate`
  - `db::migrate(c: &Connection) -> rusqlite::Result<()>`
  - `db::kv_get(c, key) -> Option<String>` and `db::kv_set(c, key, value)`
- **Schema v1:**
  - `todos(id INTEGER PRIMARY KEY, text TEXT NOT NULL, done INT NOT NULL DEFAULT 0, due_at INT, created_at INT NOT NULL)`
  - `notes(id INTEGER PRIMARY KEY, text TEXT NOT NULL DEFAULT '', color TEXT NOT NULL DEFAULT 'yellow', pinned INT NOT NULL DEFAULT 0, images TEXT NOT NULL DEFAULT '[]', updated_at INT NOT NULL, deleted_at INT)`
  - `alerts(id INTEGER PRIMARY KEY, source TEXT, title TEXT, body TEXT, at INT, missed INT, read INT DEFAULT 0)`
  - `timers(id INTEGER PRIMARY KEY, label TEXT, duration_ms INT, ends_at INT, remaining_ms INT)`
  - `screen_time(day TEXT, exe TEXT, seconds INT, PRIMARY KEY(day, exe))`
  - `kv(key TEXT PRIMARY KEY, value TEXT)`
  - `fired(key TEXT PRIMARY KEY, at INT)`
  - All timestamps are Unix ms.
- **Alerts:**
  - `alerts::notify(app: &AppHandle, source: &str, title: &str, body: &str, missed: bool)` inserts a row and emits `alerts:changed`. Unless `missed` or `dnd`, it shows a toast and emits `sound:play` with payload `{ kind: "chime" | "azan" }`.
  - Commands: `alerts_list() -> Vec<Alert{id,source,title,body,at,missed,read}>` (newest first, max 200), `alerts_mark_read()`, `alerts_dismiss(id)`, `alerts_clear()`.
  - DND is `config.dnd`, set through the existing `save_config`.

- [ ] **Step 1: Write failing tests in `db.rs`:**
  - `migrate_is_idempotent_and_keeps_rows`: migrate, insert a todo, migrate again, assert 1 row and `user_version == 1`.
  - `kv_round_trip`.
  - `open_does_not_truncate_existing_file`: write a todo, reopen, row still there.
- [ ] **Step 2:** Run `cargo test db::` and expect it to fail to compile (no `db` module).
- [ ] **Step 3:** Implement `db.rs`. Migrations are a `&[&str]` list indexed by `user_version`, each run in a transaction.
- [ ] **Step 4:** Run `cargo test db::`. Expect 3 pass.
- [ ] **Step 5:** Write failing tests in `config.rs`:
  - `uses_finds_widget_in_any_profile`
  - `widget_settings_prefers_active_profile`
- [ ] **Step 6:** Run them and expect them to fail. Implement both, run them, and expect a pass.
- [ ] **Step 7:** Implement `alerts.rs` and the commands. Register the notification plugin. `sound.ts` exports `chime()` (two-tone WebAudio, about 0.6 s) and `playFile(dataUrl)`. The bar window listens to `sound:play`.
- [ ] **Step 8:** Build the Notifications widget:
  - **Pill:** bell icon with an unread badge.
  - **Popup:** the list (relative time, a "missed" tag), dismiss ×, Clear all, a DND switch, and mark-read on open.
- [ ] **Step 9:** Run `cargo test`, `npx vitest run` and `npm run check`. Expect all green with 0 errors.
- [ ] **Step 10:** Commit: `feat: SQLite storage, alert center, notifications widget`.

### Task 2: Scheduler + To-Do

**Files:**
- Create: `src-tauri/src/scheduler.rs`, `src-tauri/src/widgets/todo.rs`, `src/widgets/todo/{index.ts,Pill.svelte,Popup.svelte}`
- Modify: `main.rs`, `widgets/mod.rs`, `ipc.ts`

**Interfaces:**
- **Due items:** `pub struct Due { pub key: String, pub at: i64, pub source: &'static str, pub title: String, pub body: String }`
- **Pure decision:** `pub fn due_actions(now: i64, items: &[Due], fired: &HashSet<String>) -> (Vec<(Due, bool /*missed*/)>, Option<i64> /*next wake*/)`. Items with `at <= now` and not fired fire, with `missed = now - at > 15*60*1000`. The next wake is the earliest future `at`.
- **Item sources:** `scheduler::collect(app) -> Vec<Due>` gathers to-dos (`todo:{id}`, the due ones that aren't done), timers (`timer:{id}:{ends_at}`), Pomodoro (`pomo:{ends_at}`), prayers (`solat:{date}:{name}`, only if `uses("solat")`), and daily housekeeping (`house:{date}`).
- **Waking:** `scheduler::start(app)` runs a thread that sleeps until `min(next wake, now+60s)`, or until `scheduler::poke()` (a `Condvar`) wakes it. Waking from sleep is caught by the 60 s cap.
- **Firing:** each fire calls `alerts::notify` and inserts into `fired`. Housekeeping runs note expiry and screen-time retention.
- **To-Do commands:**
  - `todo_list() -> Vec<Todo{id,text,done,due_at:Option<i64>,created_at}>`
  - `todo_add(text, due_at)`
  - `todo_update(id, text, done, due_at)`
  - `todo_delete(id)`
  - `todo_clear_done()`
  - Every write emits `todos:changed` and calls `poke()`.

- [ ] **Step 1: Write failing tests in `scheduler.rs`:**
  - `fires_due_items_once`: `fired` contains the key, so nothing fires.
  - `late_over_15_min_is_missed`: an item at `now-16min` gives `missed == true`; one at `now-14min` gives `false`.
  - `next_wake_is_earliest_future`.
  - `empty_means_no_wake`.
- [ ] **Step 2:** Run them and expect failure. Implement `due_actions`, run again, and expect a pass.
- [ ] **Step 3:** Implement `collect`, `start` and `poke`, and the to-do commands. Start the scheduler in setup.
- [ ] **Step 4:** Build the To-Do widget:
  - **Pill:** check icon plus the open count. Medium and large also show the next due item.
  - **Popup:** quick add with Enter, an optional `datetime-local` due date, a checkbox list (overdue in red), delete, and "Clear completed".
  - Size: 340×460.
- [ ] **Step 5:** Run all tests and the check, then commit `feat: scheduler and to-do widget`.

### Task 3: Quick Note

**Files:**
- Create: `src-tauri/src/widgets/notes.rs`, `src/widgets/notes/{index.ts,Pill.svelte,Popup.svelte,SettingsForm.svelte}`
- Modify: `Cargo.toml` (`drag`), `main.rs`, `ipc.ts`

**Interfaces:**
- **Note type:** `Note{id,text,color,pinned,images:Vec<String>,updated_at,deleted_at:Option<i64>}`
- **Commands:**
  - `notes_list(trash: bool)`: pinned first, then `updated_at` descending.
  - `note_create() -> Note`
  - `note_update(id, text, color, pinned)`
  - `note_delete(id)`: soft delete, sets `deleted_at`.
  - `note_restore(id)`
  - `note_add_image(id, png_base64) -> String`: returns the file name. Errors when the decoded size is over 10 MB or the data is not a PNG/JPEG signature.
  - `note_image(name) -> String`: a data URL.
  - `note_drag_image(window, name)`: native OLE drag of the file, via the `drag` crate.
- **Expiry (pure):** `expired(now, updated_at, pinned, days: Option<u32>) -> bool` and `purge_due(now, deleted_at) -> bool`, which is true after 7 days.
- **Housekeeping:** `notes::housekeeping(app)` soft-deletes expired notes, then hard-deletes purged notes and their image files. It reads `days` from `widget_settings("notes").temporaryDays`.
- **Storage:** images go in `%APPDATA%\GlassBar\notes\{uuid-ish}.png`. Names are generated in Rust; `note_image` and `note_drag_image` reject names containing path separators.

- [ ] **Step 1: Write failing tests:**
  - `pinned_never_expires`
  - `expires_after_days`
  - `off_means_never`
  - `purge_after_7_days`
  - `rejects_oversize_image`
  - `rejects_non_image`
  - `rejects_path_traversal_name`
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the widget:
  - **Pill:** note icon plus the count.
  - **Popup** (360×480):
    - a note list as color chips;
    - the selected note's textarea, auto-saved with a 500 ms debounce;
    - an image strip, filled by Ctrl+V of an image. The webview reads clipboard image items as a blob, then base64, then `note_add_image`. Dragging a thumbnail calls `note_drag_image`;
    - controls for color, pin, delete, new note, and a trash view with restore.
  - **SettingsForm:** a "Temporary notes" day count (0 means off).
- [ ] **Step 4:** Run all tests, then commit `feat: quick note widget`.

### Task 4: Pomodoro + Timer/Stopwatch

**Files:**
- Create: `src-tauri/src/widgets/focus.rs`, `src/widgets/pomodoro/*`, `src/widgets/timer/*`
- Modify: `main.rs`, `ipc.ts`

**Interfaces:**
- **Pomodoro state:**
  - `Pomo{phase: Phase(Work|Short|Long), done: u32 /*work sessions completed in set*/, ends_at: Option<i64>, paused_ms: Option<i64>}`
  - `PomoCfg{work_min:25, short_min:5, long_min:15, every:4, auto_start:true}`, read from `widget_settings("pomodoro")`.
- **Pure transitions:** `start(now)`, `pause(now)`, `reset()`, `skip(now, cfg)`, and `finish(now, cfg) -> Pomo`.
  - Work moves to Short, or to Long when `done+1 == every`. Long resets `done` to 0.
  - With `auto_start`, the next phase starts immediately.
- **Storage and events:** state is stored in `kv["pomodoro"]`. The `pomodoro:changed` event carries `{pomo, cfg}`.
- **Pomodoro commands:** `pomo_get`, `pomo_start`, `pomo_pause`, `pomo_skip`, `pomo_reset`.
- **Firing:** the scheduler fires `pomo:{ends_at}`, then calls `focus::on_pomo_due` to apply `finish`.
- **Timers:**
  - `timers_list`
  - `timer_add(label, duration_ms)`: starts immediately.
  - `timer_pause(id)` and `timer_resume(id)`, using `remaining_ms`.
  - `timer_delete(id)`
  - Event `timers:changed`.
- **Stopwatch:**
  - State: `kv["stopwatch"] = {started_at:Option<i64>, acc_ms:i64, laps:Vec<i64>}`.
  - Commands: `sw_get`, `sw_toggle`, `sw_lap`, `sw_reset`.
  - Event `stopwatch:changed`.

- [ ] **Step 1: Write failing tests:**
  - `work_to_short_then_long_on_fourth`: assert phases over 8 finishes and `done` counts 1, 2, 3, then 0 after the long break.
  - `pause_keeps_remaining`
  - `skip_advances_without_counting_work`: skipping work does not increment `done`.
  - `counter_label`: `label(&pomo, &cfg) == "2/4"` during the third work session.
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the Pomodoro widget:
  - **Pill:** a phase-colored ring with mm:ss and "2/4". It ticks once a second only while running and visible.
  - **Popup** (320×360): a big ring, start/pause/skip/reset, and a phase label.
  - **SettingsForm:** durations, how often the long break comes, and auto-start.
- [ ] **Step 4:** Build the Timer widget:
  - **Pill:** the nearest running timer's countdown, or the stopwatch when one is running, else an icon.
  - **Popup** (340×440) has two tabs:
    - Timers: presets 1/5/10/15/30 min plus a custom `mm:ss` field, and a running list with pause and delete.
    - Stopwatch: start/stop, lap, reset, and the lap list.
- [ ] **Step 5:** Run all tests, then commit `feat: pomodoro and timer widgets`.

### Task 5: Location + Weather

**Files:**
- Create: `src-tauri/src/net.rs`, `src-tauri/src/widgets/location.rs`, `src-tauri/src/widgets/weather.rs`, `src-tauri/src/platform/windows/location.rs`, `src-tauri/tests/fixtures/` (saved JSON responses), `src/widgets/weather/*`
- Modify: `Cargo.toml` (reqwest blocking + rustls + json, features `Devices_Geolocation`), `platform/mod.rs` (`trait Location { fn locate(&self) -> Res<(f64,f64)> }`), `ipc.ts`

**Interfaces:**
- **HTTP client:** `net::get_json(url: &str) -> Result<serde_json::Value, String>` with a 10 s timeout and the user agent `GlassBar/<version>`.
- **Location:**
  - `Place{lat,lon,name,source: "windows"|"ip"|"manual"}`.
  - Pure `resolve_location(cfg: &LocationCfg, windows: Option<(f64,f64)>, ip: Option<Place>) -> Option<Place>`. Manual wins when `!auto`; then Windows; then IP.
  - `location::current(app) -> Option<Place>` caches in `kv["place"]` for 6 h. Windows coordinates are named by reverse lookup from the IP response city, or "Current location".
- **City search:** `location_search(query) -> Vec<Place>`, using Open-Meteo geocoding, count 5.
- **Weather:**
  - `Weather{fetched_at, current{temp, code, is_day}, hourly: Vec<{time, temp, code}> (12), daily: Vec<{date, min, max, code}> (3), unit}`
  - Pure `parse_weather(v: &Value, unit) -> Option<Weather>`.
  - `weather_get(force: bool) -> Option<Weather>` returns the cache when it is under 30 min old, or when the fetch fails.
  - Units come from `widget_settings("weather").unit` (`"c"|"f"`).
- **Weather codes:** WMO code to label/icon mapping in `src/widgets/weather/codes.ts`, as a pure function with a vitest test.

- [ ] **Step 1:** Save fixture `open-meteo-forecast.json` (a real response captured with curl). Write failing tests:
  - `parse_weather_reads_current_hourly_daily`
  - `resolve_prefers_manual_when_auto_off`
  - `resolve_falls_back_to_ip_then_none`
  - plus vitest `codes.test.ts` (0 → Clear, 61 → Rain, 95 → Thunderstorm).
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the Weather widget:
  - **Pill:** icon plus temperature.
  - **Popup** (340×420): place, "updated hh:mm" (with "offline, cached" when the fetch failed), current conditions, a scrolling 12-hour row, and 3 day rows. Show "Approximate location from your IP" when `source=="ip"`.
  - **SettingsForm:** a unit choice, an auto-location switch, and city search that sets the manual location in `config.location`.
- [ ] **Step 4:** Run all tests, then commit `feat: location and weather widget`.

### Task 6: Waktu Solat

**Files:**
- Create: `src-tauri/src/widgets/solat.rs`, fixtures `jakim-takwim.json` and `aladhan.json`, `src/widgets/solat/*`
- Modify: `scheduler.rs` (prayer `Due` items), `ipc.ts`

**Interfaces:**
- **Times:**
  - `PrayerDay{date: String /*YYYY-MM-DD*/, zone: String, source:"jakim"|"aladhan", times: Vec<(String /*Imsak|Subuh|Syuruk|Zohor|Asar|Maghrib|Isyak*/, i64)>}`
  - Pure `parse_jakim(v) -> Option<PrayerDay>`, using `prayerTime[0]` fields imsak, fajr, syuruk, dhuhr, asr, maghrib, isha (local HH:MM:SS, Asia/Kuala_Lumpur).
  - Pure `parse_aladhan(v) -> Option<PrayerDay>`.
- **Next prayer:** pure `next_prayer(day: &PrayerDay, now: i64, include_imsak: bool) -> Option<(String, i64)>`. It skips Syuruk and Imsak unless asked. After Isyak it returns `None`, and the caller then fetches tomorrow.
- **Zone:**
  - `zone_for(app) -> Option<String>` uses `widget_settings("solat").zone` when set; otherwise the waktusolat.app lookup `https://api.waktusolat.app/zones/{lat}/{lon}`, cached per place in `kv["solat:zone"]`.
  - When there is no zone (outside Malaysia), use Aladhan `https://api.aladhan.com/v1/timings/{DD-MM-YYYY}?latitude=..&longitude=..&method=3`.
  - JAKIM URL: `https://www.e-solat.gov.my/index.php?r=esolatApi/takwimsolat&period=today&zone={ZONE}`.
- **Commands and caching:**
  - `solat_today() -> Option<PrayerDay>` is cached in `kv["solat:{date}:{zone}"]`, fetched once a day.
  - `solat_zones() -> Vec<(code, name)>` is a static list of JAKIM zones, embedded as a JSON file `src-tauri/src/widgets/jakim_zones.json`.
- **Alerts:** the scheduler adds a `Due` for each Subuh/Zohor/Asar/Maghrib/Isyak time. The alert sound is `azan` when `widget_settings("solat").azanFile` is set; otherwise `chime`.

- [ ] **Step 1:** Capture fixtures. Write failing tests:
  - `parse_jakim_reads_all_times`: with the WLY01 fixture, Zohor equals the fixture value.
  - `next_prayer_skips_syuruk`
  - `after_isyak_is_none`
  - `parse_aladhan_reads_times`
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the widget:
  - **Pill:** next prayer name and countdown, refreshed every minute; every second in the last 10 minutes.
  - **Popup** (320×400): zone name, today's list with the next prayer highlighted, and the Hijri date when JAKIM gives it.
  - **SettingsForm:** zone picker (Auto + the JAKIM list), a show-Imsak switch, an alerts switch, and an azan file path (a text input plus a "Test" button).
- [ ] **Step 4:** Run all tests, then commit `feat: waktu solat widget`.

### Task 7: IPU

**Files:**
- Create: `src-tauri/src/widgets/ipu.rs`, fixtures `doe-apims.json` and `open-meteo-aq.json`, `src/widgets/ipu/*`
- Modify: `ipc.ts`

**Interfaces:**
- **Readings:**
  - `Station{id,name,state,lat,lon,api:Option<f64>,class:String,pollutant:String,at:i64}`
  - `IpuReading{kind:"ipu"|"aqi_us", value, band: Band, station: Option<Station>, distance_km: Option<f64>, nearby: Vec<(Station,f64)> /*3*/, fetched_at}`
- **Pure helpers:**
  - `band(v: f64) -> Band` uses the bands from Global Constraints. `Band` serializes as `good|moderate|unhealthy|very_unhealthy|hazardous`.
  - `parse_doe(v) -> Vec<Station>`, fed by the DOE query with `outFields=STATION_ID,STATION_LOCATION,STATE_NAME,API,CLASS,PARAM_SELECTED,DATETIME,LATITUDE,LONGITUDE&returnGeometry=false&f=json&where=1%3D1`.
  - `nearest(stations, lat, lon) -> Vec<(Station, f64)>`: haversine, ascending, skipping stations with no API value.
- **Command:** `ipu_get(force) -> Option<IpuReading>` is cached 15 min.
  - It uses the station from `widget_settings("ipu").station` if set, else the nearest.
  - If the nearest station is more than 150 km away, or DOE fails, it falls back to Open-Meteo `us_aqi` with `kind="aqi_us"`.
- **Crossing alert:** pure `crossing(prev: Option<Band>, now: Band) -> Option<Band>`. It returns `Unhealthy` when entering >100 and `VeryUnhealthy` when entering >200. `kv["ipu:band"]` holds the previous band. The alert is skipped when `widget_settings("ipu").alerts == false`.

- [ ] **Step 1:** Capture fixtures. Write failing tests:
  - `band_edges`: 50 → good, 51 → moderate, 100 → moderate, 101 → unhealthy, 201 → very_unhealthy, 301 → hazardous.
  - `nearest_orders_by_distance`: a KL point gives a KL-area station first.
  - `nearest_skips_null_api`
  - `crossing_fires_on_entry_only`: moderate to unhealthy gives Some; unhealthy to unhealthy gives None; unhealthy to very_unhealthy gives Some; very_unhealthy to moderate gives None.
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the widget:
  - **Pill:** a band-colored dot plus the value, with the label "IPU" or "AQI".
  - **Popup** (320×380): a big value, band name, station, distance, reading time, the pollutant, 3 nearby stations, and the source line "Data: JAS/DOE APIMS" or "Open-Meteo (US AQI)".
  - **SettingsForm:** station picker (Auto + the list from the last fetch) and an alerts switch.
  - The pill refreshes every 15 min while it is placed.
- [ ] **Step 4:** Run all tests, then commit `feat: IPU air quality widget`.

### Task 8: Battery + System Monitor

**Files:**
- Create: `src-tauri/src/widgets/battery.rs`, `src-tauri/src/widgets/sysmon.rs`, `src-tauri/src/platform/windows/{power,gpu}.rs`, `src/lib/Sparkline.svelte`, `src/widgets/battery/*`, `src/widgets/sysmon/*`
- Modify: `Cargo.toml` (`sysinfo`, features `Win32_System_Power`, `Win32_System_Performance`), `platform/mod.rs` (`trait Power { fn battery(&self) -> Option<Battery> }` and `trait Gpu { fn usage(&self) -> Option<f32> }`), `ipc.ts`

**Interfaces:**
- **Battery:**
  - `Battery{percent:u8, charging:bool, on_ac:bool, secs_left:Option<u32>}`. `None` means there is no battery.
  - `battery_get()`, plus a watcher that polls every 30 s and emits `battery:changed` on change.
- **System monitor:**
  - `Sample{cpu:f32, mem:f32, gpu:Option<f32>, rx_bps:u64, tx_bps:u64}`.
  - `sysmon::start(app)` runs a thread that samples every 2 s and emits `sysmon:sample`, only while `config.uses("sysmon")`. Otherwise it parks, checking every 5 s.
  - GPU is sampled only when `widget_settings("sysmon").gpu == true`.
- **Sparkline:** `<Sparkline values={number[]} max={100} />`, an SVG polyline of the last 60 values.

- [ ] **Step 1:** Write a failing vitest `sparkline.test.ts` for the pure helper `points(values, w, h, max) -> string`: an empty array gives `''`, and `[0,100]` with w=10, h=10 gives `"0,10 10,0"`. Run it, implement, and expect a pass.
- [ ] **Step 2:** Implement power, gpu (PDH `\GPU Engine(*engtype_3D)\Utilization Percentage` summed and capped at 100) and sysmon.
- [ ] **Step 3:** Build the Battery widget:
  - **Pill:** battery icon plus %, a lightning bolt while charging, and "AC power" when there is no battery.
  - **Popup** (300×200): percentage, status, and time left.
- [ ] **Step 4:** Build the System Monitor widget:
  - **Pill:** CPU and RAM % with small sparklines; large size adds network.
  - **Popup** (340×360): four sparkline cards.
  - **SettingsForm:** GPU and Network switches.
- [ ] **Step 5:** Run all tests and the check, then commit `feat: battery and system monitor widgets`.

### Task 9: Screen Time

**Files:**
- Create: `src-tauri/src/widgets/screentime.rs`, `src/widgets/screentime/*`
- Modify: `platform/mod.rs` (`Shell::idle_ms(&self) -> u64` via `GetLastInputInfo`, `Shell::foreground_exe(&self) -> Option<String>`), `ipc.ts`

**Interfaces:**
- **Crediting (pure):** `credit(gap_ms: i64, idle_ms: u64, exe: Option<&str>, interval_ms: i64) -> i64 /*seconds*/`. It credits `min(gap, 2*interval)` seconds, and 0 when `idle_ms >= 300_000`, when `exe` is None, or when `exe` is `LockApp.exe`.
- **Tracker:** `screentime::start(app)` runs a thread that samples every 5 s while `uses("screentime")`. It accumulates in memory and upserts into `screen_time` once a minute and on exit.
- **Commands:**
  - `screentime_today() -> Vec<(exe, name, seconds)>`: top 8, plus the total.
  - `screentime_week() -> Vec<(day, seconds)>`: 7 days.
- **Retention:** housekeeping deletes rows older than 30 days.

- [ ] **Step 1:** Write failing tests:
  - `idle_gives_zero`
  - `lockapp_gives_zero`
  - `sleep_gap_is_capped`: a 3-hour gap credits 10 s.
  - `normal_gap_credits_gap`
- [ ] **Step 2:** Run them and expect failure. Implement, run again, and expect a pass.
- [ ] **Step 3:** Build the widget:
  - **Pill:** an hourglass plus today's total (`2h 14m`).
  - **Popup** (340×420): the total, top apps with icons (`app_icon`) and bars, and a 7-day bar graph.
- [ ] **Step 4:** Run all tests, then commit `feat: screen time widget`.

### Task 10: Jump lists

**Files:**
- Create: `src-tauri/src/widgets/jumplist.rs`, `src-tauri/src/platform/windows/recent.rs`
- Modify: `platform/mod.rs` (`Shell::recent_for(&self, exe: &str, limit: usize) -> Vec<RecentItem{name, path}>`), `src/widgets/apps/Pill.svelte` (menu section "Recent"), `ipc.ts`

**Interfaces:**
- **Command:** `jumplist(exe: String) -> Vec<RecentItem>`, max 8. It scans `%APPDATA%\Microsoft\Windows\Recent\*.lnk`, newest first, up to 300 files. It resolves each target with `IShellLinkW` and keeps targets whose extension's `AssocQueryStringW(ASSOCSTR_EXECUTABLE)` matches `exe` case-insensitively. Folders and missing targets are skipped. Associations are cached per extension.
- **Menu:** clicking an item calls `app_launch(path, "")`.

- [ ] **Step 1:** Write a failing test for pure `matches_exe(assoc_exe: &str, exe: &str) -> bool`: case-insensitive, compares file names when a full path is given on either side. Run it, implement, and expect a pass.
- [ ] **Step 2:** Implement `recent.rs` and the menu section. Check by hand: right-click Notepad shows recent .txt files.
- [ ] **Step 3:** Commit `feat: recent-file jump lists in app launcher`.

### Task 11: Auto-update + release workflow

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `Cargo.toml` (`tauri-plugin-updater`, `tauri-plugin-process`), `tauri.conf.json` (`plugins.updater.pubkey`, `endpoints`, `bundle.createUpdaterArtifacts: true`), `capabilities/default.json` (`updater:default`, `process:allow-restart`), `package.json` (`@tauri-apps/plugin-updater`, `@tauri-apps/plugin-process`), `src/app/Settings.svelte` (Updates section), `bar.rs` (tray "Check for updates…"), `README.md` (release steps)

**Interfaces:**
- **Signing key:** `npx tauri signer generate -w %USERPROFILE%\.tauri\glassbar.key`, with no password. The public key goes into `tauri.conf.json`. The private key never enters the repo.
- **Settings section:** shows the current version, a "Check now" button, and "Install & restart" when an update is available. A background check runs 10 s after startup and every 24 h; when it finds an update it fires an `alerts::notify` with source `update`, once per version via `fired` `update:{version}`.
- **Workflow:** `release.yml` runs on tag `v*`. It uses `tauri-apps/tauri-action@v0` on `windows-latest` with the secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (empty), plus `tagName`, `releaseName` and `includeUpdaterJson: true`.

- [ ] **Step 1:** Generate the key and wire the config. `npm run tauri build` must produce `*-setup.exe.sig`; check that the file exists.
- [ ] **Step 2:** Add the Settings section, tray item and background check.
- [ ] **Step 3:** Write the README "Releasing" section: add the secret once, bump the version, push tag `v0.2.0`.
- [ ] **Step 4:** Commit `feat: auto-update via GitHub releases`.

### Task 12: Docs, verification, review

- [ ] **Step 1:** Update `docs/ARCHITECTURE.md` (DB, scheduler, new traits) and the README widget list.
- [ ] **Step 2:** Run `cargo test`, `npx vitest run` and `npm run check`, and expect all green. Run `npm run tauri build` and check the installer stays under 15 MB.
- [ ] **Step 3:** Run the app. Add every new widget from the gallery, open each popup, and confirm a 1-minute timer alerts exactly once with the popup closed. Measure idle CPU over 10 s.
- [ ] **Step 4:** Run the final whole-branch review (fresh reviewer, most capable model). Fix Critical and Important findings test-first.
