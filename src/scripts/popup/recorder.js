import { el } from './dom.js';

export const RECORDERS = ['hotkey', 'selectionHotkey', 'replaceHotkey', 'ocrHotkey', 'swapHotkey'];

const NAMED_KEYS = {
  Space: 'Space', Enter: 'Enter', Tab: 'Tab', Backspace: 'Backspace', Delete: 'Delete',
  Insert: 'Insert', Home: 'Home', End: 'End', PageUp: 'PageUp', PageDown: 'PageDown',
  ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right'
};

let recording = null;

export function isRecording() {
  return Boolean(recording);
}

export function keyFromEvent(event) {
  const code = event.code || '';
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  return NAMED_KEYS[code] || null;
}

function stopRecording() {
  if (!recording) return;
  recording.classList.remove('is-recording');
  recording.textContent = recording.dataset.value;
  recording = null;
  document.removeEventListener('keydown', onRecordKey, true);
}

function onRecordKey(event) {
  event.preventDefault();
  event.stopPropagation();

  if (event.key === 'Escape') {
    stopRecording();
    return;
  }

  const key = keyFromEvent(event);
  if (!key) return;

  const parts = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  if (!parts.length && !/^F\d{1,2}$/.test(key)) {
    recording.textContent = 'Add Ctrl, Alt or Shift…';
    return;
  }

  parts.push(key);
  const target = recording;
  target.dataset.value = parts.join('+');
  stopRecording();
}

export function startRecording(button) {
  if (recording === button) {
    stopRecording();
    return;
  }
  stopRecording();
  recording = button;
  button.classList.add('is-recording');
  button.textContent = 'Press a combination…';
  document.addEventListener('keydown', onRecordKey, true);
}

export function setRecorder(id, value) {
  const button = el(id);
  button.dataset.value = value;
  button.textContent = value;
}

export function matchesHotkey(event, combo) {
  if (!combo) return false;
  const parts = combo.split('+').map((part) => part.trim().toLowerCase()).filter(Boolean);
  if (!parts.length) return false;

  const key = parts[parts.length - 1];
  const need = { alt: false, ctrl: false, shift: false, meta: false };
  parts.slice(0, -1).forEach((modifier) => {
    if (modifier === 'alt' || modifier === 'option') need.alt = true;
    else if (['ctrl', 'control', 'commandorcontrol', 'cmdorctrl'].includes(modifier)) need.ctrl = true;
    else if (modifier === 'shift') need.shift = true;
    else if (['meta', 'cmd', 'command', 'super', 'win'].includes(modifier)) need.meta = true;
  });

  if (event.altKey !== need.alt || event.ctrlKey !== need.ctrl ||
      event.shiftKey !== need.shift || event.metaKey !== need.meta) return false;

  return (keyFromEvent(event) || '').toLowerCase() === key;
}
