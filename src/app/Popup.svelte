<script lang="ts">
  import { onMount } from 'svelte';
  import { profile } from '../lib/config.svelte';
  import { events, ipc } from '../lib/ipc';
  import { GALLERY } from '../lib/popup';
  import { settingsOf, widgets } from '../widgets/registry';
  import Gallery from './Gallery.svelte';
  import WindowPreviews from './WindowPreviews.svelte';

  // Which widget's panel is showing. Cleared when Rust hides the popup, so hidden
  // panels stop their timers. (Rust sends hide before the next show, so no race.)
  let instance = $state<string | null>(null);
  const w = $derived(instance ? profile().widgets.find((x) => x.instance === instance) : undefined);
  const m = $derived(w ? widgets[w.id] : undefined);
  const close = () => ipc.popupHide();
  const WINDOWS = '__windows:';
  const hover = $derived(instance?.startsWith(WINDOWS) ?? false);

  // Hover popups close shortly after the pointer leaves both the icon and the popup.
  let leaveTimer: ReturnType<typeof setTimeout>;
  const leaveSoon = () => { clearTimeout(leaveTimer); if (hover) leaveTimer = setTimeout(close, 150); };
  const stay = () => clearTimeout(leaveTimer);

  onMount(() => {
    const offs = [
      events.popupShow((i) => { instance = i; stay(); }),
      events.popupHide(() => { instance = null; }),
      events.popupLeave(leaveSoon),
    ];
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
    addEventListener('keydown', onKey);
    return () => { offs.forEach((p) => p.then((off) => off())); removeEventListener('keydown', onKey); };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="surface popup" class:hover onpointerenter={stay} onpointerleave={leaveSoon}>
  {#if hover && instance}
    {#key instance}<WindowPreviews appKey={instance.slice(WINDOWS.length)} {close} />{/key}
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
