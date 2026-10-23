<script lang="ts">
  // Elapsed dots solid, the rest faint (reference design). Colours follow the theme's text colour.
  import { grid, type Mode } from './period';
  let { mode, frac, dot = 22, gap = 7 }: { mode: Mode; frac: number; dot?: number; gap?: number } = $props();
  const g = $derived(grid(mode, frac));
</script>

<span class="grid" style:--c={g.cols} style:--d="{dot}px" style:--g="{gap}px" aria-hidden="true">
  {#each { length: g.total } as _, i (i)}<i class:on={i < g.filled}></i>{/each}
</span>

<style>
  .grid { display: grid; grid-template-columns: repeat(var(--c), var(--d)); gap: var(--g); }
  i { width: var(--d); height: var(--d); border-radius: 50%; background: color-mix(in srgb, var(--fg) 14%, transparent); transition: background 0.4s var(--ease); }
  i.on { background: var(--fg); box-shadow: 0 0 calc(var(--d) / 4) color-mix(in srgb, var(--fg) 25%, transparent); }
</style>
