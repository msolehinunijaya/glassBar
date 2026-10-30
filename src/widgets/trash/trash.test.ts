import { describe, expect, it } from 'vitest';
import { binLook, dropSummary, fromLabel } from './trash';

describe('Recycle Bin helpers', () => {
  it('summarises a drop', () => {
    expect(dropSummary({ moved: 3, failed: [] })).toBe('Moved 3 items to the Recycle Bin');
    expect(dropSummary({ moved: 1, failed: [] })).toBe('Moved 1 item to the Recycle Bin');
    expect(dropSummary({ moved: 1, failed: [['C:\\a.txt', 'in use']] })).toBe("Moved 1 item · 1 couldn't be moved (a.txt: in use)");
    expect(dropSummary({ moved: 0, failed: [['C:\\a.txt', 'in use'], ['C:\\b', 'denied']] })).toBe("2 couldn't be moved (a.txt: in use)");
  });
  it('shows where an item came from as the last two folders', () => {
    expect(fromLabel('C:\\Users\\me\\Documents\\Work')).toBe('Documents › Work');
    expect(fromLabel('C:\\Temp')).toBe('C: › Temp');
    expect(fromLabel('D:\\')).toBe('D:');
    expect(fromLabel('')).toBe('');
  });
  it('is full only with items, unknown without an answer', () => {
    expect(binLook(null)).toBe('unknown');
    expect(binLook({ size: 0, items: 0, drives: [] })).toBe('empty');
    expect(binLook({ size: 10, items: 2, drives: [] })).toBe('full');
  });
});
