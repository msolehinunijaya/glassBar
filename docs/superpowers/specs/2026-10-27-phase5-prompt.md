# GlassBar Phase 5 — dock polish, dock alerts, two new widgets (build prompt)

> Paste this whole file into a new session as the task. It was written against branch `phase2`
> at `34b4cb3` (v0.3.0). Every path below exists today unless it says **new**.

## Before you start

- Read `docs/DESIGN_RULES.md`, `docs/WIDGET_SDK.md`, `docs/ARCHITECTURE.md` and
  `docs/superpowers/specs/2026-10-27-phase4-dock-expression-design.md`. Phase 4's anti-goals still
  apply: no extra idle CPU, no dock that jumps, no unmounting widgets mid-animation, no silent
  network calls, no fake toggles.
- Reuse what exists before writing anything new. The pieces this work leans on:
  | Need | Already here |
  |---|---|
  | Whole-dock takeover with fill + timing | charger flash in `src/app/Bar.svelte` + `src/app/chargeFlash.ts` |
  | Reveal an auto-hidden dock for a few seconds | `HOLD_UNTIL` + `reveal()` in `src-tauri/src/bar.rs` (`jump_to`) |
  | Cursor position / "is the pointer over X" in Rust | `shell().cursor_pos()`, `autohide::contains()` |
  | Hover-opened panels | `src/lib/hoverPanel.ts`, `popup_leave_close` in `bar.rs` |
  | Expand/collapse with "+N" | App Launcher `src/widgets/apps/Pill.svelte` |
  | Seek the playing track | `ipc.mediaSeek()` |
  | Lyrics lines + current index | `src/widgets/music/lyrics.ts`, `lyrics.svelte.ts` |
  | Rings, liquid fill, sparkline | `src/lib/Ring.svelte`, `src/lib/liquid.ts`, `src/lib/Sparkline.svelte` |
  | Daily data that survives a restart | `src-tauri/src/db.rs` (SQLite), `src-tauri/src/scheduler.rs` |
  | App icons, pins, file picker | `src/widgets/apps/*` |
- No new npm or cargo dependency unless a spike proves there is no other way.
- Every new setting has a default, and configs saved before it existed must load as that default
  (see the `lyricsOf()` / `partsOf()` pattern in `src/widgets/music/settings.ts`).

## Motion rules for everything in this phase

| Moment | Duration | Easing |
|---|---|---|
| Something appears (panel, alert, expanded row) | 160–200 ms, scale 0.96→1 + fade | `--ease` |
| Something leaves | 120–160 ms fade | ease-in |
| A size change (width grows/shrinks, row expands) | 220–280 ms on `width`/`transform` | `--spring` |
| Items arriving in a row (shelf, folder grid) | 30–40 ms stagger per item, max 6 staggered | `--ease` |
| A value changing (%, ml) | number ticks, fill animates 300 ms | `--ease` |

- Animate only `transform`, `opacity`, and `width` on the one element that resizes. Neighbours
  slide with the dock's existing `animate:flip`.
- `prefers-reduced-motion` and `data-motion='reduced'`: everything appears/disappears instantly,
  timeouts (not `transitionend`) own every schedule, as the charger flash already does.
- Hover timings are the same everywhere: open after **350 ms**, close **300 ms** after the pointer
  leaves (re-entering cancels the close). Put the numbers in one place and reuse them.
- Every control: hover, pressed (scale 0.96), focus-visible ring, disabled.

---

## Part A — Bugs to fix first (each its own commit)

### A1. Hover panel stays open after a right-click
**Now:** hover the Music widget → its panel opens. Right-click the dock or the widget → the panel
never closes by itself again.
**Do:** find the root cause first (likely the native context menu swallows the pointer-leave, so
`popup_leave_close` is never called). Then:
- A right-click anywhere on the dock closes any hover panel at once; a hover panel never opens
  while a context menu is up.
- Safety net in Rust: while a hover panel is open, check the cursor on the existing timer; if it
  has been outside both the panel and its widget for 300 ms, close it. This also catches any
  other lost leave event.
**Done when:** hover → right-click → Esc/click away → panel closes within ~300 ms. Repeat on every
widget that has a hover panel (Music, Capture menus, launcher window previews).

### A2. Scrolling the dock changes the volume
**Now:** `Bar.svelte` scrolls the dock on wheel, but when the pointer passes over the Sound part
(`SoundPart.svelte`, if its wheel option is on) or the Music volume button
(`PlayerPill.svelte` `wheelVolume`, always on) they grab the wheel and change the volume.
**Do:** one shared rule (`src/lib/wheelIntent.ts`, **new**, with a test): a wheel gesture belongs
to whatever it **started** on. A gesture = wheel events less than 500 ms apart. If it started on the
dock it keeps scrolling the dock even when it passes over a volume control. A volume control only
reacts to a gesture that started on it, after the pointer has rested on it ≥ 250 ms. Horizontal
wheel / trackpad sideways swipe never changes volume.
Give the Music volume button the same "Mouse wheel changes volume" option the Sound part has.
**Done when:** fast scroll across the whole dock never changes the volume; resting on the speaker
and scrolling still does.

