let voices = [];

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
  utterance.lang = language === 'pt' ? 'pt-BR' : 'en-US';
  const match = voices
    .filter((voice) => voice.lang && voice.lang.toLowerCase().startsWith(language))
    .sort((a, b) => scoreVoice(b) - scoreVoice(a))[0];
  if (match) utterance.voice = match;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(utterance);
}

export function guessSpokenLanguage(text) {
  return /[ãõáàâêôçéíóú]/i.test(text || '') ? 'pt' : 'en';
}

export function isSpeechAvailable() {
  return 'speechSynthesis' in window;
}
