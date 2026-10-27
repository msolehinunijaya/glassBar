# Performance page and performance modes: design

Date: 2026-10-27 · Branch: phase2 · Status: awaiting approval

## Why

GlassBar's owner wants to see what GlassBar itself costs their PC, and to keep it light
when the PC is short on resources. Today the only way to know is to profile by hand. That is
what this session did: it found that four always-running animations caused two-thirds of the
bar's CPU, and that RAM crept from ~395 MB to ~624 MB over one session.

The page is a **monitor for the owner**, not a feature for end users to tune. Accuracy and
honesty matter more than polish. A number that looks good but is wrong is the worst outcome.

## Goals

1. A new **Performance** section in Settings showing GlassBar's own **RAM, CPU, GPU and
   network** use, live, plus a graph of the **last hour**, so slow creep is visible.
2. Three **performance modes**: **Low**, **Normal**, **High**.
3. Modes can be picked **manually**, or left on **Auto**, which picks for you from the
   state of the PC.
4. For each mode, the page says **per widget what it changes** (what it reduces or improves),
   in plain words.

## Non-goals

- Per-widget usage numbers ("Equalizer uses 2.1% CPU"). Windows can't split one WebView's
  cost by widget without the profiler setup used in this session. This session's measured
  ranking is the basis for which widgets get mode rules.
- Monitoring other apps, or the whole PC. The sysmon widget already does that.
- History that survives a restart. The one-hour history is in memory only.
- Notifications or alerts when usage is high.

## Anti-goals (a failure even if it works)

- **The monitor itself costing noticeable CPU.** Sampling must be cheap, and it must slow
  down when the page is closed.
- **Wrong or misleading numbers.** Count the whole process tree (GlassBar plus its WebView2
  helpers), not only `glassbar.exe`. That process alone was 27 MB of ~400 MB. When a number is
  an approximation, the page says so.
- **Modes that silently change the user's widget settings.** Modes put temporary limits on
  top of settings. They never rewrite the config, and switching back to High restores
  everything exactly.
- **Auto flipping back and forth** every few seconds.

## Layout (chosen: option B from the mockups)

Mockup: `.superpowers/brainstorm/496-1793115511/content/performance-layout.html`, card B.

Top to bottom, in the existing Settings shell (left nav + `main` with `.group` blocks;
follow `docs/DESIGN_RULES.md`):

1. **Usage strip.** One line: `RAM 338 MB · CPU 4.8% · GPU 2.6% · Net 1.2 MB/hr`.
2. **One graph** under it, with chips to switch between RAM / CPU / GPU / Network. It covers
   the last hour.
3. **Mode row.** The label "Mode" and an **Auto** toggle on the right.
4. **Three mode cards:** Low / Normal / High, each with a one-line summary. The selected
   card is highlighted. Clicking a card picks that mode and turns Auto off.
5. **Auto reason line**, shown only when Auto is on: e.g. "Auto chose Low · system memory
   88% used".
6. **Change list** for the selected mode: one row per affected widget saying what changes
   **compared to Normal**, then one greyed line listing the widgets this mode leaves
   unchanged. For Normal itself, the list says what Normal does differently from High.

## What is measured, and how

All four numbers cover the **GlassBar process tree**: `glassbar.exe` and every descendant
process (WebView2 browser, GPU, renderer and utility processes). The tree is found by
parent PID, the same way this session measured it.

| Metric | Shown as | Source (builder may choose a better one) |
|---|---|---|
| RAM | MB, **private** memory summed over the tree | `GetProcessMemoryInfo` → `PrivateUsage` per process. Working set double-counts shared pages, so it is not used. |
| CPU | % of the **whole** CPU (same scale as Task Manager), summed over the tree | `sysinfo` process CPU (already a dependency) |
| GPU | % (3D engine), summed over the tree's PIDs | The PDH `GPU Engine` counter already used in `platform/windows/gpu.rs`, filtered to `pid_<n>` instances in the tree |
| Network | MB downloaded per hour, and total this session | **Counted in-app.** Bytes received through `net.rs` (all widget fetches) plus the other direct `reqwest` uses. Windows only exposes per-process network to admin tools. The page labels this "data GlassBar downloaded" and does not claim it is all traffic. |

Sampling (**my call**):
- Every **5 s** while the Performance page is closed, and every **1 s** while it is open.
- History is a ring buffer of the last hour, kept in Rust memory. The page fetches it once on
  open, then receives live samples.
