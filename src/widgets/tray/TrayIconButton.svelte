<script lang="ts">
  import { ipc, type TrayIcon } from '../../lib/ipc';
  let { icon, size = 18 }: { icon: TrayIcon; size?: number } = $props();
  const send = (e: MouseEvent, b: 'left' | 'right' | 'double') => { e.preventDefault(); e.stopPropagation(); ipc.trayClick(icon.id, b); };
</script>

<button class="ti" title={icon.tip || icon.exe.split('\\').pop()}
        onpointerdown={(e) => e.stopPropagation()}
        onclick={(e) => send(e, 'left')} ondblclick={(e) => send(e, 'double')} oncontextmenu={(e) => send(e, 'right')}>
  {#if icon.icon}<img src={icon.icon} alt="" width={size} height={size} draggable="false" />{:else}<span class="blank" style:width="{size}px" style:height="{size}px"></span>{/if}
</button>

<style>
  .ti { display: grid; place-items: center; padding: 4px; border-radius: 8px; }
  .ti:hover { background: var(--pill-hover); }
  .ti:active { transform: scale(0.9); }
  .blank { display: block; border-radius: 4px; background: var(--track); }
</style>
