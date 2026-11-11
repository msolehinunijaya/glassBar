// Privacy Monitor: what the dock icon shows, the panel's one-line status, and which changes
// deserve a notification. Pure; the states themselves come from Rust (widgets/privacy.rs).
import type { PrivacyKey, PrivacyReport, PrivacyResource, PrivacyState } from '../../lib/ipc';
import type { PrivacySettings } from './settings';

export const KEYS: PrivacyKey[] = ['camera', 'mic', 'location', 'recording', 'sharing'];
export const LABEL: Record<PrivacyKey, string> = {
  camera: 'Camera', mic: 'Microphone', location: 'Location', recording: 'Screen recording', sharing: 'Screen sharing',
};
export const STATE_LABEL: Record<PrivacyState, string> = {
  active: 'In use', inactive: 'Not in use', blocked: 'Blocked', disabled: 'Off', unavailable: 'Unavailable', unknown: 'Unknown', limited: 'Limited',
};
const SHOW: Record<PrivacyKey, keyof PrivacySettings> = {
  camera: 'showCamera', mic: 'showMic', location: 'showLocation', recording: 'showRecording', sharing: 'showSharing',
};

/** The resources this widget shows, in order. */
export const visible = (r: PrivacyReport | null, s: PrivacySettings): PrivacyResource[] =>
  (r?.resources ?? []).filter((x) => s[SHOW[x.key]] !== false);

/** Dock icon: lit while anything shown is in use; a warning when something can't be checked
 *  or a service has a problem (or the report itself failed); otherwise dimmed. */
export type Level = 'active' | 'attention' | 'idle' | 'loading';
export function dockLevel(list: PrivacyResource[], failed: boolean, loaded: boolean): Level {
  if (list.some((r) => r.state === 'active')) return 'active';
  if (failed || list.some((r) => r.attention)) return 'attention';
  return loaded ? 'idle' : 'loading';
}

/** The panel's status line. Never "protected": only what was actually observed. */
export function summary(list: PrivacyResource[], failed: boolean): string {
  const on = list.filter((r) => r.state === 'active');
  if (on.length) return `Privacy activity detected: ${on.map((r) => LABEL[r.key]).join(', ')}`;
  if (failed) return 'Privacy status couldn\'t be read';
  if (list.some((r) => r.attention)) return 'Some statuses unavailable';
  if (list.some((r) => r.state === 'unknown')) return 'No activity detected · some statuses can\'t be verified';
  if (list.some((r) => r.state === 'limited')) return 'No activity detected · screen capture detection is limited';
  return list.length ? 'All monitored resources inactive' : 'Nothing selected to monitor';
}

export interface Note { key: string; title: string; body: string }
const NOTIFY: Record<PrivacyKey, keyof PrivacySettings> = {
  camera: 'notifyCamera', mic: 'notifyMic', location: 'notifyLocation', recording: 'notifyScreen', sharing: 'notifyScreen',
};
const WHAT: Record<PrivacyKey, string> = {
  camera: 'started using the camera', mic: 'started using the microphone', location: 'started using your location',
  recording: 'started capturing your screen', sharing: 'started capturing your screen (likely sharing)',
};
const join = (n: string[]) => (n.length < 2 ? n.join('') : `${n.slice(0, -1).join(', ')} and ${n[n.length - 1]}`);

/** Notifications for what changed between two reports: an app newly using something
 *  (as Windows reports it, not unverified ones), and a resource newly needing a look or
 *  becoming unavailable / turned off. Nothing on the first report, nothing for ongoing use. */
export function transitions(prev: PrivacyReport | null, next: PrivacyReport, s: PrivacySettings): Note[] {
  if (!prev) return [];
  const out: Note[] = [];
  for (const r of visible(next, s)) {
    const p = prev.resources.find((x) => x.key === r.key);
    if (!p) continue;
    const fresh = r.apps.filter((a) => a.verified && !p.apps.some((b) => b.verified && b.id === a.id && b.since === a.since));
    if (fresh.length && s[NOTIFY[r.key]]) {
      out.push({ key: `${r.key}:${fresh.map((a) => `${a.id}@${a.since}`).join(',')}`, title: `${LABEL[r.key]} in use`, body: `${join(fresh.map((a) => a.name))} ${WHAT[r.key]}.` });
    }
    const problem = (x: PrivacyResource) => x.attention || (['camera', 'mic', 'location'].includes(x.key) && ['unavailable', 'disabled'].includes(x.state));
    if (s.notifyProblems && problem(r) && (!problem(p) || p.state !== r.state)) {
      out.push({ key: `${r.key}:${r.state}:${r.headline}`, title: `${LABEL[r.key]}: ${r.headline}`, body: r.reason });
    }
  }
  return out;
}

/** "since 14:05" / "until 14:30" in the viewer's clock format. */
export const clock = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' }).format(ms);
export function ago(ms: number, now: number): string {
  const m = Math.round((now - ms) / 60_000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m} min ago`;
  const h = Math.round(m / 60);
  return h < 24 ? `${h} h ago` : `${Math.round(h / 24)} d ago`;
}
