export type TempUnit = 'c' | 'f';

/** "52°C" / "122°F", or "N/A" when Windows exposes no thermal sensor. */
export function fmtTemp(c: number | null | undefined, unit: TempUnit): string {
  if (c == null) return 'N/A';
  return unit === 'f' ? `${Math.round(c * 9 / 5 + 32)}°F` : `${Math.round(c)}°C`;
}

/** Warning threshold is always in °C. */
export const isHot = (c: number | null | undefined, warnC: number) => c != null && c >= warnC;
