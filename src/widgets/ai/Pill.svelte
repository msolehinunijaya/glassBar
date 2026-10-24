<script lang="ts">
  import { Sparkles } from '@lucide/svelte';
  import { ipc, type AiUsage } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import type { AiSettings } from './index';
  import { tokens } from './tokens';

  let { size, settings }: PillProps<AiSettings> = $props();
  const u = live<AiUsage | null>('ai', ipc.aiUsage, null);
  const today = $derived(u.v ? u.v.dayIn + u.v.dayOut : 0);
</script>

<div class="pill" title={u.v ? `Today: ${tokens(u.v.dayIn)} in / ${tokens(u.v.dayOut)} out · ${u.v.dayRequests} requests` : 'AI'}>
  <Sparkles size={16} />
  {#if settings.showTokens && (size !== 'small' || today)}<span class="t">{tokens(today)}{#if size !== 'small'} <span class="dim">today</span>{/if}</span>{/if}
</div>

<style>
  .t { font-size: 12px; font-variant-numeric: tabular-nums; }
</style>
