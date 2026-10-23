// The only file that talks to Rust. Types mirror src-tauri structs (camelCase).
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Edge } from './geometry';

export type WidgetSize = 'small' | 'medium' | 'large';
export interface WidgetInst { id: string; instance: string; size: WidgetSize; settings: any }
export interface BarCfg {
  edge: Edge; monitor: number; length: number; size: 'compact' | 'normal' | 'large';
  floating: { x: number; y: number } | null; reserveSpace: boolean;
  nativeTaskbar: 'show' | 'autohide' | 'hidden';
}
export interface Profile { bar: BarCfg; widgets: WidgetInst[] }
export interface Theme { mode: 'auto' | 'light' | 'dark'; accent: string; blur: number; opacity: number; nativeBlur: boolean }
export interface LocationCfg { auto: boolean; lat: number | null; lon: number | null; name: string }
export interface Config {
  version: number; activeProfile: string; profiles: Record<string, Profile>;
  theme: Theme; startup: { launchAtLogin: boolean };
  dnd: boolean; location: LocationCfg;
}

export interface Alert { id: number; source: string; title: string; body: string; at: number; missed: boolean; read: boolean }
export interface Todo { id: number; text: string; done: boolean; dueAt: number | null; createdAt: number }
export type Sound = { kind: 'chime' } | { kind: 'azan'; src: string };

export interface AppWindow { id: number; title: string; exe: string; active: boolean; minimized: boolean }
export interface AudioDevice { id: string; name: string; isDefault: boolean }
export interface AudioSession { pid: number; name: string; exe: string; volume: number; muted: boolean }
export interface AudioState { volume: number; muted: boolean; devices: AudioDevice[] }
export interface MediaInfo {
  app: string; title: string; artist: string; album: string; art: string | null; playing: boolean;
  positionMs: number; durationMs: number; updatedAtMs: number;
  canNext: boolean; canPrev: boolean;
}
export interface RadioInfo { kind: 'wifi' | 'bluetooth' | 'other'; name: string; on: boolean }
export type DisplayMode = 'internal' | 'clone' | 'extend' | 'external';

export const ipc = {
  // config
  getConfig: () => invoke<Config>('get_config'),
  saveConfig: (config: Config) => invoke<void>('save_config', { config }),
  exportConfig: () => invoke<string>('export_config'),
  importConfig: (text: string) => invoke<void>('import_config', { text }),
  // bar + windows
  applyBar: () => invoke<void>('apply_bar'),
  barPreview: (length: number) => invoke<void>('bar_preview', { length }),
  barDrag: (on: boolean) => invoke<void>('bar_drag', { on }),
  mouseDown: () => invoke<boolean>('mouse_down'),
  restoreTaskbar: () => invoke<void>('restore_taskbar'),
  setEditMode: (on: boolean) => invoke<void>('set_edit_mode', { on }),
  openSettings: () => invoke<void>('open_settings_cmd'),
  popupOpen: (instance: string, anchor: DOMRect, width: number, height: number) =>
    invoke<void>('popup_open', { instance, x: anchor.x, y: anchor.y, w: anchor.width, h: anchor.height, width, height }),
  popupHide: () => invoke<void>('popup_hide'),
  // alerts
  alertsList: () => invoke<Alert[]>('alerts_list'),
  alertsMarkRead: () => invoke<void>('alerts_mark_read'),
  alertsDismiss: (id: number) => invoke<void>('alerts_dismiss', { id }),
  alertsClear: () => invoke<void>('alerts_clear'),
  // to-do
  todoList: () => invoke<Todo[]>('todo_list'),
  todoAdd: (text: string, dueAt: number | null) => invoke<void>('todo_add', { text, dueAt }),
  todoUpdate: (t: Todo) => invoke<void>('todo_update', { id: t.id, text: t.text, done: t.done, dueAt: t.dueAt }),
  todoDelete: (id: number) => invoke<void>('todo_delete', { id }),
  todoClearDone: () => invoke<void>('todo_clear_done'),
  // apps
  appsList: () => invoke<AppWindow[]>('apps_list'),
  appIcon: (exe: string) => invoke<string | null>('app_icon', { exe }),
  appActivate: (id: number) => invoke<void>('app_activate', { id }),
  appClose: (id: number) => invoke<void>('app_close', { id }),
  appLaunch: (path: string, args = '') => invoke<void>('app_launch', { path, args }),
  // audio
  audioState: () => invoke<AudioState>('audio_state'),
  audioSetVolume: (volume: number) => invoke<void>('audio_set_volume', { volume }),
  audioSetMute: (muted: boolean) => invoke<void>('audio_set_mute', { muted }),
  audioSetDefault: (id: string) => invoke<void>('audio_set_default', { id }),
  audioSessions: () => invoke<AudioSession[]>('audio_sessions'),
  audioSetSessionVolume: (pid: number, volume: number) => invoke<void>('audio_set_session_volume', { pid, volume }),
  audioSetSessionMute: (pid: number, muted: boolean) => invoke<void>('audio_set_session_mute', { pid, muted }),
  // media
  mediaInfo: () => invoke<MediaInfo | null>('media_info'),
  mediaControl: (action: 'toggle' | 'next' | 'prev') => invoke<void>('media_control', { action }),
  mediaSeek: (positionMs: number) => invoke<void>('media_seek', { positionMs }),
  // quick settings
  radios: () => invoke<RadioInfo[]>('radios_list'),
  setRadio: (kind: RadioInfo['kind'], on: boolean) => invoke<void>('radio_set', { kind, on }),
  setAirplane: (on: boolean) => invoke<void>('airplane_set', { on }),
  displayMode: () => invoke<DisplayMode | null>('display_mode'),
  setDisplayMode: (mode: DisplayMode) => invoke<void>('display_set_mode', { mode }),
  brightness: () => invoke<number | null>('brightness_get'),
  setBrightness: (value: number) => invoke<void>('brightness_set', { value }),
};

export const events = {
  configChanged: (cb: (c: Config) => void) => listen<Config>('config:changed', (e) => cb(e.payload)),
  editMode: (cb: (on: boolean) => void) => listen<boolean>('editmode', (e) => cb(e.payload)),
  popupShow: (cb: (instance: string) => void) => listen<string>('popup:show', (e) => cb(e.payload)),
  popupHide: (cb: () => void) => listen('popup:hide', () => cb()),
  appsChanged: (cb: (w: AppWindow[]) => void) => listen<AppWindow[]>('apps:changed', (e) => cb(e.payload)),
  audioChanged: (cb: (s: AudioState) => void) => listen<AudioState>('audio:changed', (e) => cb(e.payload)),
  mediaChanged: (cb: (m: MediaInfo | null) => void) => listen<MediaInfo | null>('media:changed', (e) => cb(e.payload)),
  alertsChanged: (cb: () => void) => listen('alerts:changed', () => cb()),
  soundPlay: (cb: (s: Sound) => void) => listen<Sound>('sound:play', (e) => cb(e.payload)),
  /** Any "<name>:changed" event from Rust (todos, notes, timers, ...). */
  changed: (name: string, cb: () => void) => listen(`${name}:changed`, () => cb()),
};
export type { UnlistenFn };
