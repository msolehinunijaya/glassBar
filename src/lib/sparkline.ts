/** SVG polyline points for `values` scaled into w×h (0 at the bottom), newest at the right. */
export function points(values: number[], w: number, h: number, max: number): string {
  if (!values.length) return '';
  const step = values.length > 1 ? w / (values.length - 1) : 0;
  return values
    .map((v, i) => {
      const x = values.length > 1 ? i * step : w;
      const y = h - (Math.min(max, Math.max(0, v)) / max) * h;
      return `${+x.toFixed(2)},${+y.toFixed(2)}`;
    })
    .join(' ');
}
