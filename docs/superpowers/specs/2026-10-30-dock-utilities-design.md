# Dock utilities: design

Date: 2026-10-30 · Branch: `phase2` (on top of the uncommitted Mirror widget, v0.6.7) · Status: building

## Why

The owner wants eight small, everyday tools on the dock, each placed where it already belongs
instead of as eight new pills:

| # | Tool | Where it lives |
|---|---|---|
| 1 | Recycle Bin | new widget **Recycle Bin** (`trash`) |
| 2 | Keep Awake | Quick Settings tile |
| 3 | Mic mute + "on air" light | Quick Settings tile, and a light on the Quick Settings pill |
| 4 | Bluetooth battery | Quick Settings panel and Bluetooth page |
| 5 | Countdowns | new widget **Countdowns** (`countdown`), dot style from Time Progress |
| 6 | Colour picker | fifth Capture button (`color`), frozen-screen overlay from OCR Snap |
| 7 | Hijri date + Tasbih | Waktu Solat panel, and a Tasbih segment joined to the prayer pill |
| 8 | Inline calculator | Start & Search: the Search button opens a GlassBar search panel |

Everything follows `docs/DESIGN_RULES.md` (tokens, 40 px rows, one primary button per view,
Lucide icons, tabular numbers, hover/pressed/focus/disabled states, light and dark).

## Goals

- Each tool is one click (or one hover) from the dock and does its job without a window.
- No new pill unless the tool is a place of its own (the bin, countdowns). The rest join a
  widget that already shows that kind of thing.
- Every tool reads the real state from Windows and never shows a guessed value.
- Every on-dock addition can be switched off.

## Non-goals

- Restoring files that were deleted permanently, or deleting anything permanently without
  Windows' own confirmation.
- A camera off switch: Windows has no system-wide camera mute. The panel shows which apps use
  the camera and links to its privacy page.
- Moon-sighting accuracy for Hijri dates. JAKIM's date is used when it is there; otherwise
  the Umm al-Qura calculation, marked as an estimate.
- A full launcher (files, settings, web results inline). The search panel does maths, units,
  currency and installed apps; everything else is handed to Windows Search or the browser.
- A global hotkey for the mic. Add it when someone asks.

## Anti-goals (a failure even if it works)

- **Losing files.** The bin never deletes without a confirm, a drop on the bin only ever
  *recycles*, and Restore only acts on the exact item clicked.
- **A PC that never sleeps.** Keep Awake ends on its timer, on Quit and on a crash (the power
  request dies with the process). It is never restored on the next start.
- **Typing a tasbih count into config.json on every tap.** Counts live in the database's `kv`
  table; config only holds the target and on/off switches.
- **A search box that runs code.** The calculator is a parser, never `eval`.
- **Dock clutter.** New dock marks appear only when they carry news: a muted or in-use mic, an
  active Keep Awake, a full bin.

---

## 1. Recycle Bin widget (`trash`)

### Compact card (dock)
- A 3D trash can in the house style of the To-Do and Notes icons (`lib/icons/TrashIcon3D.svelte`):
  glass-grey body, ribbed, a lid.
  - **Empty:** clean can, lid closed.
  - **Full:** crumpled paper peeks over the rim.
  - **Drop target:** while files are dragged over it the lid lifts (8°) and the can glows
    with the accent; on drop the lid closes with the dock's spring and the can does a small
    bounce.
- Sizes: **small** = icon only; **medium** = icon + size ("2.4 GB"), or "Empty".
- Tooltip: "Recycle Bin · 1,204 items · 2.4 GB".
- Left/right dock: one `.side` tile, icon + `.side-c` size.

