import * as api from '../shared/api.js';
import { DEFAULT_APPEARANCE, applyAppearance } from '../shared/theme.js';
import { el, status } from './dom.js';
import { DEFAULT_SETTINGS, state } from './state.js';
import { appearanceFromControls, appearanceToControls } from './appearance.js';
import { setRecorder } from './recorder.js';
import { updateBadge } from './translate.js';
import { fail, fitWindow, needsWelcome, showView, toast } from './views.js';

function format(value) {
  if (value >= 1000000) return (value / 1000000).toFixed(1) + 'M';
  if (value >= 1000) return Math.round(value / 1000) + 'k';
  return String(value);
}

export async function refreshUsage() {
  const box = el('usage');
  if (!el('deeplKey').value.trim()) {
    box.hidden = true;
    return;
  }
  try {
    const usage = await api.getUsage();
    const percent = usage.characterLimit ? (usage.characterCount / usage.characterLimit) * 100 : 0;
    el('usageFill').style.width = Math.min(percent, 100) + '%';
    el('usageFill').classList.toggle('is-high', percent >= 85);
    el('usageText').textContent =
      `${format(usage.characterCount)} / ${format(usage.characterLimit)} characters this month`;
    box.hidden = false;
  } catch (_) {
    box.hidden = true;
  }
  fitWindow();
}

export async function loadSettings() {
  state.settings = { ...DEFAULT_SETTINGS, ...(await api.getSettings()) };
  if (!state.settings.appearance) state.settings.appearance = { ...DEFAULT_APPEARANCE };

  setRecorder('hotkey', state.settings.hotkey);
  setRecorder('swapHotkey', state.settings.swapHotkey);
  setRecorder('selectionHotkey', state.settings.selectionHotkey);
  setRecorder('replaceHotkey', state.settings.replaceHotkey);
  setRecorder('ocrHotkey', state.settings.ocrHotkey);

  el('deeplKey').value = state.settings.deeplKey || '';
  el('autoTranslateClipboard').checked = state.settings.autoTranslateClipboard === true;
  try { el('autostart').checked = await api.getAutostart(); } catch (_) {}
  try { el('appVersion').textContent = `Lexo ${await api.appVersion()}`; } catch (_) {}

  appearanceToControls(state.settings.appearance);
  applyAppearance(state.settings.appearance);

  try {
    if (!(await api.ocrAvailable())) {
      el('btnOcr').hidden = true;
      el('ocrHotkeyField').hidden = true;
    }
  } catch (_) {}

  updateBadge();
  if (needsWelcome()) showView('welcome');
  fitWindow();
}

export async function saveAll(statusNode) {
  const updated = {
    hotkey: el('hotkey').dataset.value,
    swapHotkey: el('swapHotkey').dataset.value,
    selectionHotkey: el('selectionHotkey').dataset.value,
    replaceHotkey: el('replaceHotkey').dataset.value,
    ocrHotkey: el('ocrHotkey').dataset.value,
    autoTranslateClipboard: el('autoTranslateClipboard').checked,
    deeplKey: el('deeplKey').value.trim(),
    appearance: appearanceFromControls()
  };

  try {
    await api.saveSettings(updated);
    try { await api.setAutostart(el('autostart').checked); } catch (_) {}

    state.settings = { ...state.settings, ...updated };
    applyAppearance(state.settings.appearance);
    updateBadge();

    try {
      await api.updateShortcuts();
    } catch (error) {
      fail(statusNode, 'Saved, but a shortcut could not be registered: ' + api.describeError(error));
      return;
    }

    showView('main');
    toast(status, 'Saved!');
  } catch (error) {
    fail(statusNode, 'Error saving: ' + api.describeError(error));
  }
}
