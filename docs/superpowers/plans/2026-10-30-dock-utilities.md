# Dock Utilities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Recycle Bin widget, Keep Awake / Microphone / Bluetooth battery to Quick Settings, a Countdowns widget, a colour picker in Capture, Hijri date + Tasbih in Waktu Solat, and a calculator search panel behind Start & Search.

**Architecture:** Frontend widgets in `src/widgets/<id>/` (Svelte 5), OS access through `crate::platform` traits with Windows code in `src-tauri/src/platform/windows/`, commands in `src-tauri/src/widgets/<id>.rs`, typed wrappers in `src/lib/ipc.ts`. Pure logic (parsers, date maths, formatting) lives in `.ts` files with `*.test.ts` beside them, and in Rust `#[cfg(test)]` modules.

**Tech Stack:** Tauri 2, Rust (windows-rs 0.62), Svelte 5 runes, TypeScript, Vitest, rusqlite.

**Spec:** `docs/superpowers/specs/2026-10-30-dock-utilities-design.md`

## Global Constraints

- Follow `docs/DESIGN_RULES.md`: tokens from `src/styles/glass.css`, 40 px rows, one primary button per view, Lucide icons, `font-variant-numeric: tabular-nums` on every number, hover/pressed/focus-visible/disabled states, light and dark.
- Widgets never call `invoke` directly: only through `ipc` / `events` in `src/lib/ipc.ts`.
- Blocking OS calls run through `crate::widgets::blocking`. Errors are `Result<T, String>` with Windows' reason.
- Never a guessed value: when Windows refuses, show its reason and hide the dock mark.
- Never delete files permanently without Windows' confirmation; a drop on the bin only recycles.
- Tasbih counts live in the `kv` table, never in config.json.
- The calculator never uses `eval` / `Function`.
- No commits: the owner commits (the Mirror widget is still uncommitted on this branch).
- Verification after every task: `npm test` and `npx svelte-check` pass; after Rust tasks `cargo test` and `cargo build` in `src-tauri` pass.

## Review Focus

1. **Dropping a folder or many files on the bin** — each path is recycled; one failure doesn't stop the rest; the card reports how many moved and why one failed.
2. **Keep Awake left on, then GlassBar quits or crashes** — the power request ends with the process; nothing is restored next start.
3. **Calculator input that is half-typed** ("12 +", "(3", "5 km to") — no result row, no error banner, apps and fallbacks still listed.
4. **A countdown whose date has passed** — repeating ones roll to the next year; one-off ones show "Done" and never fire twice.
5. **Hijri around Maghrib and midnight** — after Maghrib the next Hijri day shows; at midnight it doesn't jump twice.

---

### Task 1: Recycle Bin backend (watch, recent items, restore, recycle many)

**Files:**
- Modify: `src-tauri/src/platform/mod.rs` (Files trait, new `BinItem` struct)
- Modify: `src-tauri/src/platform/windows/files.rs`, `src-tauri/src/platform/windows/mod.rs`
- Create: `src-tauri/src/widgets/trash.rs`; Modify: `src-tauri/src/widgets/mod.rs`, `src-tauri/src/main.rs`, `src-tauri/src/widgets/downloads.rs` (make `recycle_stale` `pub`)
- Modify: `src-tauri/Cargo.toml` (windows features `Win32_Security`, `Win32_Security_Authorization`, `Win32_Storage_EnhancedStorage` as needed)

