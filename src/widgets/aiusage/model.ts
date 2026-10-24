// AI Usage: readings from several sources → one view per provider. Pure, see model.test.ts.
import type { AiuLimit, AiuReading, AiuSource, AiuTotals } from '../../lib/ipc';

export type Provider = 'claude' | 'chatgpt';

export const PROVIDERS: Record<Provider, { name: string; usage: AiuSource[]; web: AiuSource }> = {
  claude: { name: 'Claude', usage: ['claudecode', 'anthropic'], web: 'claudeplan' },
  chatgpt: { name: 'ChatGPT', usage: ['openai'], web: 'chatgptweb' },
};

export const SOURCE_LABEL: Record<AiuSource, string> = {
  anthropic: 'Admin API', claudecode: 'Local logs', openai: 'Admin API', claudeplan: 'Claude Code login, unofficial', chatgptweb: 'Web, unofficial',
};

export interface View {
  provider: Provider;
  name: string;
  /** The limit closest to full, if the provider publishes any. */
  limit: AiuLimit | null;
  limits: AiuLimit[];
  /** Token data (local logs or Admin API), the chosen one of `usageSources`. */
  usage: AiuReading | null;
  usageSources: AiuReading[];
  web: AiuReading | null;
  /** Never read yet: show the skeleton. */
  loading: boolean;
  /** Problems to show, worst first (auth, broken web page, offline). */
  problems: { kind: string; text: string; source: AiuSource }[];
  problem: { kind: string; text: string; source: AiuSource } | null;
  /** Some shown data is old (a refresh failed). */
  stale: boolean;
  updatedAt: number;
  /** Tokens of the current window (session if one is running, else today). */
  hot: number;
}

const sum = (t: AiuTotals) => t.input + t.output + t.cacheRead + t.cacheWrite;
export const totalOf = sum;

/** One provider's view, or null when none of its sources is connected. `prefer` picks the
 *  usage source when it has several (else the first with data). */
export function view(p: Provider, rs: AiuReading[], prefer?: string): View | null {
  const by = (s: AiuSource) => rs.find((r) => r.source === s && r.connected) ?? null;
  const usageSources = PROVIDERS[p].usage.map(by).filter((r): r is AiuReading => !!r);
  const web = by(PROVIDERS[p].web);
  if (!usageSources.length && !web) return null;
  const usage = usageSources.find((r) => r.source === prefer) ?? usageSources.find((r) => r.updatedAt) ?? usageSources[0] ?? null;
  const limits = web?.updatedAt ? web.limits : [];
  const limit = limits.reduce<AiuLimit | null>((a, l) => (!a || l.pct > a.pct ? l : a), null);
  const shown = [usage, web].filter((r): r is AiuReading => !!r);
  const failing = shown.filter((r) => r.error);
  const rank = (k: string | null) => ({ auth: 0, broken: 1, offline: 2 } as Record<string, number>)[k ?? ''] ?? 3;
  const problems = failing.sort((a, b) => rank(a.errorKind) - rank(b.errorKind)).map((r) => ({ kind: r.errorKind ?? 'other', text: r.error!, source: r.source }));
  return {
    provider: p, name: PROVIDERS[p].name, limit, limits, usage, usageSources, web,
    loading: shown.every((r) => !r.updatedAt && !r.error),
    problems, problem: problems[0] ?? null,
    stale: failing.some((r) => r.updatedAt > 0),
    updatedAt: Math.max(0, ...shown.map((r) => r.updatedAt)),
    hot: usage ? sum(usage.session?.totals ?? usage.today) : 0,
  };
}

export const views = (rs: AiuReading[], prefer: Partial<Record<Provider, string>> = {}) =>
  (['claude', 'chatgpt'] as Provider[]).map((p) => view(p, rs, prefer[p])).filter((v): v is View => !!v);

/** Most urgent first: closest to its limit; without limits, most used today. */
export function byUrgency(vs: View[]): View[] {
  return [...vs].sort((a, b) => {
    if (a.limit || b.limit) return (b.limit?.pct ?? -1) - (a.limit?.pct ?? -1);
    return b.hot - a.hot;
  });
}

export type Tone = 'accent' | 'amber' | 'red' | 'none';
/** Colour by how full, not by brand: < 70 accent, 70–90 amber, > 90 red. */
export const tone = (pct: number | null | undefined): Tone => (pct == null ? 'none' : pct > 90 ? 'red' : pct >= 70 ? 'amber' : 'accent');
export const TONE: Record<Tone, string> = { accent: 'var(--accent)', amber: '#ff9f0a', red: '#ff453a', none: 'var(--fg-3)' };

