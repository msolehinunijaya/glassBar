<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { profile } from '../lib/config.svelte';
  import { events, ipc } from '../lib/ipc';
  import { GALLERY } from '../lib/popup';
  import { settingsOf, widgets } from '../widgets/registry';
  import Gallery from './Gallery.svelte';

  // Which widget's panel is showing. Cleared on blur so hidden panels stop their timers.
  let instance = $state<string | null>(null);
  const w = $derived(instance ? profile().widgets.find((x) => x.instance === instance) : undefined);
  const m = $derived(w ? widgets[w.id] : undefined);
  const close = () => ipc.popupHide();

  onMount(() => {
    const offs = [
      events.popupShow((i) => (instance = i)),
      getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (!focused) setTimeout(() => (instance = null), 150);
      }),
    ];
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
    addEventListener('keydown', onKey);
    return () => { offs.forEach((p) => p.then((off) => off())); removeEventListener('keydown', onKey); };
  });
</script>

<div class="surface popup">
  {#if instance === GALLERY}
    <Gallery {close} />
  {:else if w && m?.Popup}
    {#key instance}
      <m.Popup.component instance={w.instance} settings={settingsOf(m, w)} {close} />
    {/key}
  {/if}
</div>

<style>
  .popup { padding: 14px; overflow: auto; }
</style>
