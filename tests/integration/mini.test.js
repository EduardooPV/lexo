import { describe, it, expect } from 'vitest';
import { mountMini, flush, el } from '../helpers/mount.js';

describe('bubble content', () => {
  it('shows the translation with both detected languages', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'preciso revisar isso', origin: 'selection' });
    await flush();

    expect(el('srcText').textContent).toBe('preciso revisar isso');
    expect(el('outText').textContent).toBe('translated: preciso revisar isso');
    expect(el('outLang').textContent).toBe('English');
    expect(el('miniStatus').textContent).toBe('');
  });

  it('trims the incoming selection before translating it', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: '   com espacos   ', origin: 'selection' });
    await flush();

    expect(mock.callsFor('translate')[0].args.text).toBe('com espacos');
  });

  it('lets DeepL decide the direction, never pinning one', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    expect(mock.callsFor('translate')[0].args.target).toBeNull();
  });
});

describe('empty and error states', () => {
  it('explains an empty selection differently from an empty OCR grab', async () => {
    const mock = await mountMini();

    await mock.emit('mini-translate', { text: '', origin: 'selection' });
    await flush();
    expect(el('miniStatus').textContent).toBe('No text selected — nothing was copied.');

    await mock.emit('mini-translate', { text: '', origin: 'ocr' });
    await flush();
    expect(el('miniStatus').textContent).toBe('No text found in that region.');
  });

  it('never spends a translation on an empty payload', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: '   ', origin: 'selection' });
    await flush();

    expect(mock.callsFor('translate')).toHaveLength(0);
  });

  it('runs a backend error through the same mapping the popup uses', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', {
      text: '',
      origin: 'ocr',
      error: 'ocr_unavailable: no language pack'
    });
    await flush();

    expect(el('miniStatus').textContent).toContain('Windows has no OCR language pack installed');
    expect(el('miniStatus').classList.contains('error')).toBe(true);
  });

  it('reports a failed translation without wiping the source text', async () => {
    const mock = await mountMini({
      translate: () => {
        throw 'limit: DeepL monthly character quota reached.';
      }
    });
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    expect(el('srcText').textContent).toBe('algo');
    expect(el('outText').textContent).toBe('');
    expect(el('miniStatus').textContent).toBe('DeepL monthly character quota reached.');
  });
});

describe('actions', () => {
  it('copies the translation and flips the icon back after the flash', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    el('copyOut').click();
    await flush();

    expect(mock.callsFor('set_clipboard')[0].args.text).toBe('translated: algo');
    expect(el('copyOut').dataset.icon).toBe('check');

    await new Promise((resolve) => setTimeout(resolve, 950));
    expect(el('copyOut').dataset.icon).toBe('copy');
  });

  it('does nothing when there is no translation to copy', async () => {
    const mock = await mountMini();
    el('copyOut').click();
    await flush();
    expect(mock.callsFor('set_clipboard')).toHaveLength(0);
  });

  it('closes from the button and from Escape', async () => {
    const mock = await mountMini();

    el('close').click();
    await flush();
    expect(mock.callsFor('hide_mini')).toHaveLength(1);

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await flush();
    expect(mock.callsFor('hide_mini')).toHaveLength(2);
  });

  it('copies with Ctrl+C so you do not have to aim at the button', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'c', ctrlKey: true, bubbles: true })
    );
    await flush();

    expect(mock.callsFor('set_clipboard')[0].args.text).toBe('translated: algo');
  });
});

describe('window sizing', () => {
  it('asks the backend to refit the bubble after rendering', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    expect(mock.callsFor('resize_mini').length).toBeGreaterThan(0);
  });
});
