// Pure helpers for the Downloads widget (tested in format.test.ts).

/** Cut a long name in the middle so the extension stays visible: "Invoice_Oct…2026.pdf". */
export function cutMiddle(name: string, max = 22): string {
  if (name.length <= max) return name;
  const dot = name.lastIndexOf('.');
  const ext = dot > 0 && name.length - dot <= 8 ? name.slice(dot) : '';
  const base = ext ? name.slice(0, dot) : name;
  const room = Math.max(2, max - ext.length - 1);
  const head = Math.ceil(room * 0.6), tail = room - head;
  return `${base.slice(0, head)}…${tail ? base.slice(-tail) : ''}${ext}`;
}

/** 0 B, 512 B, 1.5 KB, 23 MB, 2.4 GB (1 KB = 1024 B, like File Explorer). */
export function fmtSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 1024) return `${Math.max(0, Math.round(bytes || 0))} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let v = bytes / 1024, i = 0;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return `${v < 10 ? Math.round(v * 10) / 10 : Math.round(v)} ${units[i]}`;
}

/** "just now", "2 min ago", "3 h ago", "yesterday", "4 days ago". */
export function ago(at: number, now = Date.now()): string {
  const s = Math.max(0, Math.round((now - at) / 1000));
  if (s < 60) return 'just now';
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  const d = Math.floor(s / 86400);
  return d === 1 ? 'yesterday' : `${d} days ago`;
}

/** Row time: "Today 2:05 PM", "Yesterday", or the date. */
export function when(at: number, now = Date.now(), locale?: string): string {
  const day = (t: number) => new Date(t).toDateString();
  if (day(at) === day(now)) return `Today ${new Intl.DateTimeFormat(locale, { hour: 'numeric', minute: '2-digit' }).format(at)}`;
  if (day(at) === day(now - 86_400_000)) return 'Yesterday';
  return new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'short', year: new Date(at).getFullYear() === new Date(now).getFullYear() ? undefined : 'numeric' }).format(at);
}

export type Kind = 'documents' | 'images' | 'videos' | 'archives' | 'installers' | 'other';
const KINDS: Record<Exclude<Kind, 'other'>, string[]> = {
  documents: ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'csv', 'ppt', 'pptx', 'txt', 'rtf', 'odt', 'ods', 'odp', 'md', 'epub', 'json', 'xml', 'html', 'htm'],
  images: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg', 'heic', 'heif', 'tif', 'tiff', 'ico', 'avif', 'raw'],
  videos: ['mp4', 'mkv', 'mov', 'avi', 'webm', 'wmv', 'm4v', 'flv', '3gp'],
  archives: ['zip', 'rar', '7z', 'tar', 'gz', 'tgz', 'bz2', 'xz', 'iso', 'cab'],
  installers: ['exe', 'msi', 'msix', 'msixbundle', 'appx', 'appxbundle', 'apk', 'dmg'],
};
/** Which filter a file belongs to, by its extension. */
export function kindOf(name: string): Kind {
  const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : '';
  for (const [k, list] of Object.entries(KINDS)) if (list.includes(ext)) return k as Kind;
  return 'other';
}
