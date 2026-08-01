// Tauri global API (withGlobalTauri = true)
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const el = (id) => document.getElementById(id);

const src = el('src');
const out = el('out');
const outputCard = el('outputCard');
const status = el('status');
const btnTranslate = el('btnTranslate');
const dirBadge = el('dirBadge');
const outDir = el('outDir');
const btnCopy = el('btnCopy');

const mainView = el('mainView');
const settingsView = el('settingsView');
const appearanceView = el('appearanceView');

// Defaults: Dracula theme
const DEFAULT_APPEARANCE = {
  opacity: 1,
  bg: '#282a36',
  accent: '#bd93f9',
  text: '#f8f8f2',
  font: "'Segoe UI', system-ui, sans-serif"
};

let settings = {
  hotkey: 'Alt+R',
  swapHotkey: 'Alt+E',
  selectionHotkey: 'Alt+T',
  autoTranslateClipboard: true,
  deeplKey: '',
  appearance: { ...DEFAULT_APPEARANCE }
};

// null = auto-detect; otherwise a forced direction: 'pt-en' or 'en-pt'
let forcedDir = null;
// Direction of the last produced translation (for the output speaker)
let lastResultDir = 'pt-en';

// Pick white or near-black for text over `hex`, whichever has more contrast.
function readableOn(hex) {
  const c = hex.replace('#', '');
  if (c.length < 6) return '#ffffff';
  const ch = (i) => parseInt(c.substr(i, 2), 16) / 255;
  const lin = (x) => (x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4));
  const L = 0.2126 * lin(ch(0)) + 0.7152 * lin(ch(2)) + 0.0722 * lin(ch(4));
  return 1.05 / (L + 0.05) >= (L + 0.05) / 0.05 ? '#ffffff' : '#1b1b1f';
}

/* ---------- Fit window to content (no empty space) ---------- */
function fitWindow() {
  requestAnimationFrame(() => {
    const h = Math.ceil(document.getElementById('app').scrollHeight);
    invoke('resize_window', { height: h }).catch(() => {});
  });
}

/* ---------- Transient status toast ---------- */
function toast(elem, text, ms = 2200) {
  elem.classList.remove('error');
  elem.textContent = text;
  clearTimeout(elem._t);
  elem._t = setTimeout(() => { elem.textContent = ''; }, ms);
}

/* ---------- Appearance ---------- */
function applyAppearance(a) {
  const r = document.documentElement.style;
  r.setProperty('--parchment', a.bg);
  r.setProperty('--gold', a.accent);
  r.setProperty('--ink', a.text);
  r.setProperty('--ui-font', a.font);
  r.setProperty('--app-opacity', String(a.opacity));
  r.setProperty('--on-accent', readableOn(a.accent));
}
function appearanceFromControls() {
  return {
    opacity: parseFloat(el('opacity').value),
    bg: el('colBg').value,
    accent: el('colAccent').value,
    text: el('colText').value,
    font: el('font').value
  };
}
function appearanceToControls(a) {
  el('opacity').value = a.opacity;
  el('opacityVal').textContent = Math.round(a.opacity * 100) + '%';
  el('colBg').value = a.bg;
  el('colAccent').value = a.accent;
  el('colText').value = a.text;
  const sel = el('font');
  if (![...sel.options].some(o => o.value === a.font)) {
    const opt = document.createElement('option');
    opt.value = a.font; opt.textContent = 'Custom';
    sel.appendChild(opt);
  }
  sel.value = a.font;
}
function previewAppearance() {
  const a = appearanceFromControls();
  el('opacityVal').textContent = Math.round(a.opacity * 100) + '%';
  applyAppearance(a);
  syncThemeSelect();
}
['opacity', 'colBg', 'colAccent', 'colText', 'font'].forEach(id => {
  el(id).addEventListener('input', previewAppearance);
});

