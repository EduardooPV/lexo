import { expect, it } from 'vitest';
import { LANGUAGES, baseOf, languageName } from '../../src/scripts/shared/languages.js';
import { detectedSpeechLanguage, guessSpokenLanguage, speechLocale } from '../../src/scripts/shared/speech.js';

it('names detected base codes and full target codes independently', () => {
  expect(languageName('pt')).toBe('Portuguese');
  expect(languageName('pt-br')).toBe('Portuguese (Brazilian)');
  expect(languageName('EN')).toBe('English');
  expect(languageName('zh-hant')).toBe('Chinese (traditional)');
  expect(languageName('sw')).toBe('Swahili');
  expect(languageName('unexpected')).toBe('unexpected');
  expect(baseOf('es-419')).toBe('ES');
  expect(baseOf('zh-Hans')).toBe('ZH');
  expect(baseOf(null)).toBe('');
});

it('uses unique target codes and keeps legacy EN and PT as source aliases', () => {
  const codes = LANGUAGES.map(({ code }) => code);
  expect(new Set(codes).size).toBe(codes.length);
  expect(codes).toContain('EN-US');
  expect(codes).toContain('PT-BR');
  expect(codes).not.toContain('EN');
  expect(codes).not.toContain('PT');
});

it('uses the correct locale for speech and honors a detected language variant', () => {
  expect(speechLocale('PT-BR')).toBe('pt-BR');
  expect(speechLocale('ZH-HANS')).toBe('zh-CN');
  expect(speechLocale('ZH-HANT')).toBe('zh-TW');
  expect(speechLocale('ES-419')).toBe('es-419');
  expect(detectedSpeechLanguage('PT', 'PT-PT', 'EN-GB')).toBe('PT-PT');
  expect(detectedSpeechLanguage('EN-US', 'PT-PT', 'EN-GB')).toBe('EN-US');
  expect(guessSpokenLanguage('Bom dia, irmão.', 'PT-PT', 'EN-GB')).toBe('PT-PT');
  expect(guessSpokenLanguage('Hello!', 'PT-PT', 'EN-GB')).toBe('EN-GB');
  expect(guessSpokenLanguage('Bonjour', 'FR', 'DE')).toBe('FR');
});
