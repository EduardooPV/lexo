import * as api from '../shared/api.js';
import { setLabel } from '../shared/icons.js';
import { baseOf, languageName } from '../shared/languages.js';
import { dirBadge, dirLabel, el, out, outputCard, src, status } from './dom.js';
import { state } from './state.js';
import { fail, fitWindow } from './views.js';

let pendingConfirmation = null;
let latestRequest = 0;

export function updateBadge() {
  const { primaryLang, secondaryLang, swapHotkey } = state.settings;
  if (state.forcedTarget) {
    const from = state.forcedTarget === primaryLang ? secondaryLang
      : state.forcedTarget === secondaryLang ? primaryLang : state.lastResult?.detectedSource;
    dirLabel.textContent = `${baseOf(from) || '?'} → ${state.forcedTarget}`;
  } else if (state.lastResult) {
    dirLabel.textContent = `${state.lastResult.detectedSource || '?'} → ${state.lastResult.target}`;
  } else {
    dirLabel.textContent = 'Auto';
  }
  const hint = state.forcedTarget
    ? `pinned direction — click or press ${swapHotkey} to change`
    : `detected automatically — click or press ${swapHotkey} to pin a direction`;
  setLabel(dirBadge, `${dirLabel.textContent}: ${hint}`);
  dirBadge.classList.toggle('is-forced', Boolean(state.forcedTarget));
  el('targetSelect').value = state.forcedTarget || '';
}

export function cycleDirection() {
  const { primaryLang, secondaryLang } = state.settings;
  const shown = state.forcedTarget || state.lastResult?.target || secondaryLang;
  selectTarget(shown === primaryLang ? secondaryLang : primaryLang);
}

export function selectTarget(target) {
  state.forcedTarget = target || null;
  resetLongTextConfirmation();
  updateBadge();
  if (!outputCard.hidden && src.value.trim()) translate();
}

export function resetLongTextConfirmation() {
  if (!pendingConfirmation) return;
  pendingConfirmation = null;
  el('btnTranslate').textContent = 'Translate';
  status.textContent = '';
  fitWindow();
}

export async function translate(allowConfirmation = false) {
  const text = src.value.trim();
  const target = state.forcedTarget;
  if (!text) {
    fail(status, 'Type something before translating.');
    return;
  }

  const confirmed = allowConfirmation && pendingConfirmation?.text === text &&
    pendingConfirmation.target === target;
  const request = ++latestRequest;
  resetLongTextConfirmation();
  clearTimeout(status._timer);
  status.classList.remove('error');
  status.textContent = 'Translating…';
  const button = el('btnTranslate');
  button.disabled = true;
  const isCurrent = () => src.value.trim() === text && state.forcedTarget === target;

  try {
    const result = await api.translate(text, target, confirmed);
    if (request !== latestRequest || !isCurrent()) return;
    state.lastResult = { ...result, sourceText: text };
    outputCard.hidden = false;
    out.textContent = result.text;
    el('outDir').textContent = languageName(result.target);
    status.textContent = '';
    updateBadge();
  } catch (error) {
    if (request !== latestRequest || !isCurrent()) return;
    if (api.longTextLength(error) !== null) {
      pendingConfirmation = { text, target };
      status.textContent = api.describeError(error);
      button.textContent = 'Translate anyway';
    } else {
      fail(status, api.describeError(error));
    }
  } finally {
    if (request === latestRequest) {
      if (!isCurrent() && status.textContent === 'Translating…') status.textContent = '';
      button.disabled = false;
      fitWindow();
    }
  }
}
