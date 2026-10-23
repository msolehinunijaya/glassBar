<script lang="ts">
  // Installed input languages; click one to switch the app you were using.
  import { ipc, type KeyboardLayout } from '../../lib/ipc';

  let list = $state<KeyboardLayout[]>([]);
  const load = () => ipc.langList().then((l) => (list = l)).catch(() => {});
  $effect(() => { load(); });
  async function pick(id: number) {
    await ipc.langSet(id);
    setTimeout(load, 200); // the app applies it asynchronously
  }
</script>

<div class="langs">
  {#each list as l (l.id)}
    <button class="lang" class:sel={l.active} onclick={() => pick(l.id)} title={l.locale}>
      <b>{l.code}</b><span>{l.name}</span>
    </button>
  {:else}
    <p class="dim small">No input languages reported by Windows.</p>
  {/each}
</div>

<style>
  .langs { display: flex; flex-direction: column; gap: 4px; }
  .lang { display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 10px; background: var(--pill); text-align: left; font-size: 12px; }
  .lang b { min-width: 24px; font-size: 12px; }
  .lang.sel { background: var(--accent); color: #fff; }
  .small { font-size: 12px; }
</style>
