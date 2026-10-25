import { describe, expect, it } from 'vitest';
import { CAP_MENU, parseCapMenu } from './menu';

describe('capture hover menu key', () => {
  it('reads the tab and widget instance', () => {
    expect(parseCapMenu(`${CAP_MENU}shot:ocr-1`)).toEqual({ tab: 'shot', instance: 'ocr-1' });
    expect(parseCapMenu(`${CAP_MENU}clip:a:b`)).toEqual({ tab: 'clip', instance: 'a:b' });
  });
  it('ignores other panels and unknown tabs', () => {
    expect(parseCapMenu('__hover:music-1')).toBeNull();
    expect(parseCapMenu(`${CAP_MENU}nope:x`)).toBeNull();
    expect(parseCapMenu(null)).toBeNull();
  });
});
