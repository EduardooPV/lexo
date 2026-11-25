import * as api from '../shared/api.js';
import { LANGUAGES, baseOf } from '../shared/languages.js';
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

const QUOTA_WARNING_PERCENT = 80;

function usagePercent(usage) {
  return usage.characterLimit > 0 ? (usage.characterCount / usage.characterLimit) * 100 : 0;
}

export function renderUsageHint(usage) {
  const hint = el('usageHint');
  const visible = Boolean(state.settings.deeplKey?.trim() && usage?.characterLimit > 0);
  const percent = visible ? usagePercent(usage) : 0;
  hint.hidden = !visible;
  hint.textContent = visible ? `${Math.floor(percent)}% of your monthly quota used` : '';
  hint.classList.toggle('is-high', visible && percent >= QUOTA_WARNING_PERCENT);
  fitWindow();
}

export function buildLanguageSelects() {
  const languages = [...LANGUAGES].sort((a, b) => a.name.localeCompare(b.name, 'en'));
  for (const id of ['primaryLang', 'secondaryLang', 'targetSelect']) {
    el(id).append(...languages.map(({ code, name }) => new Option(name, code)));
  }
  applyLanguageSettings(DEFAULT_SETTINGS);
}

export function applyLanguageSettings(settings) {
  el('primaryLang').value = settings.primaryLang;
  el('secondaryLang').value = settings.secondaryLang;
  el('formality').value = settings.formality;
}

export async function refreshUsage() {
  const box = el('usage');
  if (!el('deeplKey').value.trim()) {
    box.hidden = true;
    renderUsageHint(null);
    return;
  }
  try {
    const usage = await api.getUsage();
    const percent = usagePercent(usage);
    const fill = el('usageFill');
    const progress = fill.parentElement;
    fill.style.width = Math.min(percent, 100) + '%';
    fill.classList.toggle('is-high', percent >= QUOTA_WARNING_PERCENT);
    progress.setAttribute('aria-valuemin', '0');
    progress.setAttribute('aria-valuemax', String(usage.characterLimit || 100));
    progress.setAttribute('aria-valuenow', String(Math.min(usage.characterCount, usage.characterLimit)));
    el('usageText').textContent =
      `${format(usage.characterCount)} / ${format(usage.characterLimit)} characters this month`;
    box.hidden = false;
    renderUsageHint(usage);
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
  el('keepHistory').checked = state.settings.keepHistory !== false;
  applyLanguageSettings(state.settings);
  try { el('autostart').checked = await api.getAutostart(); } catch (_) {}
  try { el('appVersion').textContent = `Lexo ${await api.appVersion()}`; } catch (_) {}

  appearanceToControls(state.settings.appearance);
  applyAppearance(state.settings.appearance);
  if ((state.settings.appearance.zoom || 1) !== 1) {
    api.setZoom(state.settings.appearance.zoom).catch(() => {});
  }

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
  const languageStatus = el('settingsStatus');
  const primaryLang = el('primaryLang').value;
  const secondaryLang = el('secondaryLang').value;
  if (!primaryLang || !secondaryLang || baseOf(primaryLang) === baseOf(secondaryLang)) {
    if (statusNode !== languageStatus) showView('settings');
    fail(languageStatus, 'Choose two different languages.');
    return;
  }
  const updated = {
    hotkey: el('hotkey').dataset.value,
    swapHotkey: el('swapHotkey').dataset.value,
    selectionHotkey: el('selectionHotkey').dataset.value,
    replaceHotkey: el('replaceHotkey').dataset.value,
    ocrHotkey: el('ocrHotkey').dataset.value,
    autoTranslateClipboard: el('autoTranslateClipboard').checked,
    keepHistory: el('keepHistory').checked,
    deeplKey: el('deeplKey').value.trim(),
    primaryLang,
    secondaryLang,
    formality: el('formality').value,
    appearance: appearanceFromControls()
  };

  try {
    const warning = await api.saveSettings(updated);
    try { await api.setAutostart(el('autostart').checked); } catch (_) {}

    const keyChanged = updated.deeplKey !== state.settings.deeplKey;
    state.settings = { ...state.settings, ...updated };
    if (keyChanged) renderUsageHint(null);
    applyAppearance(state.settings.appearance);
    updateBadge();

    try {
      await api.updateShortcuts();
    } catch (error) {
      const shortcutError = 'Saved, but a shortcut could not be registered: ' + api.describeError(error);
      fail(statusNode, warning ? shortcutError + ' ' + warning : shortcutError);
      return;
    }

    showView('main');
    const destination = needsWelcome() ? el('welcomeStatus') : status;
    toast(destination, warning || 'Saved!', warning ? 6000 : undefined);
  } catch (error) {
    fail(statusNode, 'Error saving: ' + api.describeError(error));
  }
}
