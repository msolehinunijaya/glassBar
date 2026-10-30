// Unit conversion for the search panel: length, mass, temperature, data, time, speed, area
// and volume. Each unit is a factor to its kind's base unit (temperature has offsets).

export interface Unit { key: string; kind: string; label: string; f: number; names: string[] }

const U = (key: string, kind: string, label: string, f: number, names: string[]): Unit => ({ key, kind, label, f, names: [key, ...names] });

export const UNITS: Unit[] = [
  U('mm', 'length', 'mm', 0.001, ['millimeter', 'millimeters', 'millimetre', 'millimetres']),
  U('cm', 'length', 'cm', 0.01, ['centimeter', 'centimeters', 'centimetre', 'centimetres']),
  U('m', 'length', 'm', 1, ['meter', 'meters', 'metre', 'metres']),
  U('km', 'length', 'km', 1000, ['kms', 'kilometer', 'kilometers', 'kilometre', 'kilometres']),
  U('in', 'length', 'in', 0.0254, ['inch', 'inches', '"']),
  U('ft', 'length', 'ft', 0.3048, ['foot', 'feet', "'"]),
  U('yd', 'length', 'yd', 0.9144, ['yard', 'yards']),
  U('mi', 'length', 'mi', 1609.344, ['mile', 'miles']),
  U('nmi', 'length', 'nmi', 1852, ['nautical mile', 'nautical miles']),
  U('mg', 'mass', 'mg', 1e-6, ['milligram', 'milligrams']),
  U('g', 'mass', 'g', 0.001, ['gram', 'grams', 'gm']),
  U('kg', 'mass', 'kg', 1, ['kgs', 'kilo', 'kilos', 'kilogram', 'kilograms']),
  U('t', 'mass', 't', 1000, ['tonne', 'tonnes', 'ton', 'tons']),
  U('oz', 'mass', 'oz', 0.028349523125, ['ounce', 'ounces']),
  U('lb', 'mass', 'lb', 0.45359237, ['lbs', 'pound', 'pounds']),
  U('st', 'mass', 'st', 6.35029318, ['stone', 'stones']),
  U('c', 'temp', '°C', 1, ['°c', 'celsius', 'degc']),
  U('f', 'temp', '°F', 1, ['°f', 'fahrenheit', 'degf']),
  U('k', 'temp', 'K', 1, ['kelvin']),
  U('b', 'data', 'B', 1, ['byte', 'bytes']),
  U('kb', 'data', 'KB', 1e3, ['kilobyte', 'kilobytes']),
  U('mb', 'data', 'MB', 1e6, ['megabyte', 'megabytes']),
  U('gb', 'data', 'GB', 1e9, ['gigabyte', 'gigabytes']),
  U('tb', 'data', 'TB', 1e12, ['terabyte', 'terabytes']),
  U('kib', 'data', 'KiB', 1024, []),
  U('mib', 'data', 'MiB', 1024 ** 2, []),
  U('gib', 'data', 'GiB', 1024 ** 3, []),
  U('tib', 'data', 'TiB', 1024 ** 4, []),
  U('ms', 'time', 'ms', 0.001, ['millisecond', 'milliseconds']),
  U('s', 'time', 's', 1, ['sec', 'secs', 'second', 'seconds']),
  U('min', 'time', 'min', 60, ['mins', 'minute', 'minutes']),
  U('h', 'time', 'h', 3600, ['hr', 'hrs', 'hour', 'hours']),
  U('d', 'time', 'days', 86400, ['day', 'days']),
  U('wk', 'time', 'weeks', 604800, ['week', 'weeks']),
  U('yr', 'time', 'years', 31557600, ['year', 'years']),
  U('kmh', 'speed', 'km/h', 1 / 3.6, ['km/h', 'kph']),
  U('mph', 'speed', 'mph', 0.44704, []),
  U('mps', 'speed', 'm/s', 1, ['m/s']),
  U('kn', 'speed', 'knots', 0.514444, ['knot', 'knots']),
  U('m2', 'area', 'm²', 1, ['sqm', 'm²']),
  U('km2', 'area', 'km²', 1e6, ['km²']),
  U('ft2', 'area', 'ft²', 0.09290304, ['sqft', 'ft²']),
  U('ha', 'area', 'ha', 1e4, ['hectare', 'hectares']),
  U('acre', 'area', 'acres', 4046.8564224, ['acres']),
  U('ml', 'volume', 'ml', 0.001, ['milliliter', 'milliliters', 'millilitre', 'millilitres']),
  U('l', 'volume', 'L', 1, ['liter', 'liters', 'litre', 'litres']),
  U('m3', 'volume', 'm³', 1000, ['m³']),
  U('gal', 'volume', 'gal', 3.785411784, ['gallon', 'gallons']),
  U('qt', 'volume', 'qt', 0.946352946, ['quart', 'quarts']),
  U('cup', 'volume', 'cups', 0.2365882365, ['cups']),
  U('floz', 'volume', 'fl oz', 0.0295735295625, ['fl oz']),
];

const BY_NAME = new Map(UNITS.flatMap((u) => u.names.map((n) => [n, u] as const)));
export const unitOf = (word: string): Unit | null => BY_NAME.get(word.trim().toLowerCase()) ?? null;

const toC = (v: number, u: string) => (u === 'c' ? v : u === 'f' ? ((v - 32) * 5) / 9 : v - 273.15);
const fromC = (v: number, u: string) => (u === 'c' ? v : u === 'f' ? (v * 9) / 5 + 32 : v + 273.15);

/** `v` in `from` as `to` (unit keys or names), or null across kinds / unknown units. */
export function convert(v: number, from: string, to: string): number | null {
  const a = unitOf(from), b = unitOf(to);
  if (!a || !b || a.kind !== b.kind) return null;
  if (a.kind === 'temp') return fromC(toC(v, a.key), b.key);
  return (v * a.f) / b.f;
}

const SUGGEST: Record<string, string[]> = {
  km: ['mi', 'm', 'ft'], mi: ['km', 'm', 'ft'], m: ['ft', 'cm', 'in'], cm: ['in', 'mm', 'm'], mm: ['in', 'cm'], in: ['cm', 'mm', 'ft'],
  ft: ['m', 'cm', 'in'], yd: ['m', 'ft'], kg: ['lb', 'g', 'oz'], lb: ['kg', 'g', 'oz'], g: ['oz', 'mg', 'kg'], oz: ['g', 'lb', 'kg'],
  c: ['f', 'k'], f: ['c', 'k'], k: ['c', 'f'], gb: ['mb', 'gib', 'tb'], mb: ['kb', 'gb', 'mib'], tb: ['gb', 'tib'],
  min: ['h', 's'], h: ['min', 'd'], d: ['h', 'wk'], kmh: ['mph', 'mps'], mph: ['kmh', 'mps'], l: ['ml', 'gal', 'cup'], gal: ['l', 'qt'],
};

/** A bare "5 km": a few useful conversions. */
export function suggestions(v: number, unit: string): { to: string; value: number }[] {
  const u = unitOf(unit);
  if (!u) return [];
  const to = SUGGEST[u.key] ?? UNITS.filter((x) => x.kind === u.kind && x.key !== u.key).slice(0, 3).map((x) => x.key);
  return to.map((t) => ({ to: t, value: convert(v, u.key, t)! }));
}

export const labelOf = (key: string) => unitOf(key)?.label ?? key;
