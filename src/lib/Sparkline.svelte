<script lang="ts">
  import { points } from './sparkline';

  let { values, max = 100, width = 60, height = 18, color = 'var(--accent)', fill = true }: {
    values: number[]; max?: number; width?: number; height?: number; color?: string; fill?: boolean;
  } = $props();
  const line = $derived(points(values, width, height, max));
</script>

<svg {width} {height} viewBox="0 0 {width} {height}" aria-hidden="true" preserveAspectRatio="none">
  {#if fill && line}
    <polygon points="0,{height} {line} {width},{height}" fill={color} opacity="0.18" />
  {/if}
  <polyline points={line} fill="none" stroke={color} stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
</svg>
