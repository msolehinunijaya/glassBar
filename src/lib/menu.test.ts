import { describe, expect, it, vi } from 'vitest';

// Fake Tauri menus that log every resource call; popups stay open until closeMenu().
const log: string[] = [];
const popups: (() => void)[] = [];
let ids = 0;
vi.mock('@tauri-apps/api/menu', () => ({
  Menu: {
    new: async (opts: { items: unknown[] }) => {
      const id = ++ids;
      log.push(`new${id}:${JSON.stringify(opts.items.map((i: any) => i.item ?? i.id))}`);
      return {
        close: async () => { await Promise.resolve(); log.push(`closed${id}`); },
        popup: () => { log.push(`popup${id}`); return new Promise<void>((ok) => popups.push(ok)); },
      };
    },
  },
}));

const { showMenu } = await import('./menu');
const tick = () => new Promise((r) => setTimeout(r, 0));
const closeMenu = async () => { popups.shift()!(); await tick(); };

describe('showMenu', () => {
  // Regression: a menu resource call while a menu was open deadlocked GlassBar's main thread.
  it('touches no menu resource while a menu is open, and frees the old menu before the next popup', async () => {
    const first = showMenu([{ text: 'A' }, '-', { text: 'B' }]);
    await tick();
    expect(log).toEqual(['new1:["menu1-0","Separator","menu1-2"]', 'popup1']);

    const second = showMenu([{ text: 'C' }]);
    await tick();
    expect(log.length).toBe(2); // waits: nothing created or freed under the open menu

    await closeMenu();
    await first;
    expect(log.slice(2)).toEqual(['new2:["menu2-0"]', 'closed1', 'popup2']);

    await closeMenu();
    await second;
  });
});
