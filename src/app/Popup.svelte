<script lang="ts">
  import { onMount } from 'svelte';
  import { profile } from '../lib/config.svelte';
  import { events, ipc } from '../lib/ipc';
  import { GALLERY } from '../lib/popup';
  import { settingsOf, widgets } from '../widgets/registry';
  import Gallery from './Gallery.svelte';
  import WindowPreviews from './WindowPreviews.svelte';
  import Feedback from '../widgets/ocr/Feedback.svelte';
  import type { CaptureDone } from '../lib/ipc';

  // Which widget's panel is showing. Cleared when Rust hides the popup, so hidden
  // panels stop their timers. (Rust sends hide before the next show, so no race.)
  let instance = $state<string | null>(null);
  /** "__feedback": the card shown right after a screenshot / OCR snap. */
  const FEEDBACK = '__feedback';
  let feedback = $state<CaptureDone | null>(null);
  const close = () => ipc.popupHide();
  const WINDOWS = '__windows:';
  /** "__hover:<instance>" = that widget's panel opened by hovering (closes on mouse-out). */
  const HOVER = '__hover:';
  const hover = $derived(!!instance && (instance.startsWith(WINDOWS) || instance.startsWith(HOVER)));
  const target = $derived(instance?.startsWith(HOVER) ? instance.slice(HOVER.length) : instance);
  const w = $derived(target ? profile().widgets.find((x) => x.instance === target) : undefined);
  const m = $derived(w ? widgets[w.id] : undefined);

  // Hover popups close shortly after the pointer leaves both the icon and the popup.
  let leaveTimer: ReturnType<typeof setTimeout>;
  const leaveSoon = () => { clearTimeout(leaveTimer); if (hover) leaveTimer = setTimeout(close, 150); };
  const stay = () => clearTimeout(leaveTimer);

  onMount(() => {
    const offs = [
      events.popupShow((i) => { instance = i; stay(); }),
      events.popupHide(() => { instance = null; }),
      events.popupLeave(leaveSoon),
      events.captureDone((d) => (feedback = d)),
    ];
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
    addEventListener('keydown', onKey);
    return () => { offs.forEach((p) => p.then((off) => off())); removeEventListener('keydown', onKey); };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="surface popup" class:hover onpointerenter={stay} onpointerleave={leaveSoon}>
  {#if instance?.startsWith(WINDOWS)}
    {#key instance}<WindowPreviews appKey={instance.slice(WINDOWS.length)} {close} />{/key}
  {:else if instance === FEEDBACK && feedback}
    {#key feedback}<Feedback data={feedback} {close} />{/key}
  {:else if instance === GALLERY}
    <Gallery {close} />
  {:else if w && m?.Popup}
    {#key instance}
      <m.Popup.component instance={w.instance} settings={settingsOf(m, w)} {close} />
    {/key}
  {/if}
</div>

<style>
  .popup { padding: 14px; overflow: auto; }
  .popup.hover { padding: 8px; }
</style>
