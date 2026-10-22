// The widget contract. A widget is a folder in src/widgets/<id>/ whose index.ts
// default-exports `defineWidget({...})`. See docs/WIDGET_SDK.md.
import type { Component } from 'svelte';
import type { WidgetInst, WidgetSize } from '../lib/ipc';

/** Props every bar pill receives. */
export interface PillProps<S = any> {
  instance: string;
  size: WidgetSize;
  settings: S;
  vertical: boolean;
}

/** Props for the optional popup panel. */
export interface PopupProps<S = any> {
  instance: string;
  settings: S;
  close: () => void;
}

/** Props for the optional per-widget settings form (Settings window). */
export interface SettingsProps<S = any> {
  instance: string;
  settings: S;
  save: (patch: Partial<S>) => void;
}

export interface WidgetManifest<S = any> {
  id: string;
  name: string;
  description: string;
  icon: Component<any>;
  sizes: WidgetSize[];
  defaultSize: WidgetSize;
  defaults: S;
  Pill: Component<PillProps<S>>;
  Popup?: { component: Component<PopupProps<S>>; width: number; height: number };
  SettingsForm?: Component<SettingsProps<S>>;
}

export const defineWidget = <S>(m: WidgetManifest<S>) => m;

/** Stored settings layered over the widget's defaults. */
export const settingsOf = <S>(m: WidgetManifest<S>, w: WidgetInst): S => ({ ...m.defaults, ...(w.settings ?? {}) });
