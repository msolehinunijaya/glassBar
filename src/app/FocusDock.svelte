<script lang="ts">
  // The whole dock during a focus session: who (Focusing, session dots), the time left, a
  // timeline from start to end, and Pause/Resume · Skip · Stop. The controls sit in
  // `.focus-acts`, which the dock leaves alone when deciding to bring the widgets back.
  import { Pause, Play, SkipForward, Square, Target } from '@lucide/svelte';
  import { fade } from 'svelte/transition';
  import { ipc } from '../lib/ipc';
  import { run } from '../widgets/pomodoro/pomo.svelte';
  import type { FocusDock } from './focusDock';

  let { f, vertical = false }: { f: FocusDock; vertical?: boolean } = $props();
  const at = (ms: number | null) => (ms == null ? '' : new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' }).format(ms));
  const stop = (e: Event) => e.stopPropagation();
  const act = (e: MouseEvent, f: () => Promise<unknown>) => { e.stopPropagation(); run(f); };
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="focus" class:vertical class:paused={f.status === 'paused'} style:--p={f.progress} role="status" aria-live="off"
     aria-label="Focus session, {f.left} left" transition:fade={{ duration: 220 }} onpointerdown={stop}>
  <div class="fill"></div>
  <div class="body">
    <span class="who">
      <span class="badge"><Target size={vertical ? 14 : 15} strokeWidth={2.4} /></span>
      {#if !vertical}
        <span class="col">
          <b>{f.status === 'paused' ? 'Paused' : 'Focusing'}</b>
          <span class="dots" aria-label={f.session}>{#each f.dots as d, i (i)}<i class={d}></i>{/each}</span>
        </span>
      {/if}
    </span>
    <strong class="time">{f.left}</strong>
    <span class="line" aria-hidden="true">
      <span class="track"><span class="done"></span><span class="knob"></span></span>
      {#if !vertical}
        <span class="ends"><span>{at(f.start)}</span><span class="mid">{f.session}</span><span>{f.status === 'paused' ? 'paused' : at(f.end)}</span></span>
      {/if}
    </span>
    <span class="focus-acts">
      {#if f.status === 'running'}
        <button class="act primary" onclick={(e) => act(e, ipc.pomoPause)} aria-label="Pause focus" title="Pause"><Pause size={13} fill="currentColor" /><span>Pause</span></button>
      {:else}
        <button class="act primary" onclick={(e) => act(e, ipc.pomoStart)} aria-label="Resume focus" title="Resume"><Play size={13} fill="currentColor" /><span>Resume</span></button>
      {/if}
      <button class="act quiet" onclick={(e) => act(e, ipc.pomoSkip)} aria-label="Skip to break" title="Skip to break"><SkipForward size={13} /><span>Skip</span></button>
      <button class="act quiet" onclick={(e) => act(e, ipc.pomoReset)} aria-label="Stop this session" title="Stop (back to the start of this session)"><Square size={11} fill="currentColor" /><span>Stop</span></button>
    </span>
  </div>
</div>

<style>
  /* Over the whole dock, like a dock alert, in the focus colour. */
  .focus {
    --c: var(--phase-focus);
    position: absolute; inset: 0; z-index: 4; display: grid; place-items: center; container-type: inline-size;
    border-radius: var(--r-window); overflow: hidden; cursor: default;
    background: color-mix(in srgb, var(--c) 8%, transparent);
  }
  /* Elapsed time as a soft tint growing along the dock (transform only: no reflow). */
  .fill {
    position: absolute; inset: 0; transform-origin: left; transform: scaleX(var(--p));
    background: linear-gradient(90deg, color-mix(in srgb, var(--c) 22%, transparent), color-mix(in srgb, var(--c) 12%, transparent));
    transition: transform 1s linear;
  }
  .body { position: relative; display: flex; align-items: center; gap: 14px; width: 100%; max-width: 1040px; padding: 0 16px; color: var(--fg); min-width: 0; }
  .who { display: flex; align-items: center; gap: 8px; flex: none; }
  .badge { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 50%; background: var(--c); color: var(--phase-ink); flex: none; }
  .paused .badge { background: color-mix(in srgb, var(--c) 30%, var(--pill)); color: var(--c); }
  .col { display: flex; flex-direction: column; gap: 3px; line-height: 1; }
  .col b { font-size: 12px; font-weight: 600; }
  .dots { display: flex; gap: 3px; }
  .dots i { width: 6px; height: 6px; border-radius: 50%; background: color-mix(in srgb, var(--fg) 18%, transparent); }
  .dots i.done { background: var(--c); }
  .dots i.current { background: color-mix(in srgb, var(--c) 55%, transparent); box-shadow: 0 0 0 1.5px var(--c); }
  .time { font-size: 24px; font-weight: 600; line-height: 1; font-variant-numeric: tabular-nums; flex: none; letter-spacing: 0.01em; }
  .paused .time { color: var(--fg-dim); }
  /* The timeline: start time · session · end time under a track with the elapsed part. */
  .line { flex: 1; min-width: 60px; display: flex; flex-direction: column; gap: 5px; }
  .track { position: relative; height: 5px; border-radius: 3px; background: color-mix(in srgb, var(--fg) 12%, transparent); }
  .done { position: absolute; inset: 0; border-radius: inherit; transform-origin: left; transform: scaleX(var(--p)); background: var(--c); transition: transform 1s linear; }
  .knob { position: absolute; top: 50%; left: calc(var(--p) * 100%); width: 11px; height: 11px; margin: -5.5px 0 0 -5.5px; border-radius: 50%;
          background: var(--c); box-shadow: 0 0 0 2px color-mix(in srgb, var(--fg) 85%, transparent), 0 0 8px color-mix(in srgb, var(--c) 60%, transparent); transition: left 1s linear; }
  .paused .done, .paused .knob { filter: saturate(0.4); }
  .ends { display: flex; justify-content: space-between; gap: 8px; font-size: 10.5px; color: var(--fg-3); font-variant-numeric: tabular-nums; white-space: nowrap; }
  .ends .mid { overflow: hidden; text-overflow: ellipsis; color: var(--fg-dim); }
  .focus-acts { display: flex; align-items: center; gap: 4px; flex: none; }
  .act { display: inline-flex; align-items: center; gap: 6px; height: 30px; padding: 0 12px; border-radius: var(--r-chip); font-size: 12px; font-weight: 500;
         border: 1px solid transparent; transition: background 0.2s var(--ease), transform 0.15s var(--ease); }
  .act.primary { background: var(--c); color: var(--phase-ink); font-weight: 600; }
  .act.primary:hover { filter: brightness(1.08); }
  .act.quiet { color: var(--fg-dim); }
  .act.quiet:hover { background: var(--pill-hover); color: var(--fg); }
  .act:active { transform: scale(0.95); }
  .act:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  /* Narrow docks: drop the words, then the timeline labels, keep time and controls. */
  @container (max-width: 640px) { .ends .mid, .act.quiet span { display: none; } .act.quiet { padding: 0 8px; } }
  @container (max-width: 470px) { .ends, .col { display: none; } .act span { display: none; } .act { padding: 0 9px; } }
  @container (max-width: 300px) { .line { display: none; } }
  :global(.bar[data-size='compact']) .time { font-size: 19px; }
  :global(.bar[data-size='compact']) .act { height: 26px; }
  /* Left/right dock: a column of icon, time, a short track, and icon-only controls. */
  .vertical .fill { transform-origin: top; transform: scaleY(var(--p)); background: linear-gradient(180deg, color-mix(in srgb, var(--c) 22%, transparent), color-mix(in srgb, var(--c) 12%, transparent)); }
  .vertical .body { flex-direction: column; gap: 8px; padding: 10px 4px; }
  .vertical .time { font-size: 15px; }
  .vertical .line { flex: none; width: 70%; min-width: 0; }
  .vertical .focus-acts { flex-direction: column; }
  .vertical .act { padding: 0; width: 28px; justify-content: center; }
  .vertical .act span { display: none; }
  @media (prefers-reduced-motion: reduce) { .fill, .done, .knob { transition: none; } }
</style>
