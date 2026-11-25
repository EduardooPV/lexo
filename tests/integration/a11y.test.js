import { expect, it } from 'vitest';
import { el, flush, mountMini, mountOverlay, mountPopup } from '../helpers/mount.js';
import { DEFAULT_SETTINGS, sampleHistory } from '../helpers/tauri.js';

function expectNamedButtons() {
  for (const button of document.querySelectorAll('button')) {
    const labelIds = button.getAttribute('aria-labelledby');
    const label = labelIds
      ? labelIds.split(' ').map((id) => el(id)?.textContent).join(' ')
      : button.getAttribute('aria-label') || button.textContent;
    expect(label.trim(), button.id || button.outerHTML).not.toBe('');
  }
}

function expectLiveStatuses() {
  for (const status of document.querySelectorAll('.status')) {
    expect(status.getAttribute('aria-live'), status.id).toBe('polite');
    expect(status.getAttribute('aria-atomic'), status.id).toBe('true');
  }
}

it('names popup controls and labels dynamically rendered history actions', async () => {
  await mountPopup({ get_history: sampleHistory });
  el('btnHistory').click();
  await flush();
  expectNamedButtons();
  expectLiveStatuses();
  for (const button of el('historyList').querySelectorAll('button')) {
    expect(button.getAttribute('aria-label')).toBe(button.title);
  }
  expect(el('historyList').querySelector('[data-action="pin"]').getAttribute('aria-label')).toBe('Unpin');
  expect(el('out').getAttribute('aria-live')).toBe('polite');
});

it('moves focus to the active view and reflects monthly usage in the progressbar', async () => {
  const tauri = await mountPopup();
  el('btnSettings').click();
  await flush();
  expect(document.activeElement).toBe(el('settingsView').querySelector('.panel-title'));
  expect([...document.querySelectorAll('[aria-current="page"]')]).toEqual([el('btnSettings')]);
  const progress = document.querySelector('[role="progressbar"]');
  expect(progress.getAttribute('aria-label')).toBe('DeepL monthly usage');
  expect(progress.getAttribute('aria-valuemin')).toBe('0');
  expect(progress.getAttribute('aria-valuemax')).toBe('500000');
  expect(progress.getAttribute('aria-valuenow')).toBe('412345');

  el('btnRevealKey').click();
  expect(el('btnRevealKey').getAttribute('aria-label')).toBe('Hide the key');

  await tauri.emit('popup-shown');
  expect(document.activeElement).toBe(el('src'));
  expect([...document.querySelectorAll('[aria-current="page"]')]).toEqual([el('btnHome')]);
  el('dirBadge').click();
  const badge = el('dirBadge');
  expect(badge.getAttribute('aria-label')).toBe(badge.title);
  expect(badge.getAttribute('aria-label')).toContain(el('dirLabel').textContent);
  await flush();
});

it('focuses the key field when the popup opens to Welcome', async () => {
  const tauri = await mountPopup({ get_settings: () => ({ ...DEFAULT_SETTINGS, deeplKey: '' }) });
  await tauri.emit('popup-shown');
  expect(document.activeElement).toBe(el('welcomeKey'));
  expect(document.querySelector('[aria-current="page"]')).toBeNull();
  await flush();
});

it('names the mini controls and focuses its live translation region', async () => {
  const tauri = await mountMini();
  expectNamedButtons();
  expectLiveStatuses();
  await tauri.emit('mini-translate', { text: 'Olá' });
  expect(document.activeElement).toBe(el('bubble'));
  expect(el('bubble').getAttribute('role')).toBe('region');
  expect(el('outText').getAttribute('aria-live')).toBe('polite');
  await flush();
});

it('exposes the region capture hint as a status', async () => {
  await mountOverlay();
  expect(el('hint').getAttribute('role')).toBe('status');
});
