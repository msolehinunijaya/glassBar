import { describe, expect, it } from 'vitest';
import { reallyLeft } from './hoverPanel';

const el = { getBoundingClientRect: () => ({ left: 100, right: 130, top: 10, bottom: 40 }) } as unknown as HTMLElement;
const at = (x: number, y: number) => ({ clientX: x, clientY: y }) as PointerEvent;

describe('reallyLeft', () => {
  it('ignores a leave reported while the pointer is still on the button', () => {
    expect(reallyLeft(at(115, 25), el)).toBe(false);
  });
  it('counts a leave outside the button', () => {
    expect(reallyLeft(at(140, 25), el)).toBe(true);
    expect(reallyLeft(at(115, 45), el)).toBe(true);
    expect(reallyLeft(undefined, el)).toBe(true);
  });
});
