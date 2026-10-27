<script lang="ts">
  // The whole song's lyrics in the music panel. The line being sung sits in the middle,
  // larger; sung lines are dimmer, coming ones dimmest. It follows the song with a smooth
  // scroll. Scrolling yourself pauses that: a "Back to now" chip jumps back, and following
  // resumes on its own 4 s after you stop. Click a line (or ↑/↓ then Enter) to jump the song
  // there, when the playing app allows seeking.
  import { ArrowDownToLine } from '@lucide/svelte';
  import { ipc, type LyricLine, type MediaInfo } from '../../lib/ipc';
  import { currentIndex } from './lyrics';

  let { lines, media, pos, source }: { lines: LyricLine[]; media: { m: MediaInfo | null }; pos: number; source: string } = $props();

  // Words only: the empty "gap" lines would show as blank rows.
  const rows = $derived(lines.map((l, i) => ({ ...l, i })).filter((l) => l.text.trim()));
  const now = $derived(currentIndex(lines, pos));
  /** The row being sung: the last one at or before the current line. */
  const sung = $derived(rows.reduce((at, r, k) => (r.i <= now ? k : at), -1));
  // A clicked row is "now" straight away, until the song's position catches up with it
  // (or 2 s pass): no flash of the old line, no scroll back to it first.
  let jumped = $state<number | null>(null);
  let jumpTimer: ReturnType<typeof setTimeout>;
  $effect(() => { if (jumped !== null && sung === jumped) jumped = null; });
  $effect(() => () => clearTimeout(jumpTimer));
  const cur = $derived(jumped ?? sung);
  // Some apps (and live streams) can't seek: then lines aren't buttons.
  const canSeek = $derived(!!media.m && media.m.durationMs > 0);

  let list = $state<HTMLElement>();
  let following = $state(true);
  let resume: ReturnType<typeof setTimeout>;
  let focusRow = $state(-1);

  function center(k: number, smooth = true) {
    const el = list?.querySelectorAll<HTMLElement>('.row')[k];
    if (!list || !el) return;
    list.scrollTo({ top: el.offsetTop - list.clientHeight / 2 + el.offsetHeight / 2, behavior: smooth ? 'smooth' : 'auto' });
  }
  // Follow the song. The first placement is instant, so the panel opens on the right line.
  let placed = false;
  $effect(() => {
    const k = cur;
    if (!following || k < 0) return;
    center(k, placed);
    placed = true;
  });

  /** The user scrolled: stop following for a while. */
  function userScrolled() {
    following = false;
    clearTimeout(resume);
    resume = setTimeout(() => (following = true), 4000);
  }
  $effect(() => () => clearTimeout(resume));
  function backToNow() { clearTimeout(resume); following = true; center(cur); }

  function seek(k: number) {
    const r = rows[k];
    if (!r || !canSeek || !media.m) return;
    // A hair past the line's start, so the position lands inside it, not on the boundary.
    const at = r.atMs + 30;
    jumped = k;
    clearTimeout(jumpTimer);
    jumpTimer = setTimeout(() => (jumped = null), 2000);
    clearTimeout(resume);
    following = true;
    focusRow = -1;
    media.m = { ...media.m, positionMs: at, updatedAtMs: Date.now() };
    ipc.mediaSeek(at);
  }
  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const from = focusRow < 0 ? Math.max(cur, 0) : focusRow;
      focusRow = Math.max(0, Math.min(rows.length - 1, from + (e.key === 'ArrowDown' ? 1 : -1)));
      userScrolled();
      center(focusRow);
    } else if (e.key === 'Enter' && focusRow >= 0) seek(focusRow);
  }
</script>

<section class="lyrics" aria-label="Lyrics">
  <div class="list" bind:this={list} tabindex="0" role="listbox" aria-label="Lyrics, ↑/↓ to choose a line, Enter to play from it" onwheel={userScrolled} onpointerdown={userScrolled} onkeydown={key}>
    {#each rows as r, k (r.i)}
      <button class="row" role="option" aria-selected={k === cur} class:now={k === cur} class:past={k < cur} class:focus={k === focusRow}
              disabled={!canSeek} tabindex="-1" onclick={() => seek(k)}
              title={canSeek ? 'Play from here' : "This app doesn't allow jumping to a moment in the song"}>{r.text}</button>
    {/each}
  </div>
  {#if !following && cur >= 0}
    <button class="back" onclick={backToNow}><ArrowDownToLine size={12} /> Back to now</button>
  {/if}
  <p class="src dim">Lyrics: {source === 'local' ? 'local .lrc file' : 'LRCLIB'}</p>
</section>

<style>
  .lyrics { position: relative; display: flex; flex-direction: column; min-height: 0; }
  .list {
    flex: 1; min-height: 0; overflow-y: auto; scrollbar-width: none; padding: 40% 2px;
    /* Lines fade towards the top and bottom edges. */
    mask-image: linear-gradient(transparent, #000 18%, #000 82%, transparent);
    outline: none;
  }
  .list::-webkit-scrollbar { display: none; }
  .row {
    display: block; width: 100%; text-align: left; padding: 6px 10px; border-radius: var(--r-inner);
    font-size: 14px; font-weight: 500; line-height: 1.35; color: var(--fg-3, var(--fg-dim)); opacity: 0.75;
    transform-origin: left center;
    transition: color 0.3s var(--ease), opacity 0.3s var(--ease), transform 0.3s var(--ease), background 0.2s;
  }
  .row.past { color: var(--fg-dim); opacity: 0.8; }
  .row.now { color: var(--fg); opacity: 1; font-weight: 600; transform: scale(1.12); }
  .row:not(:disabled):hover { background: var(--pill-hover); }
  .row.focus { box-shadow: inset 0 0 0 2px var(--accent); }
  .row:disabled { cursor: default; }
  .back {
    position: absolute; left: 50%; bottom: 30px; translate: -50% 0; display: inline-flex; align-items: center; gap: 6px;
    padding: 6px 12px; border-radius: var(--r-chip); background: var(--accent); color: var(--on-accent); font-size: 11.5px; font-weight: 500;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.25); animation: rise 0.18s var(--ease);
  }
  .back:active { transform: scale(0.96); }
  @keyframes rise { from { opacity: 0; translate: -50% 6px; } }
  .src { font-size: 10.5px; margin: 6px 0 0; text-align: center; }
  @media (prefers-reduced-motion: reduce) { .row, .back { transition: none; animation: none; } }
</style>
