import { describe, it, expect } from 'vitest';
import { mountPopup, flush, el } from '../helpers/mount.js';
import { sampleHistory } from '../helpers/tauri.js';

const press = (node, init) => node.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, ...init }));

describe('startup', () => {
  it('renders every icon in the markup', async () => {
    await mountPopup();
    const placeholders = [...document.querySelectorAll('[data-icon]')];
    expect(placeholders.length).toBeGreaterThan(0);
    expect(placeholders.every((node) => node.querySelector('svg'))).toBe(true);
  });

  it('opens on the translator with the output card hidden', async () => {
    await mountPopup();
    expect(el('mainView').hidden).toBe(false);
    expect(el('historyView').hidden).toBe(true);
    expect(el('settingsView').hidden).toBe(true);
    expect(el('outputCard').hidden).toBe(true);
  });

  it('loads the saved shortcuts into the recorder buttons', async () => {
    await mountPopup();
    expect(el('hotkey').textContent).toBe('Alt+R');
    expect(el('selectionHotkey').textContent).toBe('Alt+T');
    expect(el('replaceHotkey').textContent).toBe('Alt+Shift+T');
    expect(el('ocrHotkey').textContent).toBe('Alt+S');
  });

  it('hides the OCR affordances when the backend reports no OCR', async () => {
    await mountPopup({ ocr_available: () => false });
    expect(el('btnOcr').hidden).toBe(true);
    expect(el('ocrHotkeyField').hidden).toBe(true);
  });
});

describe('translating', () => {
  it('shows the result and the direction DeepL actually detected', async () => {
    await mountPopup();
    el('src').value = 'preciso revisar isso';
    el('btnTranslate').click();
    await flush();

    expect(el('out').textContent).toBe('translated: preciso revisar isso');
    expect(el('outputCard').hidden).toBe(false);
    expect(el('dirLabel').textContent).toBe('PT → EN');
    expect(el('outDir').textContent).toBe('English');
    expect(el('status').textContent).toBe('');
  });

  it('refuses to call the backend with an empty box', async () => {
    const mock = await mountPopup();
    el('src').value = '   ';
    el('btnTranslate').click();
    await flush();

    expect(mock.callsFor('translate')).toHaveLength(0);
    expect(el('status').classList.contains('error')).toBe(true);
  });

  it('translates on Enter but not on Shift+Enter', async () => {
    const mock = await mountPopup();
    el('src').value = 'algo';
    press(el('src'), { key: 'Enter' });
    await flush();
    expect(mock.callsFor('translate')).toHaveLength(1);

    press(el('src'), { key: 'Enter', shiftKey: true });
    await flush();
    expect(mock.callsFor('translate')).toHaveLength(1);
  });

  it('surfaces a backend error through the shared error mapping', async () => {
    await mountPopup({
      translate: () => {
        throw 'no_key: Add your DeepL API key in Settings to translate.';
      }
    });
    el('src').value = 'algo';
    el('btnTranslate').click();
    await flush();

    expect(el('status').textContent).toBe('Add your DeepL API key in Settings to translate.');
    expect(el('status').classList.contains('error')).toBe(true);
  });
});

describe('direction badge', () => {
  it('rests on Auto before anything has been translated', async () => {
    await mountPopup();
    expect(el('dirLabel').textContent).toBe('Auto');
    expect(el('dirBadge').classList.contains('is-forced')).toBe(false);
  });

  it('flips on every single click, never needing a second one to take effect', async () => {
    await mountPopup();
    el('src').value = 'preciso revisar isso';
    el('btnTranslate').click();
    await flush();
    expect(el('dirLabel').textContent).toBe('PT → EN');

    el('dirBadge').click();
    await flush();
    expect(el('dirLabel').textContent).toBe('EN → PT');
    expect(el('dirBadge').classList.contains('is-forced')).toBe(true);

    el('dirBadge').click();
    await flush();
    expect(el('dirLabel').textContent).toBe('PT → EN');

    el('dirBadge').click();
    await flush();
    expect(el('dirLabel').textContent).toBe('EN → PT');
  });

  it('pins the direction it shows, and sends it as the target', async () => {
    const mock = await mountPopup();
    el('src').value = 'algo';
    el('btnTranslate').click();
    await flush();

    el('dirBadge').click();
    await flush();

    const last = mock.callsFor('translate').at(-1);
    expect(last.args.target).toBe('PT');
  });

  it('falls back to Auto once the text no longer matches the result', async () => {
    await mountPopup();
    el('src').value = 'preciso revisar isso';
    el('btnTranslate').click();
    await flush();
    expect(el('dirLabel').textContent).toBe('PT → EN');

    el('src').value = 'something completely different';
    el('src').dispatchEvent(new Event('input'));
    await flush();
    expect(el('dirLabel').textContent).toBe('Auto');
  });
});

