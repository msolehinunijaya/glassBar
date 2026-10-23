import { expect, it } from 'vitest';
import { progress } from './progress';

it('todo progress = done / total, null when empty', () => {
  expect(progress([])).toBeNull();
  expect(progress([{ done: true }, { done: false }, { done: false }, { done: true }])).toBe(0.5);
  expect(progress([{ done: true }])).toBe(1);
});