/* ---------- Theme presets (popular developer color schemes) ---------- */
const THEMES = [
  { name: 'Dracula (default)', bg: '#282a36', accent: '#bd93f9', text: '#f8f8f2' },
  { name: 'Nord',             bg: '#2e3440', accent: '#88c0d0', text: '#eceff4' },
  { name: 'One Dark',         bg: '#282c34', accent: '#61afef', text: '#abb2bf' },
  { name: 'Tokyo Night',      bg: '#1a1b26', accent: '#7aa2f7', text: '#c0caf5' },
  { name: 'Monokai',          bg: '#272822', accent: '#a6e22e', text: '#f8f8f2' },
  { name: 'Gruvbox',          bg: '#282828', accent: '#fabd2f', text: '#ebdbb2' },
  { name: 'Catppuccin',       bg: '#1e1e2e', accent: '#cba6f7', text: '#cdd6f4' },
  { name: 'Night Owl',        bg: '#011627', accent: '#82aaff', text: '#d6deeb' },
  { name: 'Solarized Dark',   bg: '#002b36', accent: '#268bd2', text: '#93a1a1' },
  { name: 'Solarized Light',  bg: '#fdf6e3', accent: '#268bd2', text: '#586e75' },
  { name: 'GitHub Light',     bg: '#ffffff', accent: '#0969da', text: '#1f2328' }
];

function syncThemeSelect() {
  const sel = el('themeSelect');
  if (!sel) return;
  const bg = (el('colBg').value || '').toLowerCase();
  const ac = (el('colAccent').value || '').toLowerCase();
  const tx = (el('colText').value || '').toLowerCase();
  const idx = THEMES.findIndex(t => t.bg === bg && t.accent === ac && t.text === tx);
  sel.value = idx >= 0 ? String(idx) : 'custom';
}

function buildThemeSelect() {
  const sel = el('themeSelect');
  if (!sel) return;
  sel.innerHTML = '';
  THEMES.forEach((t, i) => {
    const o = document.createElement('option');
    o.value = String(i);
    o.textContent = t.name;
    sel.appendChild(o);
  });
  const custom = document.createElement('option');
  custom.value = 'custom';
  custom.textContent = 'Custom';
  sel.appendChild(custom);
  sel.addEventListener('change', () => {
    if (sel.value === 'custom') return;
    const t = THEMES[+sel.value];
    el('colBg').value = t.bg;
    el('colAccent').value = t.accent;
    el('colText').value = t.text;
    previewAppearance();
  });
}
buildThemeSelect();

/* ---------- View switching ---------- */
function showMain() {
  settingsView.hidden = true;
  appearanceView.hidden = true;
  mainView.hidden = false;
  src.focus();
  src.select();
  fitWindow();
}
function showSettings() {
  mainView.hidden = true;
  appearanceView.hidden = true;
  settingsView.hidden = false;
  el('settingsStatus').textContent = '';
  el('hotkey').focus();
  fitWindow();
}
function showAppearance() {
  mainView.hidden = true;
  settingsView.hidden = true;
  appearanceView.hidden = false;
  el('appearanceStatus').textContent = '';
  appearanceToControls(settings.appearance);
  applyAppearance(settings.appearance);
  syncThemeSelect();
  fitWindow();
}

/* ---------- Language direction ---------- */
const PT_WORDS = [' o ', ' a ', ' os ', ' as ', ' um ', ' uma ', ' de ', ' do ', ' da ', ' dos ', ' das ', ' em ', ' no ', ' na ', ' que ', ' e ', ' ou ', ' mas ', ' com ', ' sem ', ' por ', ' para ', ' se ', ' eu ', ' voce ', ' ele ', ' ela ', ' nos ', ' eles ', ' meu ', ' minha ', ' seu ', ' sua ', ' isso ', ' isto ', ' aqui ', ' ali ', ' nao ', ' sim ', ' muito ', ' mais ', ' menos ', ' tudo ', ' nada ', ' bom ', ' boa ', ' dia ', ' noite ', ' obrigado ', ' obrigada ', ' ola ', ' porque ', ' quando ', ' onde ', ' como ', ' quem ', ' qual ', ' ser ', ' estar ', ' ter ', ' fazer ', ' vou ', ' vai ', ' esta ', ' sao ', ' foi ', ' tem ', ' quero ', ' preciso ', ' gosto ', ' casa ', ' agua ', ' hoje ', ' amanha '];
const EN_WORDS = [' the ', ' an ', ' of ', ' to ', ' in ', ' on ', ' at ', ' is ', ' are ', ' was ', ' were ', ' be ', ' been ', ' and ', ' or ', ' but ', ' with ', ' without ', ' for ', ' if ', ' i ', ' you ', ' he ', ' she ', ' we ', ' they ', ' it ', ' my ', ' your ', ' his ', ' her ', ' this ', ' that ', ' these ', ' those ', ' here ', ' there ', ' no ', ' yes ', ' not ', ' very ', ' more ', ' less ', ' all ', ' nothing ', ' good ', ' day ', ' night ', ' thanks ', ' thank ', ' hello ', ' hi ', ' because ', ' when ', ' where ', ' how ', ' who ', ' which ', ' do ', ' does ', ' did ', ' have ', ' has ', ' had ', ' will ', ' would ', ' can ', ' want ', ' need ', ' like ', ' house ', ' water ', ' today ', ' tomorrow ', ' go ', ' going ', ' me ', ' please '];