describe('clipboard on open', () => {
  it('translates the clipboard when the backend hands one over', async () => {
    const mock = await mountPopup();
    await mock.emit('popup-shown', { clipboard: 'texto da area de transferencia' });
    await flush();

    expect(el('src').value).toBe('texto da area de transferencia');
    expect(mock.callsFor('translate')).toHaveLength(1);
  });

  it('never spends a DeepL call when the clipboard payload is empty', async () => {
    const mock = await mountPopup();
    await mock.emit('popup-shown', { clipboard: '' });
    await flush();

    expect(mock.callsFor('translate')).toHaveLength(0);
  });
});

describe('history', () => {
  it('renders entries newest-detail first with both languages', async () => {
    await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    const items = document.querySelectorAll('.history-item');
    expect(items).toHaveLength(2);
    expect(items[0].querySelector('.history-pair').textContent).toBe('PT → EN');
    expect(items[0].querySelector('.history-translated').textContent).toBe(
      'I need to review this pull request'
    );
  });

  it('marks pinned entries so they read differently', async () => {
    await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    const [pinned, plain] = document.querySelectorAll('.history-item');
    expect(pinned.querySelector('[data-action=pin]').classList.contains('is-pinned')).toBe(true);
    expect(plain.querySelector('[data-action=pin]').classList.contains('is-pinned')).toBe(false);
  });

  it('filters on both the source and the translation', async () => {
    await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    el('historySearch').value = 'runner';
    el('historySearch').dispatchEvent(new Event('input'));
    await flush();
    expect(document.querySelectorAll('.history-item')).toHaveLength(1);

    el('historySearch').value = 'revisar';
    el('historySearch').dispatchEvent(new Event('input'));
    await flush();
    expect(document.querySelectorAll('.history-item')).toHaveLength(1);
  });

  it('explains an empty search differently from an empty history', async () => {
    await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    el('historySearch').value = 'zzzzz';
    el('historySearch').dispatchEvent(new Event('input'));
    await flush();
    expect(document.querySelector('.empty-state').textContent).toBe('Nothing matches that search.');

    await mountPopup({ get_history: () => [] });
    el('btnHistory').click();
    await flush();
    expect(document.querySelector('.empty-state').textContent).toBe(
      'Translations you make will show up here.'
    );
  });

  it('reopens a translation when its row is clicked', async () => {
    await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    document.querySelector('.history-item .history-body').click();
    await flush();

    expect(el('mainView').hidden).toBe(false);
    expect(el('src').value).toBe('Preciso revisar esse pull request');
    expect(el('out').textContent).toBe('I need to review this pull request');
    expect(el('dirLabel').textContent).toBe('PT → EN');
  });

  it('routes the row action buttons to the matching command', async () => {
    const mock = await mountPopup({ get_history: () => sampleHistory() });
    el('btnHistory').click();
    await flush();

    document.querySelector('[data-action=pin]').click();
    await flush();
    expect(mock.callsFor('toggle_history_pin')[0].args).toEqual({ id: '1' });

    el('btnClearHistory').click();
    await flush();
    expect(mock.callsFor('clear_history')).toHaveLength(1);
  });
});

