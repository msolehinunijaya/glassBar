<script lang="ts">
  // The lyric line in the dock, changing in time with the song.
  //
  // Each line is keyed, so the old one leaves while the new one arrives. Both sit absolutely
  // inside a fixed-size box, so they cross over without ever moving the dock, and a long
  // line can never push the other widgets off it. A line too long for the box first drops one
  // size, then wraps onto two lines (fitLine in lyrics.ts); a compact dock keeps one line.
  // A long instrumental stretch shows three soft dots instead of an empty box.
  //
  // The karaoke wipe is not a transition: it is a clip on a bright copy of the line, moved
  // once a second with a 1 s linear transition -- the same trick the progress line uses. That
  // keeps it smooth without a frame loop. Karaoke never wraps: a wipe across two lines reads
  // as both filling at once.
  //
  // No ARIA live region on purpose: announcing every lyric line would talk over everything
  // else a screen reader has to say. The text is in the DOM to be read, not shouted.
  import { cubicOut } from 'svelte/easing';
  import { fade, fly, type TransitionConfig } from 'svelte/transition';
  import { fitLine } from './lyrics';
  import type { LyricsSettings } from './settings';

  let { text, idx, progress, side, opts, gap = false }: { text: string; idx: number; progress: number; side: 'left' | 'right'; opts: LyricsSettings; gap?: boolean } = $props();

  /** Reveals the line left to right without laying anything out per character. */
  const typewriter = (_: Element, { duration = 300 }: { duration?: number } = {}): TransitionConfig => ({
    duration, easing: cubicOut,
    css: (t: number) => `clip-path: inset(0 ${(1 - t) * 100}% 0 0)`,
  });

  /** One directive, the user's choice inside it. */
  function arrive(node: Element, p: { anim: LyricsSettings['anim']; side: 'left' | 'right' }): TransitionConfig {
    switch (p.anim) {
      case 'slide-up': return fly(node, { y: 10, duration: 260, easing: cubicOut });
      case 'slide-side': return fly(node, { x: p.side === 'left' ? -14 : 14, duration: 260, easing: cubicOut });
      case 'typewriter': return typewriter(node, { duration: 300 });
      case 'none': return { duration: 0 };
      // Karaoke still fades in; the wipe is what makes it karaoke.
      default: return fade(node, { duration: 220 });
    }
  }

  // Measure with a canvas in the dock's own font: no layout, no reflow.
  let box = $state<HTMLElement>();
  let ctx: CanvasRenderingContext2D | null = null;
  const fit = $derived.by(() => {
    const size = +opts.size;
    if (!box || !text) return { size, wrap: false };
    const family = getComputedStyle(box).fontFamily;
    const shown = opts.caps ? text.toUpperCase() : text;
    ctx ??= document.createElement('canvas').getContext('2d');
    const measure = (px: number) => {
      if (!ctx) return 0;
      ctx.font = `${opts.italic ? 'italic ' : ''}${opts.weight} ${px}px ${family}`;
      return ctx.measureText(shown).width;
    };
    return fitLine(measure, opts.maxWidth, size, opts.anim !== 'karaoke');
  });
</script>

<span class="lyr" class:left={side === 'left'} bind:this={box}
      style:--lyr-w="{opts.maxWidth}px" style:--lyr-size="{+opts.size}px" style:--lyr-fit="{fit.size}px" style:--lyr-weight={opts.weight}
      style:--lyr-style={opts.italic ? 'italic' : 'normal'} style:--lyr-case={opts.caps ? 'uppercase' : 'none'}>
  <!-- Keyed on the line's position, not its words. A chorus line printed twice in a row, or
       "Oh" then "Oh", kept the same element: the arrival never played, and the karaoke wipe
       un-filled right to left for a whole second as --k dropped back to zero. -->
  {#key idx}
    {#if text}
      <span class="line" class:wrap={fit.wrap} in:arrive={{ anim: opts.anim, side }} out:fade={{ duration: opts.anim === 'none' ? 0 : 160 }}>
        <span class="base">{text}</span>
        {#if opts.anim === 'karaoke'}
          <span class="wipe" style:--k="{progress * 100}%" aria-hidden="true">{text}</span>
        {/if}
      </span>
    {:else if gap}
      <span class="line dots" in:fade={{ duration: 300 }} out:fade={{ duration: 160 }} aria-label="Instrumental"><i></i><i></i><i></i></span>
    {/if}
  {/key}
</span>

<style>
  .lyr {
    position: relative; flex: none; display: block; overflow: hidden;
    /* Room for two lines (at the size the user chose), so a wrapped line never changes the box. */
    width: var(--lyr-w); height: min(calc(var(--lyr-size) * 2.5), 100%); align-self: center;
    font-size: var(--lyr-fit); font-weight: var(--lyr-weight); line-height: 1.25;
    font-style: var(--lyr-style); text-transform: var(--lyr-case);
    transition: font-size 0.2s var(--ease);
  }
  /* A compact dock has room for one line only. */
  :global(.bar[data-size='compact']) .lyr { height: calc(var(--lyr-size) * 1.35); }
  .line { position: absolute; inset: 0; display: flex; align-items: center; overflow: hidden; }
  .lyr.left .line { justify-content: flex-end; text-align: right; }
  .base { color: var(--fg-dim); min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .wrap .base {
    white-space: normal; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; line-clamp: 2;
  }
  :global(.bar[data-size='compact']) .wrap .base { -webkit-line-clamp: 1; line-clamp: 1; }
  /* The sung part so far, clipped to how far through the line the song is. */
  .wipe {
    position: absolute; inset: 0; display: flex; align-items: center; color: var(--accent); white-space: nowrap; overflow: hidden;
    clip-path: inset(0 calc(100% - var(--k)) 0 0);
    transition: clip-path 1s linear;
  }
  /* Low: no karaoke wipe, and the instrumental dots stay still. */
  :global(:root[data-perf='low']) .wipe { display: none; }
  :global(:root[data-perf='low']) .dots i { animation: none; }
  .lyr.left .wipe { justify-content: flex-end; clip-path: inset(0 0 0 calc(100% - var(--k))); }
  /* Instrumental: three dots breathing in turn. */
  .dots { gap: 4px; }
  .dots i { width: 4px; height: 4px; border-radius: 50%; background: var(--fg-dim); opacity: 0.35; animation: breathe 1.6s ease-in-out infinite; }
  .dots i:nth-child(2) { animation-delay: 0.2s; }
  .dots i:nth-child(3) { animation-delay: 0.4s; }
  @keyframes breathe { 50% { opacity: 0.9; transform: scale(1.25); } }
  @media (prefers-reduced-motion: reduce) { .dots i { animation: none; opacity: 0.6; } .lyr { transition: none; } }
</style>
