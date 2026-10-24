import { describe, expect, it } from 'vitest';
import md from '../../CHANGELOG.md?raw';
import { boldParts, parseChangelog } from './changelog';

describe('changelog', () => {
  it('reads versions, groups and items', () => {
    const r = parseChangelog('# Changelog\n\n## 0.2.0 — 2026-10-24\n\n### New\n- **A:** one\n- two\n\n### Fixed\n- three\n\n## 0.1.0\n- first\n');
    expect(r.map((x) => [x.version, x.date])).toEqual([['0.2.0', '2026-10-24'], ['0.1.0', '']]);
    expect(r[0].groups).toEqual([{ title: 'New', items: ['**A:** one', 'two'] }, { title: 'Fixed', items: ['three'] }]);
    expect(r[1].groups[0].items).toEqual(['first']);
  });
  it('splits bold text', () => {
    expect(boldParts('**Settings › Location:** one place')).toEqual([['Settings › Location:', true], [' one place', false]]);
  });
  it('the real CHANGELOG starts with the app version', async () => {
    const pkg = await import('../../package.json');
    expect(parseChangelog(md)[0].version).toBe(pkg.version);
  });
});