- The sampler reports its own cost honestly: it is part of the tree it measures.

## Modes

Config gets a `performance: { mode: 'low' | 'normal' | 'high', auto: boolean }` block on the
config root (not per profile). Defaults: `mode: 'normal'`, `auto: false`.

The **effective mode** (what is actually in force) is decided in Rust and broadcast to the
bar and popup windows. Widgets read the effective mode, never the config directly. The root
element of each window gets `data-perf="low|normal|high"` so CSS-only changes need no
JavaScript.

### What each mode means

- **High:** everything at full smoothness. Widgets use exactly their own settings, plus the
  extra motion listed below.
- **Normal:** a little lighter than today (owner's decision). It keeps the four CPU fixes
  from `4953f1e`, **caps the Equalizer and Music visualizer at 30 fps** (both default to 60
  today), and **pauses animations while the bar is auto-hidden**. It is the default, so every
  existing user gets this on upgrade. Today's 60 fps is what High gives. The Equalizer's fps
  setting gets a hint that 60 fps applies only in High.
- **Low:** the lightest it can be. Things move only when they must, polling slows down, and
  hidden windows give memory back.

### Per-widget changes (starting set, based on this session's measurements)

| Widget | Low | Normal | High |
|---|---|---|---|
| Equalizer | max 15 fps; stops drawing when silent; idle look forced to "line" | max 30 fps | the user's fps setting (30 or 60) |
| Music | visualizer off; playing badge still; lyrics without the per-second wipe | visualizer at max 30 fps | visualizer at the user's fps |
| Battery (Quick settings, liquid style) | water still, even while charging | wave only while charging | wave always |
| Companion | no idle motion (no sway, bob or shiver) | sway a few times, then still | motion as long as the mood lasts |
| System monitor | every 10 s; skips the temperature and GPU counters unless shown | every 2 s | every 1 s |
| Solat | "approaching / reached" glow is static | glow animates | glow animates |
| Timer | stopwatch shimmer off | the user's setting | the user's setting |
| Popup / banner windows | freed when hidden (WebView2 `TrySuspend` + low memory level); opening takes a moment longer | kept ready | kept ready |
| Whole bar | all animations paused while the bar is auto-hidden | same as Low | animations run while hidden |

**Other widgets** (clock, weather, currency, IPU, todo, notes, progress, downloads, shelf,
apps, tray, notifications, …): unchanged in every mode, because this session measured them at
roughly 0. The builder audits each one. Any widget that turns out to have a recurring
animation or polls faster than every 5 s gets a row.

### Where the change text lives

Each widget declares its own mode text next to its code, as an optional field on the widget
manifest (`src/widgets/types.ts`), e.g. `perf?: { low?: string; high?: string }`. The page
builds the change list from the manifests, so the description can't drift away from the
widget that implements it. Widgets with no `perf` field appear on the "unchanged" line.

### Auto

When Auto is on, Rust decides the effective mode once per sample. **Auto only moves between
Normal and Low. High is only ever a manual choice.** All thresholds below are **my call**:

- **Go to Low** when any of these holds for **15 s**:
  - system memory used ≥ **85%**
  - system CPU ≥ **80%**
  - on battery with battery saver on (Windows' own setting)
- **Go back to Normal** only after **60 s** with memory < 75%, CPU < 60%, and no battery
  saver.
- The reason line names the condition that triggered the switch, with its value.

## Constraints

- Tauri 2 + Svelte 5 + Rust, Windows only. Use what's already in the project (`sysinfo`, the
  `windows` crate, PDH in `gpu.rs`). No new dependencies unless one is clearly smaller than
  the code it replaces.
- Follow `docs/DESIGN_RULES.md` and the existing Settings markup (`.group`, `.field`, `.seg`,
  `.hint`).
- Mode changes take effect live, without a restart.
- Existing tests keep passing. The Auto decision logic and the process-tree sum each get a
  unit test.

## Left to the builder

- The exact API used for each metric, as long as it covers the whole process tree and
  meets the anti-goals.
- How the effective mode reaches the widgets (a shared store like `live.svelte.ts`, an
  event, or both).
- The graph component (the existing `Sparkline` may be enough).
- Wording of the mode summaries and change rows: plain, short, in the voice of the existing
  Settings hints.