### A3. Quick Settings tiles unreadable with a light accent (see screenshot: white accent)
**Now:** `.tile.on` hard-codes `color: #fff` (`src/styles/glass.css`), and the chevron button
(`.more`, beside Wi-Fi and Bluetooth) sits outside the tile so it keeps the normal text colour —
white on white.
**Do (root cause):** compute `--on-accent` in `src/lib/theme.ts` from the accent's luminance
(dark text `#1c1c1e` on light accents, white on dark). Use it for every surface filled with the
accent: `.tile.on` and its chevron, switches, primary buttons, slider thumbs, selected segments.
In the Liquid preset, give the chevron the same colour as the tile text
(`.tilewrap:has(.tile.on) .more`).
**Done when:** accent = white, yellow, light grey, black, default blue → every label, state and
chevron is readable (≥ 4.5:1), in dark and light, in both presets.

### A4. Tray icons don't collapse when the mouse leaves
**Now:** the tray icons inside Quick Settings (`src/widgets/quicksettings/Pill.svelte`,
`expanded`) only collapse on a click outside.
**Do:** collapse 600 ms after the pointer leaves the widget (re-entering cancels). Never collapse
while a tray icon's own menu is open or while the pointer is over a GlassBar panel opened from it.
Same rule for any other inline expansion in the dock (the Sound slider already does this —
match its timing). Panels opened by a **click** keep closing on click-outside/Esc, like Windows.
**Done when:** expand tray → move away → it slides closed smoothly; right-click a tray icon →
its menu stays usable.

---

## Part B — Dock alerts (one system, used by five things)

Today only the charger can take over the dock. Make that one reusable system instead of five
copies: **"Dock alerts"**.

**Design**
- One alert at a time, drawn over the widget row exactly like the charger flash (widgets stay
  mounted and fade). Layout: icon · big value · label · optional 1–2 action buttons · optional
  fill showing a level.
- Priority: **Prayer > Water > Charger > Volume/Brightness**. A higher one replaces a lower one
  (the lower one is dropped, not queued — stale volume info is useless). A lower one never
  interrupts a higher one. The same kind arriving again **updates in place** and restarts its hold
  (pressing volume-up 5 times = one alert whose number counts up, not five).
- Hover pauses the hold; leaving resumes it. Alerts with buttons hold 8 s, others ~2.5 s
  (volume/brightness: 1.2 s after the last change).
- **Auto-hidden dock:** the dock slides in (existing reveal animation), shows the alert, then
  slides back out — unless the pointer is on it. Use `HOLD_UNTIL` + `reveal()`.
- Not over full-screen apps (games, slideshows, videos), except Prayer.
- `role="status"`, `aria-live="polite"`.
- Dragging a widget: no alert (already the rule for the charger).

**Kinds**
| Kind | Shows | Fires when |
|---|---|---|
| Charger | (existing) | unchanged — just moved onto the new system |
| Volume | speaker icon, `48%` or "Muted", fill = level | volume/mute changed **outside GlassBar**: keyboard keys, Windows' flyout, another app. Changes made from GlassBar's own slider/wheel don't fire it (ignore our own echo for ~500 ms). |
| Brightness | sun icon, `70%`, fill = level | built-in screen brightness changed outside GlassBar (WMI brightness event). External monitors send no event: skip them, say so in the setting's hint. |
| Prayer | crescent icon, "Zohor", time, button **Stop adhan** while it plays | at prayer time, same moment as the adhan; not Syuruk (sunrise) |
| Water | drop icon, "Time to drink · 750 / 2,000 ml", fill = today's progress, buttons **+250 ml** and **Later** | water reminder (Part D2) |

**Settings › Dock alerts (new section)** — move the charger-flash toggle here from Sounds (keep
the `chargerFlash` config key). One switch per kind, "Show when the dock is hidden" (on), length
Short / Normal / Long, and a **Test** button per kind (also how you test without a real charger or
waiting for prayer time). Don't duplicate anything already in Sounds or Notifications.

Windows' own volume/brightness popup still appears; hiding it needs an undocumented hack — out
of scope unless asked.

---

## Part C — Widget changes

