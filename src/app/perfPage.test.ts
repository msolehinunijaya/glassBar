import { describe, expect, it } from 'vitest';
import { changeList, fmtBytes, fmtPct, netLastHour } from './perfPage';
const s = (netBytes: number) => ({ at: 0, ramMb: 0, cpu: 0, gpu: null, netBytes });
describe('changeList', () => {
  const items = [{ name: 'Whole bar', perf: { low: 'paused' } }, { name: 'Equalizer', perf: { low: '15 fps', high: '60 fps' } }, { name: 'Clock' }, { name: 'Apps' }];
  it('lists what the mode changes, in order, and the rest A–Z', () => {
    expect(changeList('low', items)).toEqual({ rows: [{ name: 'Whole bar', text: 'paused' }, { name: 'Equalizer', text: '15 fps' }], unchanged: ['Apps', 'Clock'] });
  });
  it('a mode with no text for a widget leaves it unchanged', () => {
    expect(changeList('high', items).unchanged).toEqual(['Apps', 'Clock', 'Whole bar']);
  });
});
describe('formatting', () => {
  it('bytes', () => { expect(fmtBytes(348_000)).toBe('340 KB'); expect(fmtBytes(1.2 * 1048576)).toBe('1.2 MB'); });
  it('missing GPU shows a dash', () => { expect(fmtPct(null)).toBe('—'); expect(fmtPct(4.83)).toBe('4.8%'); });
  it('net in the last hour', () => { expect(netLastHour([s(100), s(400), s(1100)])).toBe(1000); expect(netLastHour([s(5)])).toBe(0); });
});
