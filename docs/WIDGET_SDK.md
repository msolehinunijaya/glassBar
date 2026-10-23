# Widget SDK

This guide shows how to add a widget to GlassBar. A working example lives in `src/widgets/_template/`. Folders that start with `_` appear in the gallery only in dev builds.

## 1. Copy the template

```bash
cp -r src/widgets/_template src/widgets/pomodoro
```

## 2. Edit the manifest (`index.ts`)

```ts
export default defineWidget<PomodoroSettings>({
  id: 'pomodoro',                 // unique; saved in config — never change it after release
  name: 'Pomodoro',
  description: '25/5 focus timer',
  icon: Timer,                    // any @lucide/svelte icon
  sizes: ['small', 'medium'],     // sizes the user can cycle in edit mode
  defaultSize: 'medium',
  defaults: { work: 25, rest: 5 },// merged under saved settings
  Pill,                           // required: what sits in the bar
  Popup: { component: Popup, width: 320, height: 300 },  // optional
  SettingsForm,                   // optional: shown in Settings → Widgets
});
```

That's it. The registry picks the folder up automatically, so it appears in the gallery (edit mode → **+**) and in Settings → Widgets.

## 3. Components and props

| Component | Props | Notes |
|---|---|---|
| `Pill.svelte` | `instance, size, settings, vertical` | Wrap in `<div class="pill">`. `vertical` = bar docked left/right (size is forced to `small`). Clicking the pill opens `Popup` if one is declared; call `e.stopPropagation()` on your own buttons. |
| `Popup.svelte` | `instance, settings, close` | Rendered in the shared popup window. Unmounted when the popup hides, so timers stop. |
| `SettingsForm.svelte` | `instance, settings, save` | `save(patch)` merges into this instance's settings and persists. |

Settings are per **instance**, so the same widget can be added twice with different settings.

### Left/right dock (`vertical`)

A side dock is 56, 72 or 96 px wide (Settings › Bar › Side width; the dock carries `data-side="slim|regular|wide"`). A normal pill is squeezed into a narrow column there, so widgets that show numbers should draw a side design instead: `{#if vertical} … {:else} <div class="pill"> … {/if}`, using the same state.

Use the shared side-tile classes from `glass.css` (not `.pill`):

| Class | Use |
|---|---|
| `.side` | One tile: full width, rounded, column, centred. |
| `.side-stack` | Several tiles of one widget, joined (only the outer corners round). |
| `.side-k` / `.side-v` / `.side-c` | Small label / value / caption. Slim hides captions. |
| `.side-meter > i` | A 3 px meter under the value; width = level, colour from `--c` (see `loadTone()` in `src/lib/side.ts`). |

Example: `<div class="side"><span class="side-k">CPU</span><span class="side-v">53<small>%</small></span><span class="side-meter"><i style:width="53%"></i></span></div>`.

## 4. Styling

Use the tokens from `src/styles/glass.css` so the widget follows the theme:

- **Colors:** `--fg`, `--fg-dim`, `--accent`, `--pill`, `--pill-hover`, `--line`, `--line-soft`, `--track`
- **Shape:** `--radius-pill` (14px), `--radius-card` (20px), `--gap`
- **Motion:** `--spring` (iOS-like overshoot), `--ease`
- **Ready-made classes:** `.pill`, `.card`, `.tile` (+`.on`), `.icon-btn`, `input.slider` (set `style:--fill="40%"`), `.dim`, `.ellipsis`

## 5. Backend (optional)

If your widget needs OS access:

1. Copy `src-tauri/src/widgets/_template.rs.example` to `src-tauri/src/widgets/<id>.rs`.
2. Add `pub mod <id>;` to `src-tauri/src/widgets/mod.rs`.
3. Add the commands to `generate_handler![...]` in `src-tauri/src/main.rs`.
4. Add a typed wrapper in `src/lib/ipc.ts`. Widgets never call `invoke` directly.

Follow these rules:

- **OS calls:** they go through a trait in `crate::platform`. Add a method to the right trait, or a new trait, with the Windows code in `platform/windows/`.
- **Blocking commands:** make them `async`. COM, WinRT, disk, and network calls all block.
- **Errors:** return `Result<T, String>`.
- **Live updates:** emit an event from a background thread (`app.emit("myid:changed", data)`) and add a listener to `events` in `ipc.ts`.

## 6. Test

- Put pure logic (parsing, math, grouping) in a `.ts` file next to the widget with a `*.test.ts` beside it. See `apps/groups.test.ts`. Run `npm test`.
- For Rust logic, use `#[cfg(test)]` and run `cargo test` in `src-tauri`.
