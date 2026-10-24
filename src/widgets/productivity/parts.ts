// The three parts of the Productivity widget (each was its own widget). Their settings sit
// flat in this widget's settings (workMin…, temporaryDays), so the parts work unchanged.
import type { Component } from 'svelte';
import PomoPill from '../pomodoro/Pill.svelte';
import PomoPopup from '../pomodoro/Popup.svelte';
import TodoPill from '../todo/Pill.svelte';
import TodoPopup from '../todo/Popup.svelte';
import NotesPill from '../notes/Pill.svelte';
import NotesPopup from '../notes/Popup.svelte';

export type PartKey = 'pomodoro' | 'todo' | 'notes';
export interface Part { key: PartKey; label: string; show: 'showPomo' | 'showTodo' | 'showNotes'; Pill: Component<any>; Popup: Component<any>; width: number; height: number }

export const PARTS: Part[] = [
  { key: 'pomodoro', label: 'Pomodoro', show: 'showPomo', Pill: PomoPill, Popup: PomoPopup, width: 340, height: 560 },
  { key: 'todo', label: 'To-Do', show: 'showTodo', Pill: TodoPill, Popup: TodoPopup, width: 340, height: 460 },
  { key: 'notes', label: 'Notes', show: 'showNotes', Pill: NotesPill, Popup: NotesPopup, width: 360, height: 480 },
];

/** Popup id of a part's panel: "__part:<key>:<widget instance>". */
export const PART_PANEL = '__part:';
export const partPanel = (key: PartKey, instance: string) => `${PART_PANEL}${key}:${instance}`;
export function parsePartPanel(id: string): { part: Part; instance: string } | null {
  if (!id.startsWith(PART_PANEL)) return null;
  const [key, ...rest] = id.slice(PART_PANEL.length).split(':');
  const part = PARTS.find((p) => p.key === key);
  return part ? { part, instance: rest.join(':') } : null;
}
