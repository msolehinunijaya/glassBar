// Volume is the middle segment of the Quick Settings pill (it was its own widget). Its
// settings live in the Quick Settings widget under `sound`.

/** 'vertical' was an older look (a small upright slider); it now shows as 'horizontal'. */
export type SoundLayout = 'icon' | 'horizontal' | 'vertical';
/** `showSlider` is the older on/off setting; kept so existing bars keep their slider.
 *  `sliderWidth`: length in px of the horizontal slider on the bar. */
export interface SoundSettings { step: number; layout: SoundLayout; wheel: boolean; sliderWidth: number; showSlider?: boolean }

export const SOUND_DEFAULTS: SoundSettings = { step: 2, layout: 'horizontal', wheel: false, sliderWidth: 90 };

/** The sound settings stored in a Quick Settings widget's settings. */
export const soundOf = (qs: { sound?: Partial<SoundSettings> } | null | undefined): SoundSettings => ({ ...SOUND_DEFAULTS, ...qs?.sound });

export const layoutOf = (s: SoundSettings): 'icon' | 'horizontal' => (s.layout === 'vertical' || (s.layout === 'icon' && s.showSlider) ? 'horizontal' : s.layout);

/** Popup id of the sound panel, opened from the volume segment of Quick Settings `qs`. */
export const SOUND_PANEL = '__sound:';
/** Popup id of the battery panel, opened from the battery segment of Quick Settings. */
export const BATTERY_PANEL = '__battery:';
