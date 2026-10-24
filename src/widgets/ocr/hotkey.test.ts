import { describe, expect, it } from 'vitest';
import { fromEvent, hotkeyLabel } from './hotkey';

const ev = (key: string, code: string, m: Partial<{ ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }> = {}) =>
  ({ key, code, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...m });

describe('hotkey', () => {
  it('labels Super as Win', () => expect(hotkeyLabel('Super+Shift+T')).toBe('Win+Shift+T'));
  it('builds a shortcut from a key event', () => {
    expect(fromEvent(ev('T', 'KeyT', { metaKey: true, shiftKey: true }))).toBe('Shift+Super+T');
    expect(fromEvent(ev('1', 'Digit1', { ctrlKey: true, altKey: true }))).toBe('Control+Alt+1');
  });
  it('ignores bare keys and lone modifiers', () => {
    expect(fromEvent(ev('t', 'KeyT'))).toBeNull();
    expect(fromEvent(ev('Shift', 'ShiftLeft', { shiftKey: true }))).toBeNull();
  });
});
