<script lang="ts">
  // One lyric line beside the music pill, changing in time with the song.
  //
  // Each line is keyed, so the old one leaves while the new one arrives. Both sit absolutely
  // inside a fixed-width box, so they cross over without ever moving the dock, and a long
  // line can never push the other widgets off it.
  //
  // The karaoke wipe is not a transition: it is a clip on a bright copy of the line, moved
  // once a second with a 1 s linear transition -- the same trick the progress line uses. That
  // keeps it smooth without a frame loop.
  //
  // No ARIA live region on purpose: announcing every lyric line would talk over everything
  // else a screen reader has to say. The text is in the DOM to be read, not shouted.
  import { cubicOut } from 'svelte/easing';
  import { fade, fly, type TransitionConfig } from 'svelte/transition';
  import type { LyricsSettings } from './settings';

  let { text, progress, side, opts }: { text: string; progress: number; side: 'left' | 'right'; opts: LyricsSettings } = $props();

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
</script>

<span class="lyr" class:left={side === 'left'}
      style:--lyr-w="{opts.maxWidth}px" style:--lyr-size="{opts.size}px" style:--lyr-weight={opts.weight}
      style:--lyr-style={opts.italic ? 'italic' : 'normal'} style:--lyr-case={opts.caps ? 'uppercase' : 'none'}>
  {#key text}
    {#if text}
      <span class="line" in:arrive={{ anim: opts.anim, side }} out:fade={{ duration: opts.anim === 'none' ? 0 : 160 }}>
        <span class="base">{text}</span>
        {#if opts.anim === 'karaoke'}
          <span class="wipe" style:--k="{progress * 100}%" aria-hidden="true">{text}</span>
        {/if}
      </span>
    {/if}
  {/key}
</span>

<style>
  .lyr {
    position: relative; flex: none; display: block; overflow: hidden;
    width: var(--lyr-w); height: 1.35em; align-self: center;
    font-size: var(--lyr-size); font-weight: var(--lyr-weight);
    font-style: var(--lyr-style); text-transform: var(--lyr-case);
  }
  .lyr.left { order: -1; }
  .line { position: absolute; inset: 0; display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .lyr.left .line { text-align: right; }
  .base { color: var(--fg-dim); }
  /* The sung part so far, clipped to how far through the line the song is. */
  .wipe {
    position: absolute; inset: 0; color: var(--accent); white-space: nowrap; overflow: hidden;
    clip-path: inset(0 calc(100% - var(--k)) 0 0);
    transition: clip-path 1s linear;
  }
  .lyr.left .wipe { clip-path: inset(0 0 0 calc(100% - var(--k))); }
</style>
