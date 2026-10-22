// Theme settings -> CSS variables on <html>.
import type { Theme } from './ipc';

const dark = matchMedia('(prefers-color-scheme: dark)');
let current: Theme | null = null;
dark.addEventListener('change', () => current && applyTheme(current));

export function applyTheme(t: Theme) {
  current = t;
  const isDark = t.mode === 'dark' || (t.mode === 'auto' && dark.matches);
  const root = document.documentElement;
  root.dataset.theme = isDark ? 'dark' : 'light';
  root.dataset.native = String(t.nativeBlur);
  root.style.setProperty('--accent', t.accent);
  root.style.setProperty('--blur', `${t.blur}px`);
  root.style.setProperty('--tint-alpha', String(t.opacity));
}
