/** Rates are "1 base = r units"; amount is in the base currency. */
export function convert(amount: number, rates: Record<string, number>, to: string): number | null {
  const r = rates[to];
  return r == null || !Number.isFinite(amount) ? null : amount * r;
}

/** 2 decimals normally; enough significant digits for tiny values. */
export function fmtAmount(n: number): string {
  const abs = Math.abs(n);
  const opts: Intl.NumberFormatOptions = abs > 0 && abs < 0.01
    ? { maximumSignificantDigits: 4 } : { minimumFractionDigits: 2, maximumFractionDigits: 2 };
  return new Intl.NumberFormat('en-US', opts).format(n);
}

export const pairKey = (from: string, to: string) => `${from.toUpperCase()}>${to.toUpperCase()}`;
export function parsePair(k: string): [string, string] | null {
  const m = /^([A-Z]{3})>([A-Z]{3})$/.exec(k);
  return m ? [m[1], m[2]] : null;
}

const names = (() => { try { return new Intl.DisplayNames(undefined, { type: 'currency' }); } catch { return null; } })();
export const currencyName = (code: string) => names?.of(code) ?? code;
