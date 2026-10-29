/** A tile's status line: the one thing connected, the first and "+N others", or `none`. */
export function connectedText(names: string[], none: string): string {
  names = [...new Set(names)]; // a device can be listed as classic and LE Bluetooth
  if (!names.length) return none;
  const more = names.length - 1;
  return more ? `${names[0]} +${more} ${more === 1 ? 'other' : 'others'}` : names[0];
}