function detectDirection(text) {
  const s = text.toLowerCase();
  // Strong signal: Portuguese-only diacritics
  if (/[ãõáàâêôçéíóú]/.test(s)) return 'pt-en';

  const t = ' ' + s.replace(/[^a-z'\s]/g, ' ').replace(/\s+/g, ' ').trim() + ' ';
  let pt = 0, en = 0;
  PT_WORDS.forEach(w => { if (t.includes(w)) pt++; });
  EN_WORDS.forEach(w => { if (t.includes(w)) en++; });

  // Morphology: endings/clusters characteristic of each language
  pt += (s.match(/ç|lh|nh|ção|ções|mente\b|ando\b|endo\b|inho\b|inha\b/g) || []).length;
  en += (s.match(/\bth|wh|ght|ing\b|tion\b|ed\b|ly\b|'s\b|n't\b/g) || []).length;

  if (pt !== en) return pt > en ? 'pt-en' : 'en-pt';
  // Tie-breaker: w/y/k are far more common in English than Portuguese
  return /[wyk]/.test(s) ? 'en-pt' : 'pt-en';
}
function currentDir() {
  return forcedDir || detectDirection(src.value);
}
function updateBadge() {
  const dir = forcedDir || (src.value.trim() ? detectDirection(src.value) : null);
  dirBadge.textContent = dir ? (dir === 'pt-en' ? 'PT → EN' : 'EN → PT') : 'Auto ⇄';
}
function flipDirection() {
  forcedDir = currentDir() === 'pt-en' ? 'en-pt' : 'pt-en';
  updateBadge();
  if (!outputCard.hidden && src.value.trim()) translate();
}
// Disabled: Auto-language detection on input
// src.addEventListener('input', updateBadge);
dirBadge.addEventListener('click', flipDirection);

/* ---------- Hotkey matching (for the in-app swap shortcut) ---------- */
function matchesHotkey(e, combo) {
  if (!combo) return false;
  const parts = combo.split('+').map(s => s.trim().toLowerCase()).filter(Boolean);
  if (!parts.length) return false;
  const key = parts[parts.length - 1];
  const mods = parts.slice(0, -1);
  const need = { alt: false, ctrl: false, shift: false, meta: false };
  mods.forEach(m => {
    if (m === 'alt' || m === 'option') need.alt = true;
    else if (['ctrl', 'control', 'commandorcontrol', 'cmdorctrl'].includes(m)) need.ctrl = true;
    else if (m === 'shift') need.shift = true;
    else if (['meta', 'cmd', 'command', 'super', 'win'].includes(m)) need.meta = true;
  });
  if (e.altKey !== need.alt || e.ctrlKey !== need.ctrl ||
      e.shiftKey !== need.shift || e.metaKey !== need.meta) return false;
  const k = e.key.toLowerCase();
  if (key === 'space') return e.code === 'Space';
  return k === key;
}

/* ---------- Translate ---------- */
async function translate() {
  const text = src.value.trim();
  if (!text) {
    status.classList.add('error');
    status.textContent = 'Type something before translating.';
    fitWindow();
    return;
  }
  const dir = currentDir();
  const langpair = dir === 'pt-en' ? 'pt|en' : 'en|pt';

  status.classList.remove('error');
  status.textContent = 'Translating…';
  btnTranslate.disabled = true;

  try {
    const translated = await invoke('translate', { text, langpair });
    out.textContent = translated;
    outputCard.hidden = false;
    outDir.textContent = dir === 'pt-en' ? 'English' : 'Portuguese';
    lastResultDir = dir;
    status.textContent = '';
  } catch (err) {
    const msg = String(err);
    if (msg.includes('no_key')) {
      status.textContent = 'Add your DeepL API key in Settings to translate.';
    } else if (msg.includes('deepl_auth')) {
      status.textContent = 'DeepL rejected the key. Check the DeepL key in Settings.';
    } else if (msg.includes('limit:')) {
      status.textContent = msg.replace(/^.*limit:\s*/, '');
    } else if (msg.includes('network_error') || msg.includes('http_error')) {
      const detail = msg.replace(/^.*(network_error|http_error):\s*/, '');
      status.textContent = 'Connection to the translator failed. Detail: ' + detail;
    } else if (msg.includes('api_error')) {
      status.textContent = "Couldn't translate that text. Try rephrasing.";
    } else {
      status.textContent = 'Error: ' + msg;
    }
    status.classList.add('error');
  } finally {
    btnTranslate.disabled = false;
    fitWindow();
  }
}

btnTranslate.addEventListener('click', translate);
src.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); translate(); }
});

