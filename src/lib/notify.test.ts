import { describe, expect, it } from 'vitest';
import { dismiss, expire, jumpTarget, nextDeadline, pause, push, resume, shown, type Banner, type Note } from './notify';

const n = (id: number, source = 'timer'): Note => ({ id, source, title: `t${id}`, body: '', at: 0, data: null });

describe('banner queue', () => {
  it('shows at most 3, newest on top, with "+N more"', () => {
    let q: Banner[] = [];
    for (let i = 1; i <= 5; i++) q = push(q, n(i), 0, 5000);
    const s = shown(q);
    expect(s.list.map((b) => b.id)).toEqual([5, 4, 3]);
    expect(s.more).toBe(2);
  });
  it('one notification, one banner (no duplicates)', () => {
    let q = push([], n(1), 0, 5000);
    q = push(q, n(1), 10, 5000);
    expect(q).toHaveLength(1);
  });
  it('times out', () => {
    let q = push([], n(1), 0, 5000);
    expect(nextDeadline(q)).toBe(5000);
    expect(expire(q, 4999, 5000)).toHaveLength(1);
    expect(expire(q, 5000, 5000)).toHaveLength(0);
  });
  it('hover pauses the countdown and leaving resumes it', () => {
    let q = push([], n(1), 0, 5000);
    q = pause(q, 2000);                       // 3 s left
    expect(nextDeadline(q)).toBeNull();        // no timer while paused
    expect(expire(q, 60_000, 5000)).toHaveLength(1);
    q = resume(q, 60_000);
    expect(q[0].until).toBe(63_000);
  });
  it('a waiting banner gets its full time when it comes up', () => {
    let q: Banner[] = [];
    for (let i = 1; i <= 4; i++) q = push(q, n(i), 0, 5000); // 1 waits
    expect(q[3].waiting).toBe(true);
    q = expire(q, 6000, 5000);                 // 4, 3, 2 time out; 1 comes up now with its full time
    expect(q.map((b) => b.id)).toEqual([1]);
    expect(q[0].until).toBe(11_000);
    q = push(push(push(push([], n(1), 0, 5000), n(2), 0, 5000), n(3), 0, 5000), n(4), 0, 5000);
    q = dismiss(q, 4, 2000, 5000);             // closing one brings the waiting one up
    expect(q.find((b) => b.id === 1)).toMatchObject({ waiting: false, until: 7000 });
  });
});

describe('jumpTarget', () => {
  const bar = [
    { id: 'clock', instance: 'clock-1' },
    { id: 'productivity', instance: 'productivity-1', settings: { showPomo: true, showTodo: false } },
    { id: 'timer', instance: 'timer-1' },
    { id: 'notifications', instance: 'notifications-1' },
  ];
  it('goes to the widget that sent it (even far along the bar)', () => {
    expect(jumpTarget('timer', bar)).toEqual({ kind: 'widget', instance: 'timer-1', index: 2, part: undefined });
    expect(jumpTarget('pomodoro', bar)).toEqual({ kind: 'widget', instance: 'productivity-1', index: 1, part: 'pomodoro' });
  });
  it('widget removed (or part hidden): the Notification Center instead', () => {
    expect(jumpTarget('solat', bar)).toEqual({ kind: 'center', instance: 'notifications-1', index: 3 });
    expect(jumpTarget('todo', bar)).toEqual({ kind: 'center', instance: 'notifications-1', index: 3 });
    expect(jumpTarget('update', bar).kind).toBe('center');
  });
  it('nowhere to go', () => expect(jumpTarget('solat', [{ id: 'clock', instance: 'c' }])).toEqual({ kind: 'none' }));
});
