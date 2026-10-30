import { describe, expect, it } from 'vitest';
import { clock, cropOf } from './mirror';

describe('cropOf', () => {
  it('shows the whole frame at 1× in a rectangle', () => {
    expect(cropOf(1280, 720, 1, 'rect')).toEqual({ x: 0, y: 0, w: 1280, h: 720 });
  });
  it('takes the centre square for a circle', () => {
    expect(cropOf(1280, 720, 1, 'circle')).toEqual({ x: 280, y: 0, w: 720, h: 720 });
  });
  it('zooms into the centre and keeps the zoom between 1 and 3', () => {
    expect(cropOf(1000, 800, 2, 'rect')).toEqual({ x: 250, y: 200, w: 500, h: 400 });
    expect(cropOf(900, 900, 9, 'rect').w).toBe(300);
    expect(cropOf(900, 900, 0, 'rect').w).toBe(900);
  });
});

it('clock shows minutes and seconds', () => {
  expect(clock(0)).toBe('0:00');
  expect(clock(75)).toBe('1:15');
});
