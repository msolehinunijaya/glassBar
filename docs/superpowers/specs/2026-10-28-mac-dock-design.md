# Mac-style dock behaviour: design

Date: 2026-10-28 · Branch: `dock-effects` (from `phase2`, worktree `C:\laragon\www\glassBar-dock`) · Status: awaiting approval

## Why

GlassBar's owner wants the dock to feel like the macOS Dock: icons that swell under the
pointer, windows that visibly fly into the dock when minimized, apps that bounce while they
start, and a dock on every screen. Each behaviour gets a setting.

GlassBar is Tauri 2 + Svelte 5 on WebView2, Windows only. Windows cannot bend or animate
another app's window, so the minimize effects are drawn by GlassBar on top of the screen
using DWM's live window pictures (the same mechanism as the hover previews in
`platform/windows/thumbs.rs`).

## Goals, in build order

1. **Magnification**: App Launcher icons grow under the pointer, with neighbours growing less
   in a wave, popping out *above* the dock (outward from the screen edge), like macOS.
2. **Launch bounce**: the icon of an app that is starting bounces until its first window
   appears.
3. **Minimize / restore effects**: Genie, Scale or Suck, flying the window into its app icon
   (or, with "into application icon" off, into its own live-picture spot at the end of the
   launcher). Restore plays the effect in reverse.
4. **Dock on every screen**: Settings › Bar › Display gets a third choice, "All displays",
   which shows a full copy of the dock on every monitor at the same time.

Build and land them in this order. Each is usable on its own.

## Non-goals

- **Taskbar pin syncing.** Dropped by the owner. The launcher's pinned list stays exactly as
  it is today. Clicking an open app already focuses or restores it, and running dots
  already show; nothing changes there.
- One dock that moves between screens (the macOS behaviour). The owner chose a copy on every
  screen instead.
- Changing the system-wide Windows animation setting (`SPI_SETANIMATION`). We may suppress
  the native minimize animation **per window** only (see 3).
- Animating windows from apps GlassBar can't get a DWM picture of (elevated apps while
  GlassBar is not elevated, UWP shells, etc.). They minimize normally without our effect.

## Anti-goals (a failure even if it works)

- **The dock or screen stuttering** because of an effect. Effects must stay smooth on a normal
  PC and drop to the cheapest form in Low performance mode.
- **A leftover picture or overlay** stuck on screen after an effect (crash, window closed
  mid-animation, rapid minimize/restore). The overlay must always clean itself up.
- **The overlay or the magnification strip stealing clicks** meant for other windows longer
  than the hover lasts. The effect overlay is always click-through.
- **Duplicated side effects with several docks**: two adhans, two chimes, two water
  reminders, two dock alerts. Only the main dock plays sounds and alerts.
- **Shifting other windows around**: magnification must not change the space reserved for
  the dock.

## Settings

