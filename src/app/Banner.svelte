<script lang="ts">
  // Pop-up banners next to the dock for GlassBar's own notifications. Up to 3 stacked
  // (newest on top), "+N more" beyond. Each shows its widget's icon, title, one line and
  // the time, with a thin line for the time left. Hover pauses; Esc closes the hovered
  // one; × closes without opening anything; a click jumps to the widget. The window never
  // takes focus. One timer at most, and only while something is counting down.
  import { onMount } from 'svelte';
  import { AlarmClock, Bell, Check, Download, Moon, Play, Timer, Wind, X, ListChecks } from '@lucide/svelte';
  import { profile, store, updateWidgetSettings } from '../lib/config.svelte';
  import { events, ipc, type Todo } from '../lib/ipc';
  import { dismiss, expire, nextDeadline, pause, push, resume, shown, SOURCE_WIDGET, type Banner, type Note } from '../lib/notify';
  import { stopSound } from '../lib/sound';

  const ICON: Record<string, typeof Bell> = { timer: AlarmClock, pomodoro: Timer, todo: ListChecks, solat: Moon, ipu: Wind, downloads: Download };
  const ms = $derived((store.config?.banners.seconds ?? 5) * 1000);
  let q = $state<Banner[]>([]);
  let hovered = $state<number | null>(null);
  const view = $derived(shown(q));
  const edge = $derived(store.config ? profile().bar.edge : 'bottom');
  const time = (at: number) => new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' }).format(at);

  // One timeout for the soonest deadline; none while nothing counts down.
  let timer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    clearTimeout(timer);
    const d = nextDeadline(q);
    if (d !== null) timer = setTimeout(() => (q = expire(q, Date.now(), ms)), Math.max(0, d - Date.now()) + 20);
    return () => clearTimeout(timer);
  });
  $effect(() => { if (!q.length) ipc.bannerHide(); });

  // Fit the window to the stack.
  let stack = $state<HTMLElement>();
  $effect(() => {
    if (!stack) return;
    const ro = new ResizeObserver(() => { if (q.length) ipc.bannerFit(340, Math.ceil(stack!.getBoundingClientRect().height) + 4); });
    ro.observe(stack);
    return () => ro.disconnect();
  });

  onMount(() => {
    const off = events.bannerPush((n: Note) => { q = push(q, n, Date.now(), ms); });
    const key = (e: KeyboardEvent) => { if (e.key === 'Escape' && hovered !== null) close(hovered); };
    addEventListener('keydown', key);
    return () => { off.then((f) => f()); removeEventListener('keydown', key); };
  });

  const close = (id: number) => { q = dismiss(q, id, Date.now(), ms); hovered = null; };
  function open(b: Banner) { close(b.id); ipc.jumpTo(b.source, b.data); }
  function enter(id: number) { hovered = id; q = pause(q, Date.now()); }
  function leave() { hovered = null; q = resume(q, Date.now()); }

  // ---- Buttons per type ----
  // A break just ended -> start work; work ended -> start the break (as the pill does).
  const nextPomo = (title: string) => (title.toLowerCase().includes('break') ? 'Start focus' : 'Start break');
  const dataOf = (b: Banner) => (b.data ?? {}) as { timerId?: number; key?: string; auto?: boolean };
  async function act(e: Event, b: Banner, f: () => unknown) { e.stopPropagation(); await f(); close(b.id); }
  async function todoDone(key: string) {
    const id = Number(key.split(':')[1]);
    const t = (await ipc.todoList()).find((x: Todo) => x.id === id);
    if (t) await ipc.todoUpdate({ ...t, done: true });
  }
  function ackPrayer(key: string) {
    // "solat:<date>:<zone>:<name>" -> the prayer pill's own key "<date>:<name>"
    const [, date, , name] = key.split(':');
    const w = profile().widgets.find((x) => x.id === 'solat');
    stopSound();
    if (w && date && name) updateWidgetSettings(w.instance, { ackKey: `${date}:${name}` });
  }
</script>

