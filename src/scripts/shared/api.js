const { invoke } = window.__TAURI__.core;

export const listen = window.__TAURI__.event.listen;

export const getSettings = () => invoke('get_settings');
export const saveSettings = (settings) => invoke('save_settings', { settings });
export const updateShortcuts = () => invoke('update_shortcuts');

export const translate = (text, target = null, confirmed = false) =>
  invoke('translate', { text, target, confirmed });
export const getUsage = () => invoke('get_usage');
export const verifyDeeplKey = (key) => invoke('verify_deepl_key', { key });
export const openDeeplSignup = () => invoke('open_deepl_signup');

export const getHistory = () => invoke('get_history');
export const clearHistory = () => invoke('clear_history');
export const deleteHistoryEntry = (id) => invoke('delete_history_entry', { id });
export const toggleHistoryPin = (id) => invoke('toggle_history_pin', { id });
export const exportHistory = (format) => invoke('export_history', { format });

export const setClipboard = (text) => invoke('set_clipboard', { text });
export const hidePopup = () => invoke('hide_popup');
export const hideMini = () => invoke('hide_mini');
export const resizeWindow = (height) => invoke('resize_window', { height });
export const setZoom = (factor) => invoke('set_zoom', { factor });
export const resizeMini = (height) => invoke('resize_mini', { height });

export const setAutostart = (enabled) => invoke('set_autostart', { enabled });
export const getAutostart = () => invoke('get_autostart');

export const pendingUpdate = () => invoke('pending_update');
export const installUpdate = () => invoke('install_update');
export const appVersion = () => invoke('app_version');

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
  ['open_error', 'Could not open the browser. Go to deepl.com/pro-api manually.'],
  ['updater_none', 'You are already on the latest version.'],
  ['updater_error', 'Could not reach the update server. Try again later.'],
  ['api_error', "Couldn't translate that text. Try rephrasing."],
  ['decode_error', 'Unexpected response from DeepL.'],
  ['too_long', 'This text has {N} characters and counts toward your DeepL quota. Translate anyway?'],
  ['too_large', 'That text is too long for a single request. Split it into smaller parts.'],
  ['empty', 'Nothing to translate.']
];

export function longTextLength(error) {
  const match = String(error ?? '').match(/^too_long:\s*(\d+)$/);
  return match ? Number(match[1]) : null;
}

export function describeError(error) {
  const raw = String(error ?? '');

  const length = longTextLength(raw);
  for (const [prefix, message] of MESSAGES) {
    if (prefix === 'too_long') {
      if (length !== null) return message.replace('{N}', String(length));
      continue;
    }
    if (raw.includes(prefix)) return message;
  }
  if (raw.includes('limit:')) return raw.replace(/^.*limit:\s*/, '');
  if (raw.includes('network_error') || raw.includes('http_error')) {
    return 'Connection to DeepL failed. ' + raw.replace(/^.*(network_error|http_error):\s*/, '');
  }
  return raw.replace(/^[a-z_]+:\s*/, '');
}
