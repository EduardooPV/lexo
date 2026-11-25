import * as api from './shared/api.js';
import { hydrate, setIcon } from './shared/icons.js';
import { languageName } from './shared/languages.js';
import { setErrorStatus } from './shared/status.js';
import { applyAppearance } from './shared/theme.js';
import { detectedSpeechLanguage, isSpeechAvailable, loadVoices, speak } from './shared/speech.js';

const el = (id) => document.getElementById(id);

const EMPTY_MESSAGE = {
  selection: 'No text selected — nothing was copied.',
  ocr: 'No text found in that region.'
};

let lastResult = null;
let shownAt = 0;
let settings = null;
let currentEvent = 0;
let pendingText = null;

hydrate();

function fit() {
  requestAnimationFrame(() => {
    api.resizeMini(Math.ceil(el('bubble').getBoundingClientRect().height)).catch(() => {});
  });
}

function setStatus(message, isError = Boolean(message)) {
  const node = el('miniStatus');
  if (isError) setErrorStatus(node, message);
  else {
    node.textContent = message;
    node.classList.remove('error');
  }
}

// An OCR grab is read straight off the screen, so the source text is still
// sitting right behind the bubble — showing it again only makes the bubble
// taller. That mode drops the source block, and the close button moves down to
// the translation header so it never disappears with it.
function setCompact(compact) {
  el('bubble').classList.toggle('is-compact', compact);
  const actions = el(compact ? 'outActions' : 'srcActions');
  const close = el('close');
  if (close.parentElement !== actions) actions.append(close);
}

function reset() {
  el('srcLang').textContent = '—';
  el('outLang').textContent = '—';
  el('srcText').textContent = '';
  el('outText').textContent = '';
  setStatus('');
  lastResult = null;
}

if (isSpeechAvailable()) {
  loadVoices();
  window.speechSynthesis.addEventListener('voiceschanged', loadVoices);
} else {
  el('speakSrc').hidden = true;
  el('speakOut').hidden = true;
}

el('speakSrc').addEventListener('click', () => {
  const language = lastResult
    ? detectedSpeechLanguage(lastResult.detectedSource, settings?.primaryLang, settings?.secondaryLang)
    : settings?.primaryLang;
  speak(el('srcText').textContent, language);
});

el('speakOut').addEventListener('click', () => {
  speak(el('outText').textContent, lastResult?.target || settings?.secondaryLang);
});

el('copyOut').addEventListener('click', async (event) => {
  const button = event.currentTarget;
  const text = el('outText').textContent;
  if (!text) return;

  try {
    await api.setClipboard(text);
  } catch (_) {
    try { await navigator.clipboard.writeText(text); } catch (_) {}
  }

  button.classList.add('is-success');
  setIcon(button, 'check');
  clearTimeout(button._timer);
  button._timer = setTimeout(() => {
    button.classList.remove('is-success');
    setIcon(button, 'copy');
  }, 900);
});

el('close').addEventListener('click', () => api.hideMini());

document.addEventListener('keydown', (event) => {
  const key = (event.key || '').toLowerCase();
  if (key === 'escape') api.hideMini();
  if (key === 'c' && event.ctrlKey && !String(window.getSelection() || '')) {
    el('copyOut').click();
  }
});

let grabbedAt = 0;
document.addEventListener(
  'mousedown',
  (event) => {
    const target = event.target;
    if (target instanceof Element && target.closest('[data-tauri-drag-region="false"]')) return;
    grabbedAt = Date.now();
  },
  true
);

window.addEventListener('blur', () => {
  if (Date.now() - shownAt < 500) return;
  if (Date.now() - grabbedAt < 1500) return;
  api.hideMini();
});

async function run(text, confirmed = false, eventId = currentEvent) {
  pendingText = null;
  el('miniConfirm').hidden = true;
  el('srcText').textContent = text;
  el('outText').textContent = '…';
  el('srcLang').textContent = '—';
  el('outLang').textContent = '—';
  lastResult = null;
  setStatus('');
  fit();

  try {
    const result = await api.translate(text, null, confirmed);
    if (eventId !== currentEvent) return;
    lastResult = result;
    el('outText').textContent = result.text;
    el('srcLang').textContent = languageName(result.detectedSource);
    el('outLang').textContent = languageName(result.target);
  } catch (error) {
    if (eventId !== currentEvent) return;
    el('outText').textContent = '';
    const needsConfirmation = api.longTextLength(error) !== null;
    setStatus(api.describeError(error), !needsConfirmation);
    if (needsConfirmation) {
      pendingText = text;
      el('miniConfirm').hidden = false;
    }
  }
  fit();
}

el('miniConfirm').addEventListener('click', () => {
  if (pendingText) {
    el('bubble').focus({ preventScroll: true });
    run(pendingText, true);
  }
});

api.listen('mini-translate', async (event) => {
  const eventId = ++currentEvent;
  pendingText = null;
  el('miniConfirm').hidden = true;
  shownAt = Date.now();
  const { text = '', origin = 'selection', error = null } = event.payload || {};
  setCompact(origin === 'ocr');
  el('bubble').focus({ preventScroll: true });

  try {
    const updated = await api.getSettings();
    if (eventId !== currentEvent) return;
    settings = updated;
    applyAppearance(settings?.appearance);
  } catch (_) {}
  if (eventId !== currentEvent) return;

  if (error) {
    reset();
    el('srcText').textContent = text;
    setStatus(api.describeError(error));
    fit();
    return;
  }

  if (!text.trim()) {
    reset();
    setStatus(EMPTY_MESSAGE[origin] || EMPTY_MESSAGE.selection);
    fit();
    return;
  }

  await run(text.trim());
});