/** 950 → "950", 12_340 → "12k", 1_240_000 → "1.24M", 1_190_000_000 → "1.19B". */
export function tokens(n: number): string {
  if (n < 1000) return String(n);
  if (n < 1_000_000) return `${+(n / 1000).toFixed(n < 10_000 ? 1 : 0)}k`;
  if (n < 1_000_000_000) return `${+(n / 1_000_000).toFixed(n < 10_000_000 ? 2 : 1)}M`;
  return `${+(n / 1_000_000_000).toFixed(2)}B`;
}

export const pct = (p: number) => `${Math.round(p)}%`;

/** "claude-opus-5-5" → "Opus 5.5"; other names unchanged. */
export function modelName(id: string): string {
  const m = /^claude-([a-z]+)-(\d+)(?:-(\d{1,2}))?(?:-\d{8})?$/.exec(id);
  if (!m) return id;
  return `${m[1][0].toUpperCase()}${m[1].slice(1)} ${m[2]}${m[3] ? `.${m[3]}` : ''}`;
}

/** Time left: "1h 12m", "12m", "2d 4h". */
export function until(at: number, now: number): string {
  const m = Math.max(0, Math.round((at - now) / 60_000));
  if (m >= 48 * 60) return `${Math.floor(m / 1440)}d ${Math.floor((m % 1440) / 60)}h`;
  if (m >= 60) return `${Math.floor(m / 60)}h ${m % 60}m`;
  return `${m}m`;
}

/** When a limit resets, short: time left under a day ("1h 25m"), else the day and hour ("Tue 6 am"). */
export function resetShort(at: number, now: number): string {
  if (at - now < 86_400_000) return until(at, now);
  const d = new Date(at);
  const h = d.getHours(), m = d.getMinutes();
  const t = `${h % 12 || 12}${m ? `:${String(m).padStart(2, '0')}` : ''} ${h < 12 ? 'am' : 'pm'}`;
  return `${d.toLocaleDateString('en', { weekday: 'short' })} ${t}`;
}

/** Plan-limit row labels: "Session", "Weekly", "Weekly · Opus". */
export const limitName = (l: { key: string; label: string }) => (l.key === 'session' ? 'Session' : l.key === 'week' ? 'Weekly' : l.label);

/** "3:40 pm" today, "Mon 3:40 pm" on another day. */
export function at(ms: number, now: number): string {
  const d = new Date(ms);
  const t = d.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' }).toLowerCase();
  return d.toDateString() === new Date(now).toDateString() ? t : `${d.toLocaleDateString(undefined, { weekday: 'short' })} ${t}`;
}

/** The dock tooltip: "Claude 62% · ChatGPT 18% · resets 3:40 pm". */
export function tooltip(vs: View[], now: number): string {
  if (!vs.length) return 'AI Usage: connect Claude or ChatGPT';
  const parts = vs.map((v) => `${v.name} ${v.limit ? pct(v.limit.pct) : `${tokens(v.hot)} tok`}`);
  const r = byUrgency(vs)[0].limit?.resetsAt;
  return [...parts, ...(r ? [`resets ${at(r, now)}`] : [])].join(' · ');
}

/** Usage windows for the flyout: label, tokens (if known), limit (if published). */
export function windows(v: View): { key: string; label: string; tokens: number | null; limit: AiuLimit | null; current: boolean }[] {
  const u = v.usage;
  const lim = (k: string) => v.limits.find((l) => l.key === k) ?? null;
  const session = u?.session ?? null;
  const rows = [];
  if (session || lim('session')) rows.push({ key: 'session', label: 'Session (5h)', tokens: session ? sum(session.totals) : null, limit: lim('session') });
  rows.push({ key: 'today', label: 'Today', tokens: u ? sum(u.today) : null, limit: null });
  rows.push({ key: 'week', label: 'This week', tokens: u ? sum(u.week) : null, limit: lim('week') });
  rows.push({ key: 'month', label: 'This month', tokens: u ? sum(u.month) : null, limit: null });
  for (const l of v.limits) if (!['session', 'week'].includes(l.key)) rows.push({ key: l.key, label: l.label, tokens: null, limit: l });
  const cur = session || lim('session') ? 'session' : 'today';
  return rows.filter((r) => r.tokens != null || r.limit).map((r) => ({ ...r, current: r.key === cur }));
}

/** 30 days of several series → stacked bars: each day's parts and the tallest day. */
export function stack(series: [string, number][][]): { days: { date: string; parts: number[]; total: number }[]; max: number } {
  const dates = [...new Set(series.flatMap((s) => s.map((d) => d[0])))].sort().slice(-30);
  const days = dates.map((date) => {
    const parts = series.map((s) => s.find((d) => d[0] === date)?.[1] ?? 0);
    return { date, parts, total: parts.reduce((a, b) => a + b, 0) };
  });
  return { days, max: Math.max(1, ...days.map((d) => d.total)) };
}

/** Money: "USD 1.23", "RM 5.80". */
export const money = (usd: number, cur: 'USD' | 'MYR', rate = 1) =>
  `${cur === 'MYR' ? 'RM' : 'USD'} ${(usd * rate).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
