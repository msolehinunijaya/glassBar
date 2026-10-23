<script lang="ts">
  // Far-left buttons: Windows Start (= Win key) and Search (= Win+S).
  import { Search } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';

  let {}: PillProps = $props();
  const press = (f: () => Promise<void>) => (e: MouseEvent) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<div class="pill start">
  <button class="b" title="Start" aria-label="Start" onpointerdown={down} onclick={press(ipc.startMenu)}>
    <svg viewBox="0 0 16 16" width="17" height="17" aria-hidden="true">
      <rect x="0.5" y="0.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
      <rect x="8.5" y="0.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
      <rect x="0.5" y="8.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
      <rect x="8.5" y="8.5" width="7" height="7" rx="0.8" fill="#0a84ff" />
    </svg>
  </button>
  <button class="b" title="Search" aria-label="Search" onpointerdown={down} onclick={press(ipc.windowsSearch)}><Search size={17} /></button>
</div>

<style>
  .start { gap: 2px; padding: 0 3px; }
  .b { display: grid; place-items: center; width: 34px; height: 100%; max-height: 38px; border-radius: 10px;
       transition: background 0.2s var(--ease), transform 0.25s var(--spring); }
  .b:hover { background: var(--pill-hover); }
  .b:active { transform: scale(0.88); }
</style>
