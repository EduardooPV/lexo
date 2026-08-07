import { DEFAULT_APPEARANCE } from '../shared/theme.js';

export const DEFAULT_SETTINGS = {
  hotkey: 'Alt+R',
  swapHotkey: 'Alt+E',
  selectionHotkey: 'Alt+T',
  replaceHotkey: 'Alt+Shift+T',
  ocrHotkey: 'Alt+S',
  autoTranslateClipboard: false,
  deeplKey: '',
  appearance: { ...DEFAULT_APPEARANCE }
};

export const LANGUAGE_NAMES = { PT: 'Portuguese', EN: 'English' };

export const state = {
  settings: { ...DEFAULT_SETTINGS },
  forcedTarget: null,
  lastResult: null,
  historyEntries: [],
  updateDismissed: false,
  skippedWelcome: false
};
