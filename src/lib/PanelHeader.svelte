<script lang="ts">
  // The one header every widget panel starts with: title on the left, its few icon actions
  // on the right (settings, refresh…). See docs/DESIGN_RULES.md.
  import type { Snippet } from 'svelte';

  let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<header class="ph">
  <h3>{title}</h3>
  {#if children}<div class="acts">{@render children()}</div>{/if}
</header>

<style>
  .ph { display: flex; align-items: center; justify-content: space-between; gap: var(--s-2); min-height: 32px; margin-bottom: var(--s-2); }
  h3 { margin: 0; font-size: 16px; font-weight: 600; }
  .acts { display: flex; align-items: center; gap: var(--s-1); }
</style>
