<script lang="ts">
  // Privacy Monitor on the dock: a shield, dimmed while nothing is detected, lit (and gently
  // glowing) while an app uses the camera, microphone, location or screen, and a warning
  // shield when something can't be checked. Medium also shows which resources are in use.
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Shield, ShieldAlert } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { isDockWindow } from '../../lib/dock';
  import type { PillProps } from '../types';
  import type { PrivacySettings } from './settings';
  import { ICON, monitor } from './monitor.svelte';
  import { dockLevel, LABEL, summary, transitions, visible } from './status';

  let { size, settings, vertical }: PillProps<PrivacySettings> = $props();
  // Only real docks notify (not the previews in Settings); Rust drops repeats from other docks.
  const onDock = isDockWindow(getCurrentWindow().label);
  const m = monitor((prev, next) => {
    if (!onDock) return;
    for (const n of transitions(prev, next, settings)) ipc.privacyNotify(n.key, n.title, n.body).catch(() => {});
  });
  const list = $derived(visible(m.v, settings));
  const level = $derived(dockLevel(list, !!m.error, !!m.v));
  const on = $derived(list.filter((r) => r.state === 'active'));
  const tip = $derived(m.error ? `Privacy Monitor: ${m.error}` : m.v ? `Privacy Monitor\n${summary(list, false)}` : 'Privacy Monitor: checking…');
  // The lit colour: the first resource in use (per-resource tone) or the accent.
  const tone = $derived(settings.tone === 'accent' ? 'accent' : on[0]?.key ?? 'accent');
</script>

{#snippet shield(px: number)}
  <span class="ic">
    {#if level === 'attention'}<ShieldAlert size={px} />{:else}<Shield size={px} />{/if}
    {#if level === 'active'}<i class="dot" aria-hidden="true"></i>{/if}
  </span>
{/snippet}

{#if vertical}
  <div class="side pv" data-level={level} data-tone={tone} class:glow={settings.glow} title={tip}>
    {@render shield(18)}
    <span class="side-c">{level === 'active' ? `${on.length} in use` : level === 'attention' ? 'Check' : 'Privacy'}</span>
  </div>
{:else}
  <div class="pill pv" data-level={level} data-tone={tone} class:glow={settings.glow} title={tip}
       aria-label={level === 'active' ? `Privacy: ${on.map((r) => LABEL[r.key]).join(', ')} in use` : 'Privacy Monitor'}>
    {@render shield(16)}
    {#if size !== 'small' && on.length}
      <span class="res">
        {#each on as r (r.key)}{@const Icon = ICON[r.key]}<span class="r" data-tone={settings.tone === 'accent' ? 'accent' : r.key}><Icon size={13} /></span>{/each}
      </span>
    {/if}
  </div>
{/if}

<style>
  /* Lit colours: the theme accent, or Windows/macOS-style status colours per resource. */
  .pv, .r { --lit: var(--accent); }
  [data-tone='camera'] { --lit: #30d158; }
  [data-tone='mic'] { --lit: #ff9f0a; }
  [data-tone='location'] { --lit: #0a84ff; }
  [data-tone='recording'] { --lit: #ff453a; }
  [data-tone='sharing'] { --lit: #bf5af2; }
  .pv { --warn: #ff9f0a; }

  /* Sized to the dock's height (Settings › Bar › Size): large keeps the usual 16 px icon,
     normal and compact get a smaller shield, dot and spacing so it sits like its neighbours.
     Idle it's a square tile like the other one-icon widgets; it widens for the in-use icons. */
  .pill.pv { --ic: 16px; --ri: 13px; --dot: 6px; padding: 0 10px; gap: 7px; aspect-ratio: 1; justify-content: center; }
  :global(.bar[data-size='normal']) .pill.pv { --ic: 15px; --ri: 12px; --dot: 5px; padding: 0 8px; gap: 6px; }
  :global(.bar[data-size='compact']) .pill.pv { --ic: 14px; --ri: 11px; --dot: 5px; padding: 0 6px; gap: 5px; }
  .pill.pv .ic :global(svg) { width: var(--ic); height: var(--ic); }
  .pill.pv .r :global(svg) { width: var(--ri); height: var(--ri); }

  .ic { position: relative; display: grid; place-items: center; color: var(--fg);
        opacity: 0.5; transition: opacity 0.3s var(--ease), color 0.3s var(--ease), filter 0.3s var(--ease); }
  [data-level='loading'] .ic { opacity: 0.35; }
  [data-level='active'] .ic { opacity: 1; color: var(--lit); }
  [data-level='active'].glow .ic { filter: drop-shadow(0 0 3px color-mix(in srgb, var(--lit) 75%, transparent)) drop-shadow(0 0 7px color-mix(in srgb, var(--lit) 40%, transparent)); }
  [data-level='attention'] .ic { opacity: 1; color: var(--warn); }
  .dot { --d: var(--dot, 6px); position: absolute; top: calc(var(--d) * -0.4); right: calc(var(--d) * -0.6); width: var(--d); height: var(--d); border-radius: 50%; background: var(--lit);
         box-shadow: 0 0 0 1.5px rgba(var(--tint), 0.9); }
  .glow .dot { box-shadow: 0 0 0 1.5px rgba(var(--tint), 0.9), 0 0 6px var(--lit); }

  .res { display: flex; align-items: center; gap: calc(var(--ri, 13px) * 0.3); }
  .r { display: grid; place-items: center; color: var(--lit); }
  .glow .r { filter: drop-shadow(0 0 3px color-mix(in srgb, var(--lit) 60%, transparent)); }
  .side.pv { gap: 4px; }
</style>
