<script lang="ts">
  import type { PopupProps } from '../types';
  import LangList from './LangList.svelte';
  let {}: PopupProps = $props();
</script>

<h3>Keyboard language</h3>
<LangList />
<p class="dim note">Switches the app you were last using. Add languages in Windows Settings › Time &amp; language.</p>

<style>
  h3 { margin: 0 0 10px; font-size: 14px; font-weight: 600; }
  .note { font-size: 11px; margin: 10px 0 0; }
</style>
