import { describe, expect, it } from 'vitest';
import type { AiuReading } from '../../lib/ipc';
import { byUrgency, resetShort, modelName, stack, tokens, tone, tooltip, until, view, views, windows } from './model';

const z = { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 };
const r = (p: Partial<AiuReading>): AiuReading => ({
  source: 'claudecode', connected: true, hint: null, updatedAt: 1, error: null, errorKind: null, hasUsage: true, limits: [],
  session: null, today: z, week: z, month: z, daily: [], models: [], projects: [], costToday: null, costMonth: null, costEstimated: false, ...p,
});

describe('aiusage model', () => {
  it('merges web limits with token data and hides unconnected providers', () => {
    const rs = [
      r({ source: 'claudecode', today: { ...z, input: 1000 } }),
      r({ source: 'claudeplan', hasUsage: false, limits: [{ key: 'session', label: '5-hour limit', pct: 62, resetsAt: 5 }, { key: 'week', label: 'Weekly limit', pct: 18, resetsAt: 9 }] }),
      r({ source: 'openai', connected: false }),
    ];
    const vs = views(rs);
    expect(vs.map((v) => v.provider)).toEqual(['claude']);
    expect(vs[0].limit?.pct).toBe(62);
    expect(vs[0].hot).toBe(1000);
    expect(windows(vs[0]).map((w) => [w.key, w.current])).toEqual([['session', true], ['today', false], ['week', false], ['month', false]]);
  });

  it('shows claude.ai limits when the Claude Code login has expired', () => {
    const v = view('claude', [
      r({ source: 'claudeplan', updatedAt: 0, error: 'Claude Code login expired', errorKind: 'auth' }),
      r({ source: 'claudeweb', hasUsage: false, limits: [{ key: 'session', label: 'Current session', pct: 2, resetsAt: 5 }, { key: 'week', label: 'This week', pct: 89, resetsAt: 9 }] }),
    ])!;
    expect(v.web?.source).toBe('claudeweb');
    expect(v.limit?.pct).toBe(89);
    expect(v.problem).toBeNull();
  });

  it('never makes up a limit, and shows the worst problem', () => {
    const v = view('chatgpt', [r({ source: 'openai', error: 'OpenAI rejected this key', errorKind: 'auth', today: { ...z, output: 5 } })])!;
    expect(v.limit).toBeNull();
    expect(v.problem?.kind).toBe('auth');
    expect(v.stale).toBe(true);
    expect(windows(v).every((w) => !w.limit)).toBe(true);
    expect(view('chatgpt', [r({ source: 'openai', updatedAt: 0 })])!.loading).toBe(true);
  });

  it('ranks by closeness to limit, else by use today', () => {
    const a = view('claude', [r({ source: 'claudeplan', limits: [{ key: 'session', label: 'x', pct: 40, resetsAt: null }] })])!;
    const b = view('chatgpt', [r({ source: 'chatgptweb', limits: [{ key: 'session', label: 'x', pct: 91, resetsAt: null }] })])!;
    expect(byUrgency([a, b])[0].provider).toBe('chatgpt');
    const c = view('claude', [r({ today: { ...z, input: 10 } })])!;
    const d = view('chatgpt', [r({ source: 'openai', today: { ...z, input: 99 } })])!;
    expect(byUrgency([c, d])[0].provider).toBe('chatgpt');
    expect(tooltip([a, b], 0)).toMatch(/^Claude 40% · ChatGPT 91%/);
  });

  it('formats', () => {
    expect([tone(69), tone(70), tone(90), tone(91), tone(null)]).toEqual(['accent', 'amber', 'amber', 'red', 'none']);
    expect([tokens(950), tokens(12_340), tokens(1_240_000), tokens(25_000_000), tokens(1_193_614_332)]).toEqual(['950', '12k', '1.24M', '25M', '1.19B']);
    expect([modelName('claude-opus-5-5'), modelName('claude-sonnet-5'), modelName('gpt-5.1')]).toEqual(['Opus 5.5', 'Sonnet 5', 'gpt-5.1']);
    expect([until(72 * 60_000, 0), until(5 * 60_000, 0), until(3 * 86_400_000, 0)]).toEqual(['1h 12m', '5m', '3d 0h']);
  });

  it('says when a limit resets, short', () => {
    const now = new Date(2026, 9, 24, 10, 0).getTime();
    expect(resetShort(now + 85 * 60_000, now)).toBe('1h 25m');
    expect(resetShort(new Date(2026, 9, 27, 6, 0).getTime(), now)).toBe('Tue 6 am');
    expect(resetShort(new Date(2026, 9, 27, 18, 30).getTime(), now)).toBe('Tue 6:30 pm');
  });

  it('stacks daily series', () => {
    const s = stack([[['2026-10-23', 1], ['2026-10-24', 2]], [['2026-10-24', 5]]]);
    expect(s.days.map((d) => d.parts)).toEqual([[1, 0], [2, 5]]);
    expect(s.max).toBe(7);
  });
});