### C1. Music: lyrics on every design, two-part pill, lyrics in the panel
**Dock**
- Lyrics work on all four designs (Player, Classic, Mini, Custom). Remove the style gate
  (`Pill.svelte` only passes lyrics to `PlayerPill`). The on/off switch stays in the music panel's
  Lyrics options. Side docks still hide the line (no room) but the panel still shows lyrics.
- One widget, two parts — like Quick Settings' Wi-Fi and Sound parts: `[player] │ [lyric]` in
  one outline with a hairline divider. Lyric side (left/right) stays a setting.
- Long line: first try one size smaller (12→11), then wrap to **2 lines** inside the lyric part's
  width (`maxWidth`), then clamp with a fade. Compact dock height: 1 line. The part never grows
  past `maxWidth`, so the dock doesn't jump between lines.
- No lyrics for this track → the lyric part shrinks away (width animation), not an empty box.
  Instrumental gap longer than 5 s → three soft dots instead of a stale line.
- Click the lyric part → opens the panel at the lyrics.

**Panel (hover/click)**
- When the track has lyrics the panel widens (≈ 620 px): player on the left, lyrics on the right.
  No lyrics → today's 320 px panel, unchanged.
- Lyrics list: current line centred, larger, semibold; past lines secondary; next lines tertiary.
  It follows the song with a smooth scroll.
- The user can scroll freely; auto-follow pauses for 4 s, then a **Back to now** chip appears
  (click = jump back). Click any line → `mediaSeek` to its time; if the app can't seek, lines
  aren't clickable and the tooltip says why. Keyboard: ↑/↓ move, Enter seeks.
- Footer: source ("LRCLIB" / "Local .lrc file").

### C2. Equalizer: shrink when silent, radial fits itself
- After **2 s** of silence the pill shrinks to a small square (dock thickness) showing the idle
  look; sound returns → it grows back to its length within ~200 ms. The 2 s delay stops it pumping
  between songs. Setting "Shrink when silent" (on). With idle look "Hide", it shrinks to nothing.
- Radial look: the pill becomes a square sized to the dock thickness (side dock: to the side
  width), and the ring's radius follows that box. Hide the Length slider for Radial (with a hint).
  Re-size the canvas for device pixel ratio on every size change so it never blurs.
- Shrinking must not wake the capture loop: same idle/battery rules as commit `0b79c31`.

