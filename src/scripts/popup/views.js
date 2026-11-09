import * as api from '../shared/api.js';
import { applyAppearance } from '../shared/theme.js';
import { el, src, VIEWS } from './dom.js';
import { state } from './state.js';
import { loadHistory } from './history.js';
import { refreshUsage } from './settings.js';
import { appearanceToControls, syncThemeSelect } from './appearance.js';

// Height the app last asked for, in CSS pixels. A window height that differs
// from it was set by the user dragging the edge, not by us.
let requestedHeight = 0;
let customHeight = false;
let resizeTimer = 0;

function currentZoom() {
  return (state.settings.appearance && state.settings.appearance.zoom) || 1;
}

export function fitWindow() {
  requestAnimationFrame(() => {
    if (customHeight) return;
    requestedHeight = Math.ceil(el('app').scrollHeight);
    api.resizeWindow(Math.ceil(requestedHeight * currentZoom())).catch(() => {});
  });
}

export function hasCustomHeight() {
  return customHeight;
}

// After a zoom change the window keeps its logical height, so a user-chosen
// height is re-sent in the new scale; an auto-fitted one just refits.
export function refitAfterZoom(cssHeight) {
  if (customHeight) {
    requestedHeight = cssHeight;
    api.resizeWindow(Math.ceil(cssHeight * currentZoom())).catch(() => {});
  } else {
    fitWindow();
  }
}

window.addEventListener('resize', () => {
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    if (!requestedHeight) return;
    if (Math.abs(window.innerHeight - requestedHeight) > 4) {
      customHeight = true;
      el('app').classList.add('is-custom-height');
    } else if (!customHeight) {
      fitWindow();
    }
  }, 120);
});

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
