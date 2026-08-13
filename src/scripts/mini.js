import * as api from './shared/api.js';
import { hydrate, setIcon } from './shared/icons.js';
import { applyAppearance } from './shared/theme.js';
import { isSpeechAvailable, loadVoices, speak } from './shared/speech.js';

const el = (id) => document.getElementById(id);

const LANGUAGE_NAMES = { PT: 'Portuguese', EN: 'English' };
const EMPTY_MESSAGE = {
  selection: 'No text selected — nothing was copied.',
  ocr: 'No text found in that region.'
};

let lastResult = null;
let shownAt = 0;

hydrate();

function fit() {
  requestAnimationFrame(() => {
    api.resizeMini(Math.ceil(el('bubble').getBoundingClientRect().height)).catch(() => {});
  });
}

function setStatus(message) {
  const node = el('miniStatus');
  node.textContent = message || '';
  node.classList.toggle('error', Boolean(message));
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
  const language = lastResult ? lastResult.detectedSource.toLowerCase().slice(0, 2) : 'en';
  speak(el('srcText').textContent, language);
});

el('speakOut').addEventListener('click', () => {
  speak(el('outText').textContent, lastResult ? lastResult.target.toLowerCase() : 'en');
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

async function run(text) {
  el('srcText').textContent = text;
  el('outText').textContent = '…';
  el('srcLang').textContent = '—';
  el('outLang').textContent = '—';
  setStatus('');
  fit();

  try {
    const result = await api.translate(text);
    lastResult = result;
    el('outText').textContent = result.text;
    el('srcLang').textContent = LANGUAGE_NAMES[result.detectedSource] || result.detectedSource;
    el('outLang').textContent = LANGUAGE_NAMES[result.target] || result.target;
  } catch (error) {
    el('outText').textContent = '';
    setStatus(api.describeError(error));
  }
  fit();
}

api.listen('mini-translate', async (event) => {
  shownAt = Date.now();
  const { text = '', origin = 'selection', error = null } = event.payload || {};
  setCompact(origin === 'ocr');

  try {
    const settings = await api.getSettings();
    applyAppearance(settings && settings.appearance);
  } catch (_) {}

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

  run(text.trim());
});
