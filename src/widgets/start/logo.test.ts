import { describe, expect, it } from 'vitest';
import { logoFills } from './logo';

describe('logoFills', () => {
  it('is Windows blue unless told otherwise', () => {
    expect(logoFills(undefined)).toEqual(Array(4).fill('#0a84ff'));
  });
  it('follows the accent or the text colour', () => {
    expect(logoFills('accent')[0]).toBe('var(--accent)');
    expect(logoFills('text')[3]).toBe('currentColor');
  });
  it('has the classic four colours in their places', () => {
    expect(logoFills('classic')).toEqual(['#f25022', '#7fba00', '#00a4ef', '#ffb900']);
  });
  it('uses a custom colour, and ignores anything that is not one', () => {
    expect(logoFills('custom', '#ff2bd6')[2]).toBe('#ff2bd6');
    expect(logoFills('custom', 'red; x')[0]).toBe('#0a84ff');
  });
});