**Interfaces:**
- Produces (Rust): `pub struct BinItem { id: String /* parsing name */, name: String, from: String, deleted_at: i64 /* ms */, size: u64 }` (camelCase); `Files::recycle_items(limit: usize) -> Res<Vec<BinItem>>`, `Files::recycle_restore(id: &str) -> Res<()>`, `Files::watch_recycle(f: Box<dyn Fn() + Send>)`.
- Produces (commands): `trash_items(limit: u32) -> Vec<BinItem>`, `trash_restore(ids: Vec<String>)`, `trash_recycle(paths: Vec<String>) -> TrashResult { moved: u32, failed: Vec<(String, String)> }`, `trash_empty_confirm()` (Windows' own dialog), and `trash::start(app)` that starts the watcher only while `Config::uses("trash")`; every change emits `recycle:changed`.
- Produces (ipc.ts): `ipc.trashItems(limit)`, `ipc.trashRestore(ids)`, `ipc.trashRecycle(paths)`, `ipc.trashEmptyConfirm()`, types `BinItem`, `TrashResult`.

- [ ] Step 1: Rust test `trash::tests::recycle_result_counts_failures` — `summarize(vec![Ok(()), Err("in use".into()), Ok(())], &["a","b","c"])` returns `moved == 2`, `failed == [("b","in use")]`.
- [ ] Step 2: run `cargo test trash` → fails (no module).
- [ ] Step 3: implement `trash.rs` + platform methods. Items: `SHGetKnownFolderItem(FOLDERID_RecycleBinFolder)` → `BHID_EnumItems`; per `IShellItem2`: `SIGDN_DESKTOPABSOLUTEPARSING` id, `SIGDN_NORMALDISPLAY` name, `PKEY_Recycle_DeletedFrom`, `PKEY_Recycle_DateDeleted`, `PKEY_Size`; sort newest first, take `limit`. Restore: `SHParseDisplayName`/`SHCreateItemFromParsingName` → `BHID_SFUIObject` `IContextMenu` → `InvokeCommand` verb `"undelete"`. Watch: per fixed drive `FindFirstChangeNotificationW("X:\\$Recycle.Bin\\<SID>", subtree, FILE_NAME|DIR_NAME|SIZE)`, `WaitForMultipleObjects`, 1.5 s debounce. Confirmed empty: `SHEmptyRecycleBinW(None, null, 0)`.
- [ ] Step 4: `cargo test trash` passes; `cargo build` passes.

### Task 2: Recycle Bin widget (frontend)

**Files:**
- Create: `src/lib/icons/TrashIcon3D.svelte` (props `size`, `full`, `open`)
- Create: `src/widgets/trash/{index.ts,Pill.svelte,Popup.svelte,trash.ts,trash.test.ts}`
- Reuse: `src/widgets/sysmon/recycle.ts`, `src/widgets/sysmon/RecycleCard.svelte` confirm pattern, `src/widgets/shelf/shelf.ts` `hits()`

**Interfaces:**
- Consumes: Task 1 ipc.
- Produces: `trash.ts`: `dropSummary(r: TrashResult): string`, `fromLabel(path: string): string` ("Documents › Work").

- [ ] Step 1: tests — `dropSummary({moved:3,failed:[]})` = `"Moved 3 items to the Recycle Bin"`; `dropSummary({moved:1,failed:[["C:\\a.txt","in use"]]})` = `"Moved 1 item · 1 couldn't be moved (a.txt: in use)"`; `fromLabel("C:\\Users\\me\\Documents\\Work")` = `"Documents › Work"`; `fromLabel("D:\\")` = `"D:"`.
- [ ] Step 2: `npx vitest run src/widgets/trash` fails; Step 3 implement; Step 4 passes.
- [ ] Step 5: Pill (sizes small/medium, side tile, drop target with lid open, right-click menu), Popup (hero, Empty with inline confirm, Recently deleted with Restore, per drive). `svelte-check` clean.

### Task 3: Keep Awake (Rust + Quick Settings tile + sub-page + dock cup)

**Files:**
- Create: `src-tauri/src/platform/windows/awake.rs`; Modify platform `mod.rs` (trait `Awake { fn set(&self, system: bool, display: bool) -> Res<()>; fn clear(&self); }`)
- Create: `src-tauri/src/widgets/awake.rs` (commands `awake_get() -> AwakeState`, `awake_set(minutes: Option<u32>, screen: bool) -> AwakeState`, `awake_off()`; `AwakeState { on: bool, until: Option<i64>, screen: bool }`; emits `awake:changed`; expiry thread)
- Create: `src/widgets/quicksettings/AwakePanel.svelte`, `src/widgets/quicksettings/awake.ts` + test
- Modify: `src/widgets/quicksettings/{index.ts,Popup.svelte,Pill.svelte,SettingsForm.svelte}`

**Interfaces:** `awake.ts`: `awakeText(s: AwakeState, now: number): string` → `"Off"`, `"On · until you turn it off"`, `"On · 1:24 left"`, `"On · 42m left"`.

- [ ] Tests for `awakeText` with those four cases; Rust test that `until` from `minutes` = now + minutes·60 000 (`awake::until_of`).
- [ ] Implement with `PowerCreateRequest` (REASON_CONTEXT simple string "GlassBar: Keep awake") + `PowerSetRequest(PowerRequestSystemRequired)` (+ `PowerRequestDisplayRequired` when screen) and `PowerClearRequest` / `CloseHandle` on off. Tiles add key `awake` to `TILES`.

### Task 4: Microphone (mute, level, devices, in-use watcher, on-air light)

**Files:**
- Modify: `src-tauri/src/platform/windows/audio.rs` (capture endpoint), platform `mod.rs` (Audio trait: `mic_state() -> Res<MicState>`, `mic_set_mute(bool)`, `mic_set_volume(f32)`, `mic_peak() -> Option<f32>`, `watch_mic(Box<dyn Fn() + Send>)`)
- Create: `src-tauri/src/platform/windows/privacy.rs` (`in_use(kind: "microphone"|"webcam") -> Vec<UsingApp>`, `watch(f)` via `RegNotifyChangeKeyValue`)
- Create: `src-tauri/src/widgets/mic.rs` (commands `mic_state`, `mic_set_mute`, `mic_set_volume`, `mic_set_default`, `mic_peak`, `privacy_in_use`; emits `mic:changed`)
- Create: `src/widgets/quicksettings/MicPanel.svelte`, `src/widgets/quicksettings/mic.ts` + test
- Modify: Quick Settings files (tile `mic`, dock light, setting `micOnDock`)

**Interfaces:** `MicState { muted: bool, volume: f32, devices: AudioDevice[] }`, `UsingApp { name: string, exe: string, since: number }`, `InUse { mic: UsingApp[], cam: UsingApp[] }`. `mic.ts`: `appName(key: string): string` (packaged `Microsoft.WindowsCamera_8wekyb3d8bbwe` → `"Camera"`; NonPackaged `C:#Program Files#Zoom#bin#Zoom.exe` → `"Zoom"`), `micDock(mode, muted, inUse): 'muted' | 'live' | null`.

- [ ] Tests: `appName` both forms; `micDock('auto', false, false)` = null, `('auto', true, false)` = `'muted'`, `('auto', false, true)` = `'live'`, `('never', true, true)` = null, `('always', false, false)` = `'live'`.
- [ ] Rust test for `privacy::parse_key` decoding the `#`-separated NonPackaged key and `LastUsedTimeStop == 0` meaning in use.

### Task 5: Bluetooth battery

**Files:**
- Create: `src-tauri/src/platform/windows/btbattery.rs` (`levels() -> Vec<(String /* mac 12 hex lower */, String /* node name */, u8)>` with `CM_Get_Device_ID_ListW` filtered to `BTHENUM` and `BTHLE`, `CM_Locate_DevNodeW`, `CM_Get_DevNode_PropertyW` for `{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2`)
- Modify: `BtDevice` gets `battery: Option<u8>`; `wireless.rs` fills it by matching the MAC in the WinRT id; new command `bt_batteries() -> Vec<BtBattery { name, kind, percent }>` for connected devices; low-battery check thread in `widgets/quick.rs` (every 2 min while Quick Settings is used; notify once ≤ 15 %, re-arm above 30 %).
- Modify: `BtPanel.svelte` (chip), Quick Settings `Popup.svelte` (Devices section), `src/widgets/quicksettings/battery.ts` + test.
- Modify: `src-tauri/Cargo.toml` feature `Win32_Devices_DeviceAndDriverInstallation`.

**Interfaces:** `battery.ts`: `levelTone(p: number): 'ok' | 'low' | 'critical'` (≤10 critical, ≤20 low).

- [ ] Rust tests: `mac_from_instance("BTHENUM\\{0000111F-…}_VID&00010075_PID&0100\\7&169FE51&0&C4776487D293_C00000000")` = `Some("c4776487d293")`; `mac_from_winrt_id("Bluetooth#Bluetooth9c:b6:d0:11:22:33-c4:77:64:87:d2:93")` = `Some("c4776487d293")`; `low_alert(prev_armed, percent)` transitions.
- [ ] TS tests for `levelTone` at 50, 20, 10.

### Task 6: Countdowns widget

**Files:**
- Create: `src/widgets/countdown/{index.ts,Pill.svelte,Popup.svelte,Editor.svelte,SettingsForm.svelte,dates.ts,dates.test.ts,presets.ts}`
- Create: `src-tauri/src/widgets/countdown.rs` (`due_items(app) -> Vec<Due>` from widget settings); Modify `scheduler.rs` `collect()`.

**Interfaces:** `dates.ts`: `nextHijri(month: number, day: number, from: Date): Date`, `nextChinese(month: number, day: number, from: Date): Date`, `nextYearly(mmdd: string, from: Date): Date`, `targetOf(c: Countdown, now: Date): Date | null` (rolled over for repeats; null time = 00:00 for counting, 09:00 for the alert), `left(ms: number): { big: string; small: string }` (`"132 days"/"18 weeks 6 days"`, `"5h 20m"`, `"42m"`, `"Today"`). Rust: `countdown::target_ms(date, time, repeat, now)` mirrors fixed/yearly; hijri/chinese presets are stored with the computed `date` refreshed by the frontend on open (Rust uses the stored date).

- [ ] Tests: `nextYearly('08-31', new Date(2026, 9, 30))` → 31 Aug 2027; `nextHijri(10, 1, new Date(2026, 9, 30))` → a date in March 2027 (1 Syawal 1448, Umm al-Qura: 2027-03-10 ±1); `nextChinese(1, 1, new Date(2026, 9, 30))` → 6 Feb 2027; `left(5*3600e3 + 20*60e3).big` = `"5h 20m"`; `left(132*86400e3).big` = `"132 days"`, `.small` = `"18 weeks 6 days"`; a one-off past date → `targetOf` returns the date and `left` says `"Done"` for negative.
- [ ] Rust test: a countdown on 2026-11-02 without time gives a Due at local 09:00 that day keyed `countdown:<id>:2026-11-02`; with `remindDayBefore` a second Due the day before.

### Task 7: Colour picker

**Files:**
- Modify: `src-tauri/src/widgets/ocr.rs` (purpose `color`; second hotkey `colorHotkey` default `Super+Shift+C`, fallback `Control+Alt+C`; `on_hotkey(app, shortcut)` dispatches by shortcut; command `color_pick_finish(hex: String)` hides the overlay, writes the formatted value to the clipboard, emits `capture:done {kind:'color', hex}`)
- Modify: `src/main.rs` handler passes the shortcut; `src/app/Snap.svelte` (loupe); `src/widgets/ocr/{index.ts,Pill.svelte,Popup.svelte,HoverMenu.svelte,Feedback.svelte,look.ts,menu.ts,SettingsForm.svelte}`; Create `src/widgets/ocr/ColorTab.svelte`, `src/widgets/ocr/color.ts` + test; `src/app/Bar.svelte` feedback height for `color`.

**Interfaces:** `color.ts`: `toHex(r,g,b): string` (`"#3B82F6"`), `formatColor(hex: string, f: 'hex'|'rgb'|'hsl'): string`, `pushColor(list: string[], hex: string, max = 16): string[]` (dedupe, newest first). `CaptureDone` gains `{ kind: 'color'; hex: string }`.

- [ ] Tests: `toHex(59,130,246)` = `"#3B82F6"`; `formatColor('#3B82F6','rgb')` = `"rgb(59, 130, 246)"`; `formatColor('#3B82F6','hsl')` = `"hsl(217, 91%, 60%)"`; `pushColor(['#000000','#FFFFFF'], '#FFFFFF')` = `['#FFFFFF','#000000']`; cap at 16.

### Task 8: Hijri date and Tasbih

**Files:**
- Create: `src/widgets/solat/hijri.ts` + test, `src/widgets/solat/tasbih.ts` + test, `src/widgets/solat/TasbihPart.svelte` (dock segment), `src/widgets/solat/TasbihSection.svelte` (panel)
- Create: `src-tauri/src/widgets/tasbih.rs` (`tasbih_get`, `tasbih_add(delta: i32, target: u32, steps: Vec<u32>) -> TasbihState`, `tasbih_reset`; kv key `tasbih`; emits `tasbih:changed`)
- Modify: `src/widgets/solat/{index.ts,Pill.svelte,Popup.svelte,SettingsForm.svelte}`

**Interfaces:** `TasbihState { count: number; round: number; step: number; today: number; day: string }`. `hijri.ts`: `MONTHS_MS` (12 Malay names), `fromJakim('1448-05-12')` = `{ y:1448, m:5, d:12 }`, `hijriLabel(h)` = `"12 Jamadilawal 1448 H"`, `hijriOf(date: Date)` (Umm al-Qura via Intl), `hijriNow(today: PrayerDay | undefined, now: number): { label: string; estimate: boolean; afterMaghrib: boolean }`. `tasbih.ts`: `DHIKR` list, `stepsOf(dhikr: string, target: number): number[]` (`tasbih_fatimah` → `[33,33,34]`, else `[target]`).

- [ ] Tests: `hijriLabel(fromJakim('1448-05-12'))` = `"12 Jamadilawal 1448 H"`; `hijriNow` before Maghrib uses today's JAKIM date, after Maghrib gives day 13 with `afterMaghrib: true`; `stepsOf('tasbih_fatimah', 33)` = `[33,33,34]`.
- [ ] Rust tests: adding past the step target wraps to round+1/next step; a new day resets `today` but not `count`; `delta = -1` at 0 stays 0.

### Task 9: Calculator search panel

**Files:**
- Create: `src/widgets/start/{calc.ts,calc.test.ts,units.ts,units.test.ts,query.ts,query.test.ts,SearchPanel.svelte}`
- Modify: `src/widgets/start/{index.ts,StartButtons.svelte,Pill.svelte}`, Create `src/widgets/start/SettingsForm.svelte`; `src/widgets/apps/` inline Start options (`searchOpens`); `src/app/Popup.svelte` route `__search:<instance>`; Rust `quick.rs` `windows_search_text(text)` (Win+S then `SendInput` unicode) and Shell `type_text`.

**Interfaces:** `calc.ts`: `evaluate(src: string): number | null` (null on anything incomplete). `units.ts`: `convert(v: number, from: string, to: string): number | null`, `unitOf(word: string): Unit | null`, `suggestions(v, unit): {to: string; value: number}[]`. `query.ts`: `parse(q: string): Answer | null` where `Answer = { kind: 'math', value } | { kind: 'unit', value, to, from } | { kind: 'units', list } | { kind: 'fx', amount, from, to }`; `formatNumber(n): string`.

- [ ] Tests: `evaluate('2+3*4')` = 14; `'(2+3)*4'` = 20; `'2^10'` = 1024; `'-3+5'` = 2; `'15% of 200'` = 30; `'200 + 10%'` = 220; `'1,000 x 3'` = 3000; `'sqrt(16)'` = 4; `'sin(30)'` ≈ 0.5; `'12 +'` = null; `'(3'` = null; `'alert(1)'` = null. `convert(5,'km','mi')` ≈ 3.10686; `convert(100,'f','c')` ≈ 37.7778; `convert(1.5,'gb','mb')` = 1500; `convert(1,'kg','m')` = null. `parse('100 usd to myr')` = `{kind:'fx', amount:100, from:'USD', to:'MYR'}`; `parse('rm 50 in sgd')` → from `MYR`; `parse('5 km to')` = null; `parse('hello')` = null.

### Task 10: Docs, version, verification in the live app

- [ ] `CHANGELOG.md` 0.6.7 "New" entries for each tool; README widget list; `docs/DESIGN_RULES.md` flyout table rows for Recycle Bin and Countdowns.
- [ ] `npm test`, `npx svelte-check`, `cargo test`, `npm run tauri build` (release) all pass.
- [ ] Live check (memory: CDP port 9333 + UIA): each panel opens and shows real data; tasbih counts on the dock; the colour overlay copies a colour; the calculator answers `15% of 2400 + 18` = 378.
