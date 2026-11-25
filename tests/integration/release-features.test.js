import { expect, it } from 'vitest';
import { el, flush, mountMini, mountPopup } from '../helpers/mount.js';
import { DEFAULT_SETTINGS, sampleHistory } from '../helpers/tauri.js';

function changeTarget(target) {
  el('targetSelect').value = target;
  el('targetSelect').dispatchEvent(new Event('change'));
}

const LONG_TEXT = 'a'.repeat(5001);

function longTextHandler({ text, target, confirmed }) {
  if (text.length > 5000 && !confirmed) throw `too_long: ${text.length}`;
  return { text: 'Translated', detectedSource: 'DE', target: target || 'EN-US' };
}

it('cycles between the saved languages and synchronizes the explicit target selector', async () => {
  const mock = await mountPopup({
    get_settings: () => ({ ...DEFAULT_SETTINGS, primaryLang: 'DE', secondaryLang: 'ES' })
  });
  expect(el('targetSelect').value).toBe('');
  el('dirBadge').click();
  expect(el('targetSelect').value).toBe('DE');
  expect(el('dirLabel').textContent).toBe('ES → DE');
  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'e', code: 'KeyE', altKey: true }));
  expect(el('targetSelect').value).toBe('ES');
  expect(el('dirLabel').textContent).toBe('DE → ES');

  changeTarget('ZH-HANT');
  el('src').value = 'Guten Morgen';
  el('btnTranslate').click();
  await flush();
  expect(mock.callsFor('translate').at(-1).args.target).toBe('ZH-HANT');
  expect(el('outDir').textContent).toBe('Chinese (traditional)');

  changeTarget('');
  await flush();
  expect(mock.callsFor('translate').at(-1).args.target).toBeNull();
  expect(el('targetSelect').value).toBe('');
  expect(el('dirBadge').classList.contains('is-forced')).toBe(false);
});

it('loads and saves language settings and rejects a pair with the same base', async () => {
  const mock = await mountPopup({
    get_settings: () => ({
      ...DEFAULT_SETTINGS, primaryLang: 'EN-GB', secondaryLang: 'JA', formality: 'less'
    })
  });
  el('btnSettings').click();
  expect(el('primaryLang').value).toBe('EN-GB');
  expect(el('secondaryLang').value).toBe('JA');
  expect(el('formality').value).toBe('less');
  el('secondaryLang').value = 'EN-US';
  el('btnSaveSettings').click();
  expect(el('settingsStatus').textContent).toContain('Choose two different languages.');
  expect(mock.callsFor('save_settings')).toHaveLength(0);

  el('secondaryLang').value = 'FR';
  el('formality').value = 'more';
  el('btnSaveSettings').click();
  await flush();
  expect(mock.callsFor('save_settings').at(-1).args.settings).toMatchObject({
    primaryLang: 'EN-GB', secondaryLang: 'FR', formality: 'more'
  });
  el('btnSettings').click();
  el('btnResetAll').click();
  await flush();
  expect(mock.callsFor('save_settings').at(-1).args.settings).toMatchObject({
    primaryLang: 'PT-BR', secondaryLang: 'EN-US', formality: 'default'
  });
});

it('retries long popup input only on confirmation and resets when the source is edited', async () => {
  const mock = await mountPopup({ translate: longTextHandler });
  el('src').value = LONG_TEXT;
  el('btnTranslate').click();
  await flush();
  expect(mock.callsFor('translate')[0].args.confirmed).toBe(false);
  expect(el('btnTranslate').textContent).toBe('Translate anyway');
  expect(el('status').textContent).toContain('5001 characters');

  el('src').value = LONG_TEXT + 'b';
  el('src').dispatchEvent(new Event('input'));
  expect(el('btnTranslate').textContent).toBe('Translate');
  el('btnTranslate').click();
  await flush();
  expect(mock.callsFor('translate')[1].args.confirmed).toBe(false);
  expect(el('status').textContent).toContain('5002 characters');

  el('btnTranslate').click();
  await flush();
  expect(mock.callsFor('translate')[2].args).toMatchObject({ text: LONG_TEXT + 'b', confirmed: true });
  expect(el('btnTranslate').textContent).toBe('Translate');
  expect(el('out').textContent).toBe('Translated');
});