/* ---------- Copy ---------- */
btnCopy.addEventListener('click', async () => {
  try {
    await invoke('set_clipboard', { text: out.textContent });
  } catch (_) {
    try { await navigator.clipboard.writeText(out.textContent); } catch (_) {}
  }
  // Icon-only feedback: brief accent flash, no text.
  btnCopy.classList.add('copied');
  clearTimeout(btnCopy._t);
  btnCopy._t = setTimeout(() => btnCopy.classList.remove('copied'), 900);
});

/* ---------- Text-to-speech (pronunciation) ---------- */
let voicesCache = [];
function loadVoices() {
  try { voicesCache = window.speechSynthesis.getVoices() || []; } catch (_) {}
}
if ('speechSynthesis' in window) {
  loadVoices();
  window.speechSynthesis.addEventListener('voiceschanged', loadVoices);
}
// Prefer modern, natural-sounding voices (Windows "Online (Natural)" voices,
// macOS "Enhanced"/"Premium") over legacy robotic ones ("Desktop" SAPI5).
function scoreVoice(v) {
  const n = (v.name || '').toLowerCase();
  let score = 0;
  if (n.includes('natural')) score += 5;
  if (n.includes('online')) score += 3;
  if (n.includes('neural') || n.includes('enhanced') || n.includes('premium')) score += 3;
  if (n.includes('desktop')) score -= 2;
  return score;
}
function pickVoice(langBase) {
  const matches = voicesCache.filter(vc => vc.lang && vc.lang.toLowerCase().startsWith(langBase));
  if (!matches.length) return null;
  return matches.slice().sort((a, b) => scoreVoice(b) - scoreVoice(a))[0];
}
function speak(text, dir, side) {
  // side 'src' speaks in the source language, 'out' in the target language
  if (!text || !('speechSynthesis' in window)) return;
  const langBase = (side === 'out')
    ? (dir === 'pt-en' ? 'en' : 'pt')
    : (dir === 'pt-en' ? 'pt' : 'en');
  const u = new SpeechSynthesisUtterance(text);
  u.lang = langBase === 'pt' ? 'pt-BR' : 'en-US';
  const v = pickVoice(langBase);
  if (v) u.voice = v;
  u.rate = 1;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(u);
}
if ('speechSynthesis' in window) {
  el('btnSpeakSrc').addEventListener('click', () => speak(src.value.trim(), currentDir(), 'src'));
  el('btnSpeakOut').addEventListener('click', () => speak(out.textContent.trim(), lastResultDir, 'out'));
} else {
  el('btnSpeakSrc').hidden = true;
  el('btnSpeakOut').hidden = true;
}

/* ---------- Speech-to-text (voice input) ---------- */
const SpeechRec = window.SpeechRecognition || window.webkitSpeechRecognition;
let recognition = null;
let micOn = false;
let micTimer = null;
function setMic(on) {
  micOn = on;
  el('btnMic').classList.toggle('active', on);
  if (!on) { clearTimeout(micTimer); }
}
// Force the mic off no matter what state the recognizer is in — used on the
// second click and as a watchdog, so the button can never get stuck "on".
function stopMic() {
  clearTimeout(micTimer);
  try { recognition.abort(); } catch (_) {}
  try { recognition.stop(); } catch (_) {}
  setMic(false);
}
if (SpeechRec) {
  recognition = new SpeechRec();
  recognition.interimResults = false;
  recognition.maxAlternatives = 1;
  recognition.continuous = false;
  recognition.onstart = () => setMic(true);
  recognition.onresult = (e) => {
    const txt = e.results[0][0].transcript;
    setMic(false);
    src.value = txt;
    updateBadge();
    translate();
  };
  recognition.onerror = (e) => {
    setMic(false);
    status.classList.add('error');
    status.textContent = 'Voice input unavailable (' + (e.error || 'error') + ').';
    fitWindow();
  };
  recognition.onend = () => setMic(false);
  el('btnMic').addEventListener('click', () => {
    if (micOn) { stopMic(); return; }
    recognition.lang = currentDir() === 'pt-en' ? 'pt-BR' : 'en-US';
    try {
      recognition.start();
      setMic(true);
      clearTimeout(micTimer);
      micTimer = setTimeout(stopMic, 12000); // safety net: never stay on forever
    } catch (_) { setMic(false); }
  });
} else {
  el('btnMic').hidden = true;
}