### C3. File Shelf: one file + "+N", expand on hover
- At rest: the newest file's icon + a `+N` badge (no badge for 1 file).
- Hover 350 ms → expands, files slide out with a stagger (same feel as the launcher's +N).
  Leave → collapses 300 ms later.
- Never collapse during a drag in or out, or while its right-click menu is open. Dragging a file
  over the collapsed shelf expands it and shows where it will land.
- Setting "Collapse to one file" (on).

### C4. Quick Settings: split tiles + four more
- Every tile with more behind it is a **split tile**: toggle area │ 1 px divider │ 32 px chevron.
  Same look on and off (A3 fixes the colour). Wi-Fi, Bluetooth, Hotspot, Nearby sharing, Night
  light, Cast.
- **Hotspot** chevron → `ms-settings:network-mobilehotspot`.
- **Nearby sharing** (new): tile toggles Off ↔ My devices only (last used on-mode); state line
  shows Off / My devices / Everyone nearby. Chevron → `ms-settings:crossdevice`.
- **Night light** (new): tile toggles it; chevron → `ms-settings:nightlight`.
- **Cast** (new): tile opens a sub-view like Wi-Fi's: nearby wireless displays, each with
  Connect / Disconnect and live status (Searching… / Connecting… / Connected); when connected, a
  switch **Allow touch, mouse and keyboard from the TV/display**; footer link "Open Windows Cast".
- **Edit tiles** in the widget settings: choose and order tiles (7 is a lot for 360 px).

**Spike these three first — none has a public Windows API:**
- Night light: the CloudStore registry blob (`…bluelightreductionstate`) is the known way.
- Nearby sharing: `HKCU\Software\Microsoft\Windows\CurrentVersion\CDP` policy values.
- Cast: finding Miracast displays works from a desktop app (DeviceWatcher with the projection
  selector); **starting** a cast to a chosen display from a desktop app is not documented, and the
  touch-input switch may only exist in Windows' own Cast flyout.

For each: if the spike works reliably on Windows 11 (state reads back correctly, change applies
live), build it. If not, the tile shows the real state where it can and opens the Windows page
instead of toggling — and says so in its tooltip. Never show a toggle that lies. Report the
spike results before building on them.

---

## Part D — New widgets

### D1. App Folders
Phone-style folders on the dock.
- Dock: each folder is one tile showing a 2×2 grid of its first four app icons; a dot under it if
  any app inside is running. Optional name under/beside it; empty name = no label. Widget option
  "Show folder names".
- Click (or hover, option, off by default) → flyout grid, 4 columns: icon, name, running dot.
  Click launches/focuses (same behaviour as the launcher). Drag to reorder; drag out of the grid
  to remove; **+ Add app** (reuse the launcher's picker); drop `.exe` / `.lnk` from Explorer onto
  the folder to add. Click the title to rename inline.
- Several folders per widget; **+ New folder** in the widget menu. Deleting a folder asks first.
- Stored in widget settings like launcher pins (no DB table). Reuse `src/widgets/apps/` icons,
  `Pin`, `keyFor()` — don't copy them.

### D2. Water reminder
- **Dock:** a drop that fills with today's progress (reuse `liquid.ts`) + `750 ml` (or `38 %`,
  option). Side dock: drop + value tile. Click the drop → panel.
- **Panel:** progress ring with today / goal; quick-add buttons (cup sizes, default 150 / 250 /
  500 ml, editable); **Undo** right after adding; today's log (time, amount, remove); last 7 days
  as small bars; footer "Goal resets at midnight".
- **Goal:** Fixed (ml) or **Dynamic** = weight × 35 ml/kg, +10 % when the Weather widget's
  temperature is ≥ 30 °C (only if weather data exists; never guess). Unit ml or fl oz.
- **Reminders** (run in Rust via `scheduler.rs`, so they fire with every panel closed): on/off,
  every 30–120 min, active hours (default 08:00–22:00), skip if you logged in the last 20 min,
  quiet in Do Not Disturb. Shows a Dock alert (Part B) with **+250 ml** (uses your default cup)
  and **Later** (15 min). Goal reached → one short celebratory alert, once per day.
- **Data:** daily log in SQLite (`db.rs`), local-date rollover at midnight, survives restarts.

---

## Part E — Settings › Bar: space from the edge, two values

**Now:** one "Space from the edge" (0–48 px), used on both sides of the dock
(`place()` in `src-tauri/src/bar.rs`: strip = thickness + 2 × gap).
**Do:** two sliders for every edge, labels naming the real sides of the current edge:
| Edge | Slider 1 (screen edge) | Slider 2 (windows side) |
|---|---|---|
| Bottom | Space below the dock | Space above the dock |
| Top | Space above the dock | Space below the dock |
| Left | Space left of the dock | Space right of the dock |
| Right | Space right of the dock | Space left of the dock |
- Config `gapOuter` + `gapInner`; an old `gap` loads into both. Reserved screen space =
  thickness + outer + inner. Auto-hide peek and popup placement keep working on every edge.
- Live preview while dragging the slider (like Length), saved on release.
- Rust unit tests for `place()` with unequal gaps on all four edges.

---

## Order of work

A1 → A2 → A3 → A4 → B (charger moved first, then volume/brightness, then prayer) → C1 → C2 →
C3 → E → C4 (spikes first) → D1 → D2. One commit per item, each shippable on its own.

## Verification (do all of it, report honestly what you could not test)

Automated, after every item: `npm test`, `npm run check`, `cargo test`. New pure logic gets one
small test each: wheel intent, alert priority/replace/update-in-place, lyric fit (shrink → wrap →
clamp), equalizer silence hysteresis, shelf collapse state, folder add/remove/reorder, water goal +
midnight rollover + reminder skip rules, `place()` with two gaps, `--on-accent` contrast.

Manual, with `npm run tauri dev` — drive it yourself and take screenshots where you can:
- All four dock edges; auto-hide on and off; compact / normal / large; side widths.
- Dark and light; Default and Liquid presets; accents white, yellow, black, blue.
- Display scaling 100 %, 125 %, 150 % (125 % has broken things twice before).
- Reduced motion on: nothing animates, nothing breaks.
- Idle CPU with the dock idle is not higher than before (Task Manager, 30 s).
- Every animation in this phase: smooth, no jump of neighbouring widgets, no flash of empty space.

Needs a person (list these in the report with exact steps): real charger, keyboard volume /
brightness keys, a Miracast display, Nearby sharing with a second PC, prayer time (use the Test
buttons), water reminder over a real interval.

## Decisions taken (say if you want any changed)

1. Five takeovers share **one** Dock alerts system with priorities, not five separate flashes.
2. Folders are a **new widget** (as asked), reusing the launcher's code — not folders inside the
   launcher.
3. Click-opened panels still close on click-outside; only hover panels and inline expansions
   close when the mouse leaves.
4. Volume/brightness alerts fire only for changes made **outside** GlassBar.
5. No alerts over full-screen apps, except Prayer.
6. Night light, Nearby sharing and Cast are built only as far as a spike proves they work;
   otherwise they open the Windows page.
