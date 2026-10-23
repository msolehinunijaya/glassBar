// Native context menus.
// - A menu must stay alive until its click has been delivered, so the previous one is
//   freed only when the next one opens.
// - Every item gets a unique id (submenu items too). Tauri maps item id -> click handler;
//   when two menus shared an id ("bar:edit"), freeing the old menu deleted the new menu's
//   handler, so "Edit widgets" / "Settings…" worked once and then never again.
import { Menu, PredefinedMenuItem, type MenuItemOptions } from '@tauri-apps/api/menu';

/** A plain item, a ticked item (`checked`), a submenu (`items`), or a separator. */
export type Item =
  | (Omit<MenuItemOptions, 'id'> & { checked?: boolean })
  | { text: string; items: Item[]; enabled?: boolean }
  | PredefinedMenuItem;

let last: Menu | null = null;
let seq = 0;

function withIds(items: Item[], prefix: string): any[] {
  return items.map((it, i) => {
    if (it instanceof PredefinedMenuItem) return it;
    const id = `${prefix}-${i}`;
    return 'items' in it ? { ...it, id, items: withIds(it.items, id) } : { ...it, id };
  });
}

export async function showMenu(items: Item[]) {
  const n = ++seq;
  const prev = last;
  last = await Menu.new({ items: withIds(items, `menu${n}`) });
  prev?.close().catch(() => {});
  await last.popup();
}
