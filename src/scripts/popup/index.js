import * as api from '../shared/api.js';
import { hydrate, setIcon } from '../shared/icons.js';
import { guessSpokenLanguage, isSpeechAvailable, loadVoices, speak } from '../shared/speech.js';
import { DEFAULT_APPEARANCE, applyAppearance } from '../shared/theme.js';
import { dirBadge, el, out, outputCard, src, status, VIEWS } from './dom.js';
import { DEFAULT_SETTINGS, LANGUAGE_NAMES, state } from './state.js';
import { fail, fitWindow, showView } from './views.js';
import { cycleDirection, translate, updateBadge } from './translate.js';
import { loadHistory, renderHistory } from './history.js';
import { loadSettings, refreshUsage, saveAll } from './settings.js';
import { RECORDERS, isRecording, matchesHotkey, setRecorder, startRecording } from './recorder.js';
import {
  appearanceToControls,
  buildThemeSelect,
  previewAppearance
} from './appearance.js';
import { connectDeepl } from './welcome.js';
import {
  dismissUpdateBanner,
  installUpdate,
  showUpdateBanner,
  syncUpdateBanner
} from './updater.js';
import { wireVoiceInput } from './voice.js';

hydrate();

el('btnTranslate').addEventListener('click', translate);
dirBadge.addEventListener('click', cycleDirection);

src.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    translate();
  }
});

src.addEventListener('input', () => {
  if (state.lastResult && src.value.trim() !== state.lastResult.sourceText) {
    state.lastResult = null;
    updateBadge();
  }
});

el('btnCopy').addEventListener('click', async (event) => {
  const button = event.currentTarget;
  try {
    await api.setClipboard(out.textContent);
  } catch (_) {
    try { await navigator.clipboard.writeText(out.textContent); } catch (_) {}
  }
  button.classList.add('is-success');
  setIcon(button, 'check');
  clearTimeout(button._timer);
  button._timer = setTimeout(() => {
    button.classList.remove('is-success');
    setIcon(button, 'copy');
  }, 900);
});

el('btnOcr').addEventListener('click', async () => {
  await api.hidePopup().catch(() => {});
  api.startRegionCapture().catch(() => {});
});

if (isSpeechAvailable()) {
  loadVoices();
  window.speechSynthesis.addEventListener('voiceschanged', loadVoices);

  el('btnSpeakSrc').addEventListener('click', () => {
    const detected = state.lastResult && state.lastResult.detectedSource;
    speak(src.value.trim(), detected ? detected.toLowerCase().slice(0, 2) : guessSpokenLanguage(src.value));
  });
  el('btnSpeakOut').addEventListener('click', () => {
    speak(out.textContent.trim(), state.lastResult ? state.lastResult.target.toLowerCase() : 'en');
  });
} else {
  el('btnSpeakSrc').hidden = true;
  el('btnSpeakOut').hidden = true;
}

wireVoiceInput();

el('historySearch').addEventListener('input', renderHistory);

el('historyList').addEventListener('click', async (event) => {
  const item = event.target.closest('.history-item');
  if (!item) return;
  const id = item.dataset.id;
  const entry = state.historyEntries.find((candidate) => candidate.id === id);
  if (!entry) return;

  const action = event.target.closest('[data-action]')?.dataset.action;

  if (action === 'pin') {
    state.historyEntries = await api.toggleHistoryPin(id);
    renderHistory();
    return;
  }
  if (action === 'delete') {
    state.historyEntries = await api.deleteHistoryEntry(id);
    renderHistory();
    return;
  }
  if (action === 'copy') {
    await api.setClipboard(entry.translated).catch(() => {});
    return;
  }

  src.value = entry.source;
  out.textContent = entry.translated;
  state.lastResult = {
    text: entry.translated,
    detectedSource: entry.from,
    target: entry.to,
    sourceText: entry.source
  };
  outputCard.hidden = false;
  el('outDir').textContent = LANGUAGE_NAMES[entry.to] || entry.to;
  updateBadge();
  showView('main');
});

el('btnClearHistory').addEventListener('click', async () => {
  state.historyEntries = await api.clearHistory();
  renderHistory();
});

el('btnRefreshUsage').addEventListener('click', refreshUsage);

