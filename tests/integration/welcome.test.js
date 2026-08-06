import { describe, it, expect } from 'vitest';
import { mountPopup, flush, el } from '../helpers/mount.js';
import { DEFAULT_SETTINGS } from '../helpers/tauri.js';

const firstRun = (overrides = {}) =>
  mountPopup({
    get_settings: () => ({ ...structuredClone(DEFAULT_SETTINGS), deeplKey: '' }),
    ...overrides
  });

const type = (id, value) => {
  el(id).value = value;
};

describe('first run', () => {
  it('greets a run with no key instead of a translator that cannot translate', async () => {
    await firstRun();

    expect(el('welcomeView').hidden).toBe(false);
    expect(el('mainView').hidden).toBe(true);
  });

  it('stays out of the way once a key is stored', async () => {
    await mountPopup();

    expect(el('welcomeView').hidden).toBe(true);
    expect(el('mainView').hidden).toBe(false);
  });

  it('keeps sending the popup back to the welcome screen while no key exists', async () => {
    const mock = await firstRun();

    await mock.emit('popup-shown', { clipboard: '' });
    await flush();

    expect(el('welcomeView').hidden).toBe(false);
    expect(el('mainView').hidden).toBe(true);
  });

  it('lets someone in without a key once they choose to decide later', async () => {
    const mock = await firstRun();

    el('btnSkipWelcome').click();
    await flush();
    expect(el('mainView').hidden).toBe(false);

    await mock.emit('popup-shown', { clipboard: '' });
    await flush();
    expect(el('mainView').hidden).toBe(false);
  });
});

describe('connecting a DeepL account', () => {
  it('hands the signup page to the browser instead of opening it in the app', async () => {
    const mock = await firstRun();

    el('btnDeeplAccount').click();
    await flush();

    expect(mock.callsFor('open_deepl_signup')).toHaveLength(1);
  });

  it('asks for a key before spending a call on checking one', async () => {
    const mock = await firstRun();

    el('btnConnectDeepl').click();
    await flush();

    expect(mock.callsFor('verify_deepl_key')).toHaveLength(0);
    expect(el('welcomeStatus').classList.contains('error')).toBe(true);
    expect(el('welcomeView').hidden).toBe(false);
  });

  it('checks the key against DeepL before storing it', async () => {
    const mock = await firstRun();

    type('welcomeKey', 'brand-new-key:fx');
    el('btnConnectDeepl').click();
    await flush();

    expect(mock.callsFor('verify_deepl_key')[0].args).toEqual({ key: 'brand-new-key:fx' });
    expect(mock.callsFor('save_settings')).toHaveLength(1);
    expect(el('mainView').hidden).toBe(false);
  });

  it('saves the key without wiping the shortcuts it was not asked about', async () => {
    const mock = await firstRun();

    type('welcomeKey', 'brand-new-key:fx');
    el('btnConnectDeepl').click();
    await flush();

    const saved = mock.callsFor('save_settings')[0].args.settings;
    expect(saved.deeplKey).toBe('brand-new-key:fx');
    expect(saved.hotkey).toBe(DEFAULT_SETTINGS.hotkey);
    expect(saved.selectionHotkey).toBe(DEFAULT_SETTINGS.selectionHotkey);
    expect(saved.appearance).toEqual(DEFAULT_SETTINGS.appearance);
  });

  it('never stores a key DeepL rejects', async () => {
    const mock = await firstRun({
      verify_deepl_key: () => {
        throw 'deepl_auth: DeepL rejected the key.';
      }
    });

    type('welcomeKey', 'wrong-key');
    el('btnConnectDeepl').click();
    await flush();

    expect(mock.callsFor('save_settings')).toHaveLength(0);
    expect(el('welcomeView').hidden).toBe(false);
    expect(el('welcomeStatus').textContent).toBe('DeepL rejected the key. Check it in Settings.');
    expect(el('welcomeStatus').classList.contains('error')).toBe(true);
  });

  it('lets a rejected key be retyped instead of locking the button', async () => {
    await firstRun({
      verify_deepl_key: () => {
        throw 'network_error: offline';
      }
    });

    type('welcomeKey', 'brand-new-key:fx');
    el('btnConnectDeepl').click();
    await flush();

    expect(el('btnConnectDeepl').disabled).toBe(false);
  });

  it('carries the connected key into the settings panel', async () => {
    await firstRun();

    type('welcomeKey', 'brand-new-key:fx');
    el('btnConnectDeepl').click();
    await flush();

    expect(el('deeplKey').value).toBe('brand-new-key:fx');
  });

  it('keeps the key masked until it is asked to show it', async () => {
    await firstRun();

    expect(el('welcomeKey').type).toBe('password');
    el('btnRevealWelcomeKey').click();
    expect(el('welcomeKey').type).toBe('text');
    el('btnRevealWelcomeKey').click();
    expect(el('welcomeKey').type).toBe('password');
  });
});
