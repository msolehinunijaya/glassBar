import { expect, it } from 'vitest';
import { stateOf } from './index';

it('names every Pomodoro state and the button that moves on', () => {
  const s = (o: object) => stateOf({ phase: 'work', endsAt: null, pausedMs: null, completed: null, ...o });
  expect(s({})).toMatchObject({ key: 'ready', action: 'Start Work Session' });
  expect(s({ endsAt: 1 })).toMatchObject({ key: 'running', title: 'Working', action: 'Pause' });
  expect(s({ pausedMs: 5 })).toMatchObject({ key: 'paused', action: 'Resume' });
  expect(s({ phase: 'short', completed: 'work' })).toMatchObject({ title: 'Work Session Complete', action: 'Start Break' });
  expect(s({ phase: 'long', completed: 'work' })).toMatchObject({ action: 'Start Long Break' });
  expect(s({ phase: 'work', completed: 'short' })).toMatchObject({ title: 'Break Complete', action: 'Start Work Session' });
});
