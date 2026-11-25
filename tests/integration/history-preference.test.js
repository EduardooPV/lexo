import { expect, it } from 'vitest';
import { el, flush, mountPopup } from '../helpers/mount.js';
import { DEFAULT_SETTINGS, sampleHistory } from '../helpers/tauri.js';

it('loads the history preference and saves a changed value', async () => {
  const mock = await mountPopup({
    get_settings: () => ({ ...DEFAULT_SETTINGS, keepHistory: false })
  });

  el('btnSettings').click();
  await flush();
  expect(el('keepHistory').checked).toBe(false);

  el('keepHistory').click();
  el('btnSaveSettings').click();
  await flush();
  expect(mock.callsFor('save_settings').at(-1).args.settings.keepHistory).toBe(true);
});

it('keeps existing translations visible when saving history is disabled', async () => {
  const mock = await mountPopup({ get_history: sampleHistory });
  expect(el('keepHistory').checked).toBe(true);

  el('btnSettings').click();
  el('keepHistory').click();
  el('btnSaveSettings').click();
  await flush();
  expect(mock.callsFor('save_settings').at(-1).args.settings.keepHistory).toBe(false);

  el('btnHistory').click();
  await flush();
  expect(el('historyDisabledHint').hidden).toBe(false);
  expect(document.querySelectorAll('.history-item')).toHaveLength(2);
  expect(mock.callsFor('clear_history')).toHaveLength(0);
});

it('shows the keychain warning returned by the backend when saving settings', async () => {
  const warning = 'Saved, but the system keychain is unavailable \u2014 the key is stored in the settings file.';
  await mountPopup({ save_settings: () => warning });

  el('btnSettings').click();
  el('deeplKey').value = 'new-key:fx';
  el('btnSaveSettings').click();
  await flush();

  expect(el('mainView').hidden).toBe(false);
  expect(el('status').textContent).toBe(warning);
});
