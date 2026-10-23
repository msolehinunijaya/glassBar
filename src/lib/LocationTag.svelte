<script lang="ts">
  // "📍 Kuala Lumpur · Precise" in a widget panel. Click → Settings › Location.
  // When there is no location, says so plainly with a way to set one.
  import { MapPin } from '@lucide/svelte';
  import { ipc } from './ipc';
  import { badge, tagText } from './location';
  import { useLocation } from './location.svelte';

  const loc = useLocation();
  const b = $derived(badge(loc.s));
  const open = () => ipc.openSettings('Location');
</script>

{#if loc.s && !loc.s.place}
  <div class="none">
    <span><MapPin size={12} /> Location unavailable{loc.s.reason ? `: ${loc.s.reason}` : ''}</span>
    <button class="set" onclick={open}>Set location</button>
  </div>
{:else if loc.s}
  <button class="tag {b.kind}" onclick={open} title="Shared location ({b.label}). Click to change it in Settings.">
    <MapPin size={11} /> {tagText(loc.s)}
  </button>
{/if}

<style>
  .tag { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; color: var(--fg-dim); padding: 2px 6px; border-radius: 8px; }
  .tag:hover { background: var(--pill-hover); color: var(--fg); }
  .tag.approx :global(svg) { color: #e0a800; }
  .tag.precise :global(svg) { color: #30d158; }
  .tag.manual :global(svg) { color: #0a84ff; }
  .none { display: flex; align-items: center; justify-content: space-between; gap: 8px; font-size: 12px; color: #ff9f0a; padding: 6px 8px; border-radius: 10px; background: var(--pill); }
  .set { padding: 3px 9px; border-radius: 8px; background: var(--accent); color: #fff; flex: none; }
  .none span { display: inline-flex; align-items: center; gap: 4px; }
</style>
