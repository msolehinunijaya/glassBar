<script lang="ts">
  import { StickyNote } from '@lucide/svelte';
  import { ipc, type Note } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';

  let { size }: PillProps = $props();
  const notes = live<Note[]>('notes', () => ipc.notesList(), []);
  const first = $derived(notes.v[0]?.text.split('\n')[0] ?? '');
</script>

<div class="pill" title="{notes.v.length} notes">
  <StickyNote size={16} />
  {#if size === 'small'}
    <span class="n">{notes.v.length}</span>
  {:else}
    <span class="ellipsis prev">{first || 'New note'}</span>
  {/if}
</div>

<style>
  .n { font-weight: 600; }
  .prev { max-width: 140px; font-size: 12px; }
</style>
