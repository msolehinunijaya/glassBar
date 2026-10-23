<script lang="ts">
  import { ArchiveRestore, Pin, Plus, Trash2, X } from '@lucide/svelte';
  import { ipc, type Note } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PopupProps } from '../types';
  import { COLORS } from './index';

  let {}: PopupProps = $props();
  let trash = $state(false);
  const notes = live<Note[]>('notes', () => ipc.notesList(trash), []);
  $effect(() => { trash; ipc.notesList(trash).then((v) => (notes.v = v)); });

  let selectedId = $state<number | null>(null);
  const note = $derived(notes.v.find((n) => n.id === selectedId) ?? notes.v[0]);
  let draft = $state('');
  let draftFor = -1;
  let error = $state('');
  let images = $state<Record<string, string>>({});
  const requested = new Set<string>();

  // Keep the textarea's text local while typing; save 500 ms after the last key.
  $effect(() => {
    if (note && note.id !== draftFor) { draft = note.text; draftFor = note.id; }
  });
  let saveTimer: ReturnType<typeof setTimeout>;
  function onInput(v: string) {
    draft = v;
    const n = note;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => n && ipc.noteUpdate({ ...n, text: v }), 500);
  }
  $effect(() => () => { clearTimeout(saveTimer); if (note && draft !== note.text) ipc.noteUpdate({ ...note, text: draft }); });

  $effect(() => {
    for (const name of note?.images ?? []) {
      if (requested.has(name)) continue;
      requested.add(name);
      ipc.noteImage(name).then((src) => (images[name] = src));
    }
  });

  async function create() {
    const n = await ipc.noteCreate();
    selectedId = n.id;
  }

  // Ctrl+V of an image (e.g. a screenshot) attaches it; text pastes normally.
  async function onPaste(e: ClipboardEvent) {
    const item = [...(e.clipboardData?.items ?? [])].find((i) => i.type.startsWith('image/'));
    if (!item || !note) return;
    e.preventDefault();
    const file = item.getAsFile();
    if (!file) return;
    if (file.size > 10 * 1024 * 1024) { error = 'Image is larger than 10 MB'; return; }
    const data = await new Promise<string>((ok) => { const r = new FileReader(); r.onload = () => ok(String(r.result)); r.readAsDataURL(file); });
    try { await ipc.noteAddImage(note.id, data); error = ''; } catch (err) { error = String(err); }
  }

  function startDrag(e: DragEvent, name: string) {
    e.preventDefault(); // hand the drag to Windows so the file can land in other apps
    ipc.noteDragImage(name);
  }
</script>

<div class="head">
  <h3>{trash ? 'Trash' : 'Notes'}</h3>
  <div class="tools">
    {#if !trash}<button class="icon-btn" onclick={create} aria-label="New note"><Plus size={16} /></button>{/if}
    <button class="icon-btn" class:on={trash} onclick={() => { trash = !trash; selectedId = null; }} aria-label="Trash"><Trash2 size={15} /></button>
  </div>
</div>

<div class="chips">
  {#each notes.v as n (n.id)}
    <button class="chip" class:sel={n.id === note?.id} style:--c={COLORS[n.color] ?? COLORS.yellow} onclick={() => (selectedId = n.id)}>
      {#if n.pinned}<Pin size={10} />{/if}
      <span class="ellipsis">{n.text.split('\n')[0] || 'Empty note'}</span>
    </button>
  {/each}
</div>

{#if note}
  {#if trash}
    <div class="card preview">{note.text || 'Empty note'}</div>
    <button class="btn" onclick={() => ipc.noteRestore(note.id)}><ArchiveRestore size={14} /> Restore</button>
    <p class="dim small">Notes in the trash are deleted for good after 7 days.</p>
  {:else}
    <div class="paper" style:--c={COLORS[note.color] ?? COLORS.yellow}>
      <textarea value={draft} oninput={(e) => onInput(e.currentTarget.value)} onpaste={onPaste}
                placeholder="Type or paste text and images…" aria-label="Note"></textarea>
      {#if note.images.length}
        <div class="strip">
          {#each note.images as name (name)}
            <div class="thumb">
              {#if images[name]}<img src={images[name]} alt="Pasted" draggable="true" ondragstart={(e) => startDrag(e, name)} title="Drag into another app" />{/if}
              <button class="rm" onclick={() => ipc.noteRemoveImage(note.id, name)} aria-label="Remove image"><X size={10} /></button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
    <div class="bar">
      {#each Object.entries(COLORS) as [name, c] (name)}
        <button class="dot" class:sel={note.color === name} style:background={c} onclick={() => ipc.noteUpdate({ ...note, text: draft, color: name })} aria-label={name}></button>
      {/each}
      <span class="grow"></span>
      <button class="icon-btn" class:on={note.pinned} onclick={() => ipc.noteUpdate({ ...note, text: draft, pinned: !note.pinned })} aria-label="Pin"><Pin size={15} /></button>
      <button class="icon-btn" onclick={() => ipc.noteDelete(note.id)} aria-label="Delete note"><Trash2 size={15} /></button>
    </div>
  {/if}
  {#if error}<p class="err">{error}</p>{/if}
{:else}
  <div class="empty dim">
    {#if trash}Trash is empty.{:else}<button class="btn" onclick={create}><Plus size={14} /> New note</button>{/if}
  </div>
{/if}

<style>
  .head { display: flex; align-items: center; justify-content: space-between; }
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .tools { display: flex; }
  .icon-btn.on { color: var(--accent); }
  .chips { display: flex; gap: 6px; overflow-x: auto; padding: 8px 0; scrollbar-width: none; }
  .chip { flex: none; display: flex; align-items: center; gap: 4px; max-width: 120px; padding: 4px 10px; border-radius: 10px;
          background: color-mix(in srgb, var(--c) 30%, transparent); border: 1px solid transparent; font-size: 12px; }
  .chip.sel { border-color: var(--c); }
  .paper { border-radius: 16px; background: color-mix(in srgb, var(--c) 22%, var(--pill)); padding: 10px; display: flex; flex-direction: column; gap: 8px; }
  textarea { width: 100%; height: 220px; resize: none; border: 0; background: none; outline: none; line-height: 1.5; }
  .strip { display: flex; gap: 6px; overflow-x: auto; }
  .thumb { position: relative; flex: none; }
  .thumb img { height: 64px; border-radius: 8px; display: block; cursor: grab; }
  .rm { position: absolute; top: 2px; right: 2px; width: 16px; height: 16px; border-radius: 50%; background: rgba(0,0,0,.55); color: #fff; display: grid; place-items: center; }
  .bar { display: flex; align-items: center; gap: 6px; margin-top: 8px; }
  .dot { width: 18px; height: 18px; border-radius: 50%; border: 2px solid transparent; }
  .dot.sel { border-color: var(--fg); }
  .grow { flex: 1; }
  .preview { white-space: pre-wrap; max-height: 260px; overflow: auto; margin-bottom: 8px; }
  .small { font-size: 12px; }
  .err { color: #ff453a; font-size: 12px; }
  .empty { display: grid; place-items: center; padding: 60px 0; }
</style>
