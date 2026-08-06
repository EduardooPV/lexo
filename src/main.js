import * as api from './api.js';
import { hydrate, setIcon } from './icons.js';
import { DEFAULT_APPEARANCE, THEMES, applyAppearance } from './theme.js';

const el = (id) => document.getElementById(id);

const src = el('src');
const out = el('out');
const outputCard = el('outputCard');
const status = el('status');
const dirBadge = el('dirBadge');
const dirLabel = el('dirLabel');

const VIEWS = {
  main: el('mainView'),
  history: el('historyView'),
  settings: el('settingsView'),
  appearance: el('appearanceView')
};

const DEFAULT_SETTINGS = {
  hotkey: 'Alt+R',
  swapHotkey: 'Alt+E',
  selectionHotkey: 'Alt+T',
  replaceHotkey: 'Alt+Shift+T',
  ocrHotkey: 'Alt+S',
  autoTranslateClipboard: false,
  deeplKey: '',
  appearance: { ...DEFAULT_APPEARANCE }
};

let settings = { ...DEFAULT_SETTINGS };
let forcedTarget = null;
let lastResult = null;
let historyEntries = [];
let updateDismissed = false;

hydrate();

function fitWindow() {
  requestAnimationFrame(() => {
    api.resizeWindow(Math.ceil(el('app').scrollHeight)).catch(() => {});
  });
}

function toast(node, text, ms = 2200) {
  node.classList.remove('error');
  node.textContent = text;
  clearTimeout(node._timer);
  node._timer = setTimeout(() => { node.textContent = ''; }, ms);
}

function fail(node, text) {
  node.classList.add('error');
  node.textContent = text;
}

function showView(name) {
  Object.entries(VIEWS).forEach(([key, node]) => { node.hidden = key !== name; });

  if (name === 'main') {
    src.focus();
    src.select();
  }
  if (name === 'history') {
    el('historySearch').value = '';
    loadHistory();
  }
  if (name === 'settings') {
    el('settingsStatus').textContent = '';
    refreshUsage();
  }
  if (name === 'appearance') {
    el('appearanceStatus').textContent = '';
    appearanceToControls(settings.appearance);
    applyAppearance(settings.appearance);
    syncThemeSelect();
  }
  fitWindow();
}

function guessSpokenLanguage(text) {
  return /[ãõáàâêôçéíóú]/i.test(text || '') ? 'pt' : 'en';
}

const LANGUAGE_NAMES = { PT: 'Portuguese', EN: 'English' };

function updateBadge() {
  if (forcedTarget) {
    dirLabel.textContent = forcedTarget === 'EN' ? 'PT → EN' : 'EN → PT';
    dirBadge.title = `Pinned direction — click or press ${settings.swapHotkey} to change`;
  } else if (lastResult) {
    dirLabel.textContent = `${lastResult.detectedSource || '?'} → ${lastResult.target}`;
    dirBadge.title = `Detected automatically — click or press ${settings.swapHotkey} to pin a direction`;
  } else {
    dirLabel.textContent = 'Auto';
    dirBadge.title = `Detected automatically — click or press ${settings.swapHotkey} to pin a direction`;
  }
  dirBadge.classList.toggle('is-forced', Boolean(forcedTarget));
}

function cycleDirection() {
  const shown = forcedTarget || (lastResult && lastResult.target) || 'EN';
  forcedTarget = shown === 'EN' ? 'PT' : 'EN';
  updateBadge();
  if (!outputCard.hidden && src.value.trim()) translate();
}

async function translate() {
  const text = src.value.trim();
  if (!text) {
    fail(status, 'Type something before translating.');
    fitWindow();
    return;
  }

  status.classList.remove('error');
  status.textContent = 'Translating…';
  el('btnTranslate').disabled = true;

  try {
    const result = await api.translate(text, forcedTarget);
    lastResult = { ...result, sourceText: text };
    out.textContent = result.text;
    outputCard.hidden = false;
    el('outDir').textContent = LANGUAGE_NAMES[result.target] || result.target;
    status.textContent = '';
    updateBadge();
  } catch (error) {
    fail(status, api.describeError(error));
  } finally {
    el('btnTranslate').disabled = false;
    fitWindow();
  }
}

