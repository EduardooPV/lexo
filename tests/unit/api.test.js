import { describe, it, expect, beforeEach, vi } from 'vitest';
import { installTauriMock } from '../helpers/tauri.js';

const importApi = async () => {
  vi.resetModules();
  return import('../../src/scripts/shared/api.js');
};

describe('describeError', () => {
  it('maps a known backend prefix to its human message', async () => {
    const { describeError } = await importApi();
    expect(describeError('no_key: Add your DeepL API key in Settings to translate.')).toBe(
      'Add your DeepL API key in Settings to translate.'
    );
    expect(describeError('deepl_auth: DeepL rejected the key.')).toBe(
      'DeepL rejected the key. Check it in Settings.'
    );
  });

  it('passes the backend wording through for quota errors', async () => {
    const { describeError } = await importApi();
    expect(
      describeError('limit: DeepL monthly character quota reached. It resets next month.')
    ).toBe('DeepL monthly character quota reached. It resets next month.');
  });

  it('keeps the underlying detail on connection failures', async () => {
    const { describeError } = await importApi();
    expect(describeError('network_error: connection refused')).toBe(
      'Connection to DeepL failed. connection refused'
    );
    expect(describeError('http_error: DeepL http 502')).toBe(
      'Connection to DeepL failed. DeepL http 502'
    );
  });

  it('distinguishes the OCR failure modes from each other', async () => {
    const { describeError } = await importApi();
    expect(describeError('region_too_small: drag a larger area.')).toBe(
      'That region is too small — drag a larger area.'
    );
    expect(describeError('capture_error: could not read that screen region.')).toBe(
      'Could not read that screen region.'
    );
    expect(describeError('ocr_error: recognize (something)')).toBe(
      'Could not read the text in that region.'
    );
    expect(describeError('ocr_unsupported: only on Windows.')).toBe(
      'Screen OCR is only available on Windows.'
    );
  });

  it('prefers api_error over the bare word "empty" that its message contains', async () => {
    const { describeError } = await importApi();
    expect(describeError('api_error: empty response from DeepL')).toBe(
      "Couldn't translate that text. Try rephrasing."
    );
  });

  it('strips the prefix from unrecognised errors instead of leaking it', async () => {
    const { describeError } = await importApi();
    expect(describeError('some_unknown_error: the real detail')).toBe('the real detail');
  });

  it('survives null and undefined', async () => {
    const { describeError } = await importApi();
    expect(describeError(null)).toBe('');
    expect(describeError(undefined)).toBe('');
  });
});

describe('command wrappers', () => {
  let mock;

  beforeEach(() => {
    mock = installTauriMock();
  });

  it('lets DeepL pick the direction when no target is given', async () => {
    const api = await importApi();
    await api.translate('bom dia');
    expect(mock.callsFor('translate')[0].args).toEqual({ text: 'bom dia', target: null });
  });

  it('forwards a pinned direction as the target', async () => {
    const api = await importApi();
    await api.translate('good morning', 'PT');
    expect(mock.callsFor('translate')[0].args).toEqual({ text: 'good morning', target: 'PT' });
  });

  it('names every argument the way the Rust commands expect', async () => {
    const api = await importApi();
    await api.deleteHistoryEntry('123');
    await api.toggleHistoryPin('456');
    await api.setClipboard('text');
    await api.resizeWindow(300);
    await api.setAutostart(true);

    expect(mock.callsFor('delete_history_entry')[0].args).toEqual({ id: '123' });
    expect(mock.callsFor('toggle_history_pin')[0].args).toEqual({ id: '456' });
    expect(mock.callsFor('set_clipboard')[0].args).toEqual({ text: 'text' });
    expect(mock.callsFor('resize_window')[0].args).toEqual({ height: 300 });
    expect(mock.callsFor('set_autostart')[0].args).toEqual({ enabled: true });
  });

  it('sends the OCR rectangle as flat x/y/width/height', async () => {
    const api = await importApi();
    await api.ocrRegion({ x: 10, y: 20, width: 100, height: 50 });
    expect(mock.callsFor('ocr_region')[0].args).toEqual({
      x: 10,
      y: 20,
      width: 100,
      height: 50
    });
  });
});
