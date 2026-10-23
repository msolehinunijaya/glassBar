// Native context menus.
// - A menu must stay alive until its click has been delivered, so the previous one is
//   freed only when the next one opens.
// - Every item gets a unique id. Tauri maps item id -> click handler; when two menus
//   shared an id ("bar:edit"), freeing the old menu deleted the new menu's handler, so
//   "Edit widgets" / "Settings…" worked once and then never again.
import { Menu, PredefinedMenuItem, type MenuItemOptions } from '@tauri-apps/api/menu';

export type Item = Omit<MenuItemOptions, 'id'> | PredefinedMenuItem;

let last: Menu | null = null;
let seq = 0;

export async function showMenu(items: Item[]) {
  const n = ++seq;
  const prev = last;
  last = await Menu.new({
    items: items.map((it, i) => (it instanceof PredefinedMenuItem ? it : { ...it, id: `menu${n}-${i}` })),
  });
  prev?.close().catch(() => {});
  await last.popup();
}
