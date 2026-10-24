import { describe, expect, it } from 'vitest';
import { binView, confirmText, freeShare } from './recycle';

describe('Recycle Bin card', () => {
  it('empty: says so and disables Empty', () => {
    expect(binView({ size: 0, items: 0, drives: [] }, null)).toEqual({ title: '0 B', detail: 'Recycle Bin is empty', canEmpty: false });
  });
  it('shows size and count', () => {
    expect(binView({ size: 2.4 * 1024 ** 3, items: 312, drives: [] }, null)).toEqual({ title: '2.4 GB', detail: '312 items', canEmpty: true });
    expect(binView({ size: 10, items: 1, drives: [] }, null).detail).toBe('1 item');
  });
  it("can't be read: N/A with the reason, nothing to empty", () => {
    expect(binView(null, 'Blocked by policy')).toEqual({ title: 'N/A', detail: 'Blocked by policy', canEmpty: false });
  });
  it('share of free space and the confirm text', () => {
    expect(freeShare(4, 100)).toBe(4);
    expect(freeShare(1, 1000)).toBe(1);
    expect(freeShare(5, 0)).toBe(0);
    expect(confirmText(312, 2.4 * 1024 ** 3)).toBe("Permanently delete 312 items (2.4 GB)? This can't be undone.");
  });
});
