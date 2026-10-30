// What the search panel's input means: a sum, a unit conversion, a currency conversion, or
// nothing (then only apps and "search Windows / the web"). Tested in calc.test.ts.
import { evaluate } from './calc';
import { convert, suggestions, unitOf } from './units';

export type Answer =
  | { kind: 'math'; value: number }
  | { kind: 'unit'; value: number; amount: number; from: string; to: string }
  | { kind: 'units'; amount: number; from: string; list: { to: string; value: number }[] }
  | { kind: 'fx'; amount: number; from: string; to: string | null };

const CODES = new Set(['MYR', 'USD', 'EUR', 'GBP', 'SGD', 'JPY', 'CNY', 'HKD', 'IDR', 'THB', 'PHP', 'VND', 'INR', 'AUD', 'NZD', 'CAD',
  'CHF', 'KRW', 'TWD', 'SAR', 'AED', 'QAR', 'KWD', 'BND', 'PKR', 'BDT', 'LKR', 'TRY', 'EGP', 'ZAR', 'SEK', 'NOK', 'DKK', 'PLN', 'BRL', 'MXN', 'RUB']);
const SYMBOLS: Record<string, string> = { $: 'USD', 'us$': 'USD', '€': 'EUR', '£': 'GBP', '¥': 'JPY', rm: 'MYR', 's$': 'SGD', rp: 'IDR', '฿': 'THB', '₹': 'INR', '₩': 'KRW', a$: 'AUD' };
const cur = (w: string | undefined) => {
  if (!w) return null;
  const s = w.toLowerCase();
  if (SYMBOLS[s]) return SYMBOLS[s];
  const u = w.toUpperCase();
  return CODES.has(u) ? u : null;
};
const num = (s: string) => parseFloat(s.replace(/,/g, ''));
const SEP = '(?:to|in|into|as|=|->|→)';

export function parse(q: string): Answer | null {
  const s = q.trim();
  if (!s) return null;
  // Money: "100 usd to myr", "rm 50 in sgd", "$20", "€15 to gbp".
  const fx = new RegExp(`^(us\\$|s\\$|a\\$|[$€£¥฿₹₩]|rm|rp|[a-z]{3})?\\s*([\\d.,]+)\\s*([a-z]{3})?\\s*(?:${SEP}\\s*([a-z]{3}))?$`, 'i').exec(s);
  if (fx && (fx[1] || fx[3])) {
    const from = cur(fx[1]) ?? cur(fx[3]);
    const to = fx[4] ? cur(fx[4]) : null;
    if (from && (!fx[4] || to) && !(fx[1] && fx[3])) return { kind: 'fx', amount: num(fx[2]), from, to };
  }
  // Units: "5 km to mi", "100 f in c", "5 km".
  const un = new RegExp(`^(-?[\\d.,]+)\\s*([^\\d\\s][^\\s]*(?:\\s(?:oz|mile|miles))?)\\s*(?:${SEP}\\s*(.*))?$`, 'i').exec(s);
  if (un) {
    const amount = num(un[1]);
    const from = unitOf(un[2]);
    if (from && Number.isFinite(amount)) {
      if (un[3] === undefined) {
        if (/\s(to|in|into|as)\s*$/i.test(s)) return null;
        return { kind: 'units', amount, from: from.key, list: suggestions(amount, from.key) };
      }
      const to = unitOf(un[3]);
      if (!to || to.kind !== from.kind) return null;
      return { kind: 'unit', amount, from: from.key, to: to.key, value: convert(amount, from.key, to.key)! };
    }
    if (/\s(to|in)\s*$/i.test(s)) return null;
  }
  if (/\s(to|in|into)\s*$/i.test(s)) return null;
  const v = evaluate(s);
  return v === null ? null : { kind: 'math', value: v };
}

/** "1,234,567.891"; float noise (0.1 + 0.2) rounded away; huge or tiny numbers as JS writes them. */
export function formatNumber(n: number, maxDecimals = 10): string {
  if (!Number.isFinite(n)) return String(n);
  if (Math.abs(n) >= 1e21) return n.toString();
  const clean = Number(n.toPrecision(12));
  if (clean !== 0 && Math.abs(clean) < 10 ** -Math.min(6, maxDecimals)) return clean.toString();
  return clean.toLocaleString('en-US', { maximumFractionDigits: maxDecimals });
}
