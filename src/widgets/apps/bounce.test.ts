import { describe, expect, it } from 'vitest';
import { settleBounces, startBounce } from './bounce';

const g = (key: string, ...ids: number[]) => ({ key, windows: ids.map((id) => ({ id })) });

describe('launch bounce', () => {
  it('keeps bouncing while no new window has appeared', () => {
    const b = startBounce({}, 'code', [1], 0);
    expect(settleBounces(b, [g('code', 1)], 1000)).toHaveProperty('code');
  });
  it('stops once a new window shows up', () => {
    const b = startBounce({}, 'code', [1], 0);
    expect(settleBounces(b, [g('code', 1, 7)], 1000)).not.toHaveProperty('code');
  });
  it('gives up after 10 seconds', () => {
    const b = startBounce({}, 'tray-app', [], 0);
    expect(settleBounces(b, [g('tray-app')], 10_001)).not.toHaveProperty('tray-app');
  });
  it('waits for the timeout when the app is not in the list yet', () => {
    const b = startBounce({}, 'new', [], 0);
    expect(settleBounces(b, [], 5000)).toHaveProperty('new');
  });
  it('restarts the clock when launched again', () => {
    const b = startBounce(startBounce({}, 'code', [], 0), 'code', [], 9000);
    expect(settleBounces(b, [g('code')], 12_000)).toHaveProperty('code');
  });
});
