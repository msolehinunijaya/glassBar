<script lang="ts">
  import { ChevronUp } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { profile, store } from '../../lib/config.svelte';
  import type { PillProps } from '../types';

  let {}: PillProps = $props();
  const nativeVisible = $derived(!!store.config && profile().bar.nativeTaskbar !== 'hidden' && !profile().bar.autoHide);
</script>

<button class="pill" title={nativeVisible ? 'Show hidden icons (right-click: tray settings)' : 'Taskbar hidden: opens tray icon settings'}
        onclick={() => ipc.trayOverflow(nativeVisible)}
        oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); ipc.traySettings(); }}>
  <ChevronUp size={16} />
</button>
