import * as api from '../shared/api.js';
import { applyAppearance } from '../shared/theme.js';
import { el, src, VIEWS } from './dom.js';
import { state } from './state.js';
import { loadHistory } from './history.js';
import { refreshUsage } from './settings.js';
import { appearanceToControls, syncThemeSelect } from './appearance.js';

export function fitWindow() {
  requestAnimationFrame(() => {
    api.resizeWindow(Math.ceil(el('app').scrollHeight)).catch(() => {});
  });
}

// An empty status takes no room at all, so writing one grows the window and
// clearing one has to shrink it back — nothing else runs when the toast fades.
export function toast(node, text, ms = 2200) {
  node.classList.remove('error');
  node.textContent = text;
  fitWindow();
  clearTimeout(node._timer);
  node._timer = setTimeout(() => {
    node.textContent = '';
    fitWindow();
  }, ms);
}

export function fail(node, text) {
  node.classList.add('error');
  node.textContent = text;
  fitWindow();
}

export function needsWelcome() {
  return !state.skippedWelcome && !String(state.settings.deeplKey || '').trim();
}

export function showView(name) {
  const view = name === 'main' && needsWelcome() ? 'welcome' : name;
  Object.entries(VIEWS).forEach(([key, node]) => { node.hidden = key !== view; });

  if (view === 'main') {
    src.focus();
    src.select();
  }
  if (view === 'welcome') {
    el('welcomeStatus').classList.remove('error');
    el('welcomeStatus').textContent = '';
  }
  if (view === 'history') {
    el('historySearch').value = '';
    loadHistory();
  }
  if (view === 'shortcuts') {
    el('shortcutsStatus').classList.remove('error');
    el('shortcutsStatus').textContent = '';
  }
  if (view === 'settings') {
    el('settingsStatus').textContent = '';
    refreshUsage();
  }
  if (view === 'appearance') {
    el('appearanceStatus').textContent = '';
    appearanceToControls(state.settings.appearance);
    applyAppearance(state.settings.appearance);
    syncThemeSelect();
  }
  fitWindow();
}
