# Performance Page and Performance Modes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Settings › Performance page that shows GlassBar's own RAM / CPU / GPU / network use with a one-hour graph, plus Low / Normal / High modes (manual or Auto) that change what each widget does.

**Architecture:** A Rust sampler (`src-tauri/src/perf/`) measures the whole GlassBar process tree, keeps one hour of samples, and decides the **effective mode** (manual, or Auto from system load). It broadcasts `perf:changed`. Every window mirrors the mode onto `<html data-perf="…">`, so most widget changes are plain CSS. The few JavaScript ones (Equalizer fps, Music visualizer) read a small `perf` store. Rust-side changes (sysmon period, suspending hidden windows) read `perf::effective()`.

**Tech Stack:** Tauri 2.12, Svelte 5, Rust (`sysinfo` 0.37, `windows` 0.62, PDH, `webview2-com` 0.39), Vitest, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-10-27-performance-page-design.md` (approved, with Normal = lighter than today).

## Global Constraints

- Windows only. No new dependencies except `webview2-com = "0.39"` (already in the lockfile through Tauri) and the `windows` feature `Win32_System_ProcessStatus`.
- All four metrics cover the **process tree**: `glassbar.exe` plus every descendant (WebView2 browser, GPU, renderers, utilities), found by parent PID.
- RAM = **private** bytes summed over the tree. Never working set.
- CPU = % of the **whole** CPU (Task Manager scale).
- Network = bytes GlassBar's own code downloaded (`net.rs`), labelled "data GlassBar downloaded".
- Sampling every **5 s**, or **1 s** while the Performance page is open. History = last **1 hour**, in memory only.
- Config: `performance: { mode: 'low' | 'normal' | 'high', auto: boolean }` on the config root, default `{ mode: 'normal', auto: false }`.
- Modes never write widget settings. They are limits applied on top, and High restores everything exactly.
- Auto moves only between Normal and Low.
  - To Low when memory ≥ **85%**, or CPU ≥ **80%**, or battery saver is on, for **15 s**.
  - Back to Normal only after **60 s** with memory < **75%**, CPU < **60%** and no battery saver.
- Normal (the default) caps the Equalizer and Music visualizer at **30 fps** and pauses animations while the bar is auto-hidden. High gives the user's own fps (30/60).
- Follow `docs/DESIGN_RULES.md` and the Settings markup (`.group`, `.field`, `.seg`, `.hint`).
- Commit prefixes used in this repo: `feat:`, `fix:`, `change:`, `docs:`, `test:`. Every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Checks: `npm run check` (0 errors), `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`.

## Review Focus

1. **Old config file with no `performance` block** must load as Normal with Auto off, not fail or reset the config. Test in Task 1.
2. **No GPU counter** (PDH unavailable, remote desktop, no GPU) must show "—" for GPU and keep the other numbers working. Tests in Task 2 (`pid_of` rejects junk) and Task 8 (`fmtPct(null)`).
3. **Settings window closed without the page unmounting** (window destroyed) must not leave the sampler at 1 s forever. The 1 s rate is a 5 s lease the page keeps renewing. Test in Task 4.
4. **Values exactly on a threshold** (85% memory, 80% CPU, 75% / 60% for calm) must follow the spec's ≥ / < exactly. Tests in Task 3.
5. **Opening a popup or banner that Low mode suspended** must show its content on the first try, not a blank or stale window. Manual check in Task 7.

---

### Task 1: `performance` config block

**Files:**
- Modify: `src-tauri/src/config.rs` (struct `Config` ~line 210, `impl Default for Config` ~line 263, tests ~line 505)
- Modify: `src/lib/ipc.ts` (interface `Config` ~line 35)

**Interfaces:**
- Produces (Rust, `crate::config`):
  ```rust
  #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
  #[serde(rename_all = "lowercase")]
  pub enum PerfMode { Low, #[default] Normal, High }

  #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
  #[serde(rename_all = "camelCase", default)]
  pub struct Performance { pub mode: PerfMode, pub auto: bool }
  // Config gains: pub performance: Performance
  ```
- Produces (TS, `src/lib/ipc.ts`): `export type PerfMode = 'low' | 'normal' | 'high';` and `Config.performance: { mode: PerfMode; auto: boolean }`.

- [ ] **Step 1: Write the failing tests** in `config.rs`'s `mod tests`:

```rust
#[test]
fn performance_defaults_to_normal_manual() {
    let c: Config = serde_json::from_str(r#"{"version":11}"#).unwrap();
    assert_eq!(c.performance, Performance { mode: PerfMode::Normal, auto: false });
}

#[test]
fn performance_round_trips_lowercase() {
    let mut c = Config::default();
    c.performance = Performance { mode: PerfMode::Low, auto: true };
    let s = serde_json::to_string(&c).unwrap();
    assert!(s.contains(r#""performance":{"mode":"low","auto":true}"#));
    assert_eq!(serde_json::from_str::<Config>(&s).unwrap().performance, c.performance);
}
```

- [ ] **Step 2: Run them, expect FAIL** (`Performance` not found): `cargo test --manifest-path src-tauri/Cargo.toml performance_`
- [ ] **Step 3: Add `PerfMode`, `Performance`, the `performance` field and its default** as in Interfaces. No version bump: `#[serde(default)]` covers old files.
- [ ] **Step 4: Run them, expect PASS.** Then add the TS types from Interfaces and run `npm run check`. Expect 0 errors.
- [ ] **Step 5: Commit** `feat: performance mode in config (low / normal / high, auto)`

---

### Task 2: Measure the GlassBar process tree

**Files:**
- Create: `src-tauri/src/perf/usage.rs`
- Create: `src-tauri/src/perf/mod.rs` (only `pub mod usage;` for now. Task 4 fills it in.)
- Modify: `src-tauri/src/main.rs` (`mod perf;`)
- Modify: `src-tauri/src/platform/windows/gpu.rs` (add a per-PID query)
- Modify: `src-tauri/src/net.rs` (count bytes received)
- Modify: `src-tauri/Cargo.toml` (`windows` feature `"Win32_System_ProcessStatus"`)

**Interfaces:**
- Produces (`crate::perf::usage`):
  - `pub fn tree(root: u32, procs: &[(u32, u32)]) -> Vec<u32>`: input is `(pid, parent_pid)` pairs. Returns `root` plus all descendants, in any order.
  - `pub struct Snapshot { pub ram_mb: f32, pub cpu: f32, pub gpu: Option<f32>, pub sys_mem_pct: f32, pub sys_cpu_pct: f32 }`
  - `pub struct Meter` with `pub fn new() -> Meter` and `pub fn sample(&mut self) -> Snapshot`.
- Produces (`crate::platform::windows::gpu`, re-exported the way `usage()` already is):
  - `pub fn pid_of(instance: &str) -> Option<u32>`: parses `"pid_1234_luid_0x…_engtype_3D"` → `Some(1234)`.
  - `pub fn usage_for(pids: &[u32]) -> Option<f32>`: 3D utilisation summed over instances whose PID is in `pids`, clamped to 0..=100. `None` when PDH is unavailable.
- Produces (`crate::net`): `pub fn bytes_received() -> u64`, the running total since start.

- [ ] **Step 1: Write the failing tests** in `usage.rs` and `gpu.rs`:

```rust
// usage.rs
#[test]
fn tree_has_root_and_all_descendants_only() {
    let procs = [(10, 1), (20, 10), (21, 10), (30, 20), (99, 1), (98, 99)];
    let mut t = tree(10, &procs); t.sort();
    assert_eq!(t, vec![10, 20, 21, 30]);
}
#[test]
fn tree_survives_a_parent_cycle() {
    // PID reuse can make a process look like its own ancestor.
    let mut t = tree(10, &[(10, 30), (20, 10), (30, 20)]); t.sort();
    assert_eq!(t, vec![10, 20, 30]);
}

// gpu.rs
#[test]
fn pid_of_reads_engine_instance_names() {
    assert_eq!(pid_of("pid_1234_luid_0x00000000_0x0000D1A5_phys_0_eng_0_engtype_3D"), Some(1234));
    assert_eq!(pid_of("_Total"), None);
    assert_eq!(pid_of("pid_abc_luid"), None);
    assert_eq!(pid_of(""), None);
}
```

- [ ] **Step 2: Run them, expect FAIL:** `cargo test --manifest-path src-tauri/Cargo.toml tree_ pid_of`
- [ ] **Step 3: Implement `tree`** as a breadth-first walk with a visited set (so a cycle stops). **Implement `pid_of`.**
- [ ] **Step 4: Run them, expect PASS.**
- [ ] **Step 5: Implement `usage_for`** with its **own** `static` PDH query on the same counter path as `usage()`. Two callers collecting one query would break each other's rate deltas. Read item names (`szName`) from `PdhGetFormattedCounterArrayW`.
- [ ] **Step 6: Implement `Meter::sample`:**
  - Use `sysinfo::System` with `refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_cpu())`, plus `refresh_cpu_usage` and RAM refresh for `sys_*`.
  - Root = `std::process::id()`.
  - `cpu` = sum of tree `cpu_usage()` ÷ `sys.cpus().len()`. sysinfo reports % of one core.
  - `ram_mb` = sum of `private_bytes(pid)` ÷ 1 048 576, where `fn private_bytes(pid: u32) -> Option<u64>` uses `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` + `K32GetProcessMemoryInfo` with `PROCESS_MEMORY_COUNTERS_EX.PrivateUsage`.
  - `gpu` = `usage_for(&tree)`.
- [ ] **Step 7: Count network bytes in `net.rs`.** Add `static RX: AtomicU64`. Every function in `net.rs` that reads a response body reads `.bytes()`, adds `len` to `RX`, then parses with `serde_json::from_slice`. Error text stays the same. (There are no `reqwest` uses outside `net.rs`.)
- [ ] **Step 8: Verify:** `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo build --manifest-path src-tauri/Cargo.toml`. Both pass.
- [ ] **Step 9: Commit** `feat: measure GlassBar's own process tree (private RAM, CPU, GPU, downloads)`

---

### Task 3: Auto mode decision

**Files:**
- Create: `src-tauri/src/perf/auto.rs` (and `pub mod auto;` in `perf/mod.rs`)

**Interfaces:**
- Consumes: `crate::config::PerfMode` (Task 1).
- Produces:
  ```rust
  pub struct Load { pub mem_pct: f32, pub cpu_pct: f32, pub saver: bool }
  #[derive(Default)] pub struct Auto { /* low: bool, hot_ms: u32, calm_ms: u32, reason: String */ }
  impl Auto { pub fn step(&mut self, l: &Load, dt_ms: u32) -> (PerfMode, String) }
  ```
  - Returns `Normal` or `Low`, never `High`.
  - Reason text, exact copy. The first matching cause is used, in this order:
    1. `"system memory {mem:.0}% used"`
    2. `"CPU {cpu:.0}% busy"`
    3. `"battery saver on"`
  - In Normal the reason is `"your PC has room"`.
  - The reason is fixed at the moment of switching to Low and kept until the switch back.
  - Constants: `LOW_MEM = 85.0`, `LOW_CPU = 80.0`, `HOT_MS = 15_000`, `CALM_MEM = 75.0`, `CALM_CPU = 60.0`, `CALM_MS = 60_000`.

- [ ] **Step 1: Write the failing tests:**

```rust
fn run(a: &mut Auto, l: Load, secs: u32) -> (PerfMode, String) { let mut r = a.step(&l, 0); for _ in 0..secs { r = a.step(&l, 1000); } r }
const CALM: Load = Load { mem_pct: 50.0, cpu_pct: 10.0, saver: false };

#[test] fn memory_at_85_for_15s_goes_low() {
    let mut a = Auto::default();
    assert_eq!(run(&mut a, Load { mem_pct: 85.0, ..CALM }, 14).0, PerfMode::Normal);
    assert_eq!(a.step(&Load { mem_pct: 85.0, ..CALM }, 1000), (PerfMode::Low, "system memory 85% used".into()));
}
#[test] fn cpu_just_under_80_never_goes_low() {
    assert_eq!(run(&mut Auto::default(), Load { cpu_pct: 79.9, ..CALM }, 120).0, PerfMode::Normal);
}
#[test] fn a_dip_restarts_the_15s() {
    let mut a = Auto::default();
    run(&mut a, Load { cpu_pct: 90.0, ..CALM }, 10); run(&mut a, CALM, 1);
    assert_eq!(run(&mut a, Load { cpu_pct: 90.0, ..CALM }, 14).0, PerfMode::Normal);
}
#[test] fn battery_saver_goes_low_with_its_reason() {
    assert_eq!(run(&mut Auto::default(), Load { saver: true, ..CALM }, 15), (PerfMode::Low, "battery saver on".into()));
}
#[test] fn back_to_normal_only_after_60s_calm() {
    let mut a = Auto::default();
    run(&mut a, Load { mem_pct: 90.0, ..CALM }, 15);
    assert_eq!(run(&mut a, Load { mem_pct: 74.9, cpu_pct: 59.9, saver: false }, 59).0, PerfMode::Low);
    assert_eq!(a.step(&Load { mem_pct: 74.9, cpu_pct: 59.9, saver: false }, 1000), (PerfMode::Normal, "your PC has room".into()));
}
#[test] fn memory_at_75_is_not_calm() {
    let mut a = Auto::default();
    run(&mut a, Load { mem_pct: 90.0, ..CALM }, 15);
    assert_eq!(run(&mut a, Load { mem_pct: 75.0, ..CALM }, 120).0, PerfMode::Low);
}
#[test] fn low_keeps_the_reason_it_switched_for() {
    let mut a = Auto::default();
    run(&mut a, Load { mem_pct: 88.0, ..CALM }, 15);
    assert_eq!(run(&mut a, Load { mem_pct: 80.0, ..CALM }, 5).1, "system memory 88% used");
}
```

- [ ] **Step 2: Run them, expect FAIL:** `cargo test --manifest-path src-tauri/Cargo.toml perf::auto`
- [ ] **Step 3: Implement `Auto::step`** to the constants above. A hot sample resets `calm_ms`, and a non-hot sample resets `hot_ms`.
- [ ] **Step 4: Run them, expect PASS.**
- [ ] **Step 5: Commit** `feat: Auto performance mode decision (to Low after 15 s busy, back after 60 s calm)`

---

### Task 4: Sampler, effective mode and commands

**Files:**
- Modify: `src-tauri/src/perf/mod.rs`
- Modify: `src-tauri/src/main.rs` (`perf::start(h.clone())` next to `widgets::sysmon::start` ~line 181; add the three commands to `generate_handler!` ~line 207)
- Modify: `src-tauri/src/main.rs` `commit()` (~line 40): after the config is applied, call `perf::refresh(app)`
- Modify: `src/lib/ipc.ts` (types, `ipc.*`, `events.*`)

**Interfaces:**
- Consumes: `usage::Meter`, `usage::Snapshot` (Task 2); `auto::{Auto, Load}` (Task 3); `config::{PerfMode, Performance}` (Task 1); `crate::platform::power().saver_on()`; `crate::net::bytes_received()`; `crate::db::now_ms()`.
- Produces (Rust, `crate::perf`):
  ```rust
  #[derive(Serialize, Clone, Debug)] #[serde(rename_all = "camelCase")]
  pub struct PerfSample { pub at: i64, pub ram_mb: f32, pub cpu: f32, pub gpu: Option<f32>, pub net_bytes: u64 }
  #[derive(Serialize, Clone, Debug, PartialEq)] #[serde(rename_all = "camelCase")]
  pub struct PerfState { pub mode: PerfMode, pub auto: bool, pub reason: Option<String> } // reason only when auto
  pub fn effective() -> PerfMode
  pub fn refresh(app: &AppHandle)            // recompute from config now; emit "perf:changed" if PerfState changed
  pub fn period_ms(now_ms: i64, watched_at_ms: i64) -> u64   // 1000 if now - watched < 5000, else 5000
  pub fn trim(h: &mut VecDeque<PerfSample>, now_ms: i64)     // drop samples with at < now_ms - 3_600_000
  pub fn start(app: AppHandle)
  #[tauri::command] pub fn perf_history() -> Vec<PerfSample>
  #[tauri::command] pub fn perf_state() -> PerfState
  #[tauri::command] pub fn perf_watch()      // renews the 1 s lease (records now_ms)
  ```
  - Events: `perf:sample` (payload `PerfSample`, `emit_to("settings", …)`) and `perf:changed` (no payload, to all windows, so `live()`-style reloads work).
- Produces (TS, `src/lib/ipc.ts`):
  - `interface PerfSample { at: number; ramMb: number; cpu: number; gpu: number | null; netBytes: number }`
  - `interface PerfState { mode: PerfMode; auto: boolean; reason: string | null }`
  - `ipc.perfHistory(): Promise<PerfSample[]>`, `ipc.perfState(): Promise<PerfState>`, `ipc.perfWatch(): Promise<void>`
  - `events.perfSample(cb: (s: PerfSample) => void)`

- [ ] **Step 1: Write the failing tests** in `perf/mod.rs`:

```rust
#[test] fn fast_only_while_the_page_renews_its_lease() {
    assert_eq!(period_ms(10_000, 6_000), 1000);
    assert_eq!(period_ms(10_000, 5_000), 5000);  // lease expired: window closed without unmounting
    assert_eq!(period_ms(10_000, 0), 5000);      // never opened
}
#[test] fn history_keeps_one_hour() {
    let s = |at| PerfSample { at, ram_mb: 0.0, cpu: 0.0, gpu: None, net_bytes: 0 };
    let mut h: VecDeque<_> = [s(0), s(1_000), s(3_600_500)].into();
    trim(&mut h, 3_601_000);
    assert_eq!(h.iter().map(|x| x.at).collect::<Vec<_>>(), vec![1_000, 3_600_500]);
}
```

- [ ] **Step 2: Run them, expect FAIL.**
- [ ] **Step 3: Implement.**
  - State lives in statics: history `Mutex<VecDeque<PerfSample>>`, `STATE: Mutex<PerfState>`, `WATCHED: AtomicI64`, and an `Auto` inside the sampler thread.
  - The loop works like `sysmon::start`:
    1. Sample.
    2. Push, then `trim`.
    3. `emit_to("settings","perf:sample", s)`.
    4. If `config.performance.auto`, feed `Load { mem_pct: snap.sys_mem_pct, cpu_pct: snap.sys_cpu_pct, saver: power().saver_on() == Some(true) }` to `Auto::step` with the real elapsed ms.
    5. Store the new `PerfState`, emitting `perf:changed` when it changed.
    6. Sleep `period_ms`.
  - `refresh` applies manual mode at once, without waiting for the next sample.
  - Turning Auto on starts from Normal (a fresh `Auto`).
- [ ] **Step 4: Run them, expect PASS.** Then run `cargo build` and `npm run check`.
- [ ] **Step 5: Commit** `feat: perf sampler with one-hour history and effective mode`

---

### Task 5: Mode reaches every window

**Files:**
- Create: `src/lib/perf.ts`, `src/lib/perf.test.ts`, `src/lib/perf.svelte.ts`
- Modify: `src/main.ts` (call `await initPerf()` right after `await initConfig()`)
- Modify: `src-tauri/src/bar.rs` `reveal()` (~line 266): after the `revealed.swap` early-return, `let _ = app.emit_to("bar", "bar:away", !show);`
- Modify: `src/lib/ipc.ts` (`events.barAway(cb: (away: boolean) => void)`)
- Modify: `src/styles/glass.css` (next to the `data-motion` rules ~line 97)

**Interfaces:**
- Consumes: `ipc.perfState`, `events.changed('perf', …)` (Task 4).
- Produces:
  - `src/lib/perf.ts`: `export function fpsCap(mode: PerfMode, userFps: number): number`. Returns 15 in Low, `Math.min(30, userFps)` in Normal, and `userFps` in High.
  - `src/lib/perf.svelte.ts`: `export const perf = $state<PerfState>({ mode: 'normal', auto: false, reason: null })`, and `export async function initPerf(): Promise<void>`. That function loads the state, reloads it on `perf:changed`, and mirrors `perf.mode` to `document.documentElement.dataset.perf`. In the bar window it also sets or deletes `dataset.away` from `bar:away`.
  - CSS contract, used by later tasks: `:root[data-perf='low' | 'normal' | 'high']` and `:root[data-away]`.

- [ ] **Step 1: Write the failing test** `src/lib/perf.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { fpsCap } from './perf';
describe('fpsCap', () => {
  it('Low is 15 whatever the setting', () => { expect(fpsCap('low', 60)).toBe(15); expect(fpsCap('low', 30)).toBe(15); });
  it('Normal caps at 30', () => { expect(fpsCap('normal', 60)).toBe(30); expect(fpsCap('normal', 30)).toBe(30); });
  it('High is the user setting', () => { expect(fpsCap('high', 60)).toBe(60); expect(fpsCap('high', 30)).toBe(30); });
});
```

- [ ] **Step 2: Run it, expect FAIL:** `npx vitest run src/lib/perf.test.ts`
- [ ] **Step 3: Implement** `perf.ts`, `perf.svelte.ts`, the `initPerf()` call, the `bar:away` emit, and this CSS rule:

```css
/* Auto-hidden bar: nothing moves while it's tucked away (Low and Normal). */
:root[data-away]:not([data-perf='high']) * { animation-play-state: paused !important; }
```

- [ ] **Step 4: Run it, expect PASS.** Then run `npm test` and `npm run check`.
- [ ] **Step 5: Commit** `feat: performance mode on every window; pause animations while the bar is auto-hidden`

---

### Task 6: Widgets follow the mode

**Files:**
- Modify: `src/widgets/types.ts` (`WidgetManifest`)
- Modify: `src/widgets/equalizer/Spectrum.svelte`, `src/widgets/equalizer/SettingsForm.svelte:94`, `src/widgets/equalizer/index.ts`
- Modify: `src/widgets/music/PlayerPill.svelte`, `src/widgets/music/Rhythm.svelte`, `src/widgets/music/LyricLine.svelte`, `src/widgets/music/index.ts`
- Modify: `src/widgets/battery/LiquidBattery.svelte`, `src/widgets/quicksettings/index.ts`
- Modify: `src/widgets/companion/Face.svelte`, `src/widgets/companion/index.ts`
- Modify: `src/widgets/solat/Pill.svelte`, `src/widgets/solat/index.ts`
- Modify: `src/widgets/timer/Pill.svelte`, `src/widgets/timer/index.ts`
- Modify: `src-tauri/src/widgets/sysmon.rs` (`start`, ~line 86), `src/widgets/sysmon/index.ts`

**Interfaces:**
- Consumes: `perf` and `fpsCap` (Task 5), `crate::perf::effective()` (Task 4), the `data-perf` / `data-away` CSS contract (Task 5).
- Produces: `WidgetManifest.perf?: Partial<Record<PerfMode, string>>`. This is the change text shown on the Performance page for that mode. `low` and `high` say what differs from Normal; `normal` says what differs from High.

Rules, from the spec table. CSS rules use `:global(:root[data-perf='…'])` inside the component's `<style>`:

- [ ] **Step 1: Equalizer.**
  - In `Spectrum.svelte`, the loud-frame interval becomes `1000 / fpsCap(perf.mode, +s.fps)`.
  - In Low, the idle look is treated as `'line'` both for the nap decision and for `paint`. Pass `paint` a derived settings object; don't mutate `s`.
  - While `document.documentElement.dataset.away` is set and the mode isn't High, nap 1000 ms without drawing.
  - In `SettingsForm.svelte:94`, the 60 fps choice gets `hint: 'Only in High performance mode'`.
  - Manifest `perf`:
    - `low: 'Up to 15 fps, line when silent'`
    - `normal: 'Up to 30 fps'`
    - `high: 'Your frame rate (up to 60 fps)'`
- [ ] **Step 2: Music.**
  - In `PlayerPill.svelte`, Low renders neither the Visualizer nor the Spectrum (treat `p.viz` as `'none'`). The Spectrum fps cap comes free from Step 1.
  - In Low, the `.eq i` badge bars and `Rhythm.svelte`'s bars have `animation: none`. The `.wipe` in `LyricLine.svelte` gets `display: none`.
  - Manifest `perf`:
    - `low: 'No visualizer, still playing badge, no karaoke wipe'`
    - `normal: 'Visualizer up to 30 fps'`
    - `high: 'Visualizer at your frame rate'`
- [ ] **Step 3: Battery** (`LiquidBattery.svelte`, text on the `quicksettings` manifest).
  - The wave animates when `.charging` and the mode isn't Low, or always in High.
  - In Low, the bubbles and bolt pulse don't animate either.
  - Fix the file's header comment to match.
  - Manifest `perf`:
    - `low: 'Liquid battery stays still, even while charging'`
    - `normal: 'Liquid battery waves only while charging'`
    - `high: 'Liquid battery always waves'`
- [ ] **Step 4: Companion** (`Face.svelte`).
  - In Low, `.bob`, `.shiver` and `.sway` get `animation: none`.
  - In High, `.sway` gets `animation-iteration-count: infinite`.
  - Manifest `perf`:
    - `low: 'No idle motion'`
    - `normal: 'Sways a few times, then rests'`
    - `high: 'Keeps moving while the mood lasts'`
- [ ] **Step 5: Solat and Timer.**
  - In Low, solat's `.approaching` and `.reached` have `animation: none`. Their static colours stay.
  - In Low, timer's `.sw.shimmer::after` has `animation: none`.
  - Manifest `perf.low`: solat `'Prayer-time glow stays still'`, timer `'No stopwatch shimmer'`.
- [ ] **Step 6: System monitor (Rust).**
  - In `sysmon::start`, `period` comes from `crate::perf::effective()`: Low 10 s, Normal 2 s, High 1 s.
  - `rx_bps`/`tx_bps` divide by the real period.
  - `cpu_temp` is read only when `widget_settings("sysmon")["temp"] == true` (GPU is already gated by its setting).
  - Manifest `perf`:
    - `low: 'Updates every 10 s'`
    - `normal: 'Updates every 2 s'`
    - `high: 'Updates every second'`
- [ ] **Step 7: Verify:** `npm run check` (0 errors), `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 8: Commit** `feat: widgets follow the performance mode`

---

### Task 7: Low mode frees hidden popup and banner windows

**Files:**
- Create: `src-tauri/src/perf/suspend.rs` (and `pub mod suspend;`)
- Modify: `src-tauri/Cargo.toml` (`[target.'cfg(windows)'.dependencies] webview2-com = "0.39"`)
- Modify: `src-tauri/src/bar.rs`:
  - `popup_hidden` (~line 497) and `banner_hide` (~line 687): call `perf::suspend::later`.
  - Top of `popup_open` (~line 506) and top of `banner_push` (~line 639), **before** any `emit_to` or `show`: call `perf::suspend::wake`.

**Interfaces:**
- Consumes: `crate::perf::effective()`.
- Produces:
  - `pub fn later(win: WebviewWindow)`: after **30 s** (my call: avoids suspending during quick re-opens), if `effective() == PerfMode::Low` and the window is still hidden, suspend it. A generation counter per label cancels the timer when the window is shown again.
  - `pub fn wake(win: &WebviewWindow)`: no-op unless it was suspended.

- [ ] **Step 1: Implement `suspend`** in `win.with_webview`:
  - Take `controller()`.
  - `SetIsVisible(false)`.
  - `CoreWebView2().cast::<ICoreWebView2_3>()?.TrySuspend(&TrySuspendCompletedHandler::create(Box::new(|_, _| Ok(()))))`.
  - `cast::<ICoreWebView2_19>()?.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW)`.
  - Errors are ignored: an older WebView2 simply isn't freed.
- [ ] **Step 2: Implement `wake`:** `SetMemoryUsageTargetLevel(…NORMAL)`, then `Resume()`, then `SetIsVisible(true)`. Track "suspended" in a `Mutex<HashSet<String>>` keyed by label.
- [ ] **Step 3: Wire up `bar.rs`** as listed under Files.
- [ ] **Step 4: Verify:** `cargo build --manifest-path src-tauri/Cargo.toml` passes.
- [ ] **Step 5: Manual check (Review Focus 5).** Set mode Low, open a widget popup, close it, and wait 40 s.
  - The popup's renderer drops to ~0% CPU and its private memory falls. Use `procs.ps1` from this session's scratchpad or Task Manager's "Details" tab with Command line.
  - Then open the same popup and a different one. Both show their content on the first try.
  - Trigger a banner (Settings › Notifications test) after 40 s hidden. It shows its text.
- [ ] **Step 6: Commit** `feat: Low mode frees hidden popup and banner windows`

---

### Task 8: Settings › Performance page

**Files:**
- Create: `src/app/perfPage.ts`, `src/app/perfPage.test.ts`, `src/app/PerformanceSettings.svelte`
- Modify: `src/app/Settings.svelte` (`sections` line 16: insert `'Performance'` after `'Widgets'`; render `<PerformanceSettings />` like `<AppearanceSettings />` at line 140)

**Interfaces:**
- Consumes: `ipc.perfHistory`, `ipc.perfWatch`, `events.perfSample`, `PerfSample` (Task 4); `perf` (Task 5); `widgets` from `src/widgets/registry.ts` and `WidgetManifest.perf` (Task 6); `update` from `src/lib/config.svelte.ts`; `Sparkline` from `src/lib/Sparkline.svelte`.
- Produces (`src/app/perfPage.ts`):
  - `export const MODE_CARDS: { mode: PerfMode; title: string; summary: string }[]`:
    - Low: "Lightest. Things move only when they must."
    - Normal: "Smooth where it counts, calm when idle."
    - High: "Everything at full smoothness."
  - `export const APP_ROWS: { name: string; perf: Partial<Record<PerfMode, string>> }[]`:
    - `{ name: 'Popup & banner', perf: { low: 'Memory freed when hidden; opens a moment slower' } }`
    - `{ name: 'Whole bar', perf: { normal: 'Animations pause while auto-hidden', low: 'Animations pause while auto-hidden', high: 'Animations keep running while auto-hidden' } }`
  - `export function changeList(mode: PerfMode, items: { name: string; perf?: Partial<Record<PerfMode, string>> }[]): { rows: { name: string; text: string }[]; unchanged: string[] }`. Rows keep input order. `unchanged` is the names without text for `mode`, sorted A–Z.
  - `export function netLastHour(s: PerfSample[]): number`: `last.netBytes - first.netBytes`, or 0 for fewer than 2 samples.
  - `export function fmtBytes(n: number): string`: `"340 KB"`, `"1.2 MB"`, `"1.3 GB"`. Base 1024, one decimal from MB up, KB rounded.
  - `export function fmtPct(v: number | null): string`: `"4.8%"`, or `"—"` for null.

- [ ] **Step 1: Write the failing tests** `src/app/perfPage.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { changeList, fmtBytes, fmtPct, netLastHour } from './perfPage';
const s = (netBytes: number) => ({ at: 0, ramMb: 0, cpu: 0, gpu: null, netBytes });
describe('changeList', () => {
  const items = [{ name: 'Whole bar', perf: { low: 'paused' } }, { name: 'Equalizer', perf: { low: '15 fps', high: '60 fps' } }, { name: 'Clock' }, { name: 'Apps' }];
  it('lists what the mode changes, in order, and the rest A–Z', () => {
    expect(changeList('low', items)).toEqual({ rows: [{ name: 'Whole bar', text: 'paused' }, { name: 'Equalizer', text: '15 fps' }], unchanged: ['Apps', 'Clock'] });
  });
  it('a mode with no text for a widget leaves it unchanged', () => {
    expect(changeList('high', items).unchanged).toEqual(['Apps', 'Clock', 'Whole bar']);
  });
});
describe('formatting', () => {
  it('bytes', () => { expect(fmtBytes(348_000)).toBe('340 KB'); expect(fmtBytes(1.2 * 1048576)).toBe('1.2 MB'); });
  it('missing GPU shows a dash', () => { expect(fmtPct(null)).toBe('—'); expect(fmtPct(4.83)).toBe('4.8%'); });
  it('net in the last hour', () => { expect(netLastHour([s(100), s(400), s(1100)])).toBe(1000); expect(netLastHour([s(5)])).toBe(0); });
});
```

- [ ] **Step 2: Run it, expect FAIL:** `npx vitest run src/app/perfPage.test.ts`
- [ ] **Step 3: Implement `perfPage.ts`.**
- [ ] **Step 4: Run it, expect PASS.**
- [ ] **Step 5: Build `PerformanceSettings.svelte`** to the spec's layout B (mockup card B in `.superpowers/brainstorm/496-1793115511/content/performance-layout.html`):
  1. **Strip:** `RAM 338 MB · CPU 4.8% · GPU 2.6% · Net 1.2 MB/hr (8.4 MB since start)`. Values come from the latest sample, `netLastHour`, and the latest `netBytes` (the session total).
  2. **Chips:** RAM / CPU / GPU / Network, with a full-width `Sparkline` of that series over the history. Network plots the per-sample delta of `netBytes`. GPU nulls plot as 0, and the strip shows "—".
  3. **Hint**, exact copy: "Includes GlassBar's WebView2 helpers and this Settings window. Network counts data GlassBar downloaded."
  4. **Mode row:** an **Auto** toggle, `update((c) => { c.performance.auto = v; })`.
  5. **Three cards** from `MODE_CARDS`. The selected one is `perf.mode`. A click runs `update((c) => { c.performance = { mode: m, auto: false }; })`.
  6. **Reason line**, only when `perf.auto`: `Auto chose {Mode} · {perf.reason}`.
  7. **Change list** for the **selected card's** mode: `changeList(mode, [...APP_ROWS, ...Object.values(widgets).map((m) => ({ name: m.name, perf: m.perf }))])`. The unchanged names go on one greyed `.hint` line: "Unchanged: …".
  8. **Lifecycle:**
     - On mount: `perfHistory()`, subscribe to `events.perfSample` and append, call `perfWatch()` now and every 2 s.
     - On destroy: unsubscribe and clear the interval.
- [ ] **Step 6: Verify:** `npm run check` (0 errors), `npm test`.
- [ ] **Step 7: Commit** `feat: Settings › Performance page (usage, one-hour graph, modes)`

---

### Task 9: End-to-end verification

**Files:** none (fixes found here go in the task that owns the code).

- [ ] **Step 1: Run every check:** `npm run check`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`. All pass.
- [ ] **Step 2: Build:** `npx tauri build --no-bundle`. Expect `Built application at: …\target\release\glassbar.exe`.
- [ ] **Step 3: Run it with the Performance page open.**
  - RAM is within ±10% of Task Manager's summed "Memory (private working set)" for glassbar.exe and its msedgewebview2 children.
  - CPU roughly matches the sum of their CPU %.
  - Network rises after a weather refresh.
- [ ] **Step 4: Check the sampler's own cost (anti-goal).** Close Settings and wait 10 s. GlassBar's total CPU is within 0.1 percentage points of the same build with the sampler thread commented out. Measure over 60 s with `procs.ps1`.
- [ ] **Step 5: Check the modes live.**
  - Low: Equalizer visibly slower, Liquid battery still, companion still. After auto-hide, the bar's renderer CPU drops near 0.
  - High: Equalizer at 60 fps, liquid wave moving while not charging.
  - Auto on: the reason line shows "Auto chose Normal · your PC has room".
- [ ] **Step 6: Commit** any fix-ups with the owning task's prefix. Then tick off this plan's checkboxes and commit `docs: performance plan done`.