el('btnTranslate').addEventListener('click', translate);
dirBadge.addEventListener('click', cycleDirection);

src.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    translate();
  }
});

src.addEventListener('input', () => {
  if (lastResult && src.value.trim() !== lastResult.sourceText) {
    lastResult = null;
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

let voices = [];
function loadVoices() {
  try { voices = window.speechSynthesis.getVoices() || []; } catch (_) { voices = []; }
}

function scoreVoice(voice) {
  const name = (voice.name || '').toLowerCase();
  let score = 0;
  if (name.includes('natural')) score += 5;
  if (name.includes('online')) score += 3;
  if (name.includes('neural') || name.includes('enhanced') || name.includes('premium')) score += 3;
  if (name.includes('desktop')) score -= 2;
  return score;
}

function speak(text, language) {
  if (!text || !('speechSynthesis' in window)) return;
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.lang = language === 'pt' ? 'pt-BR' : 'en-US';
  const match = voices
    .filter((voice) => voice.lang && voice.lang.toLowerCase().startsWith(language))
    .sort((a, b) => scoreVoice(b) - scoreVoice(a))[0];
  if (match) utterance.voice = match;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(utterance);
}

if ('speechSynthesis' in window) {
  loadVoices();
  window.speechSynthesis.addEventListener('voiceschanged', loadVoices);

  el('btnSpeakSrc').addEventListener('click', () => {
    const detected = lastResult && lastResult.detectedSource;
    speak(src.value.trim(), detected ? detected.toLowerCase().slice(0, 2) : guessSpokenLanguage(src.value));
  });
  el('btnSpeakOut').addEventListener('click', () => {
    speak(out.textContent.trim(), lastResult ? lastResult.target.toLowerCase() : 'en');
  });
} else {
  el('btnSpeakSrc').hidden = true;
  el('btnSpeakOut').hidden = true;
}

const SpeechRec = window.SpeechRecognition || window.webkitSpeechRecognition;
let recognition = null;
let micTimer = null;

function setMic(active) {
  el('btnMic').classList.toggle('is-active', active);
  if (!active) clearTimeout(micTimer);
}

function stopMic() {
  clearTimeout(micTimer);
  try { recognition.abort(); } catch (_) {}
  try { recognition.stop(); } catch (_) {}
  setMic(false);
}

if (SpeechRec) {
  recognition = new SpeechRec();
  recognition.interimResults = false;
  recognition.maxAlternatives = 1;
  recognition.continuous = false;
  recognition.onstart = () => setMic(true);
  recognition.onend = () => setMic(false);
  recognition.onresult = (event) => {
    setMic(false);
    src.value = event.results[0][0].transcript;
    translate();
  };
  recognition.onerror = (event) => {
    setMic(false);
    fail(status, 'Voice input unavailable (' + (event.error || 'error') + ').');
    fitWindow();
  };

  el('btnMic').addEventListener('click', () => {
    if (el('btnMic').classList.contains('is-active')) {
      stopMic();
      return;
    }
    recognition.lang = forcedTarget === 'EN' ? 'pt-BR' : forcedTarget === 'PT' ? 'en-US' : 'pt-BR';
    try {
      recognition.start();
      setMic(true);
      micTimer = setTimeout(stopMic, 12000);
    } catch (_) {
      setMic(false);
    }
  });
} else {
  el('btnMic').hidden = true;
}

function renderHistory() {
  const query = el('historySearch').value.trim().toLowerCase();
  const list = el('historyList');
  const matches = historyEntries.filter(
    (entry) =>
      !query ||
      entry.source.toLowerCase().includes(query) ||
      entry.translated.toLowerCase().includes(query)
  );

  list.replaceChildren();

  if (!matches.length) {
    const empty = document.createElement('div');
    empty.className = 'empty-state';
    empty.textContent = historyEntries.length
      ? 'Nothing matches that search.'
      : 'Translations you make will show up here.';
    list.append(empty);
    fitWindow();
    return;
  }

  for (const entry of matches) {
    const item = document.createElement('div');
    item.className = 'history-item';
    item.dataset.id = entry.id;

    const body = document.createElement('div');
    body.className = 'history-body';

    const pair = document.createElement('div');
    pair.className = 'history-pair';
    pair.textContent = `${entry.from} → ${entry.to}`;

    const translated = document.createElement('div');
    translated.className = 'history-translated';
    translated.textContent = entry.translated;

    const source = document.createElement('div');
    source.className = 'history-source';
    source.textContent = entry.source;

    body.append(pair, translated, source);

    const actions = document.createElement('div');
    actions.className = 'history-actions';
    actions.append(
      historyButton('star', entry.pinned ? 'Unpin' : 'Pin', 'pin', entry.pinned),
      historyButton('copy', 'Copy the translation', 'copy', false),
      historyButton('trash', 'Delete', 'delete', false)
    );

    item.append(body, actions);
    list.append(item);
  }

  hydrate(list);
  fitWindow();
}

function historyButton(icon, title, action, active) {
  const button = document.createElement('button');
  button.className = 'btn btn--subtle btn--sm btn--icon' + (active ? ' is-pinned' : '');
  button.dataset.icon = icon;
  button.dataset.action = action;
  button.title = title;
  return button;
}

async function loadHistory() {
  try {
    historyEntries = await api.getHistory();
  } catch (_) {
    historyEntries = [];
  }
  renderHistory();
}

el('historySearch').addEventListener('input', renderHistory);

el('historyList').addEventListener('click', async (event) => {
  const item = event.target.closest('.history-item');
  if (!item) return;
  const id = item.dataset.id;
  const entry = historyEntries.find((candidate) => candidate.id === id);
  if (!entry) return;

  const action = event.target.closest('[data-action]')?.dataset.action;

  if (action === 'pin') {
    historyEntries = await api.toggleHistoryPin(id);
    renderHistory();
    return;
  }
  if (action === 'delete') {
    historyEntries = await api.deleteHistoryEntry(id);
    renderHistory();
    return;
  }
  if (action === 'copy') {
    await api.setClipboard(entry.translated).catch(() => {});
    return;
  }

  src.value = entry.source;
  out.textContent = entry.translated;
  lastResult = {
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
  historyEntries = await api.clearHistory();
  renderHistory();
});

async function refreshUsage() {
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

function format(value) {
  if (value >= 1000000) return (value / 1000000).toFixed(1) + 'M';
  if (value >= 1000) return Math.round(value / 1000) + 'k';
  return String(value);
}

el('btnRefreshUsage').addEventListener('click', refreshUsage);

el('btnRevealKey').addEventListener('click', (event) => {
  const field = el('deeplKey');
  const hidden = field.type === 'password';
  field.type = hidden ? 'text' : 'password';
  setIcon(event.currentTarget, hidden ? 'eye-off' : 'eye');
});

const RECORDERS = ['hotkey', 'selectionHotkey', 'replaceHotkey', 'ocrHotkey', 'swapHotkey'];
const NAMED_KEYS = {
  Space: 'Space', Enter: 'Enter', Tab: 'Tab', Backspace: 'Backspace', Delete: 'Delete',
  Insert: 'Insert', Home: 'Home', End: 'End', PageUp: 'PageUp', PageDown: 'PageDown',
  ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right'
};

let recording = null;

function keyFromEvent(event) {
  const code = event.code || '';
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  return NAMED_KEYS[code] || null;
}

function stopRecording() {
  if (!recording) return;
  recording.classList.remove('is-recording');
  recording.textContent = recording.dataset.value;
  recording = null;
  document.removeEventListener('keydown', onRecordKey, true);
}

function onRecordKey(event) {
  event.preventDefault();
  event.stopPropagation();

  if (event.key === 'Escape') {
    stopRecording();
    return;
  }

  const key = keyFromEvent(event);
  if (!key) return;

  const parts = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  if (!parts.length && !/^F\d{1,2}$/.test(key)) {
    recording.textContent = 'Add Ctrl, Alt or Shift…';
    return;
  }

  parts.push(key);
  const target = recording;
  target.dataset.value = parts.join('+');
  stopRecording();
}

RECORDERS.forEach((id) => {
  el(id).addEventListener('click', () => {
    const button = el(id);
    if (recording === button) {
      stopRecording();
      return;
    }
    stopRecording();
    recording = button;
    button.classList.add('is-recording');
    button.textContent = 'Press a combination…';
    document.addEventListener('keydown', onRecordKey, true);
  });
});

function setRecorder(id, value) {
  const button = el(id);
  button.dataset.value = value;
  button.textContent = value;
}

function appearanceFromControls() {
  return {
    opacity: parseFloat(el('opacity').value),
    bg: el('colBg').value,
    accent: el('colAccent').value,
    text: el('colText').value,
    font: el('font').value
  };
}

function appearanceToControls(appearance) {
  el('opacity').value = appearance.opacity;
  el('opacityVal').textContent = Math.round(appearance.opacity * 100) + '%';
  el('colBg').value = appearance.bg;
  el('colAccent').value = appearance.accent;
  el('colText').value = appearance.text;

  const select = el('font');
  if (![...select.options].some((option) => option.value === appearance.font)) {
    const custom = document.createElement('option');
    custom.value = appearance.font;
    custom.textContent = 'Custom';
    select.append(custom);
  }
  select.value = appearance.font;
}

function syncThemeSelect() {
  const index = THEMES.findIndex(
    (theme) =>
      theme.bg === el('colBg').value.toLowerCase() &&
      theme.accent === el('colAccent').value.toLowerCase() &&
      theme.text === el('colText').value.toLowerCase()
  );
  el('themeSelect').value = index >= 0 ? String(index) : 'custom';
}

function previewAppearance() {
  const appearance = appearanceFromControls();
  el('opacityVal').textContent = Math.round(appearance.opacity * 100) + '%';
  applyAppearance(appearance);
  syncThemeSelect();
}

['opacity', 'colBg', 'colAccent', 'colText', 'font'].forEach((id) => {
  el(id).addEventListener('input', previewAppearance);
});

(function buildThemeSelect() {
  const select = el('themeSelect');
  THEMES.forEach((theme, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = theme.name;
    select.append(option);
  });
  const custom = document.createElement('option');
  custom.value = 'custom';
  custom.textContent = 'Custom';
  select.append(custom);

  select.addEventListener('change', () => {
    if (select.value === 'custom') return;
    const theme = THEMES[Number(select.value)];
    el('colBg').value = theme.bg;
    el('colAccent').value = theme.accent;
    el('colText').value = theme.text;
    previewAppearance();
  });
})();

function matchesHotkey(event, combo) {
  if (!combo) return false;
  const parts = combo.split('+').map((part) => part.trim().toLowerCase()).filter(Boolean);
  if (!parts.length) return false;

  const key = parts[parts.length - 1];
  const need = { alt: false, ctrl: false, shift: false, meta: false };
  parts.slice(0, -1).forEach((modifier) => {
    if (modifier === 'alt' || modifier === 'option') need.alt = true;
    else if (['ctrl', 'control', 'commandorcontrol', 'cmdorctrl'].includes(modifier)) need.ctrl = true;
    else if (modifier === 'shift') need.shift = true;
    else if (['meta', 'cmd', 'command', 'super', 'win'].includes(modifier)) need.meta = true;
  });

  if (event.altKey !== need.alt || event.ctrlKey !== need.ctrl ||
      event.shiftKey !== need.shift || event.metaKey !== need.meta) return false;

  return (keyFromEvent(event) || '').toLowerCase() === key;
}

async function loadSettings() {
  settings = { ...DEFAULT_SETTINGS, ...(await api.getSettings()) };
  if (!settings.appearance) settings.appearance = { ...DEFAULT_APPEARANCE };

  setRecorder('hotkey', settings.hotkey);
  setRecorder('swapHotkey', settings.swapHotkey);
  setRecorder('selectionHotkey', settings.selectionHotkey);
  setRecorder('replaceHotkey', settings.replaceHotkey);
  setRecorder('ocrHotkey', settings.ocrHotkey);

  el('deeplKey').value = settings.deeplKey || '';
  el('autoTranslateClipboard').checked = settings.autoTranslateClipboard === true;
  try { el('autostart').checked = await api.getAutostart(); } catch (_) {}
  try { el('appVersion').textContent = `Lexo ${await api.appVersion()}`; } catch (_) {}

  appearanceToControls(settings.appearance);
  applyAppearance(settings.appearance);

  try {
    if (!(await api.ocrAvailable())) {
      el('btnOcr').hidden = true;
      el('ocrHotkeyField').hidden = true;
    }
  } catch (_) {}

  updateBadge();
  fitWindow();
}

async function saveAll(statusNode) {
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

    settings = { ...settings, ...updated };
    applyAppearance(settings.appearance);
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

el('btnSaveSettings').addEventListener('click', () => saveAll(el('settingsStatus')));
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

el('btnHistory').addEventListener('click', () => showView(VIEWS.history.hidden ? 'history' : 'main'));
el('btnSettings').addEventListener('click', () => showView(VIEWS.settings.hidden ? 'settings' : 'main'));
el('btnAppearance').addEventListener('click', () => showView(VIEWS.appearance.hidden ? 'appearance' : 'main'));
el('btnClose').addEventListener('click', () => api.hidePopup());
el('btnBackHistory').addEventListener('click', () => showView('main'));
el('btnBackSettings').addEventListener('click', () => showView('main'));
el('btnBackAppearance').addEventListener('click', () => {
  applyAppearance(settings.appearance);
  showView('main');
});

document.addEventListener('keydown', (event) => {
  if (recording) return;

  if (matchesHotkey(event, settings.swapHotkey)) {
    event.preventDefault();
    cycleDirection();
    return;
  }
  if (event.key === 'Escape') {
    if (VIEWS.main.hidden) showView('main');
    else api.hidePopup();
  }
});

function showUpdateBanner(update) {
  if (updateDismissed || !update || !update.version) return;
  el('updateText').textContent = `Update available · ${update.version}`;
  el('updateBanner').hidden = false;
  fitWindow();
}

function hideUpdateBanner() {
  el('updateBanner').hidden = true;
  fitWindow();
}

async function syncUpdateBanner() {
  if (updateDismissed) return;
  try {
    showUpdateBanner(await api.pendingUpdate());
  } catch (_) {}
}

el('btnDismissUpdate').addEventListener('click', () => {
  updateDismissed = true;
  hideUpdateBanner();
});

el('btnInstallUpdate').addEventListener('click', async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  button.textContent = 'Updating…';
  try {
    await api.installUpdate();
  } catch (error) {
    button.disabled = false;
    button.textContent = 'Update';
    if (String(error).includes('updater_none')) {
      updateDismissed = true;
      hideUpdateBanner();
      return;
    }
    fail(status, api.describeError(error));
    fitWindow();
  }
});

api.listen('update-available', (event) => showUpdateBanner(event.payload));

api.listen('popup-shown', (event) => {
  syncUpdateBanner();
  showView('main');
  const clipboard = (event.payload && event.payload.clipboard) || '';
  if (clipboard.trim()) {
    src.value = clipboard;
    lastResult = null;
    translate();
  }
});

api.listen('open-view', (event) => showView(String(event.payload || 'main')));

loadSettings();
syncUpdateBanner();
