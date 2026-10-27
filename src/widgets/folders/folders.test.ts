import { describe, expect, it } from 'vitest';
import { addApps, folderKey, isApp, moveApp, parseFolderKey, pinKey, removeApp, type Folder } from './folders';

const f = (...paths: string[]): Folder => ({ id: 'a', name: '', apps: paths.map((path) => ({ path })) });
const names = (x: Folder) => x.apps.map((p) => p.path);

describe('folders', () => {
  it('adds apps once, in order', () => {
    expect(names(addApps(f('C:\a.exe'), [{ path: 'C:\b.exe' }, { path: 'c:\A.EXE' }, { path: 'C:\b.exe' }]))).toEqual(['C:\a.exe', 'C:\b.exe']);
  });
  it('removes and moves by key', () => {
    const x = f('C:\a.exe', 'C:\b.exe', 'C:\c.exe');
    expect(names(removeApp(x, pinKey({ path: 'C:\b.exe' })))).toEqual(['C:\a.exe', 'C:\c.exe']);
    expect(names(moveApp(x, pinKey({ path: 'C:\c.exe' }), 0))).toEqual(['C:\c.exe', 'C:\a.exe', 'C:\b.exe']);
    expect(names(moveApp(x, pinKey({ path: 'C:\a.exe' }), 99))).toEqual(['C:\b.exe', 'C:\c.exe', 'C:\a.exe']);
    expect(moveApp(x, 'nope', 0)).toBe(x);
  });
  it('round-trips its panel key, click and hover', () => {
    expect(parseFolderKey(folderKey(false, 'folders-1', 'f9'))).toEqual({ instance: 'folders-1', id: 'f9', hover: false });
    expect(parseFolderKey(folderKey(true, 'folders-1', 'f9'))?.hover).toBe(true);
    expect(parseFolderKey('__hover:x')).toBeNull();
  });
  it('takes programs and shortcuts only', () => {
    expect(isApp('C:/x/Tool.LNK')).toBe(true);
    expect(isApp('C:/x/notes.txt')).toBe(false);
  });
});
