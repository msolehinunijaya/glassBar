// World-clock helpers. Times come from the WebView's built-in IANA time-zone data,
// so daylight saving changes are handled automatically.
/** lat/lon: stored when the city came from the search, for real sunrise/sunset. */
export interface Zone { tz: string; label: string; lat?: number; lon?: number }

/** UTC offset of `tz` at `when`, in minutes (e.g. Asia/Kuala_Lumpur → 480). */
export function offsetMinutes(tz: string, when: Date = new Date()): number {
  try {
    const name = new Intl.DateTimeFormat('en-US', { timeZone: tz, timeZoneName: 'longOffset' })
      .formatToParts(when).find((p) => p.type === 'timeZoneName')?.value ?? 'GMT';
    const m = name.match(/GMT([+-])(\d{1,2})(?::(\d{2}))?/);
    if (!m) return 0; // "GMT" = UTC
    const mins = Number(m[2]) * 60 + Number(m[3] ?? 0);
    return m[1] === '-' ? -mins : mins;
  } catch {
    return 0;
  }
}

export const localOffset = (when: Date = new Date()) => -when.getTimezoneOffset();

/** "Same time", "1h ahead", "2h 30m behind". */
export function diffLabel(local: number, other: number): string {
  const d = other - local;
  if (d === 0) return 'Same time';
  const a = Math.abs(d), h = Math.floor(a / 60), m = a % 60;
  const amount = [h ? `${h}h` : '', m ? `${m}m` : ''].filter(Boolean).join(' ');
  return `${amount} ${d > 0 ? 'ahead' : 'behind'}`;
}

/** City name from a zone id: "America/New_York" → "New York". */
export const cityOf = (tz: string) => tz.split('/').pop()!.replace(/_/g, ' ');

/** The saved world clocks; an older single "second time zone" becomes the first entry. */
export function zonesOf(s: { zones?: Zone[]; secondTz?: string }): Zone[] {
  if (s.zones?.length) return s.zones;
  return s.secondTz ? [{ tz: s.secondTz, label: cityOf(s.secondTz) }] : [];
}

export const isValidZone = (tz: string) => { try { new Intl.DateTimeFormat('en', { timeZone: tz }); return true; } catch { return false; } };
