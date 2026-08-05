import * as api from './api.js';
import { hydrate, setIcon } from './icons.js';
import { applyAppearance } from './theme.js';

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

function reset() {
  el('srcLang').textContent = '—';
  el('outLang').textContent = '—';
  el('srcText').textContent = '';
  el('outText').textContent = '';
  setStatus('');
  lastResult = null;
}

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
