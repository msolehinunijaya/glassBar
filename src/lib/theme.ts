// Theme settings -> CSS variables on <html>.
import { ipc, type Theme } from './ipc';

const dark = matchMedia('(prefers-color-scheme: dark)');
let current: Theme | null = null;
let transparency = true;
dark.addEventListener('change', () => current && applyTheme(current));

export function applyTheme(t: Theme) {
  current = t;
  const isDark = t.mode === 'dark' || (t.mode === 'auto' && dark.matches);
  const root = document.documentElement;
  root.dataset.theme = isDark ? 'dark' : 'light';
  // Native blur draws nothing while Windows' Transparency effects are off: draw the tint here then.
  root.dataset.native = String(t.nativeBlur && transparency);
  // Without transparency effects Windows paints see-through pixels black: draw a solid surface.
  root.dataset.solid = String(!transparency);
  ipc.transparencyEffects().then((on) => { if (on !== transparency) { transparency = on; if (current) applyTheme(current); } }).catch(() => {});
  root.style.setProperty('--accent', t.accent);
  root.style.setProperty('--blur', `${t.blur}px`);
  root.style.setProperty('--tint-alpha', String(t.opacity));
}
