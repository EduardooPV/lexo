import * as api from '../shared/api.js';
import { hydrate, setLabel } from '../shared/icons.js';
import { el } from './dom.js';
import { state } from './state.js';
import { fail, fitWindow, toast } from './views.js';

let exporting = false;

function updateExportButtons() {
  const disabled = exporting || state.historyEntries.length === 0;
  el('btnExportJson').disabled = disabled;
  el('btnExportCsv').disabled = disabled;
}

function historyButton(icon, title, action, active) {
  const button = document.createElement('button');
  button.className = 'btn btn--subtle btn--sm btn--icon' + (active ? ' is-pinned' : '');
  button.dataset.icon = icon;
  button.dataset.action = action;
  setLabel(button, title);
  return button;
}

export function renderHistory() {
  const query = el('historySearch').value.trim().toLowerCase();
  const list = el('historyList');
  const matches = state.historyEntries.filter(
    (entry) =>
      !query ||
      entry.source.toLowerCase().includes(query) ||
      entry.translated.toLowerCase().includes(query)
  );

  list.replaceChildren();
  el('historyDisabledHint').hidden = state.settings.keepHistory !== false;
  updateExportButtons();

  if (!matches.length) {
    const empty = document.createElement('div');
    empty.className = 'empty-state';
    empty.textContent = state.historyEntries.length
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

export async function loadHistory() {
  try {
    state.historyEntries = await api.getHistory();
  } catch (_) {
    state.historyEntries = [];
  }
  renderHistory();
}

export async function exportHistory(format) {
  if (exporting || !state.historyEntries.length) return;
  const status = el('historyStatus');
  exporting = true;
  updateExportButtons();
  try {
    const path = await api.exportHistory(format);
    if (path) toast(status, 'Exported!');
  } catch (error) {
    fail(status, api.describeError(error));
  } finally {
    exporting = false;
    updateExportButtons();
  }
}