it('confirms a long mini selection but leaves a rejected replace unconfirmed', async () => {
  const mock = await mountMini({ translate: longTextHandler });
  await mock.emit('mini-translate', { text: LONG_TEXT });
  await flush();
  expect(el('miniConfirm').hidden).toBe(false);
  expect(el('miniStatus').textContent).toContain('5001 characters');
  expect(mock.callsFor('translate')[0].args.confirmed).toBe(false);

  el('miniConfirm').click();
  await flush();
  expect(mock.callsFor('translate')[1].args).toEqual({ text: LONG_TEXT, target: null, confirmed: true });
  expect(el('outText').textContent).toBe('Translated');
  expect(el('miniConfirm').hidden).toBe(true);

  await mock.emit('mini-translate', {
    error: 'replace_too_long: Selection too long to replace automatically. Use the translate shortcut instead.'
  });
  await flush();
  expect(el('miniConfirm').hidden).toBe(true);
  expect(el('miniStatus').textContent).toContain('Use the translate shortcut instead.');
  expect(mock.callsFor('translate')).toHaveLength(2);
});

it('shows quota from either refresh or the event and warns at 80 percent', async () => {
  const mock = await mountPopup({
    get_usage: () => ({ characterCount: 62, characterLimit: 100 })
  });
  const hint = el('usageHint');
  expect(hint.hidden).toBe(true);
  el('btnSettings').click();
  await flush();
  expect(hint.hidden).toBe(false);
  expect(hint.textContent).toBe('62% of your monthly quota used');
  expect(hint.classList.contains('is-high')).toBe(false);
  expect(hint.hasAttribute('aria-live')).toBe(false);

  await mock.emit('usage-updated', { characterCount: 80, characterLimit: 100 });
  expect(hint.textContent).toBe('80% of your monthly quota used');
  expect(hint.classList.contains('is-high')).toBe(true);
  await mock.emit('usage-updated', { characterCount: 0, characterLimit: 0 });
  expect(hint.hidden).toBe(true);
  await flush();
});

it('hides the quota hint when no API key is saved', async () => {
  const mock = await mountPopup({ get_settings: () => ({ ...DEFAULT_SETTINGS, deeplKey: '' }) });
  await mock.emit('usage-updated', { characterCount: 80, characterLimit: 100 });
  expect(el('usageHint').hidden).toBe(true);
  await flush();
});

it('enables exports only when history exists and handles dialog cancellation', async () => {
  let entries = [];
  const mock = await mountPopup({
    get_history: () => entries,
    export_history: ({ format }) => format === 'csv' ? null : 'lexo-history.json'
  });
  el('btnHistory').click();
  await flush();
  expect(el('btnExportJson').disabled).toBe(true);
  expect(el('btnExportCsv').disabled).toBe(true);
  entries = sampleHistory();
  await mock.emit('open-view', 'history');
  await flush();
  el('historySearch').value = 'no such translation';
  el('historySearch').dispatchEvent(new Event('input'));
  expect(el('btnExportJson').disabled).toBe(false);
  expect(el('btnExportCsv').disabled).toBe(false);

  el('btnExportCsv').click();
  expect(el('btnExportJson').disabled).toBe(true);
  await flush();
  expect(el('historyStatus').textContent).toBe('');
  expect(el('btnExportJson').disabled).toBe(false);
  el('btnExportJson').click();
  await flush();
  expect(el('historyStatus').textContent).toBe('Exported!');
  expect(mock.callsFor('export_history').map(({ args }) => args)).toEqual([
    { format: 'csv' }, { format: 'json' }
  ]);
});

it('still auto-fits a shorter view after the native window height is capped', async () => {
  const mock = await mountPopup({ resize_window: ({ height }) => Math.min(height, 480) });
  Object.defineProperty(el('app'), 'scrollHeight', {
    get: () => el('settingsView').hidden ? 280 : 720
  });
  const initialHeight = window.innerHeight;
  try {
    el('btnSettings').click();
    await flush();
    expect(mock.callsFor('resize_window').at(-1).args.height).toBe(720);
    window.innerHeight = 480;
    window.dispatchEvent(new Event('resize'));
    await new Promise((resolve) => setTimeout(resolve, 160));
    el('btnBackSettings').click();
    await flush();
    expect(mock.callsFor('resize_window').at(-1).args.height).toBe(280);
  } finally {
    window.innerHeight = initialHeight;
  }
});
