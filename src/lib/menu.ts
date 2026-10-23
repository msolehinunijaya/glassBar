// Native context menus. A menu must stay alive until its click has been delivered,
// so we free the previous one only when the next one opens (closing right after
// popup() dropped the item handlers, and "Edit widgets" did nothing).
import { Menu, type MenuOptions } from '@tauri-apps/api/menu';

let last: Menu | null = null;

export async function showMenu(items: MenuOptions['items']) {
  const prev = last;
  last = await Menu.new({ items });
  prev?.close().catch(() => {});
  await last.popup();
}
