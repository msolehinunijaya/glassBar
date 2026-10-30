import './styles/glass.css';
import { mount } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { initConfig } from './lib/config.svelte';
import { initPerf } from './lib/perf.svelte';
import { installRangeWheel } from './lib/rangeWheel';
import './lib/liquid';
import Bar from './app/Bar.svelte';
import Popup from './app/Popup.svelte';
import Settings from './app/Settings.svelte';
import Snap from './app/Snap.svelte';
import Banner from './app/Banner.svelte';
import Magnify from './app/Magnify.svelte';

// One bundle, five windows: pick the root by window label.
const label = getCurrentWindow().label;
// The docks on other monitors ("All displays", bar-<monitor>) look and behave as the dock.
document.documentElement.dataset.window = label.startsWith('bar-') ? 'bar' : label;
// No browser menu (Back/Refresh/Inspect) anywhere except in text fields.
addEventListener('contextmenu', (e) => {
  if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault();
});
await initConfig();
// The mouse wheel moves sliders in Settings and panels (the dock has its own wheel rules).
if (label === 'settings' || label === 'popup') installRangeWheel();
await initPerf();
const Root = label === 'popup' ? Popup : label === 'settings' ? Settings : label === 'snap' ? Snap : label === 'banner' ? Banner : label === 'magnify' ? Magnify : Bar;
mount(Root, { target: document.getElementById('app')! });
