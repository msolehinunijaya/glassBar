# GlassBar design rules

One system for the dock and every widget. Each widget has two states: a **compact card** in the dock, and an **expanded flyout** that opens when it is clicked. They should feel like parts of one product, not separate apps.

> **Status:** these are the target rules. The code does not follow all of them yet; see [Current gaps](#current-gaps). New and changed widgets follow these rules.

## 1. Foundation tokens (define once, use everywhere)

All values live as CSS custom properties in `src/styles/glass.css`. Widgets never hard-code colours, radii or spacing.

### Spacing
- 4 px grid only: **4 / 8 / 12 / 16 / 24 / 32**.
- Card padding **16 px**. Gap between cards **8 px**.

### Radius (3 levels only)
| Level | Value | Used for |
|---|---|---|
| Outer | 16 px | cards, flyouts |
| Inner | 10 px | rows, inputs, buttons |
| Pill | 999 px | chips, pills, badges |

Inner radius = outer radius − the padding gap, so nested corners stay parallel.

### Surfaces (3 levels only)
| Token | Used for |
|---|---|
| `bg-0` | dock background (Mica / Acrylic blur, about 70 % opacity) |
| `bg-1` | card |
| `bg-2` | inset element: row, input, selected item |

Separate levels with a 1 px border at `rgba(255,255,255,0.06)` (dark) — not with heavy shadows.

### Text (3 levels only)
| Level | Dark theme |
|---|---|
| Primary | 95 % white |
| Secondary | 65 % white |
| Tertiary | 45 % white |

Body text never goes below 4.5 : 1 contrast.

### Type
- One family everywhere: Poppins (current), or Segoe UI Variable if we switch. Never mix.
- Scale: **11 / 12 / 14 / 16 / 20 / 32 / 48**.
- Every number uses `font-variant-numeric: tabular-nums`, so clocks and timers don't shift as digits change.

### Colour
- **One global accent** (`--accent`, following the Windows accent colour when possible).
- Status colours carry meaning only:
  - Red / orange — active timer, work
  - Green — break, success, precise location
  - Amber — warning, saved
- Never colour a card background for decoration. (Example: world-clock rows are `bg-2` with a small day/night icon, not blue or brown cards.)
- Data-driven colour is allowed where the colour *is* the value: the IPU band colour, weather-condition art.

## 2. Compact card

_To be added (not in the brief received)._

## 3. Compact card sizes (S / M / L)

_To be added (not in the brief received)._

## 4. Expanded flyout (on click)

- Anchored to its dock item like Windows 11 Quick Settings: **360–420 px** wide, same radius and surface tokens, **8 px** from the dock.
- Opens in **150–200 ms**, ease-out, scale **0.96 → 1** plus a fade, from the card's position so the card seems to grow. Respect `prefers-reduced-motion`.
- Closes on click outside, **Esc**, or clicking the same dock item again. **No close button.**
- Structure, top to bottom:
  1. the same header as the compact card;
  2. the hero, repeated larger;
  3. sections separated by 16 px space and a hairline divider — no boxes inside boxes;
  4. footer with meta and source.
- Contents per widget:

| Widget | Flyout holds |
|---|---|
| Clock | world clocks list, calendar, "Add city" |
| Pomodoro | ring, controls, session dots (●●○○), today's total, duration settings |
| Currency | converter, saved pairs as chips, 7-day sparkline, rate source |
| Waktu Solat | full prayer list, zone picker, Hijri date, adhan notification toggle |
| File | path, size, modified date; Open / Show in folder / Copy path |

## 5. Lists and rows (prayer times, world clocks)

- Row height **40 px**, horizontal padding **12 px**. Label left-aligned, value right-aligned in tabular numbers.
- Current / next item: `bg-2` + a **3 px accent bar on the left** + semibold text. Never fill the whole row with saturated colour.
- Past items (prayers already done) use tertiary text.
- One time format: `05:51` with a small "AM" in tertiary colour, or 24 h — following the system setting.

## 6. Controls

- **Primary button:** filled accent, 36 px tall, at most **one per view** (e.g. Pause).
- **Secondary:** ghost icon buttons, 32 px, with a visible hover and focus ring.
- **Inputs and selects:** `bg-2`, 10 px radius, 40 px tall. When the input *is* the hero (Currency), its value is 20–32 px semibold.
- Every interactive element has **hover, pressed, focus-visible and disabled** states.

## 7. Behaviour

- Live values update without layout shift. Countdowns tick every second; rates show a relative "updated 3m ago".
- **Loading:** a skeleton shaped like the final content.
- **Error / offline:** keep the last value, dim it, add "offline · last updated 8:02". Never leave a widget blank.
- Keyboard inside a flyout: Tab through items, arrow keys within lists.
- Follow the Windows light and dark theme and accent colour. Test both themes.
- Plain words in the user's language. Keep local terms as they are (Subuh, Zohor, …).
- Location-based widgets never show a made-up or stale place as current (see Settings › Location).

## 8. Left/right (side) dock

A side dock is **56 / 72 / 96 px** wide (Settings › Bar › Side width → `data-side="slim|regular|wide"` on the dock). Widgets don't squeeze their horizontal card into it; they draw **side tiles**:

- `.side` — one tile: full width, column, centred, outer radius.
- `.side-stack` — the tiles of one widget, joined with a 2 px gap; only the outer corners round.
- Inside a tile, at most three text roles: `.side-k` label (small, secondary), `.side-v` value (semibold, tabular), `.side-c` caption (small, secondary). Slim hides captions.
- `.side-meter` — a 3 px meter under the value for a level (CPU, RAM, disk, to-dos done); colour via `--c` (`loadTone()` in `src/lib/side.ts`).
- No text that can be cut off: short labels ("CPU", "C:", "Net"), short values ("3:53", "↓978K"), long names only in the tooltip or flyout.
- When the widgets don't fit, the cut-off end fades and shows a "⌄ 9" button that scrolls (same on top/bottom docks).
- See `docs/WIDGET_SDK.md` › Left/right dock for the markup.

## 9. Do / Don't

- ✅ One accent, one radius system, one icon set (Lucide), one header pattern.
- ✅ Whitespace and type weight for hierarchy, not colour.
- ✅ Buttons that crowd a dock item appear on hover only.
- ❌ Coloured backgrounds per card, mixed icon styles, settings icons in random places.
- ❌ More than 3 text sizes in one compact card.
- ❌ Paragraph text in the compact state (disclaimers belong in the flyout footer).

## Deliverable for a redesign

A token sheet; the compact card in S / M / L for each widget; the expanded flyout for each widget; all in dark and light themes.

## Current gaps

Where today's code differs from these rules (fix when touching the widget):

- Radii are 14 px (pill) / 18 px (bar) / 20 px (card) in `glass.css`, not 16 / 10 / 999.
- Surfaces use `--pill` / `--panel` / `--settings-bg` rather than `bg-0/1/2`; borders use `--line-soft`.
- World-clock rows use blue/brown day/night backgrounds.
- The selected prayer row is fully filled with the accent.
- The Clock card has no header; the Currency disclaimer sits in the compact state.
- Flyouts have no open animation from the card, and some have settings icons in different places.
