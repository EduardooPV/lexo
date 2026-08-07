import { el, src, status } from './dom.js';
import { state } from './state.js';
import { translate } from './translate.js';
import { fail, fitWindow } from './views.js';

const SpeechRec = window.SpeechRecognition || window.webkitSpeechRecognition;

let recognition = null;
let micTimer = null;

function setMic(active) {
  el('btnMic').classList.toggle('is-active', active);
  if (!active) clearTimeout(micTimer);
}

function stopMic() {
  clearTimeout(micTimer);
  try { recognition.abort(); } catch (_) {}
  try { recognition.stop(); } catch (_) {}
  setMic(false);
}

export function wireVoiceInput() {
  if (!SpeechRec) {
    el('btnMic').hidden = true;
    return;
  }

  recognition = new SpeechRec();
  recognition.interimResults = false;
  recognition.maxAlternatives = 1;
  recognition.continuous = false;
  recognition.onstart = () => setMic(true);
  recognition.onend = () => setMic(false);
  recognition.onresult = (event) => {
    setMic(false);
    src.value = event.results[0][0].transcript;
    translate();
  };
  recognition.onerror = (event) => {
    setMic(false);
    fail(status, 'Voice input unavailable (' + (event.error || 'error') + ').');
    fitWindow();
  };

  el('btnMic').addEventListener('click', () => {
    if (el('btnMic').classList.contains('is-active')) {
      stopMic();
      return;
    }
    recognition.lang =
      state.forcedTarget === 'EN' ? 'pt-BR' : state.forcedTarget === 'PT' ? 'en-US' : 'pt-BR';
    try {
      recognition.start();
      setMic(true);
      micTimer = setTimeout(stopMic, 12000);
    } catch (_) {
      setMic(false);
    }
  });
}
