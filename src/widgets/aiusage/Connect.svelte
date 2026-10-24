<script lang="ts">
  // Connect page: one card per source. Keys go straight to Rust (Windows Credential Manager)
  // after one test call; only their last 4 characters ever come back. Web plans sign in
  // inside GlassBar's own browser window, with its own cookie folder.
  import { Check, LogIn, TriangleAlert } from '@lucide/svelte';
  import { ipc, type AiuReading, type AiuSource } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import Logo from './Logo.svelte';
  import type { Provider } from './model';

  let { readings }: { readings: AiuReading[] } = $props();

  const CARDS: { source: AiuSource; provider: Provider; name: string; method: 'key' | 'auto' | 'web'; unlocks: string; how: string; placeholder?: string; unofficial?: boolean; off?: string }[] = [
    { source: 'anthropic', provider: 'claude', name: 'Claude (API)', method: 'key', placeholder: 'sk-ant-admin…',
      unlocks: 'Tokens in/out, cost, by model, by day', how: 'Paste an Admin API key from console.anthropic.com › Settings › Admin keys.' },
    { source: 'claudecode', provider: 'claude', name: 'Claude Code', method: 'auto', off: 'Off, or no Claude Code logs on this PC',
      unlocks: 'Session tokens, 5-hour window, by project', how: 'Read from the logs in %USERPROFILE%\\.claude\\projects. No sign-in needed.' },
    { source: 'openai', provider: 'chatgpt', name: 'ChatGPT / OpenAI (API)', method: 'key', placeholder: 'sk-admin-…',
      unlocks: 'Tokens, cost, by model, by day', how: 'Paste an Admin key from platform.openai.com › Organization › Admin keys.' },
    { source: 'claudeplan', provider: 'claude', name: 'Claude plan', method: 'auto', unofficial: true, off: 'Off, or Claude Code is not signed in here',
      unlocks: 'Session and weekly limits with reset times', how: 'Uses the Claude Code login on this PC, like the /usage command in Claude Code. No sign-in; the login is only sent to Anthropic. If it says expired, open Claude Code once.' },
    { source: 'claudeweb', provider: 'claude', name: 'Claude plan (claude.ai)', method: 'web', unofficial: true,
      unlocks: 'Current session and weekly limits with reset times', how: 'Sign in to claude.ai in a GlassBar window: use this if you use the Claude app rather than Claude Code. Its cookies stay in their own folder on this PC; usage is read at most every 5 minutes.' },
    { source: 'chatgptweb', provider: 'chatgpt', name: 'ChatGPT plan', method: 'web', unofficial: true,
      unlocks: 'Plan limit % and reset time', how: 'Sign in to chatgpt.com in a GlassBar window. Its cookies stay in their own folder on this PC; usage is read at most every 5 minutes.' },
  ];

  const keys = $state<Record<string, string>>({});
  const busy = $state<Record<string, boolean>>({});
  const msg = $state<Record<string, { ok: boolean; text: string } | undefined>>({});
  const of = (s: AiuSource) => readings.find((r) => r.source === s);

  async function run(s: AiuSource, f: () => Promise<unknown>, ok: string) {
    busy[s] = true; msg[s] = undefined;
    try { await f(); msg[s] = ok ? { ok: true, text: ok } : undefined; keys[s] = ''; }
    catch (e) { msg[s] = { ok: false, text: String(e) }; }
    busy[s] = false;
  }
</script>

