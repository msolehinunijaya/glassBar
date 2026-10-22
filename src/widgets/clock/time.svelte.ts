// A clock that wakes only on minute (or second) boundaries: no busy timers.
export function ticker(seconds: () => boolean) {
  const s = $state({ now: new Date() });
  $effect(() => {
    const everySecond = seconds();
    let t: ReturnType<typeof setTimeout>;
    const tick = () => {
      const d = new Date(); // read the local, not s.now: reading it here would re-run this effect
      s.now = d;
      const ms = d.getSeconds() * 1000 + d.getMilliseconds();
      t = setTimeout(tick, (everySecond ? 1000 - (ms % 1000) : 60000 - ms) + 5);
    };
    tick();
    return () => clearTimeout(t);
  });
  return s;
}

export function fmtTime(d: Date, hour12: boolean, seconds = false, timeZone?: string) {
  try {
    return new Intl.DateTimeFormat(undefined, {
      hour: '2-digit', minute: '2-digit', second: seconds ? '2-digit' : undefined, hour12, timeZone: timeZone || undefined,
    }).format(d);
  } catch {
    return '--:--'; // invalid time zone name
  }
}
