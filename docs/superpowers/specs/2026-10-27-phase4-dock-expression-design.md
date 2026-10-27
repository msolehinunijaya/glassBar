# GlassBar Phase 4: Dock expression (Design)

Status: approved · Date: 2026-10-27 · Branch `phase2` · Builds on v0.2.7 (see `docs/ARCHITECTURE.md`)

## Intent

The dock reports facts well and says almost nothing about them. Phase 4 gives it a voice: it
reacts when the charger moves, it lets the user build the music player they actually want
instead of picking one of three, it sings along, it holds a few files for a minute, and it
grows a small creature that has opinions about all of it.

Everything stays glanceable from the bar, costs nothing while idle, and works offline wherever
the data allows.

## Goals

1. **Charger flash.** Plugging in or unplugging turns the whole dock into a battery display for
   about 2.5 s — large percentage, the level drawn as a filled background, smooth in and out —
   then it returns to normal. On by default, one toggle to turn off.
2. **Custom music design.** A fourth music style, `custom`, where the user chooses which parts
   show: album art (and its shape and size), equalizer, title, artist, album, source app,
   progress bar, elapsed / remaining / total time, and Previous / Play-Pause / Next / volume
   each on their own.
3. **Lyrics.** Synced lyrics for the playing track, inline in the dock beside the music widget,
   with the line changing in time with the music. Style and per-line animation are the user's
   choice. Opt-in, because it makes a network call.
4. **File shelf.** A widget holding at most 5 files. Drag in, open later, drag out to take it
   off the shelf. Survives a restart.
5. **Companion.** A creature on the dock with expressive eyes and optional hands, that blinks on
   its own, can be tickled, and changes mood from things the app already knows: the charger,
   the battery level, music playing, prayer time, a Pomodoro phase, Do Not Disturb, unread
   alerts, CPU load, and being left alone.

## Non-goals

- A second font family for lyrics. `docs/DESIGN_RULES.md` says one family, never mixed; lyrics
  get weight, size, italic and caps instead.
- Lyrics for a track LRCLIB does not have. There is no second provider and no scraping.
- A lyrics window. The line lives inside the dock.
- Moving, copying or deleting a file on the user's disk. The shelf holds paths only.
- Hands painted outside the dock's outline. **User decision:** everything stays inside the
  companion's own pill, so the dock window, the AppBar reserved space and auto-hide are untouched.
- An OS toast for the charger. **User's words:** "i dont want use popup notification."

## Anti-goals (a failure even if it "works")

- **A busy bar.** None of this may raise idle CPU. The companion must cost ~0 while nothing is
  happening; the embedded equalizer must obey the same lease, idle-fps and battery-saver rules
  that commit `0b79c31` bought (~15 % of a core, down from ~120 %).
- **A dock that jumps.** The charger flash must overlay the widget row, never replace or unmount
  it. Commit `4ff00e2` already recorded what unmounting mid-animation does: leaving elements
  float over their neighbours while the row resizes.
- **A changed music widget.** The three existing designs must look and behave exactly as they do
  today. `custom` is additive.
- **Silent network calls.** Lyrics lookup is opt-in, says in plain words what leaves the machine,
  and the toggle stops the request rather than hiding the result.
- **A flash that lies.** It must fire once per real charger change — never on a battery
  percentage change while plugged in, and never at launch.
- **Lost files.** A cancelled drag must never remove a shelf entry.
- **A creature that fights itself.** Two triggers must never both win; the priority order is
  explicit and tested.

## Decisions (user)

- **Lyrics source:** LRCLIB (`lrclib.net`, free, no API key), fetched in Rust, **plus** a local
  `.lrc` override read from disk first. LRCLIB loses to a local file.
- **Lyrics placement:** inline in the dock beside the music pill. Right by default, left
  optional. No new window.
- **Companion hands:** inside its own pill, reaching the pill's rounded edge.
- **Order:** charger flash → custom music design → lyrics → file shelf → companion. Each phase
  is tested, verified and shippable on its own.

## Decisions (ours, open to correction)

- **Charger flash is global config, not a widget setting.** It is dock behaviour and must work
  whether or not the Battery widget is placed. It sits next to the existing charger-sound
  toggle in Settings › Sounds, which is where a user would look for it.
- **The flash rides the existing charger edge.** `charger_flip()` in
  `src-tauri/src/widgets/sound.rs` already detects a real plug change, debounced, with a guard
  against a false edge at launch, and it is unit-tested. The flash emits from the same place.
  Doing the edge detection in the webview would mean re-solving all of that.
- **No new pill for the custom music design.** `PlayerPill.svelte` already draws every part
  asked for; it just gates them on widget size. Replacing that gate with a resolved parts object
  is a smaller, safer change than a second component that would drift from the first.
- **The equalizer's capture/draw harness becomes a shared component** used by both the Equalizer
  widget and the custom music layout, rather than copied. The painter is already pure.
- **The shelf stores paths in widget settings,** like the App Launcher's pins — no database
  table, no migration, and it travels with a config export.
- **The companion is SVG and CSS, not canvas.** Its eyes are a few shapes and its moods are
  discrete, so transitions between them are free when nothing changes. A canvas would need a
  frame loop to show the same thing.
- **Blinking is a chained timeout, not an interval,** and stops when the dock is hidden.
- **The companion needs no Rust.** Every trigger reads state the app already publishes.

## Local .lrc, honestly

Windows' media API gives no file path for Spotify or browser playback, so there is nothing to
look "next to". The local override is therefore a folder GlassBar owns —
`%APPDATA%\GlassBar\lyrics\<artist> - <title>.lrc` — which is also the fix when LRCLIB is wrong,
and the only way to get lyrics with the lookup turned off.

## What is left to the builder

- The exact easing and the split between the flash's enter, fill, hold and exit, within the
  150–200 ms enter that `DESIGN_RULES.md` §4 requires.
- The companion's eye shapes, and how many moods are worth drawing.
- Which cut-down equalizer settings the music layout exposes (`EQ_DEFAULTS` has 30+ fields and
  the music widget should not show them all).
- Whether the real loopback spectrum or the cheaper peak meter looks better inside the music
  pill. Ship the spectrum; `Rhythm.svelte` and `Visualizer.svelte` are the documented fallback.
- Whether the dock can accept OS file drops at all. The bar window is always-on-top,
  transparent and auto-hiding, and nothing in the repo subscribes to `onDragDrop` yet. Spike it
  before building the shelf UI; the fall-back is a file-picker button, which already exists in
  the App Launcher.

## Verification

Per phase: `npm test`, `npm run check`, `cargo test`, then `npm run tauri dev` and the manual
checks in the plan — both themes, and 100 % / 125 % / 150 % display scaling, since 125 % has
already produced two bugs in this codebase. The charger flash needs a real charger; there is no
fake path.
