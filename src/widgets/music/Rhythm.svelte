<script lang="ts">
  // Small "now playing" equaliser. Animates only while music plays
  // (transform-only CSS, so the browser does it on the compositor).
  let { playing, color = 'var(--accent)' }: { playing: boolean; color?: string } = $props();
</script>

<span class="eq" class:playing style:--c={color} aria-hidden="true">
  {#each [0.9, 0.6, 1.1, 0.75] as d, i (i)}<i style:animation-duration="{d}s" style:animation-delay="{-i * 0.17}s"></i>{/each}
</span>

<style>
  .eq { display: inline-flex; align-items: flex-end; gap: 2px; height: 14px; }
  i { width: 3px; height: 100%; border-radius: calc(2px * var(--rs)); background: var(--c); transform-origin: bottom; transform: scaleY(0.25); }
  .playing i { animation: bounce ease-in-out infinite alternate; }
  @keyframes bounce { from { transform: scaleY(0.2); } to { transform: scaleY(1); } }
</style>
