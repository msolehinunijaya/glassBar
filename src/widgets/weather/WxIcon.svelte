<script lang="ts">
  import { Cloud, CloudDrizzle, CloudFog, CloudLightning, CloudMoon, CloudRain, CloudSnow, CloudSun, Moon, Sun } from '@lucide/svelte';
  import { describe } from './codes';

  let { code, isDay = true, size = 16 }: { code: number; isDay?: boolean; size?: number } = $props();
  const icons = {
    sun: Sun, moon: Moon, 'cloud-sun': CloudSun, 'cloud-moon': CloudMoon, cloud: Cloud,
    fog: CloudFog, drizzle: CloudDrizzle, rain: CloudRain, snow: CloudSnow, storm: CloudLightning,
  };
  const Icon = $derived(icons[describe(code, isDay).icon]);
</script>

<Icon {size} />