<div class="list">
  {#each CARDS as c (c.source)}
    {@const r = of(c.source)}
    <section class="src" class:on={r?.connected}>
      <div class="top">
        <Logo provider={c.provider} size={18} />
        <div class="col">
          <span class="nm">{c.name}{#if c.unofficial}<span class="badge">Unofficial, may break</span>{/if}</span>
          <span class="status">
            {#if r?.connected && r.error}
              <span class="warn"><TriangleAlert size={12} /> {r.error}</span>
            {:else if r?.connected}
              <span class="ok"><Check size={12} /> Connected</span>{#if r.hint}<span class="hint">••••{r.hint}</span>{/if}
              {#if r.updatedAt}<span class="k">· updated {relTime(r.updatedAt)}</span>{/if}
            {:else if c.method === 'auto'}
              <span class="k">{c.off}</span>
            {:else}
              <span class="k">Not connected</span>
            {/if}
          </span>
        </div>
        {#if r?.connected}
          <button class="ghost" disabled={busy[c.source]} onclick={() => run(c.source, () => ipc.aiuDisconnect(c.source), '')}>Disconnect</button>
        {:else if c.method === 'auto'}
          <button class="ghost" disabled={busy[c.source]} onclick={() => run(c.source, () => ipc.aiuConnect(c.source), c.source === 'claudecode' ? 'Reading logs…' : '✓ Connected')}>Connect</button>
        {:else if c.method === 'web'}
          <button class="ghost" disabled={busy[c.source]} onclick={() => run(c.source, () => ipc.aiuConnect(c.source), 'Sign in in the window that opened')}><LogIn size={13} /> Sign in</button>
        {/if}
      </div>
      <p class="unlocks">{c.unlocks}</p>
      {#if !r?.connected || r.errorKind === 'auth'}
        <p class="how">{c.how}</p>
        {#if c.method === 'key'}
          <form class="key" onsubmit={(e) => { e.preventDefault(); run(c.source, () => ipc.aiuConnect(c.source, keys[c.source]), '✓ Connected'); }}>
            <input type="password" autocomplete="off" spellcheck="false" placeholder={c.placeholder} aria-label="{c.name} Admin key" bind:value={keys[c.source]} />
            <button class="primary" disabled={busy[c.source] || !keys[c.source]?.trim()}>{busy[c.source] ? 'Checking…' : 'Save'}</button>
          </form>
        {:else if c.method === 'web' && r?.errorKind === 'auth'}
          <button class="ghost" onclick={() => run(c.source, () => ipc.aiuConnect(c.source), 'Sign in in the window that opened')}><LogIn size={13} /> Sign in again</button>
        {/if}
      {/if}
      {#if msg[c.source]}<p class:err={!msg[c.source]!.ok} class="msg">{msg[c.source]!.text}</p>{/if}
    </section>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: var(--s-2); }
  .src { padding: var(--s-3); border-radius: var(--r-inner); background: var(--bg-1); border: 1px solid var(--border); }
  .top { display: flex; align-items: center; gap: var(--s-3); }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .nm { font-size: 14px; font-weight: 600; display: flex; align-items: center; gap: var(--s-2); flex-wrap: wrap; }
  .badge { font-size: 11px; font-weight: 500; padding: 1px 8px; border-radius: var(--r-chip); background: var(--bg-2); color: var(--fg-dim); }
  .status { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; font-size: 12px; }
  .ok { display: inline-flex; align-items: center; gap: 4px; color: #30d158; }
  .warn { display: inline-flex; align-items: center; gap: 4px; color: #ff9f0a; }
  .hint { font-variant-numeric: tabular-nums; color: var(--fg-dim); }
  .k { color: var(--fg-3); }
  .unlocks { margin: 6px 0 0; font-size: 12px; color: var(--fg-dim); }
  .how { margin: 4px 0 0; font-size: 11px; color: var(--fg-3); line-height: 1.45; }
  .key { display: flex; gap: var(--s-2); margin-top: var(--s-2); }
  .key input { flex: 1; min-width: 0; height: 36px; padding: 0 var(--s-3); border-radius: var(--r-inner); border: 1px solid var(--border); background: var(--bg-2); font-size: 12px; }
  .key input:focus-visible { outline: 2px solid var(--accent); outline-offset: -1px; }
  button { display: inline-flex; align-items: center; gap: 4px; height: 32px; padding: 0 var(--s-3); border-radius: var(--r-inner); font-size: 12px; white-space: nowrap; }
  .ghost { background: var(--bg-2); }
  .ghost:hover:not(:disabled) { background: var(--pill-hover); }
  .primary { height: 36px; background: var(--accent); color: #fff; font-weight: 500; }
  .primary:hover:not(:disabled) { filter: brightness(1.1); }
  button:active:not(:disabled) { transform: scale(0.97); }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button:disabled { opacity: 0.45; }
  .msg { margin: 6px 0 0; font-size: 12px; color: #30d158; }
  .msg.err { color: #ff453a; }
</style>
