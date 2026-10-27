import { describe, expect, it } from 'vitest';
import { MAX, add, extOf, hits, nameOf, remove, removeAll } from './shelf';

const P = (n: number) => `C:\\Users\\me\\file${n}.txt`;
const five = [P(1), P(2), P(3), P(4), P(5)];

describe('add', () => {
  it('puts the newest file at the front', () => {
    expect(add([P(1)], [P(2)])).toEqual([P(2), P(1)]);
  });

  it('adds several at once, keeping the order they were dropped in', () => {
    expect(add([], [P(1), P(2), P(3)])).toEqual([P(1), P(2), P(3)]);
  });

  it('lets the oldest fall off at five', () => {
    const r = add(five, [P(9)]);
    expect(r).toHaveLength(MAX);
    expect(r[0]).toBe(P(9));
    expect(r).not.toContain(P(5));
    expect(r).toContain(P(1));
  });

  it('keeps only the first five of a big drop', () => {
    const r = add([], [P(1), P(2), P(3), P(4), P(5), P(6), P(7)]);
    expect(r).toEqual([P(1), P(2), P(3), P(4), P(5)]);
  });

  it('moves a file already on the shelf to the front instead of doubling it', () => {
    const r = add([P(1), P(2), P(3)], [P(3)]);
    expect(r).toEqual([P(3), P(1), P(2)]);
  });

  it('treats Windows paths as case-insensitive', () => {
    expect(add([P(1)], ['c:\\users\\me\\FILE1.TXT'])).toHaveLength(1);
  });

  it('drops blanks and trims stray spaces', () => {
    expect(add([], ['  ', '', `  ${P(1)}  `])).toEqual([P(1)]);
  });

  it('de-duplicates within one drop', () => {
    expect(add([], [P(1), P(1)])).toEqual([P(1)]);
  });
});

describe('remove', () => {
  it('takes one file off, whatever its case', () => {
    expect(remove(five, 'C:\\USERS\\ME\\FILE3.TXT')).toEqual([P(1), P(2), P(4), P(5)]);
  });

  it('leaves the shelf alone when the file is not on it', () => {
    expect(remove(five, P(9))).toEqual(five);
  });

  it('takes several off at once, the way a finished drag reports back', () => {
    expect(removeAll(five, [P(2), P(4)])).toEqual([P(1), P(3), P(5)]);
    expect(removeAll(five, [])).toEqual(five);
  });
});

describe('nameOf and extOf', () => {
  it('reads the file name off a Windows or a Unix path', () => {
    expect(nameOf('C:\\Users\\me\\Report Q4.pdf')).toBe('Report Q4.pdf');
    expect(nameOf('/home/me/notes.md')).toBe('notes.md');
    expect(nameOf('C:\\Users\\me\\folder\\')).toBe('folder');
    expect(nameOf('plain.txt')).toBe('plain.txt');
  });

  it('reads the extension, and admits when there is none', () => {
    expect(extOf('C:\\a\\Report.PDF')).toBe('PDF');
    expect(extOf('C:\\a\\archive.tar.gz')).toBe('GZ');
    expect(extOf('C:\\a\\Makefile')).toBe('');
    expect(extOf('C:\\a\\.gitignore')).toBe('');
  });
});

describe('hits', () => {
  const box = { left: 100, top: 10, right: 200, bottom: 40 };

  it('accepts a drop inside the shelf at 100 %', () => {
    expect(hits(box, 150, 25, 1)).toBe(true);
    expect(hits(box, 100, 10, 1)).toBe(true);
    expect(hits(box, 200, 40, 1)).toBe(true);
  });

  it('refuses a drop outside it', () => {
    expect(hits(box, 99, 25, 1)).toBe(false);
    expect(hits(box, 201, 25, 1)).toBe(false);
    expect(hits(box, 150, 9, 1)).toBe(false);
    expect(hits(box, 150, 41, 1)).toBe(false);
  });

  it('converts the physical pixels Tauri reports at 125 % and 150 %', () => {
    // The pointer is over the middle of the shelf: 150 CSS px is 187.5 physical at 125 %.
    expect(hits(box, 187.5, 31.25, 1.25)).toBe(true);
    expect(hits(box, 225, 37.5, 1.5)).toBe(true);
    // The same physical number, read as CSS px, would land outside the box.
    expect(hits(box, 225, 37.5, 1)).toBe(false);
  });

  it('survives a nonsense ratio rather than dropping everything', () => {
    expect(hits(box, 150, 25, 0)).toBe(true);
  });
});
