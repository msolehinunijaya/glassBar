/** Privacy Monitor options. show*: which cards the panel lists (and the dock icon counts).
 *  glow: the dock icon glows while something is in use. tone: the lit colour, the theme's
 *  accent or a colour per resource (camera green, microphone orange…). details: cards open
 *  by default. history: Windows' record of the last 7 days on each card. notify*: a
 *  notification when an app starts using it; notifyProblems: when something can't be
 *  checked any more, a device becomes unavailable or location is turned off. */
export interface PrivacySettings {
  showCamera: boolean; showMic: boolean; showLocation: boolean; showRecording: boolean; showSharing: boolean;
  glow: boolean; tone: 'status' | 'accent'; details: boolean; history: boolean;
  notifyCamera: boolean; notifyMic: boolean; notifyLocation: boolean; notifyScreen: boolean; notifyProblems: boolean;
}

export const PRIVACY_DEFAULTS: PrivacySettings = {
  showCamera: true, showMic: true, showLocation: true, showRecording: true, showSharing: true,
  glow: true, tone: 'status', details: false, history: true,
  notifyCamera: true, notifyMic: false, notifyLocation: false, notifyScreen: false, notifyProblems: true,
};
