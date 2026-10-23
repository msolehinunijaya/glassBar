import './styles/glass.css';
import { mount } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { initConfig } from './lib/config.svelte';
import Bar from './app/Bar.svelte';
import Popup from './app/Popup.svelte';
import Settings from './app/Settings.svelte';
import Snap from './app/Snap.svelte';

// One bundle, four windows: pick the root by window label.
const label = getCurrentWindow().label;
document.documentElement.dataset.window = label;
// No browser menu (Back/Refresh/Inspect) anywhere except in text fields.
addEventListener('contextmenu', (e) => {
  if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault();
});
await initConfig();
const Root = label === 'popup' ? Popup : label === 'settings' ? Settings : label === 'snap' ? Snap : Bar;
mount(Root, { target: document.getElementById('app')! });
