<script lang="ts">
  import type { PopupProps } from '../types';
  import type { HelloSettings } from './index';

  let { settings, close }: PopupProps<HelloSettings> = $props();
  let count = $state(0);
  // Backend example: `await invoke('hello_greet', { name })` after adding
  // src-tauri/src/widgets/hello.rs (see _template.rs.example).
</script>

<h3>Hello, {settings.name}!</h3>
<div class="card">
  <p>Clicked {count} times.</p>
  <button class="tile" onclick={() => count++}>Click me</button>
</div>
<button class="icon-btn" style="width:auto;padding:0 12px;margin-top:10px" onclick={close}>Close</button>

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
</style>
