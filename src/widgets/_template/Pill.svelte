<script lang="ts">
  import { Sparkles } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { HelloSettings } from './index';

  // size: 'small' | 'medium' | 'large'; vertical: bar docked left/right.
  let { size, settings }: PillProps<HelloSettings> = $props();
</script>

<div class="pill">
  <Sparkles size={16} />
  {#if size !== 'small'}<span>Hello, {settings.name}</span>{/if}
</div>
