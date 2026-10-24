<script lang="ts">
  // Small provider marks. The only place brand colour appears (Claude's orange; OpenAI's
  // mark is monochrome, so it follows the text colour).
  import type { Provider } from './model';
  let { provider, size = 14 }: { provider: Provider; size?: number } = $props();
</script>

<svg class="logo" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  {#if provider === 'claude'}
    <g fill="#d97757">
      {#each [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] as i (i)}
        <rect x="10.9" y={i % 2 ? 4 : 1.5} width="2.2" height={i % 2 ? 8 : 10.5} rx="1.1" transform="rotate({i * 36} 12 12)" />
      {/each}
    </g>
  {:else}
    <g fill="none" stroke="currentColor" stroke-width="1.7">
      {#each [0, 1, 2, 3, 4, 5] as i (i)}
        <rect x="8.6" y="2.6" width="6.8" height="11.4" rx="3.4" transform="rotate({i * 60} 12 12)" />
      {/each}
    </g>
  {/if}
</svg>

<style>
  .logo { flex: none; display: block; }
</style>
