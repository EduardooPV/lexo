import * as api from '../shared/api.js';
import { el, status } from './dom.js';
import { renderUsageHint } from './settings.js';
import { state } from './state.js';
import { fail, fitWindow, showView, toast } from './views.js';

export async function connectDeepl() {
  const statusNode = el('welcomeStatus');
  const key = el('welcomeKey').value.trim();
  if (!key) {
    fail(statusNode, 'Paste the API key from your DeepL account first.');
    fitWindow();
    return;
  }

  const button = el('btnConnectDeepl');
  button.disabled = true;
  statusNode.classList.remove('error');
  statusNode.textContent = 'Checking the key…';
  fitWindow();

  try {
    const usage = await api.verifyDeeplKey(key);
    const warning = await api.saveSettings({ ...state.settings, deeplKey: key });
    state.settings = { ...state.settings, deeplKey: key };
    el('deeplKey').value = key;
    renderUsageHint(usage);
    showView('main');
    toast(status, warning || 'Connected to DeepL.', warning ? 6000 : undefined);
  } catch (error) {
    fail(statusNode, api.describeError(error));
    fitWindow();
  } finally {
    button.disabled = false;
  }
}
