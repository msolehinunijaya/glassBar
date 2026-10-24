<script lang="ts">
  // The flyout: Claude | ChatGPT | All, or the Accounts page (settings icon, "Manage
  // accounts", or the empty state). Opening it refreshes what is due.
  import { ChevronLeft, RefreshCw, Settings2 } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type AiuReading, type AiuSource } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import Ring from '../../lib/Ring.svelte';
  import { liveRates } from '../currency/rates.svelte';
  import type { PopupProps } from '../types';
  import Bars from './Bars.svelte';
  import Connect from './Connect.svelte';
  import type { AiUsageSettings } from './index';
  import Logo from './Logo.svelte';
  import { pct, tokens, tone, TONE, until, views, type Provider } from './model';
  import Options from './Options.svelte';
  import ProviderPanel from './ProviderPanel.svelte';

  let { instance, settings }: PopupProps<AiUsageSettings> = $props();
  const st = live<AiuReading[] | null>('aiusage', ipc.aiuState, null);
  const vs = $derived(views(st.v ?? [], settings.prefer));
  const t = tickingNow(() => true, 30_000);
  const fx = liveRates(() => 'USD');
  const myr = $derived(fx.r?.rates.MYR ?? null);
  const set = (p: Partial<AiUsageSettings>) => updateWidgetSettings(instance, p);

  let manage = $state(false);
  let busy = $state(false);
  const refresh = () => { busy = true; ipc.aiuRefresh().catch(() => {}).finally(() => (busy = false)); };
  $effect(() => { refresh(); });

  type Tab = Provider | 'all';
  const tabs = $derived<{ id: Tab; name: string }[]>([...vs.map((v) => ({ id: v.provider, name: v.name })), ...(vs.length > 1 ? [{ id: 'all' as const, name: 'All' }] : [])]);
  const tab = $derived<Tab>(tabs.some((x) => x.id === settings.tab) ? settings.tab : (tabs[0]?.id ?? 'claude'));
  const current = $derived(vs.find((v) => v.provider === tab));
  const prefer = (p: Provider) => (s: AiuSource) => set({ prefer: { ...settings.prefer, [p]: s } });
  const SERIES: Record<Provider, string> = { claude: 'var(--accent)', chatgpt: 'var(--fg-dim)' };
  const series = $derived(vs.filter((v) => v.usage).map((v) => ({ name: v.name, color: SERIES[v.provider], data: v.usage!.daily })));
</script>

<PanelHeader title={manage ? 'Accounts' : 'AI Usage'}>
  {#if !manage}
    <button class="icon-btn" class:spin={busy} onclick={refresh} aria-label="Refresh" title="Refresh"><RefreshCw size={15} /></button>
  {/if}
  <button class="icon-btn" class:on={manage} onclick={() => (manage = !manage)} aria-label={manage ? 'Back to usage' : 'Accounts and alerts'} title={manage ? 'Back' : 'Accounts and alerts'}>
    {#if manage}<ChevronLeft size={16} />{:else}<Settings2 size={15} />{/if}
  </button>
</PanelHeader>

{#if manage}
  <Connect readings={st.v ?? []} />
  <div class="opts"><Options {settings} save={set} /></div>
{:else if st.v && !vs.length}
  <div class="empty">
    <div class="rings"><Logo provider="claude" size={28} /><Logo provider="chatgpt" size={28} /></div>
    <p>Connect Claude or ChatGPT to see usage</p>
    <button class="primary" onclick={() => (manage = true)}>Connect</button>
  </div>
{:else if st.v}
  {#if tabs.length > 1}
    <div class="seg" role="tablist" aria-label="Provider">
      {#each tabs as x (x.id)}
        <button role="tab" aria-selected={tab === x.id} class:sel={tab === x.id} onclick={() => set({ tab: x.id })}>{x.name}</button>
      {/each}
    </div>
  {/if}
  {#if tab === 'all'}
    <div class="cards">
      {#each vs as v (v.provider)}
        <button class="mini" class:stale={v.stale} onclick={() => set({ tab: v.provider })}>
          <span class="nm"><Logo provider={v.provider} size={14} /> {v.name}</span>
          <span class="row">
            <Ring value={v.limit ? v.limit.pct / 100 : 0} size={44} stroke={5} color={TONE[tone(v.limit?.pct)]}>
              {#if v.limit}<b class="rp">{Math.round(v.limit.pct)}</b>{/if}
            </Ring>
            <span class="col">
              <b>{v.limit ? pct(v.limit.pct) : `${tokens(v.hot)} tok`}</b>
              <span class="k">{v.limit ? (v.limit.resetsAt ? `resets ${until(v.limit.resetsAt, t.now)}` : v.limit.label) : v.usage?.session ? 'this session' : 'today'}</span>
            </span>
          </span>
        </button>
      {/each}
    </div>
    {#if series.length}<section class="sec"><Bars {series} /></section>{/if}
  {:else if current}
    <ProviderPanel v={current} now={t.now} {myr} manage={() => (manage = true)} prefer={prefer(current.provider)} />
  {/if}
{/if}

<style>
  .opts { margin-top: var(--s-3); }
  .empty { display: flex; flex-direction: column; align-items: center; gap: var(--s-3); padding: var(--s-6) var(--s-4); text-align: center; }
  .empty p { margin: 0; font-size: 14px; color: var(--fg-dim); }
  .rings { display: flex; gap: var(--s-3); padding: var(--s-3); border-radius: var(--r-chip); border: 1px dashed var(--fg-3); }
  .primary { height: 36px; padding: 0 var(--s-4); border-radius: var(--r-inner); background: var(--accent); color: #fff; font-weight: 500; }
  .primary:hover { filter: brightness(1.1); }
  .primary:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .seg { display: flex; gap: 2px; padding: 2px; margin-bottom: var(--s-3); border-radius: var(--r-inner); background: var(--bg-2); border: 1px solid var(--border); }
  .seg button { flex: 1; height: 28px; border-radius: calc(8px * var(--rs)); font-size: 12px; transition: background 0.2s var(--ease); }
  .seg button:hover:not(.sel) { background: var(--pill-hover); }
  .seg button:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .cards { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-2); }
  .mini { display: flex; flex-direction: column; gap: var(--s-2); padding: var(--s-3); border-radius: var(--r-inner); background: var(--bg-1); border: 1px solid var(--border); text-align: left; }
  .mini:hover { background: var(--pill-hover); }
  .mini:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .mini.stale { opacity: 0.6; }
  .nm { display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; }
  .row { display: flex; align-items: center; gap: var(--s-3); }
  .col { display: flex; flex-direction: column; min-width: 0; }
  .col b { font-size: 16px; font-weight: 600; }
  .rp { font-size: 12px; font-weight: 600; }
  .k { font-size: 11px; color: var(--fg-3); }
  .sec { margin-top: var(--s-3); padding-top: var(--s-3); border-top: 1px solid var(--border); }
  .icon-btn.on { background: var(--bg-2); }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
