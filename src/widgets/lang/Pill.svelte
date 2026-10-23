<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';

  let {}: PillProps = $props();
  let code = $state('');
  let name = $state('');
  // ponytail: 2 s poll; Windows has no cheap cross-process "layout changed" event.
  $effect(() => {
    const load = () => ipc.langList().then((l) => {
      const a = l.find((x) => x.active) ?? l[0];
      code = a?.code ?? ''; name = a?.name ?? '';
    }).catch(() => {});
    load();
    const t = setInterval(load, 2000);
    return () => clearInterval(t);
  });
</script>

<div class="pill" title={name || 'Keyboard language'}><b class="code">{code || '–'}</b></div>

<style>
  .code { font-size: 12px; letter-spacing: 0.04em; min-width: 18px; text-align: center; }
</style>
