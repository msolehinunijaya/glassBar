// Focus music: YouTube links played as sound during a focus session (Pomodoro settings).

/** The video id in a YouTube link (watch, youtu.be, shorts, live, embed), or null. */
export function youtubeId(link: string): string | null {
  let u: URL;
  try { u = new URL(link.trim()); } catch { return null; }
  const host = u.hostname.replace(/^(www|m|music)\./, '');
  const ok = (id: string | null | undefined) => (id && /^[\w-]{11}$/.test(id) ? id : null);
  if (host === 'youtu.be') return ok(u.pathname.split('/')[1]);
  if (host !== 'youtube.com' && host !== 'youtube-nocookie.com') return null;
  if (u.pathname === '/watch') return ok(u.searchParams.get('v'));
  const [, kind, id] = u.pathname.split('/');
  return ['shorts', 'live', 'embed', 'v'].includes(kind) ? ok(id) : null;
}

/** The player, starting on the first link. The dock moves it through the rest itself
 *  (FocusMusic: loadVideoById), since YouTube's own playlist stops at a link that can't play.
 *  enablejsapi lets the dock pause, resume and change it. */
export function playerUrl(id: string, origin: string): string {
  const q = new URLSearchParams({ autoplay: '1', controls: '0', enablejsapi: '1', origin });
  return `https://www.youtube.com/embed/${id}?${q}`;
}

/** The link after `i` that should play next: on in order, back to the first after the last. */
export const nextIndex = (i: number, n: number) => (n ? (i + 1) % n : 0);
