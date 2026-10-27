// Which Tauri API the shelf listens on for an OS file drop.
//
// This looks like a style check and is not. Tauri has three event targets -- Webview, Window
// and WebviewWindow -- and for a webview that *is* a window's content (every GlassBar window),
// the runtime sends drag-drop as a SynthesizedWindowEvent, i.e. to the **Window** target
// (tauri-runtime-wry: `kind == WebviewKind::WindowContent`).
//
// `getCurrentWebview().onDragDropEvent()` subscribes to the **Webview** target, so it is never
// called: no error, no warning, dropping a file simply does nothing. That was the bug.
// `getCurrentWebviewWindow()` subscribes to the WebviewWindow target, which receives both --
// and is what Tauri's own example app uses.
//
// There is no way to unit-test this against the real runtime without a browser and a mouse, so
// the source is the thing being asserted.
import { describe, expect, it } from 'vitest';

const fsMod = 'node:fs';
const src = async (p: string): Promise<string> => {
  const fs = await import(/* @vite-ignore */ fsMod);
  return fs.readFileSync(p, 'utf8');
};

describe('the shelf listens for drops where Tauri actually sends them', () => {
  it('uses getCurrentWebviewWindow, not getCurrentWebview', async () => {
    const pill = await src('src/widgets/shelf/Pill.svelte');
    expect(pill).toContain('onDragDropEvent');
    expect(pill).toContain('getCurrentWebviewWindow');
    // The trap: the Webview target silently never fires for a window-content webview.
    expect(pill).not.toMatch(/getCurrentWebview\s*\(/);
    expect(pill).not.toMatch(/from ['"]@tauri-apps\/api\/webview['"]/);
  });

  it('is the only place that listens for drops, so there is one thing to get right', async () => {
    const fs = await import(/* @vite-ignore */ fsMod);
    const hits: string[] = [];
    const walk = (dir: string) => {
      for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        const p = `${dir}/${e.name}`;
        if (e.isDirectory()) walk(p);
        else if (/\.(ts|svelte)$/.test(e.name) && !/\.test\.ts$/.test(e.name)
          && fs.readFileSync(p, 'utf8').includes('onDragDropEvent')) hits.push(p);
      }
    };
    walk('src');
    expect(hits).toEqual(['src/widgets/shelf/Pill.svelte']);
  });
});
