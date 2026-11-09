// GlassBar search filters the dock: while you type, only the widgets that match stay.
// A widget matches by its name, description and related words (KEYWORDS), exactly, as the
// start of a word, or with a small typo; App Launcher and Folders also by their apps' names.

/** Related words per widget id (English and Malay), beyond its name and description. */
export const KEYWORDS: Record<string, string[]> = {
  'ai-usage': ['claude', 'ai', 'tokens', 'limit', 'usage', 'llm', 'chatgpt'],
  apps: ['apps', 'launcher', 'programs', 'pinned', 'taskbar', 'running', 'aplikasi'],
  clock: ['time', 'date', 'calendar', 'alarm', 'world', 'masa', 'tarikh', 'jam', 'kalendar'],
  countdown: ['days', 'deadline', 'event', 'raya', 'merdeka', 'birthday', 'holiday', 'cuti', 'hari'],
  currency: ['money', 'exchange', 'rate', 'convert', 'usd', 'myr', 'ringgit', 'dollar', 'forex', 'duit', 'matawang'],
  devstack: ['dev', 'code', 'server', 'localhost', 'laragon', 'mysql', 'php', 'node', 'docker', 'vite', 'database'],
  downloads: ['download', 'files', 'muat', 'turun', 'fail'],
  equalizer: ['spectrum', 'audio', 'visualizer', 'bass', 'sound', 'music', 'eq'],
  folders: ['folder', 'group', 'apps', 'kumpulan'],
  gmail: ['mail', 'email', 'inbox', 'google', 'message', 'emel', 'surat'],
  ipu: ['air', 'quality', 'pollution', 'haze', 'jerebu', 'api', 'udara', 'asap'],
  mirror: ['camera', 'webcam', 'selfie', 'photo', 'video', 'kamera', 'cermin'],
  music: ['song', 'spotify', 'player', 'media', 'play', 'audio', 'lagu', 'muzik', 'youtube', 'lyrics'],
  notifications: ['alerts', 'notify', 'dnd', 'disturb', 'history', 'notifikasi', 'pemberitahuan'],
  ocr: ['screenshot', 'capture', 'record', 'recording', 'text', 'snip', 'screen', 'tangkap', 'rakam'],
  productivity: ['pomodoro', 'focus', 'timer', 'todo', 'task', 'tasks', 'notes', 'note', 'study', 'work', 'fokus', 'nota', 'tugas'],
  progress: ['day', 'month', 'year', 'percent', 'progress', 'hari', 'bulan', 'tahun'],
  quicksettings: ['wifi', 'bluetooth', 'volume', 'brightness', 'battery', 'network', 'settings', 'internet', 'sound', 'tetapan', 'bateri'],
  screentime: ['usage', 'time', 'apps', 'hours', 'screen', 'masa'],
  shelf: ['files', 'drop', 'park', 'fail'],
  solat: ['prayer', 'azan', 'adhan', 'salah', 'namaz', 'islam', 'mosque', 'hijri', 'subuh', 'zohor', 'asar', 'maghrib', 'isyak', 'waktu', 'masjid'],
  start: ['start', 'menu', 'search', 'windows', 'cari'],
  sysmon: ['cpu', 'ram', 'memory', 'disk', 'network', 'performance', 'gpu', 'storage', 'speed', 'prestasi', 'memori'],
  timer: ['stopwatch', 'countdown', 'alarm', 'lap', 'pemasa', 'jam randik'],
  trash: ['recycle', 'bin', 'delete', 'deleted', 'restore', 'sampah', 'tong', 'padam'],
  water: ['drink', 'hydrate', 'hydration', 'reminder', 'air', 'minum'],
};

const norm = (s: string) => s.toLowerCase().normalize('NFKD').replace(/[\u0300-\u036f]/g, '');
const wordsOf = (s: string) => norm(s).split(/[^\p{L}\p{N}]+/u).filter(Boolean);

/** Edit distance, stopping early once it passes `max`. */
function within(a: string, b: string, max: number): boolean {
  if (Math.abs(a.length - b.length) > max) return false;
  let prev = Array.from({ length: b.length + 1 }, (_, j) => j);
  for (let i = 1; i <= a.length; i++) {
    const cur = [i];
    let best = i;
    for (let j = 1; j <= b.length; j++) {
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      best = Math.min(best, cur[j]);
    }
    if (best > max) return false;
    prev = cur;
  }
  return prev[b.length] <= max;
}

/** One typed word against one widget word: the start of it, or a small typo (1 letter in 4+,
 *  2 in 7+). Words of 1-2 letters only match the start of a word. */
function close(typed: string, word: string, typos: boolean): boolean {
  if (word.startsWith(typed)) return true;
  if (!typos || typed.length < 4) return false;
  const max = typed.length >= 7 ? 2 : 1;
  return within(typed, word, max) || within(typed, word.slice(0, typed.length), max);
}

/** Does `query` match a widget with these texts? Every typed word must match one of them.
 *  `typos`: also allow a small typo (see `pick`, which only does when nothing matches as typed). */
export function matches(query: string, texts: string[], typos = true): boolean {
  const typed = wordsOf(query);
  if (!typed.length) return false;
  const words = texts.flatMap(wordsOf);
  return typed.every((t) => words.some((w) => close(t, w, typos)));
}

/** The items that match `query`: those that match as typed, or, only when none does, those
 *  within a small typo ("wether" finds Weather, but "solat" doesn't also bring "Solar"). */
export function pick<T>(query: string, items: T[], texts: (t: T) => string[]): T[] {
  const exact = items.filter((t) => matches(query, texts(t), false));
  return exact.length ? exact : items.filter((t) => matches(query, texts(t), true));
}

/** The searchable texts of a dock widget: its name, description, related words, and for the
 *  App Launcher and Folders the names of the apps in them. */
export function textsOf(id: string, name: string, description: string, settings: unknown): string[] {
  const s = (settings ?? {}) as { pinned?: Pin[]; folders?: { name?: string; apps?: Pin[] }[] };
  const pins = [...(s.pinned ?? []), ...(s.folders ?? []).flatMap((f) => f.apps ?? [])];
  return [id, name, description, ...(KEYWORDS[id] ?? []), ...(s.folders ?? []).map((f) => f.name ?? ''), ...pins.map(pinName)];
}

type Pin = { path: string; name?: string };
/** "C:\…\chrome.exe" → "chrome"; a named pin (web or Store app) → its name. */
const pinName = (p: Pin) => p.name ?? p.path.split(/[\\/]/).pop()?.replace(/\.(exe|lnk|url)$/i, '') ?? '';
