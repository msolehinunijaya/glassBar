<script lang="ts">
  // Windows Start (= Win key) and Search (= Win+S) buttons. Used by the standalone
  // Start & Search widget and, when placed inline, at the front of the App Launcher.
  import { Search } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';

  let { searchStyle = 'icon' }: { searchStyle?: 'icon' | 'bar' } = $props();
  const press = (f: () => Promise<void>) => (e: MouseEvent) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<button class="b" title="Start" aria-label="Start" onpointerdown={down} onclick={press(ipc.startMenu)}>
  <svg viewBox="0 0 16 16" width="17" height="17" aria-hidden="true">
    <rect x="0.5" y="0.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
    <rect x="8.5" y="0.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
    <rect x="0.5" y="8.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
    <rect x="8.5" y="8.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
  </svg>
</button>
{#if searchStyle === 'bar'}
  <button class="sbar" title="Search (Win+S)" aria-label="Search" onpointerdown={down} onclick={press(ipc.windowsSearch)}>
    <Search size={14} /><span>Search</span>
  </button>
{:else}
  <button class="b" title="Search" aria-label="Search" onpointerdown={down} onclick={press(ipc.windowsSearch)}><Search size={17} /></button>
{/if}

<style>
  .b { display: grid; place-items: center; width: 34px; height: 100%; max-height: 38px; border-radius: 10px; flex: none;
       transition: background 0.2s var(--ease), transform 0.25s var(--spring); }
  .b:hover, .b:focus-visible { background: var(--pill-hover); outline: none; }
  .b:active { transform: scale(0.88); }
  .sbar { display: flex; align-items: center; gap: 6px; height: 28px; padding: 0 12px; border-radius: 14px; flex: none;
          background: var(--pill); border: 1px solid var(--line-soft); font-size: 12px; color: var(--fg-dim);
          transition: background 0.2s var(--ease); }
  .sbar:hover, .sbar:focus-visible { background: var(--pill-hover); outline: none; }
</style>
