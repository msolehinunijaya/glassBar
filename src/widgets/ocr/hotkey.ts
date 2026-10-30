/** "Super+Shift+T" -> "Win+Shift+T", "Control+Alt+C" -> "Ctrl+Alt+C" for display. */
export const hotkeyLabel = (s: string) => (s || 'Super+Shift+T').replace(/super|meta|cmd/i, 'Win').replace(/control/i, 'Ctrl');

const MODS = new Set(['Control', 'Shift', 'Alt', 'Meta']);

/** A keydown -> the plugin's shortcut string ("Super+Shift+T"), or null while only modifiers are held. */
export function fromEvent(e: { key: string; code: string; ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }): string | null {
  if (MODS.has(e.key)) return null;
  const mods = [e.ctrlKey && 'Control', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Super'].filter(Boolean);
  if (!mods.length) return null; // a bare key would steal normal typing
  const key = e.code.startsWith('Key') ? e.code.slice(3) : e.code.startsWith('Digit') ? e.code.slice(5) : e.code;
  return [...mods, key].join('+');
}
