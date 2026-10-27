// The Windows logo's four squares, top-left, top-right, bottom-left, bottom-right.
export type LogoColor = 'blue' | 'accent' | 'text' | 'classic' | 'custom';

export function logoFills(c: LogoColor | undefined, custom = '#0a84ff'): [string, string, string, string] {
  switch (c) {
    case 'accent': return Array(4).fill('var(--accent)') as [string, string, string, string];
    case 'text': return Array(4).fill('currentColor') as [string, string, string, string];
    case 'classic': return ['#f25022', '#7fba00', '#00a4ef', '#ffb900'];
    case 'custom': return Array(4).fill(/^#[0-9a-f]{3,8}$/i.test(custom) ? custom : '#0a84ff') as [string, string, string, string];
    default: return Array(4).fill('#0a84ff') as [string, string, string, string];
  }
}