el('btnRevealKey').addEventListener('click', (event) => {
  const field = el('deeplKey');
  const hidden = field.type === 'password';
  field.type = hidden ? 'text' : 'password';
  setIcon(event.currentTarget, hidden ? 'eye-off' : 'eye');
});

RECORDERS.forEach((id) => {
  el(id).addEventListener('click', () => startRecording(el(id)));
});

['opacity', 'colBg', 'colAccent', 'colText', 'font'].forEach((id) => {
  el(id).addEventListener('input', previewAppearance);
});

buildThemeSelect();

el('btnSaveSettings').addEventListener('click', () => saveAll(el('settingsStatus')));
el('btnSaveShortcuts').addEventListener('click', () => saveAll(el('shortcutsStatus')));
el('btnSaveAppearance').addEventListener('click', () => saveAll(el('appearanceStatus')));

el('btnResetAll').addEventListener('click', () => {
  setRecorder('hotkey', DEFAULT_SETTINGS.hotkey);
  setRecorder('swapHotkey', DEFAULT_SETTINGS.swapHotkey);
  setRecorder('selectionHotkey', DEFAULT_SETTINGS.selectionHotkey);
  setRecorder('replaceHotkey', DEFAULT_SETTINGS.replaceHotkey);
  setRecorder('ocrHotkey', DEFAULT_SETTINGS.ocrHotkey);
  el('autoTranslateClipboard').checked = false;
  el('autostart').checked = true;
  appearanceToControls(DEFAULT_APPEARANCE);
  applyAppearance(DEFAULT_APPEARANCE);
  saveAll(el('settingsStatus'));
});

el('btnConnectDeepl').addEventListener('click', connectDeepl);

el('welcomeKey').addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    connectDeepl();
  }
});

el('btnDeeplAccount').addEventListener('click', async () => {
  try {
    await api.openDeeplSignup();
  } catch (error) {
    fail(el('welcomeStatus'), api.describeError(error));
    fitWindow();
  }
});

el('btnRevealWelcomeKey').addEventListener('click', (event) => {
  const field = el('welcomeKey');
  const hidden = field.type === 'password';
  field.type = hidden ? 'text' : 'password';
  setIcon(event.currentTarget, hidden ? 'eye-off' : 'eye');
});

el('btnSkipWelcome').addEventListener('click', () => {
  state.skippedWelcome = true;
  showView('main');
});

el('btnHome').addEventListener('click', () => showView('main'));
el('btnHistory').addEventListener('click', () => showView(VIEWS.history.hidden ? 'history' : 'main'));
el('btnShortcuts').addEventListener('click', () => showView(VIEWS.shortcuts.hidden ? 'shortcuts' : 'main'));
el('btnSettings').addEventListener('click', () => showView(VIEWS.settings.hidden ? 'settings' : 'main'));
el('btnAppearance').addEventListener('click', () => showView(VIEWS.appearance.hidden ? 'appearance' : 'main'));
el('btnClose').addEventListener('click', () => api.hidePopup());
el('btnBackHistory').addEventListener('click', () => showView('main'));
el('btnBackShortcuts').addEventListener('click', () => showView('main'));
el('btnBackSettings').addEventListener('click', () => showView('main'));
el('btnBackAppearance').addEventListener('click', () => {
  applyAppearance(state.settings.appearance);
  showView('main');
});

document.addEventListener('keydown', (event) => {
  if (isRecording()) return;

  if (matchesHotkey(event, state.settings.swapHotkey)) {
    event.preventDefault();
    cycleDirection();
    return;
  }
  if (event.key === 'Escape') {
    if (VIEWS.main.hidden && VIEWS.welcome.hidden) showView('main');
    else api.hidePopup();
  }
});

el('btnDismissUpdate').addEventListener('click', dismissUpdateBanner);

el('btnInstallUpdate').addEventListener('click', (event) => installUpdate(event.currentTarget));

api.listen('update-available', (event) => showUpdateBanner(event.payload));

api.listen('popup-shown', (event) => {
  syncUpdateBanner();
  showView('main');
  const clipboard = (event.payload && event.payload.clipboard) || '';
  if (clipboard.trim()) {
    src.value = clipboard;
    state.lastResult = null;
    translate();
  }
});

api.listen('open-view', (event) => showView(String(event.payload || 'main')));

loadSettings();
syncUpdateBanner();
