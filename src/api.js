const { invoke } = window.__TAURI__.core;

export const listen = window.__TAURI__.event.listen;

export const getSettings = () => invoke('get_settings');
export const saveSettings = (settings) => invoke('save_settings', { settings });
export const updateShortcuts = () => invoke('update_shortcuts');
export const getShortcutsPaused = () => invoke('get_shortcuts_paused');
export const setShortcutsPaused = (paused) => invoke('set_shortcuts_paused', { paused });

export const translate = (text, target = null) => invoke('translate', { text, target });
export const getUsage = () => invoke('get_usage');

export const getHistory = () => invoke('get_history');
export const clearHistory = () => invoke('clear_history');
export const deleteHistoryEntry = (id) => invoke('delete_history_entry', { id });
export const toggleHistoryPin = (id) => invoke('toggle_history_pin', { id });

export const setClipboard = (text) => invoke('set_clipboard', { text });
export const hidePopup = () => invoke('hide_popup');
export const hideMini = () => invoke('hide_mini');
export const resizeWindow = (height) => invoke('resize_window', { height });
export const resizeMini = (height) => invoke('resize_mini', { height });

export const setAutostart = (enabled) => invoke('set_autostart', { enabled });
export const getAutostart = () => invoke('get_autostart');

export const ocrAvailable = () => invoke('ocr_available');
export const ocrRegion = (rect) => invoke('ocr_region', rect);
export const startRegionCapture = () => invoke('start_region_capture');
export const cancelRegionCapture = () => invoke('cancel_region_capture');

const MESSAGES = [
  ['no_key', 'Add your DeepL API key in Settings to translate.'],
  ['deepl_auth', 'DeepL rejected the key. Check it in Settings.'],
  ['region_too_small', 'That region is too small — drag a larger area.'],
  ['ocr_unavailable', 'Windows has no OCR language pack installed. Add English or Portuguese under Settings › Time & language.'],
  ['ocr_unsupported', 'Screen OCR is only available on Windows.'],
  ['capture_error', 'Could not read that screen region.'],
  ['ocr_error', 'Could not read the text in that region.'],
  ['api_error', "Couldn't translate that text. Try rephrasing."],
  ['decode_error', 'Unexpected response from DeepL.'],
  ['empty', 'Nothing to translate.']
];

export function describeError(error) {
  const raw = String(error ?? '');

  for (const [prefix, message] of MESSAGES) {
    if (raw.includes(prefix)) return message;
  }
  if (raw.includes('limit:')) return raw.replace(/^.*limit:\s*/, '');
  if (raw.includes('network_error') || raw.includes('http_error')) {
    return 'Connection to DeepL failed. ' + raw.replace(/^.*(network_error|http_error):\s*/, '');
  }
  return raw.replace(/^[a-z_]+:\s*/, '');
}