<div class="stack" data-edge={edge} bind:this={stack} aria-live="polite">
  {#each view.list as b (b.id)}
    {@const I = ICON[b.source] ?? Bell}
    {@const d = dataOf(b)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="banner" role="button" tabindex="-1" onclick={() => open(b)} onpointerenter={() => enter(b.id)} onpointerleave={leave}
         aria-label="{b.title}. {b.body}. Click to go to {SOURCE_WIDGET[b.source]?.label ?? 'notifications'}.">
      <span class="ic"><I size={17} /></span>
      <div class="txt">
        <div class="row1"><b>{b.title}</b><span class="dim tm">{time(b.at)}</span></div>
        {#if b.body}<div class="dim body">{b.body}</div>{/if}
        {#if b.source === 'timer' && d.timerId != null}
          <div class="acts">
            <button onclick={(e) => act(e, b, () => ipc.timerDelete(d.timerId!))}>Dismiss</button>
            <button onclick={(e) => act(e, b, () => ipc.timerSnooze(d.timerId!, 60_000))}>+1 min</button>
          </div>
        {:else if b.source === 'pomodoro' && !d.auto}
          <div class="acts"><button onclick={(e) => act(e, b, () => ipc.pomoStart())}><Play size={11} /> {nextPomo(b.title)}</button></div>
        {:else if b.source === 'todo' && d.key}
          <div class="acts"><button onclick={(e) => act(e, b, () => todoDone(d.key!))}><Check size={11} /> Mark done</button></div>
        {:else if b.source === 'solat' && d.key}
          <div class="acts"><button onclick={(e) => act(e, b, () => ackPrayer(d.key!))}><Check size={11} /> Acknowledge</button></div>
        {/if}
      </div>
      <button class="x" onclick={(e) => { e.stopPropagation(); close(b.id); }} aria-label="Close"><X size={13} /></button>
      <span class="life" class:paused={b.left !== null} style:animation-duration="{ms}ms"></span>
    </div>
  {/each}
  {#if view.more}<div class="more dim">+{view.more} more</div>{/if}
</div>

<style>
  .stack { display: flex; flex-direction: column; gap: 6px; padding: 2px; }
  /* On a top dock the newest sits nearest the dock (top); on a bottom dock, nearest it too. */
  .stack[data-edge='bottom'] { flex-direction: column-reverse; }
  .banner {
    position: relative; overflow: hidden; display: flex; align-items: flex-start; gap: 10px; padding: 10px 10px 12px;
    border-radius: calc(16px * var(--rs)); background: rgba(var(--tint), 0.92); border: 1px solid var(--line);
    /* No drop shadow: the window is only 2 px bigger than the card, so it would be cut into a grey box. */
    color: var(--fg); cursor: default; animation: in 0.28s var(--ease);
  }
  /* Windows' transparency effects off: see-through pixels turn black, so the whole window is
     one solid surface (Windows rounds its corners) and stacked banners are split by hairlines. */
  :global(:root[data-solid='true'][data-window='banner'] body) { background: rgb(var(--tint)); }
  :global(:root[data-solid='true']) .stack { padding: 0; gap: 0; }
  :global(:root[data-solid='true']) .banner { background: transparent; border: 0; border-radius: 0; box-shadow: none; }
  :global(:root[data-solid='true']) .banner + .banner { border-top: 1px solid var(--border); }
  :global(:root[data-solid='true']) .stack[data-edge='bottom'] .banner + .banner { border-top: 0; border-bottom: 1px solid var(--border); }
  .stack[data-edge='top'] .banner { animation-name: in-top; }
  .stack[data-edge='left'] .banner { animation-name: in-left; }
  .stack[data-edge='right'] .banner { animation-name: in-right; }
  @keyframes in { from { opacity: 0; transform: translateY(14px); } }
  @keyframes in-top { from { opacity: 0; transform: translateY(-14px); } }
  @keyframes in-left { from { opacity: 0; transform: translateX(-14px); } }
  @keyframes in-right { from { opacity: 0; transform: translateX(14px); } }
  .banner:hover { border-color: color-mix(in srgb, var(--accent) 60%, transparent); }
  .ic { display: grid; place-items: center; width: 30px; height: 30px; border-radius: calc(9px * var(--rs)); flex: none; background: var(--accent); color: #fff; }
  .txt { flex: 1; min-width: 0; }
  .row1 { display: flex; align-items: baseline; gap: 8px; }
  .row1 b { flex: 1; font-size: 13px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tm { font-size: 11px; flex: none; }
  .body { font-size: 12px; margin-top: 1px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .acts { display: flex; gap: 6px; margin-top: 7px; }
  .acts button { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: calc(8px * var(--rs)); background: var(--pill); font-size: 11.5px; font-weight: 500; }
  .acts button:hover { background: var(--pill-hover); }
  .x { display: grid; place-items: center; width: 22px; height: 22px; border-radius: calc(7px * var(--rs)); opacity: 0.6; flex: none; }
  .x:hover { opacity: 1; background: var(--pill-hover); }
  /* Time left: a thin line along the bottom (CSS only; pauses with hover). */
  .life { position: absolute; left: 0; right: 0; bottom: 0; height: 2px; background: var(--accent); transform-origin: left; animation: life linear forwards; }
  .life.paused { animation-play-state: paused; }
  @keyframes life { to { transform: scaleX(0); } }
  .more { text-align: center; font-size: 11.5px; padding: 2px 0; }
</style>