describe('shortcut recorder', () => {
  it('rejects a bare letter that would swallow the key system-wide', async () => {
    await mountPopup();
    el('hotkey').click();
    press(document, { code: 'KeyG', key: 'g' });

    expect(el('hotkey').textContent).toBe('Add Ctrl, Alt or Shift…');
    expect(el('hotkey').dataset.value).toBe('Alt+R');
  });

  it('records a modified combination', async () => {
    await mountPopup();
    el('hotkey').click();
    press(document, { code: 'KeyG', key: 'g', ctrlKey: true, shiftKey: true });

    expect(el('hotkey').dataset.value).toBe('Ctrl+Shift+G');
    expect(el('hotkey').classList.contains('is-recording')).toBe(false);
  });

  it('allows a bare function key', async () => {
    await mountPopup();
    el('hotkey').click();
    press(document, { code: 'F9', key: 'F9' });
    expect(el('hotkey').dataset.value).toBe('F9');
  });

  it('leaves the old combo untouched when cancelled with Escape', async () => {
    await mountPopup();
    el('hotkey').click();
    press(document, { code: 'Escape', key: 'Escape' });

    expect(el('hotkey').dataset.value).toBe('Alt+R');
    expect(el('hotkey').classList.contains('is-recording')).toBe(false);
  });
});

describe('settings', () => {
  it('tells you which version you are running, so an update can be verified', async () => {
    await mountPopup();
    el('btnSettings').click();
    await flush();

    expect(el('appVersion').textContent).toBe('Lexo 9.9.9');
  });

  it('says nothing rather than lying about the version when the backend cannot answer', async () => {
    await mountPopup({
      app_version: () => {
        throw 'boom';
      }
    });
    el('btnSettings').click();
    await flush();

    expect(el('appVersion').textContent).toBe('');
  });

  it('saves every field and re-registers the shortcuts', async () => {
    const mock = await mountPopup();
    el('btnSettings').click();
    await flush();

    el('deeplKey').value = 'new-key:fx';
    el('autoTranslateClipboard').checked = true;
    el('btnSaveSettings').click();
    await flush();

    const saved = mock.callsFor('save_settings')[0].args.settings;
    expect(saved.deeplKey).toBe('new-key:fx');
    expect(saved.autoTranslateClipboard).toBe(true);
    expect(saved.hotkey).toBe('Alt+R');
    expect(mock.callsFor('update_shortcuts')).toHaveLength(1);
  });

  it('keeps you on the panel when a shortcut will not register', async () => {
    await mountPopup({
      update_shortcuts: () => {
        throw 'Open: could not register Alt+R (already in use)';
      }
    });
    el('btnSettings').click();
    await flush();
    el('btnSaveSettings').click();
    await flush();

    expect(el('settingsView').hidden).toBe(false);
    expect(el('settingsStatus').classList.contains('error')).toBe(true);
  });

  it('shows the DeepL quota as a filled bar', async () => {
    await mountPopup();
    el('btnSettings').click();
    await flush();

    expect(el('usage').hidden).toBe(false);
    expect(el('usageText').textContent).toBe('412k / 500k characters this month');
    expect(parseFloat(el('usageFill').style.width)).toBeCloseTo(82.469, 3);
  });

  it('toggles the key between hidden and visible', async () => {
    await mountPopup();
    el('btnSettings').click();
    await flush();

    expect(el('deeplKey').type).toBe('password');
    el('btnRevealKey').click();
    expect(el('deeplKey').type).toBe('text');
    el('btnRevealKey').click();
    expect(el('deeplKey').type).toBe('password');
  });
});

describe('navigation', () => {
  it('toggles a panel open and closed from its title-bar button', async () => {
    await mountPopup();
    el('btnHistory').click();
    await flush();
    expect(el('historyView').hidden).toBe(false);

    el('btnHistory').click();
    await flush();
    expect(el('mainView').hidden).toBe(false);
    expect(el('historyView').hidden).toBe(true);
  });

  it('sends Escape back to the translator before closing the window', async () => {
    const mock = await mountPopup();
    el('btnSettings').click();
    await flush();

    press(document, { key: 'Escape' });
    await flush();
    expect(el('mainView').hidden).toBe(false);
    expect(mock.callsFor('hide_popup')).toHaveLength(0);

    press(document, { key: 'Escape' });
    await flush();
    expect(mock.callsFor('hide_popup')).toHaveLength(1);
  });

  it('opens the view the tray asked for', async () => {
    const mock = await mountPopup();
    await mock.emit('open-view', 'settings');
    await flush();
    expect(el('settingsView').hidden).toBe(false);
  });
});
