import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mountMini, flush, el } from '../helpers/mount.js';

const VOICES = [
  { name: 'Microsoft Maria Desktop', lang: 'pt-BR' },
  { name: 'Microsoft Francisca Online (Natural)', lang: 'pt-BR' },
  { name: 'Microsoft Zira Desktop', lang: 'en-US' },
  { name: 'Google US English', lang: 'en-US' }
];

let spoken;

function stubSpeech(voices = VOICES) {
  spoken = [];
  window.SpeechSynthesisUtterance = class {
    constructor(text) {
      this.text = text;
      this.lang = '';
      this.voice = null;
    }
  };
  window.speechSynthesis = {
    getVoices: () => voices,
    addEventListener: () => {},
    cancel: () => {},
    speak: (utterance) => spoken.push(utterance)
  };
}

beforeEach(() => stubSpeech());

afterEach(() => {
  delete window.speechSynthesis;
  delete window.SpeechSynthesisUtterance;
});

describe('voice selection', () => {
  it('prefers a Natural/Online voice over a legacy Desktop one', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'preciso revisar', origin: 'selection' });
    await flush();

    el('speakSrc').click();

    expect(spoken).toHaveLength(1);
    expect(spoken[0].voice.name).toBe('Microsoft Francisca Online (Natural)');
    expect(spoken[0].lang).toBe('pt-BR');
  });

  it('speaks the translation in the target language, not the source', async () => {
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'preciso revisar', origin: 'selection' });
    await flush();

    el('speakOut').click();

    expect(spoken[0].text).toBe('translated: preciso revisar');
    expect(spoken[0].lang).toBe('en-US');
    expect(spoken[0].voice.lang).toBe('en-US');
  });

  it('falls back gracefully when no voice matches the language', async () => {
    stubSpeech([{ name: 'Some Japanese Voice', lang: 'ja-JP' }]);
    const mock = await mountMini();
    await mock.emit('mini-translate', { text: 'algo', origin: 'selection' });
    await flush();

    el('speakOut').click();

    expect(spoken).toHaveLength(1);
    expect(spoken[0].voice).toBeNull();
    expect(spoken[0].lang).toBe('en-US');
  });

  it('says nothing when there is nothing to say', async () => {
    await mountMini();
    el('speakOut').click();
    expect(spoken).toHaveLength(0);
  });
});
