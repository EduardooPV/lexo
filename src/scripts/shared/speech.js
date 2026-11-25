import { baseOf } from './languages.js';

let voices = [];
const LOCALES = new Map([
  ['EN', 'en-US'],
  ['PT', 'pt-BR'],
  ['ZH', 'zh-CN'],
  ['ZH-HANS', 'zh-CN'],
  ['ZH-HANT', 'zh-TW']
]);

export function loadVoices() {
  try { voices = window.speechSynthesis.getVoices() || []; } catch (_) { voices = []; }
}

function scoreVoice(voice) {
  const name = (voice.name || '').toLowerCase();
  let score = 0;
  if (name.includes('natural')) score += 5;
  if (name.includes('online')) score += 3;
  if (name.includes('neural') || name.includes('enhanced') || name.includes('premium')) score += 3;
  if (name.includes('desktop')) score -= 2;
  return score;
}

export function speak(text, language) {
  if (!text || !('speechSynthesis' in window)) return;
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.lang = speechLocale(language);
  const candidates = voices.filter((voice) => baseOf(voice.lang) === baseOf(utterance.lang));
  const exact = candidates.filter((voice) => voice.lang.toLowerCase() === utterance.lang.toLowerCase());
  const match = (exact.length ? exact : candidates).sort((a, b) => scoreVoice(b) - scoreVoice(a))[0];
  if (match) utterance.voice = match;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(utterance);
}

export function speechLocale(code) {
  const normalized = (code || 'EN').toUpperCase();
  return LOCALES.get(normalized) || Intl.getCanonicalLocales(normalized)[0];
}

export function detectedSpeechLanguage(detected, primary, secondary) {
  if (!detected || detected.includes('-')) return detected;
  return [primary, secondary].find((code) => baseOf(code) === baseOf(detected)) || detected;
}

export function guessSpokenLanguage(text, primary, secondary) {
  const pair = [primary, secondary];
  const portuguese = pair.find((code) => baseOf(code) === 'PT');
  const english = pair.find((code) => baseOf(code) === 'EN');
  if (!portuguese || !english) return primary;
  return /[ãõáàâêôçéíóú]/i.test(text || '') ? portuguese : english;
}

export function isSpeechAvailable() {
  return 'speechSynthesis' in window;
}
