import * as api from '../shared/api.js';
import { el, status } from './dom.js';
import { state } from './state.js';
import { fail, fitWindow } from './views.js';

export function showUpdateBanner(update) {
  if (state.updateDismissed || !update || !update.version) return;
  el('updateText').textContent = `Update available · ${update.version}`;
  el('updateBanner').hidden = false;
  fitWindow();
}

export function hideUpdateBanner() {
  el('updateBanner').hidden = true;
  fitWindow();
}

export function dismissUpdateBanner() {
  state.updateDismissed = true;
  hideUpdateBanner();
}

export async function syncUpdateBanner() {
  if (state.updateDismissed) return;
  try {
    showUpdateBanner(await api.pendingUpdate());
  } catch (_) {}
}

export async function installUpdate(button) {
  button.disabled = true;
  button.textContent = 'Updating…';
  try {
    await api.installUpdate();
  } catch (error) {
    button.disabled = false;
    button.textContent = 'Update';
    if (String(error).includes('updater_none')) {
      dismissUpdateBanner();
      return;
    }
    fail(status, api.describeError(error));
    fitWindow();
  }
}
