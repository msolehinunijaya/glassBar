import { describe, expect, it } from 'vitest';
import { matches, pick, textsOf } from './search-match';

const weather = textsOf('clock', 'Clock & Date', 'Time and date in your format, calendar, world clocks with day/night, weather', {});
const music = textsOf('music', 'Music', 'Now playing from Spotify, browsers and any media app', {});
const sysmon = textsOf('sysmon', 'System Monitor', 'CPU, RAM, network and disk storage with live graphs', {});
const apps = textsOf('apps', 'App Launcher', 'Pinned and running apps', { pinned: [{ path: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe' }, { path: 'x', name: 'WhatsApp' }] });

describe('widget search', () => {
  it('matches the name, its start and related words', () => {
    expect(matches('music', music)).toBe(true);
    expect(matches('mus', music)).toBe(true);
    expect(matches('song', music)).toBe(true);
    expect(matches('lagu', music)).toBe(true);
    expect(matches('ram', sysmon)).toBe(true);
    expect(matches('calendar', weather)).toBe(true);
  });
  it('forgives a small typo', () => {
    expect(matches('muzic', music)).toBe(true);
    expect(matches('calender', weather)).toBe(true);
    expect(matches('sistem monitr', sysmon)).toBe(true);
  });
  it('finds the launcher by the apps pinned in it', () => {
    expect(matches('chrome', apps)).toBe(true);
    expect(matches('whats', apps)).toBe(true);
  });
  it('uses typos only when nothing matches as typed', () => {
    const solat = { id: 'solat', t: textsOf('solat', 'Waktu Solat', 'Prayer times', {}) };
    const folders = { id: 'folders', t: textsOf('folders', 'App Folders', '', { folders: [{ name: 'Tools', apps: [{ path: 'C:/Tools/Solar.exe' }] }] }) };
    const all = [solat, folders];
    expect(pick('solat', all, (x) => x.t).map((x) => x.id)).toEqual(['solat']);
    expect(pick('solaat', all, (x) => x.t).map((x) => x.id)).toEqual(['solat']);
    expect(pick('solar', all, (x) => x.t).map((x) => x.id)).toEqual(['folders']);
  });
  it("doesn't match what isn't there", () => {
    expect(matches('music', sysmon)).toBe(false);
    expect(matches('2+3', music)).toBe(false);
    expect(matches('', music)).toBe(false);
    expect(matches('xyzzy', music)).toBe(false);
    // Short words only match the start of a word, not by typo.
    expect(matches('cp', music)).toBe(false);
  });
});