### Interactions
| Action | Result |
|---|---|
| Click | Panel (below) |
| Right-click | Menu: *Open Recycle Bin* · *Empty Recycle Bin…* (Windows' own confirm dialog) · *Restore last deleted* (only when there is one) |
| Drag files from Explorer onto it | Each is **moved to the Recycle Bin** (Windows' FOF_WANTNUKEWARNING: if it is too big to recycle Windows asks first). A short dock-style card says "Moved 3 items to the Recycle Bin · Undo". Undo restores those items. |
| Double-click | Open Recycle Bin in Explorer |

### Panel (380 × fit)
```
┌ Recycle Bin                         [⟳] [📂] ┐
│  2.4 GB                                       │  hero, 32 px semibold
│  1,204 items · would free 4% of C:            │  secondary
│  [ Empty Recycle Bin ]  (danger outline)      │
│ ───────────────────────────────────────────── │
│  Recently deleted                             │
│  📄 report-final.docx        2 min ago  [↺]   │  40 px rows: icon, name,
│     from Documents › Work                     │  from-folder (tertiary),
│  🖼 IMG_2033.jpg             1 h ago    [↺]   │  when, Restore on hover
│  … up to 8                                    │
│ ───────────────────────────────────────────── │
│  C: 2.1 GB · 980 items         [Empty]        │  per drive, only with >1 drive
│  D: 300 MB · 224 items         [Empty]        │
└───────────────────────────────────────────────┘
```
- Empty asks inline (the existing confirm card: Cancel focused, "Delete permanently" red).
- Restore puts the item back where it came from (Windows' "Restore" verb) and the row slides
  out. If the original folder is gone Windows recreates it, as Explorer does.
- Loading: skeleton rows. Error: Windows' reason, dimmed, never "0 B".
- The panel reuses `sysmon/recycle.ts` (`binView`, `confirmText`, `freeShare`).

### Live state
- The bin folders (`X:\$Recycle.Bin\<user SID>` on each fixed drive) are watched with Windows
  change notifications. A change (debounced 1.5 s) drops the cached count and emits
  `recycle:changed`, so deleting in Explorer updates the dock within ~2 s, with no polling.
- The count itself stays the existing 50 s shared cache (`recycle_info`).

---

## 2–4. Quick Settings additions

New tile keys: `awake`, `mic` (both in *Edit tiles*, shown by default). The tile grid stays
2 columns; new tiles go after Night light and before Cast.

### 2. Keep Awake tile
```
┌───────────────┬─┐
│ ☕ Keep awake  │›│   off:  "Off"
│ On · 1:24 left │ │   on:   "On · 1:24 left" / "On · until you turn it off"
└───────────────┴─┘
```
- Click toggles with the last chosen duration (default *Until turned off*).
- **›** opens a sub-page (same pattern as Wi-Fi / Bluetooth):
  - Duration choices as 40 px rows with a check: 30 min · 1 hour · 2 hours · 4 hours · Until I
    turn it off.
  - Switch: **Keep the screen on** (default on). Off = the PC stays awake but the screen may
    turn off (good for downloads).
  - Footer: "Ends when GlassBar closes."
- On the dock: while active a small coffee cup appears in the Quick Settings pill (after the
  radios), tooltip "Keep awake · 1:24 left". Click on it turns it off.
- Rust: `PowerCreateRequest` + `PowerSetRequest` (System, and Display when the screen stays
  on), reason "GlassBar: Keep awake" (visible in `powercfg /requests`). A timer ends it.
  State is in memory only and goes with the process.

### 3. Microphone tile and the on-air light
```
┌───────────────┬─┐
│ 🎙 Microphone  │›│   "On" · "On · Zoom" · "Muted" (red tint on the tile)
└───────────────┴─┘
```
- Click mutes / unmutes the **default recording device** (Core Audio capture endpoint).
- **›** sub-page:
  - **Input level** meter (live while the page is open: "Speak to test").
  - **Mic volume** slider.
  - **Input device** list (choose the default recording device).
  - **In use now:** apps using the microphone, and apps using the camera, each with its icon
    and "since 10:42". Empty: "No app is using your microphone or camera."
  - Footer link: *Microphone privacy settings*.
- **On-air light on the dock** (Quick Settings pill, before Wi-Fi):
  - Mic in use: an orange dot with the mic glyph (iOS's convention); camera in use: a green
    dot with the camera glyph. Both can show.
  - Mic muted: a red mic-off glyph, always, so a muted mic is never forgotten.
  - Click the mic glyph = mute/unmute (doesn't open the panel). Tooltip lists the apps.
  - Setting `micOnDock`: **Automatic** (only when muted or in use, default) · Always · Never.
- In-use detection: Windows' capability-access store
  (`HKCU\…\CapabilityAccessManager\ConsentStore\{microphone,webcam}`, packaged and
  `NonPackaged` apps; in use = `LastUsedTimeStop` is 0). Watched with
  `RegNotifyChangeKeyValue` (subtree), not polled. Mute changes come from Core Audio's
  endpoint callback, so the mute key on a headset updates the dock too.

### 4. Bluetooth battery
- **Main page:** a "Devices" section after the tiles, only when a connected device reports a
  level:
```
  Devices
  🎧 Sol S25            ▮▮▮▮▯▯▯▯▯▯  20%
  🔊 MI Portable Speaker ▮▮▮▮▯▯▯▯▯▯  40%
```
  40 px rows; the bar is green, amber at ≤ 20 %, red at ≤ 10 %; tabular %.
- **Bluetooth page:** each paired, connected device gets a battery chip next to its name.
- Source: the battery level Windows' Bluetooth stack publishes on the device node (the
  Hands-Free profile, and LE devices that report it), read through the Configuration Manager
  API and matched to the paired device by its Bluetooth address. Devices that report nothing
  show no chip (never a made-up value).
- **Low battery:** once a device drops to 15 % or below, one notification "Sol S25 battery low ·
  15%" (once per device until it is charged above 30 %). Checked every 2 minutes while Quick
  Settings is on the dock.

---

## 5. Countdowns widget (`countdown`)

### Data (widget settings; small, user-made, travels with config export)
```ts
interface Countdown { id: string; name: string; date: string /* YYYY-MM-DD */; time: string | null /* HH:MM */;
  emoji: string; repeat: 'none' | 'yearly' | 'hijri' | 'chinese'; preset: string | null; created: number;
  remindDayBefore: boolean }
interface CountdownSettings { items: Countdown[]; pinned: string | null /* null = nearest */; notify: boolean }
```
- Presets compute their next date in the browser's calendars (`Intl`):
  - Hijri (`islamic-umalqura`): **Hari Raya Aidilfitri** (1 Syawal), **Hari Raya Haji**
    (10 Zulhijjah), **Ramadan** (1 Ramadan), **Awal Muharram** (1 Muharram), **Maulidur Rasul**
    (12 Rabiulawal). Marked "estimate · ±1 day".
  - Chinese (`chinese`): **Tahun Baru Cina**.
  - Fixed: **Merdeka** (31 Aug), **Hari Malaysia** (16 Sep), **New Year** (1 Jan),
    **Christmas** (25 Dec).
  - Custom: any name and date ("Project deadline").
- Repeating ones roll over to the next occurrence the day after.

### Compact card
- small: emoji + "132d" · medium: emoji, name above "132 days" (stacked like the prayer pill).
- Last 24 h: "5h 20m"; last hour: "42m". On the day: "Today" with a soft accent glow
  (no flashing; it isn't urgent). After the day (non-repeating): "Done" dimmed until removed.
- A thin progress ring around the emoji: from when it was added to the target.
- Shows the pinned countdown, else the nearest one. Empty: hourglass + "Add".

### Panel (340 × fit)
```
┌ Countdowns                               [+] ┐
│ ┌──────────────────────────────────────────┐ │
│ │ 🌙 Hari Raya Aidilfitri         ★ pinned │ │  card per countdown
│ │ Wed, 10 Mar 2027 · estimate ±1 day       │ │
│ │ 132 days                 ●●●●●●○○○○○○○○ │ │  hero + dot row (Time Progress dots)
│ │ 18 weeks 6 days                          │ │
│ └──────────────────────────────────────────┘ │
│ ┌ 🇲🇾 Merdeka …                             ┐ │
└──────────────────────────────────────────────┘
```
- Click a card = pin it to the dock (★). Hover shows *Edit* and *Delete* (ghost icons).
- Sorted by how soon. Done ones go last, dimmed, with "Remove".
- **Add / edit** (inline, replaces the list; Back returns):
  - "Start from" preset chips (one tap fills name, emoji, date and repeat).
  - Name · Date (native date input) · Time (optional) · Emoji (8 choices) ·
    Repeat every year · Remind me a day before.
  - One primary button: **Save**.
- Empty state: the preset chips and "or make your own".

### Alerts
- With `notify` on (default), Rust adds due items from the widget settings: on the day at the
  set time, else 09:00, "Merdeka is today"; and, if chosen, a day before at 09:00
  "Merdeka is tomorrow". Normal banners (source `countdown`, mutable in Notifications).

---

## 6. Colour picker (Capture)

- A **fifth Capture button** `color` (Pipette icon; tile gradient teal → green). It joins the
  saved button order at the end automatically, like a new tile.
- **Hotkey** `Super+Shift+C` (changeable in Capture settings), falls back to `Control+Alt+C`
  when taken (PowerToys uses Win+Shift+C), like the OCR hotkey.
- **Hover menu:** *Pick a colour* (primary) + the last 8 swatches (click = copy).
- **Overlay** (the OCR Snap window, purpose `color`, no dimming):
```
            ╭───────────╮
            │ ▦▦▦▦▦▦▦▦▦ │   loupe: 11 × 11 pixels at 10×, circle, the centre pixel boxed
            │ ▦▦▦▦▣▦▦▦▦ │
            │ ▦▦▦▦▦▦▦▦▦ │
            ╰───────────╯
            [■ #3B82F6 · 59 130 246]   chip under the loupe, swatch + value
```
  - The loupe follows the pointer, flips side near screen edges.
  - Arrow keys move 1 px (Shift: 10 px); wheel changes zoom (6–20×).
  - Click or Enter picks; Esc or right-click cancels.
  - Hint at the top: "Click to copy a colour · arrows to nudge · Esc to cancel".
- **After a pick:** the value is copied in the chosen format and the feedback card shows a
  big swatch, "Colour copied", and chips **HEX · RGB · HSL** (click copies that one).
- **Colour tab** in the Capture panel: *Pick a colour* (primary) · format segmented control
  (HEX `#3B82F6` · RGB `rgb(59, 130, 246)` · HSL `hsl(217, 91%, 60%)`) · history grid of the
  last 16 swatches (click = copy, right-click = remove, "Clear").
- History lives in the widget settings (`colors`, max 16 hex strings).

---

## 7. Hijri date and Tasbih (Waktu Solat)

### Hijri date
- Panel: a hero under the header: **12 Jamadilawal 1448 H** (16 px semibold) and the
  Gregorian date below (secondary). Malay month names: Muharram, Safar, Rabiulawal,
  Rabiulakhir, Jamadilawal, Jamadilakhir, Rejab, Syaaban, Ramadan, Syawal, Zulkaedah,
  Zulhijjah.
- Source: JAKIM's date for today; otherwise Umm al-Qura with "(estimate)".
- **After Maghrib** the Islamic day has begun: the hero shows tomorrow's Hijri date with a
  small "since Maghrib" tag.
- Settings: `showHijri` (default on); `hijriOnDock` (default off) puts "12 Jamadilawal" as
  the large pill's second line instead of the clock time.

### Tasbih
- **Dock:** a segment joined to the prayer pill (the Quick Settings joined-segment style,
  4 px inner corners), only when `tasbihOnDock` is on (default on):
```
  [ 🌙 Asar · in 25m ][ 📿 27/33 ]
```
  - Click = +1, with a 120 ms press bounce. The count rides a thin progress ring.
  - Reaching the target: a soft tick sound, a green pulse, "✓ 33" and the round count
    (×2) as a tooltip; the next click starts the next round from 1.
  - Right-click: Reset · Undo (−1) · Target 33 / 99 / 100 / 1,000 · Open panel.
  - Wheel down = −1 (fixes an extra tap) only with the pointer on the segment.
- **Panel section** "Tasbih" (when `showTasbih`, default on), after the prayer list:
```
  Tasbih                                  Today 412
        ╭──────────╮
        │    27    │   120 px tap ring; Space / Enter also count
        │  of 33   │
        ╰──────────╯
   Subhanallah ▾      [33] [99] [100] [⋯]      ↶   ⟲ (hold)
```
  - Dhikr chooser: Subhanallah · Alhamdulillah · Allahuakbar · Astaghfirullah ·
    Lailahaillallah · **Tasbih Fatimah** (33 · 33 · 34, moves to the next phrase by itself).
  - Undo = −1. Reset is **press and hold 600 ms** (a ring fills) so a stray click never
    loses a count.
  - "Today" total resets at local midnight.
- Storage: count, round, phrase step and today's total in the `kv` table (`tasbih`), one
  small write per tap, `tasbih:changed` to keep dock and panel in step. Target and dhikr in
  the widget settings.

---

## 8. Inline calculator (Start & Search)

- New option on the Start & Search widget and the App Launcher's inline Start & Search:
  **Search opens:** *GlassBar search* (default) · *Windows Search*. Win+S still opens
  Windows Search.
- The Search button opens a focused panel (420 × fit):
```
┌──────────────────────────────────────────────┐
│ 🔍 15% of 2,400 + 18                          │  input, autofocus, 20 px
├──────────────────────────────────────────────┤
│ = 378                              ⏎ copy    │  answer row, 32 px semibold
│   15% of 2400 + 18                            │
│ ───────────────────────────────────────────── │
│ ▸ Apps                                        │
│   🟦 Calculator                               │
│ ───────────────────────────────────────────── │
│   🔎 Search Windows for "15% of 2,400 + 18"   │
│   🌐 Search the web                           │
└──────────────────────────────────────────────┘
```
- **Maths:** + − × ÷ ^ %, parentheses, unary minus, `x` as times, `,` thousands, functions
  sqrt, sin, cos, tan (degrees), log, ln, abs, round, floor, ceil, constants pi and e;
  "15% of 200", "200 + 10%". Recursive-descent parser; no `eval`.
- **Units:** length, mass, temperature, data, time, speed, area, volume — "5 km to mi",
  "70 kg in lb", "100 f to c", "1.5 gb in mb". A bare "5 km" lists 3 common targets.
- **Currency:** "100 usd to myr", "rm 50 in sgd", "$20" (to the Currency widget's base, MYR
  by default) through the existing rates (`fx_rates`, cached 1 h). Row footer: "rate from
  ExchangeRate-API · 12 min ago".
- **Apps:** installed Start-menu apps whose name contains the words, up to 5, with icons.
- **Fallbacks:** *Search Windows* (opens Windows Search and types the text) and *Search the
  web* (default browser).
- Keyboard: ↑ ↓ move, Enter acts (copy the answer / open the app / search), Esc closes,
  Tab completes a unit or app name. Empty input shows the last 5 calculations.
- Copying shows "Copied" on the row for 1 s; the panel stays open.

---

## Settings summary

| Widget | Key | Default |
|---|---|---|
| Quick Settings | `hiddenTiles` (+ `awake`, `mic`), `awakeMinutes` (0 = until off), `awakeScreen`, `micOnDock` (`auto`/`always`/`never`), `btBattery` (section on/off), `btLowAlert` | shown, 0, true, `auto`, true, true |
| Recycle Bin | size `small`/`medium` | medium |
| Countdowns | `items`, `pinned`, `notify` | presets none, null, true |
| Capture | `icons` (+ `color`), `colorHotkey`, `colorFormat`, `colors` | appended, `Super+Shift+C`, `hex`, [] |
| Waktu Solat | `showHijri`, `hijriOnDock`, `showTasbih`, `tasbihOnDock`, `tasbihTarget`, `dhikr` | true, false, true, true, 33, `subhanallah` |
| Start & Search, App Launcher | `searchOpens` | `glassbar` |

## Left to the builder

- Exact icon art, spacing inside the 4 px grid, motion curves (use `--spring`, `--ease`).
- File layout inside each widget folder; tests next to pure logic (`*.test.ts`, Rust
  `#[cfg(test)]`).
- Where a Windows API refuses (policy, missing driver), show its reason in the panel and
  hide the dock mark.