/* ---------- Global keys ---------- */
document.addEventListener('keydown', (e) => {
  if (matchesHotkey(e, settings.swapHotkey)) {
    e.preventDefault();
    flipDirection();
    return;
  }
  if (e.key === 'Escape') {
    if (!settingsView.hidden || !appearanceView.hidden) { showMain(); return; }
    invoke('hide_popup');
  }
});

/* ---------- Menus ---------- */
el('btnSettings').addEventListener('click', showSettings);
el('btnAppearance').addEventListener('click', showAppearance);
el('btnClose').addEventListener('click', () => invoke('hide_popup'));
el('btnBackSettings').addEventListener('click', showMain);
el('btnBackAppearance').addEventListener('click', () => {
  // Discard unsaved preview
  applyAppearance(settings.appearance);
  showMain();
});

/* ---------- Load / save ---------- */
async function loadSettings() {
  settings = await invoke('get_settings');
  if (!settings.appearance) settings.appearance = { ...DEFAULT_APPEARANCE };
  el('hotkey').value = settings.hotkey || 'Alt+R';
  el('swapHotkey').value = settings.swapHotkey || 'Alt+E';
  el('selectionHotkey').value = settings.selectionHotkey || 'Alt+T';
  el('deeplKey').value = settings.deeplKey || '';
  el('autoTranslateClipboard').checked = settings.autoTranslateClipboard !== false;
  try { el('autostart').checked = await invoke('get_autostart'); } catch (_) {}
  appearanceToControls(settings.appearance);
  applyAppearance(settings.appearance);
  dirBadge.title = `Click or press ${settings.swapHotkey || 'Alt+E'} to swap direction`;
  fitWindow();
}

async function saveAll(statusEl) {
  const newSettings = {
    hotkey: el('hotkey').value.trim() || 'Alt+R',
    swapHotkey: el('swapHotkey').value.trim() || 'Alt+E',
    selectionHotkey: el('selectionHotkey').value.trim() || 'Alt+T',
    autoTranslateClipboard: el('autoTranslateClipboard').checked,
    deeplKey: el('deeplKey').value.trim(),
    appearance: appearanceFromControls()
  };
  try {
    await invoke('save_settings', { settings: newSettings });
    try { await invoke('set_autostart', { enabled: el('autostart').checked }); } catch (_) {}
    settings = newSettings;
    applyAppearance(settings.appearance);
    dirBadge.title = `Click or press ${settings.swapHotkey} to swap direction`;

    // Apply the (possibly changed) global shortcuts immediately — no restart.
    try {
      await invoke('update_shortcuts');
    } catch (err) {
      statusEl.classList.add('error');
      statusEl.textContent = 'Saved, but a shortcut is invalid or already in use: ' + err;
      return; // stay on this panel so the shortcut can be fixed
    }

    // Saved cleanly → return to the main view.
    showMain();
    toast(status, 'Saved!');
  } catch (err) {
    statusEl.classList.add('error');
    statusEl.textContent = 'Error saving: ' + err;
  }
}

el('btnSaveSettings').addEventListener('click', () => saveAll(el('settingsStatus')));
el('btnSaveAppearance').addEventListener('click', () => saveAll(el('appearanceStatus')));

/* ---------- Reset everything to defaults (keeps the DeepL key) ---------- */
el('btnResetAll').addEventListener('click', () => {
  el('hotkey').value = 'Alt+R';
  el('swapHotkey').value = 'Alt+E';
  el('selectionHotkey').value = 'Alt+T';
  el('autoTranslateClipboard').checked = true;
  el('autostart').checked = true; // default: start with Windows
  appearanceToControls(DEFAULT_APPEARANCE);
  applyAppearance(DEFAULT_APPEARANCE);
  // The DeepL key field is intentionally left untouched.
  saveAll(el('settingsStatus'));
});

/* ---------- Home button ---------- */
el('btnHome').addEventListener('click', showMain);

/* ---------- Popup shown ---------- */
listen('popup-shown', (event) => {
  showMain();
  src.focus();
  src.select();
  fitWindow();
});

listen('open-settings', () => showSettings());

loadSettings();
