import { describe, expect, it } from 'vitest';
import { wheelStep } from './rangeWheel';

describe('wheel on a slider', () => {
  it('moves one step per notch and stays in range', () => {
    expect(wheelStep(0.5, 0.2, 1, 0.01, 1)).toBe(0.51);
    expect(wheelStep(0.2, 0.2, 1, 0.01, -1)).toBe(0.2);
    expect(wheelStep(48, 0, 48, 1, 1)).toBe(48);
    expect(wheelStep(100, 30, 400, 2, -1)).toBe(98);
    expect(wheelStep(1.2, 1, 3, 0.1, 1)).toBe(1.3);
  });
});
