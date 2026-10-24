<script lang="ts">
  // The idle Pomodoro mark: a plain tomato in the focus colour.
  let { size = 18 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  <circle cx="12" cy="13.5" r="8.5" fill="var(--phase-focus)" />
  <path d="M12 6.5V3.5M12 6.5c-1.2-1.6-3.2-2-4.8-1.4 1.2.8 2 1.7 2.6 2.8M12 6.5c1.2-1.6 3.2-2 4.8-1.4-1.2.8-2 1.7-2.6 2.8"
        fill="none" stroke="var(--fg-dim)" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
</svg>
