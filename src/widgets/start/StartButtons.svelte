<script lang="ts">
  // Windows Start (= Win key) and Search (= Win+S) buttons. Used by the standalone
  // Start & Search widget and, when placed inline, at the front of the App Launcher.
  import { Search } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';

  import { logoFills, type LogoColor } from './logo';
  import { SEARCH_PANEL } from './search';

  // searchOpens: 'glassbar' = GlassBar's search panel (maths, units, currency, apps) for the
  // widget `instance`; 'windows' = Windows Search (Win+S).
  let { searchStyle = 'icon', color = 'blue', custom = '#0a84ff', searchOpens = 'glassbar', instance = '' }:
    { searchStyle?: 'icon' | 'bar'; color?: LogoColor; custom?: string; searchOpens?: 'glassbar' | 'windows'; instance?: string } = $props();
  function search(e: MouseEvent) {
    e.stopPropagation();
    if (searchOpens === 'windows' || !instance) return ipc.windowsSearch();
    ipc.popupOpen(SEARCH_PANEL + instance, (e.currentTarget as HTMLElement).getBoundingClientRect(), 420, 360);
  }
  const fills = $derived(logoFills(color, custom));
  const press = (f: () => Promise<void>) => (e: MouseEvent) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<button class="b" title="Start" aria-label="Start" onpointerdown={down} onclick={press(ipc.startMenu)}>
  <svg viewBox="0 0 16 16" width="17" height="17" aria-hidden="true">
    <rect x="0.5" y="0.5" width="7" height="7" rx="0.8" fill={fills[0]} />
    <rect x="8.5" y="0.5" width="7" height="7" rx="0.8" fill={fills[1]} />
    <rect x="0.5" y="8.5" width="7" height="7" rx="0.8" fill={fills[2]} />
    <rect x="8.5" y="8.5" width="7" height="7" rx="0.8" fill={fills[3]} />
  </svg>
</button>
{#if searchStyle === 'bar'}
  <button class="sbar" title="Search (Win+S)" aria-label="Search" onpointerdown={down} onclick={search}>
    <Search size={14} /><span>Search</span>
  </button>
{:else}
  <button class="b" title={searchOpens === 'windows' ? 'Search' : 'Search, calculate, convert'} aria-label="Search" onpointerdown={down} onclick={search}><Search size={17} /></button>
{/if}

<style>
  .b rect { transition: fill 0.3s var(--ease); }
  .b { display: grid; place-items: center; width: 34px; height: 100%; max-height: 38px; border-radius: calc(10px * var(--rs)); flex: none;
       transition: background 0.2s var(--ease), transform 0.25s var(--spring); }
  .b:hover, .b:focus-visible { background: var(--pill-hover); outline: none; }
  .b:active { transform: scale(0.88); }
  .sbar { display: flex; align-items: center; gap: 6px; height: 28px; padding: 0 12px; border-radius: calc(14px * var(--rs)); flex: none;
          background: var(--pill); border: 1px solid var(--line-soft); font-size: 12px; color: var(--fg-dim);
          transition: background 0.2s var(--ease); }
  .sbar:hover, .sbar:focus-visible { background: var(--pill-hover); outline: none; }
</style>
