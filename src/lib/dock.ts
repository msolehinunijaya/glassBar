// Dock windows: the main dock is "bar"; with "All displays" the copies are "bar-<monitor>".
export const isDockWindow = (label: string) => label === 'bar' || label.startsWith('bar-');
/** Only the main dock plays sounds and shows alerts. */
export const isMainDock = (label: string) => label === 'bar';
