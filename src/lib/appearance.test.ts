import { describe, expect, it } from 'vitest';
import type { Theme } from './ipc';
import { accentInk, INK, onAccent, applySnapshot, changed, FLOOR, luminance, tintFor, presetDefaults, resolve, snapshot, STYLES, THEMES, windowRadius } from './appearance';

const theme = (p: Partial<Theme> = {}): Theme => ({
  ...THEMES.system.set, ...STYLES.glassbar.set, preset: 'system', style: 'glassbar', accent: '#0A84FF', saved: {}, ...p,
});
const env = { transparency: true, prefersDark: true };

describe('appearance', () => {
  it('lets blurred glass be fully see-through but keeps unblurred surfaces readable', () => {
    expect(resolve(theme({ opacity: 0 }), env).dock).toBe(0);
    const flat = resolve(theme({ nativeBlur: false, opacity: 0, panelOpacity: 0.1 }), env);
    expect([flat.dock, flat.panel, flat.raised.dock]).toEqual([FLOOR.dock, FLOOR.panel, true]);
    // Transparency effects off: nothing is blurred, so the floors apply even with blur on.
    expect(resolve(theme({ opacity: 0 }), { ...env, transparency: false }).blurred).toBe(false);
    expect(resolve(theme({ contrast: 'high' }), env).panel).toBe(FLOOR.high);
    expect(resolve(theme({ cardOpacity: 0 }), env).card).toBe(FLOOR.card);
  });

  it('raises the tint just enough for readable text over the wallpaper', () => {
    const white = { min: 1, max: 1 }, black = { min: 0, max: 0 };
    // White text over a white wallpaper needs ~83 % dark tint; over black none.
    expect(tintFor(true, luminance('#1c1c20'), white)).toBeCloseTo(0.83, 2);
    expect(tintFor(true, luminance('#1c1c20'), black)).toBe(0);
    // Dark text over black needs light tint; over white none.
    expect(tintFor(false, luminance('#f6f6fa'), black)).toBeCloseTo(0.25, 2);
    expect(tintFor(false, luminance('#f6f6fa'), white)).toBe(0);
    const r = resolve(theme({ opacity: 0.06, mode: 'dark' }), { ...env, behind: { min: 0.1, max: 0.9 } });
    expect(r.dock).toBeGreaterThan(0.75);
    expect(r.raised.dock).toBe(true);
    expect(resolve(theme({ opacity: 0.06, mode: 'dark' }), { ...env, transparency: false, behind: white }).wall).toBe(0);
  });

  it('picks text colour from a custom background, not the mode', () => {
    expect(resolve(theme({ mode: 'dark', bg: '#f0f0f0' }), env).dark).toBe(false);
    expect(resolve(theme({ mode: 'light', bg: '#101820' }), env).dark).toBe(true);
    expect(resolve(theme({ mode: 'auto' }), { ...env, prefersDark: false }).dark).toBe(false);
    expect(luminance('#ffffff')).toBeCloseTo(1);
    expect(luminance('#000000')).toBe(0);
  });

  it('rounds a material window only as Windows can (0/4/8, same as appbar.rs)', () => {
    expect([0, 3, 5, 6, 20].map((r) => windowRadius(r, true))).toEqual([0, 4, 4, 8, 8]);
    expect(windowRadius(20, false)).toBe(20);
  });

  it('every preset sets the same fields, and reset/revert/snapshots use them', () => {
    const keys = (o: object) => Object.keys(o).sort();
    for (const p of Object.values(THEMES)) expect(keys(p.set)).toEqual(keys(THEMES.system.set));
    for (const s of Object.values(STYLES)) expect(keys(s.set)).toEqual(keys(STYLES.glassbar.set));
    const t = theme({ style: 'mac', ...STYLES.mac.set });
    expect(changed(t, 'radius')).toBe(false);
    t.radius = 3;
    expect(changed(t, 'radius')).toBe(true);
    expect(presetDefaults(t).radius).toBe(STYLES.mac.set.radius);
    const copy = theme();
    applySnapshot(copy, snapshot(t));
    expect({ ...copy, saved: {} }).toEqual({ ...t, saved: {} });
  });
});

describe('accent contrast', () => {
  it('keeps white labels on dark accents', () => {
    for (const a of ['#0A84FF', '#000000', '#6E3FD8']) expect(onAccent(a)).toBe('#fff');
  });
  it('turns labels dark on light accents (white, yellow, light grey)', () => {
    for (const a of ['#FFFFFF', '#FFD60A', '#C8C8C8']) expect(onAccent(a)).toBe(INK);
  });
  it('draws the accent on white only where it shows', () => {
    expect(accentInk('#0A84FF')).toBe('#0A84FF');
    expect(accentInk('#FFFFFF')).toBe(INK);
  });
});
