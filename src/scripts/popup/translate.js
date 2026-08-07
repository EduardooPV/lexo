import * as api from '../shared/api.js';
import { dirBadge, dirLabel, el, out, outputCard, src, status } from './dom.js';
import { LANGUAGE_NAMES, state } from './state.js';
import { fail, fitWindow } from './views.js';

export function updateBadge() {
  if (state.forcedTarget) {
    dirLabel.textContent = state.forcedTarget === 'EN' ? 'PT → EN' : 'EN → PT';
    dirBadge.title = `Pinned direction — click or press ${state.settings.swapHotkey} to change`;
  } else if (state.lastResult) {
    dirLabel.textContent = `${state.lastResult.detectedSource || '?'} → ${state.lastResult.target}`;
    dirBadge.title = `Detected automatically — click or press ${state.settings.swapHotkey} to pin a direction`;
  } else {
    dirLabel.textContent = 'Auto';
    dirBadge.title = `Detected automatically — click or press ${state.settings.swapHotkey} to pin a direction`;
  }
  dirBadge.classList.toggle('is-forced', Boolean(state.forcedTarget));
}

export function cycleDirection() {
  const shown = state.forcedTarget || (state.lastResult && state.lastResult.target) || 'EN';
  state.forcedTarget = shown === 'EN' ? 'PT' : 'EN';
  updateBadge();
  if (!outputCard.hidden && src.value.trim()) translate();
}

export async function translate() {
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
    const result = await api.translate(text, state.forcedTarget);
    state.lastResult = { ...result, sourceText: text };
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
