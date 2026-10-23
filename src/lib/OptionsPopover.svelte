<script lang="ts" module>
  /** One row of the options panel. Values are read from and saved to widget settings. */
  export type Option =
    | { type: 'choice'; key: string; label: string; choices: { value: string; label: string; hint?: string }[]; menu?: boolean }
    | { type: 'toggle'; key: string; label: string; hint?: string }
    | { type: 'checks'; label: string; items: { key: string; label: string }[]; min?: number };
</script>

<script lang="ts">
  // Shared "options" panel for widget popups (Battery style, Music style, System Monitor items).
  // Every change is saved straight into the widget's settings, so it survives a restart.
  import { toggleCheck } from './options';

  // Many or long choices don't fit as buttons in a narrow panel: show those as a dropdown.
  const asMenu = (o: Extract<Option, { type: 'choice' }>) =>
    o.menu ?? (o.choices.length > 4 || o.choices.reduce((n, c) => n + c.label.length, 0) > 30);

  let { title = 'Options', options, values, save }: {
    title?: string;
    options: Option[];
    values: Record<string, any>;
    save: (patch: Record<string, unknown>) => void;
  } = $props();
</script>

<section class="opts card">
  <h4>{title}</h4>
  {#each options as o (o.label)}
    {#if o.type === 'choice'}
      <div class="row">
        <span class="lbl dim">{o.label}</span>
        {#if asMenu(o)}
          <select value={values[o.key]} onchange={(e) => save({ [o.key]: e.currentTarget.value })}
                  title={o.choices.find((c) => c.value === values[o.key])?.hint}>
            {#each o.choices as c (c.value)}<option value={c.value}>{c.label}</option>{/each}
          </select>
        {:else}
          <div class="seg">
            {#each o.choices as c (c.value)}
              <button class:sel={values[o.key] === c.value} title={c.hint} onclick={() => save({ [o.key]: c.value })}>{c.label}</button>
            {/each}
          </div>
        {/if}
      </div>
    {:else if o.type === 'toggle'}
      <label class="row toggle">
        <span class="lbl">{o.label}{#if o.hint}<span class="dim hint">{o.hint}</span>{/if}</span>
        <input type="checkbox" class="switch" checked={!!values[o.key]} onchange={(e) => save({ [o.key]: e.currentTarget.checked })} />
      </label>
    {:else}
      {@const group = Object.fromEntries(o.items.map((i) => [i.key, !!values[i.key]]))}
      {@const checked = Object.values(group).filter(Boolean).length}
      <div class="row">
        <span class="lbl dim">{o.label}</span>
        <div class="checks">
          {#each o.items as i (i.key)}
            <label class:locked={group[i.key] && checked <= (o.min ?? 0)}>
              <input type="checkbox" checked={group[i.key]} disabled={group[i.key] && checked <= (o.min ?? 0)}
                     onchange={(e) => save(toggleCheck(group, i.key, e.currentTarget.checked, o.min ?? 0))} />
              {i.label}
            </label>
          {/each}
        </div>
      </div>
    {/if}
  {/each}
</section>

<style>
  .opts { padding: 10px 12px; display: flex; flex-direction: column; gap: 8px; }
  h4 { margin: 0; font-size: 12px; font-weight: 600; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .lbl { font-size: 12px; display: flex; flex-direction: column; flex: none; }
  .hint { font-size: 10px; }
  .seg { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 2px; padding: 2px; border-radius: 9px; background: var(--pill); min-width: 0; }
  .seg button { padding: 4px 10px; border-radius: 7px; font-size: 12px; white-space: nowrap; transition: background 0.2s var(--ease); }
  select { min-width: 0; max-width: 68%; padding: 5px 8px; border-radius: 9px; border: none; background: var(--pill); color: inherit;
           font: inherit; font-size: 12px; color-scheme: light dark; cursor: pointer; text-overflow: ellipsis; }
  select:hover { background: var(--pill-hover); }
  .seg .sel { background: var(--accent); color: #fff; }
  .checks { display: flex; gap: 10px; font-size: 12px; }
  .checks label { display: flex; align-items: center; gap: 4px; }
  .checks .locked { opacity: 0.6; }
  input[type='checkbox'] { accent-color: var(--accent); }
  .switch { appearance: none; width: 34px; height: 20px; border-radius: 10px; background: var(--track); position: relative; transition: background 0.2s var(--ease); flex: none; }
  .switch::after { content: ''; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform 0.25s var(--spring); box-shadow: 0 1px 2px rgba(0,0,0,.3); }
  .switch:checked { background: var(--accent); }
  .switch:checked::after { transform: translateX(14px); }
</style>
