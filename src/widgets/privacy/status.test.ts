import { describe, expect, it } from 'vitest';
import type { PrivacyApp, PrivacyReport, PrivacyResource } from '../../lib/ipc';
import { PRIVACY_DEFAULTS } from './settings';
import { dockLevel, summary, transitions, visible } from './status';

const app = (id: string, since = 1, verified = true): PrivacyApp => ({ id, name: id, exe: '', since, until: 0, verified, window: null });
const res = (key: PrivacyResource['key'], state: PrivacyResource['state'], over: Partial<PrivacyResource> = {}): PrivacyResource =>
  ({ key, state, headline: 'h', reason: 'r', attention: false, apps: [], recent: [], devices: [], facts: [], method: '', ...over });
const report = (...resources: PrivacyResource[]): PrivacyReport => ({ resources, checked: 0, micMuted: false });
const quiet = () => report(res('camera', 'inactive'), res('mic', 'inactive'), res('location', 'inactive'), res('recording', 'limited'), res('sharing', 'limited'));

describe('privacy status', () => {
  it('lights the dock only for observed use, warns for unchecked, never claims protection', () => {
    const q = quiet().resources;
    expect(dockLevel(q, false, true)).toBe('idle');
    expect(summary(q, false)).toBe('No activity detected · screen capture detection is limited');
    expect(summary(q.slice(0, 3), false)).toBe('All monitored resources inactive');
    const on = [res('camera', 'active'), ...q.slice(1)];
    expect(dockLevel(on, false, true)).toBe('active');
    expect(summary(on, false)).toBe('Privacy activity detected: Camera');
    const unk = [res('mic', 'unknown', { attention: true })];
    expect(dockLevel(unk, false, true)).toBe('attention');
    expect(summary(unk, false)).toBe('Some statuses unavailable');
    // A report that failed is a warning, not "idle".
    expect(dockLevel([], true, true)).toBe('attention');
    expect(dockLevel([], false, false)).toBe('loading');
  });

  it('hides resources switched off in settings', () => {
    expect(visible(quiet(), { ...PRIVACY_DEFAULTS, showLocation: false }).map((r) => r.key)).toEqual(['camera', 'mic', 'recording', 'sharing']);
  });

  it('notifies once when an app starts, not for ongoing use or unverified apps', () => {
    const s = PRIVACY_DEFAULTS; // camera + problems on, mic off
    expect(transitions(null, quiet(), s)).toEqual([]);
    const cam = report(res('camera', 'active', { apps: [app('Zoom')] }), res('mic', 'active', { apps: [app('Zoom')] }));
    const n = transitions(report(res('camera', 'inactive'), res('mic', 'inactive')), cam, s);
    expect(n).toEqual([{ key: 'camera:Zoom@1', title: 'Camera in use', body: 'Zoom started using the camera.' }]);
    expect(transitions(cam, cam, s)).toEqual([]);
    const obs = report(res('recording', 'unknown', { apps: [app('obs64.exe', 0, false)] }));
    expect(transitions(report(res('recording', 'limited')), obs, { ...s, notifyScreen: true })).toEqual([]);
  });

  it('notifies about problems when they start', () => {
    const off = report(res('location', 'unavailable', { headline: 'Location service is disabled', attention: true }));
    const n = transitions(report(res('location', 'inactive')), off, PRIVACY_DEFAULTS);
    expect(n.map((x) => x.title)).toEqual(['Location: Location service is disabled']);
    expect(transitions(off, off, PRIVACY_DEFAULTS)).toEqual([]);
    expect(transitions(report(res('location', 'inactive')), off, { ...PRIVACY_DEFAULTS, notifyProblems: false })).toEqual([]);
  });
});