All App Launcher settings go in the widget's `SettingsForm.svelte`, in a new **Dock effects**
group above the pinned list. (Builder's call on exact layout; follow `docs/DESIGN_RULES.md`.)
Names below are the labels to show.

| Setting | Type | Default | Stored as (in `AppsSettings`) |
|---|---|---|---|
| Magnification | toggle | **off** | `magnify: boolean` |
| Size (shown when on) | slider 1.2×–2.0× | 1.6× | `magnifyScale: number` |
| Minimize windows using | choice: Genie / Scale / Suck | **Genie** | `minimizeEffect: 'genie' \| 'scale' \| 'suck' \| 'none'` |
| Animate opening applications | toggle | **on** | `bounce: boolean` |
| Minimize windows into application icon | toggle | **on** | `minimizeToIcon: boolean` |

`none` is offered as "Off" in the choice so the owner can turn minimize effects off
entirely. Defaults go in `src/widgets/apps/index.ts` `defaults`; old configs without these
keys get the defaults.

Minimize effects are drawn by the Rust side, which needs to know the chosen effect and
target. The front end sends them (see 3); there is no new field in `config.rs` for the
launcher.

**Settings › Bar › Display** (existing select in `src/app/Settings.svelte`, currently "Main
display" + each monitor): add **All displays**. Stored as a new `BarCfg` field
`all_displays: bool` (serde default `false`, mirrored in `src/lib/ipc.ts`). When true,
`monitor` is ignored for placement of the extra docks but still picks which one is the
**main** dock (null = primary monitor).

## 1. Magnification

**Behaviour**
- While the pointer is over the launcher's icons, each icon is scaled by
  `1 + (scale − 1) × f(d)`, where `d` is the distance from the pointer to the icon's centre
  along the dock, and `f` is a smooth bell, 1 at the pointer, falling to 0 at about
  2.5 icon widths. Use a cosine or Gaussian bell; builder's call.
- Icons grow **outward from the screen edge** (bottom dock: upward; top dock: downward;
  left/right dock: sideways away from the edge). Neighbours spread apart along the dock so
  grown icons don't overlap.
- **The dock gets longer while magnifying (owner's choice, like macOS).** The launcher's slot
  widens by the extra length its icons need; the widgets on either side slide outward; the
  bar window and its glass grow in length to fit (staying centred, as `place()` centres it),
  then everything slides back when the pointer leaves. Clamp to the monitor's length minus
  the gap; if there is no room left, icons stop spreading and grow in place.
- Floating bar (`bar.floating` set): it grows in length around its centre the same way.
- On leaving the launcher, icons ease back to normal size, about 150–200 ms.
- Off while dragging an icon to reorder, while in edit mode, and for icons folded behind
  "+N".
- **Low performance mode**: still magnifies, but with no easing transition (instant follow).
  `<html data-perf="low">` is the switch.

**Popping out above the dock**

> Planning update: the dock's native glass (acrylic/mica/blur) covers its whole window, so
> growing the bar window outward would show glass in the "empty" strip. Instead the magnified
> icons are drawn in a small transparent, click-through window (`magnify`) over the launcher;
> the dock only spreads its icon slots. The glass already has free space around the centred
> widgets, so the bar window only grows in length when the spread wouldn't fit. The bullets
> below are kept for the intent (strip unchanged, nothing clickable added, works with
> auto-hide and popups).

- The bar window is only as thick as the bar. While magnifying, the bar window grows
  **outward** (away from the screen edge) by enough to fit the largest icon, and **along the
  edge** by the launcher's extra length, and shrinks back after the icons settle. The **reserved AppBar strip does not change** (`place()` strip
  stays the same), so other windows don't move.
- The added strip must look empty: no glass, no blur, no tint, no shadow, nothing clickable
  except the magnified icons. If native blur/backdrop is on, its area must stay limited to the
  real bar rectangle while grown (for example a window region or limiting the DWM backdrop
  rect). **Builder must check this on the native-blur materials and with auto-hide on.**
- The growth is driven by a new command (e.g. `bar_magnify(outward_px, along_px)`) handled
  in `bar.rs`, which reuses `bar_rect()`: the outward side moves out by `outward_px`, both
  ends move out by `along_px / 2`. Only the real bar area (normal thickness, grown length)
  gets glass. It must cooperate with
  auto-hide (`watch_autohide`, `SLIDE`), popups positioned from the bar rect
  (`popup_rect`), and `RESERVED_AT` (Windows shoving the bar back). While grown, the hover
  zone of auto-hide must not treat the transparent strip as "pointer left the bar" in a way
  that hides it, nor keep it shown when the pointer is truly gone.
- Hover previews (`__windows:` popup) still open after the usual delay, positioned against
  the magnified icon.

## 2. Launch bounce

- When the launcher starts an app (`launch()` in `Pill.svelte`: click on a closed app,
  middle-click, "New window"), its icon bounces: a vertical hop outward from the edge,
  repeating, roughly macOS rhythm (about 0.6 s per hop).
- It stops when a window for that group appears in the window list (`appsChanged`), or
  after **10 seconds**, or if launch fails (the existing red "failed" state shows instead).
- "New window" on an already-running app bounces too, until a new window id appears.
- **Animate opening applications** off: no bounce.
- **Low mode**: no bounce; the running dot blinks instead until the window appears.
- Pure front end, CSS animation on the icon. Must pause under `data-away` like other
  animations.

## 3. Minimize / restore effects

**Detecting**
- `platform/windows/windows_list.rs` already hooks `EVENT_SYSTEM_MINIMIZESTART..END`.
  Add a platform event (behind the `platform/` trait like the rest) that reports
  `{ hwnd, kind: minimize|restore, rect }` where `rect` is the window's on-screen rect just
  before minimizing (or the restore target rect).

**Which settings apply**
- The **first App Launcher in the active profile** supplies the effect settings (builder's
  call, settled while writing this spec). No App Launcher at all: Genie, flying to the middle
  of the dock; "into application icon" doesn't apply.

**Where it flies to**
- The Rust side asks the bar for the target rect (the app's icon in the launcher on the
  screen where the window is, or that window's own spot at the end of the launcher), or the
  bar pushes the current icon rects whenever they change. Builder's call; keep it simple.
- No App Launcher: fly to the middle of the dock.
- All displays off and the window is on a screen without a dock: it flies across to the main
  dock (as macOS does), even across monitors with different scaling.
- Dock auto-hidden at that moment: fly to the dock's edge position (where it would appear).

**Drawing**
- An overlay window (one spanning the monitors involved, or one per monitor with hand-off;
  builder's call): transparent, topmost, click-through
  (`WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE`, not in taskbar/Alt-Tab), created on
  demand and hidden when idle.
- The window picture is a DWM thumbnail of the source window
  (`DwmRegisterThumbnail`, as in `thumbs.rs`). DWM keeps a minimized window's last frame, so
  the picture is available both ways.
- **Scale**: one thumbnail whose destination rect shrinks from the window rect to the target
  rect, ~250 ms, ease-in.
- **Genie**: the source cut into N horizontal strips (about 24–40; N horizontal for a
  bottom/top dock, vertical strips for left/right). Each strip is its own thumbnail with
  `rcSource` = that slice and `rcDestination` interpolated so the window's sides curve in
  toward the icon (the edge nearer the dock reaches the icon's width first), then slides
  into the icon. ~450 ms. Strip count is a tuning knob.
- **Suck**: like Genie but pulled toward one corner of the icon, faster and more abrupt
  (~300 ms). Builder's call on the exact curve; it should read as different from Genie.
- **Restore**: the same effect played in reverse, from the icon to the window's restore
  rect, **only when GlassBar starts the restore** (click on the app icon or its minimized
  spot): GlassBar plays the effect first, then restores the window, so nothing needs hiding.
  Restores started elsewhere (Alt-Tab, the Windows taskbar, the app itself) play no effect.
  GlassBar never hides or restyles another app's window; the only change it may make is the
  per-window transitions attribute below.
- Animation driven from Rust at the display's refresh (a timer around 60–120 Hz calling
  `DwmUpdateThumbnailProperties`), or from the overlay webview. Builder's call; Rust is
  expected to be simpler and avoids a fourth webview.
- **Native animation**: try `DwmSetWindowAttribute(hwnd, DWMWA_TRANSITIONS_FORCEDISABLED)`
  on the source window when minimize starts (and re-enable after). If Windows refuses for
  other processes' windows, accept the overlap and **note it in the CHANGELOG and to the
  owner**. Never change the system-wide animation setting.
- **Low mode**: always Scale, whatever is chosen.
- Must clean up: every thumbnail unregistered and the overlay hidden when an effect ends,
  when the source window is destroyed, or when a new effect for the same window starts.

**Minimize windows into application icon: off**
- Minimized windows (the existing `AppWindow.minimized` flag) get their own spot at the
  **end** of the launcher, after a thin divider (like the existing `.sep`), one spot per
  minimized window, in the order they were minimized.
- Each spot shows a **live picture** of the window (DWM thumbnail drawn into the bar window
  at the spot's rect, reusing `thumbs.rs`), with the small app icon in a corner.
- Click restores the window (existing `appActivate`); the spot then goes away. Right-click
  offers Close window.
- The app's own icon keeps its running dot while it has minimized windows.
- The minimize effect flies into the spot. Restore flies out of it.
- Bar thumbnails must follow the spot when the row moves (magnification, reorder, "+N"
  expand). If keeping a live thumbnail in sync costs too much, the fallback is a static
  picture refreshed on hover; tell the owner if the fallback is used.

## 4. Dock on every screen

- With **All displays** on, every connected monitor gets a full dock window with the same
  widgets, same profile, same theme. The existing window labelled `bar` stays the **main**
  dock (on `monitor`, or the primary when null); extra docks are new windows (e.g.
  `bar-1`, `bar-2`…) on the other monitors.
- Each dock is placed with `place()` on its own monitor, reserves its own strip when
  "Reserve space" is on, and auto-hides on its own (each reveals from its own screen edge).
  `appbar.rs`'s reservation owner window is currently single; it must become one per
  monitor.
- **Popups** open next to the dock that asked for them, on that dock's monitor (the popup
  commands must know which bar window called; Tauri commands can take the calling
  `WebviewWindow`).
- **Side effects only on the main dock**: sounds (`sound:play`), dock alerts
  (`dock:alert`), `charger:flip`, reminders and anything else that notifies stays targeted
  at `bar`. Extra docks render widgets only. **Builder must audit front-end widgets for
  timers that notify or play sound on their own** (solat, water, pomodoro, timer,
  notifications, companion, etc.) and gate them on "is main dock". Global keyboard shortcuts
  register once, from the main dock.
- Apart from that, extra docks run every widget in full (no cut-down versions; builder's
  call). Their cost shows on the Performance page; Low mode lightens them like the main
  dock.
- Events that change what the dock shows (`bar:away` per window, edit mode, config changes,
  `shelf:dropped`, `bar:jump`) go to the right dock(s). Edit mode applies to all docks at
  once (same config).
- The App Launcher on every dock shows **all** apps (like Windows' "show taskbar buttons on
  all taskbars"). The minimize effect targets the launcher on the screen the window is on.
- Monitors plugged in or removed: docks are added or removed to match, without restarting.
- Turning All displays off closes the extra docks and releases their reserved space.
- Performance: each extra dock is another WebView2. The Performance page should keep
  counting the whole process tree correctly (it already counts children).

## Testing

- `npm test`, `npm run check`, `cargo test` (in `src-tauri`) stay green.
- Unit tests for the pure parts: the magnification bell / scale per icon, the bar's
  grown-rect calculation (strip unchanged, outward growth for every edge), the Genie strip
  geometry at a few `t` values (first and last frame equal the window rect and the target
  rect), the bounce stop rule, the minimized-spot ordering, and extra-dock placement per
  monitor.
- Live check on a release build (`npx tauri build --no-bundle`) with CDP
  (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`), stopping the
  installed copy at `C:\Users\msole\AppData\Local\GlassBar\glassbar.exe` first and restarting
  it after. Each feature checked with its setting on and off, in Low and Normal mode, on a
  bottom dock at least.
- Multi-monitor needs a second screen. If none is attached during the build, say so plainly
  and leave that live check to the owner.

## Delivery

- Commit on `dock-effects`. Don't push a tag or release until the owner says so; at release
  time add the CHANGELOG entry and version bump in `package.json`, `src-tauri/Cargo.toml`
  and `tauri.conf.json`.

## What looks like macOS, and what won't

- **Same feel**: magnification wave, popping out above the dock, launch bounce, Scale,
  restore-from-icon, minimized-window spots with live pictures.
- **Close, not identical**: Genie is built from straight strips, so very close up the curve
  is slightly stepped. Suck is our own interpretation.
- **May differ**: if Windows won't let GlassBar switch off its own minimize animation per
  window, a short native shrink plays underneath ours.
- **Won't animate**: windows of apps running as administrator (when GlassBar isn't), and a
  few system windows, minimize without our effect.
