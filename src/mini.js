// Compact "translate selection" bubble (Google-extension style)
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const el = (id) => document.getElementById(id);

let lastDir = 'pt-en';
let shownAt = 0;

/* ---------- Theme (shared defaults with the main window) ---------- */
function readableOn(hex) {
  const c = (hex || '').replace('#', '');
  if (c.length < 6) return '#ffffff';
  const ch = (i) => parseInt(c.substr(i, 2), 16) / 255;
  const lin = (x) => (x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4));
  const L = 0.2126 * lin(ch(0)) + 0.7152 * lin(ch(2)) + 0.0722 * lin(ch(4));
  return 1.05 / (L + 0.05) >= (L + 0.05) / 0.05 ? '#ffffff' : '#1b1b1f';
}
function applyAppearance(a) {
  if (!a) return;
  const r = document.documentElement.style;
  r.setProperty('--parchment', a.bg);
  r.setProperty('--gold', a.accent);
  r.setProperty('--ink', a.text);
  r.setProperty('--ui-font', a.font);
  r.setProperty('--app-opacity', String(a.opacity));
  r.setProperty('--on-accent', readableOn(a.accent));
}

/* ---------- Language detection (same heuristic as the main window) ---------- */
const PT_WORDS = [' o ', ' a ', ' os ', ' as ', ' um ', ' uma ', ' de ', ' do ', ' da ', ' dos ', ' das ', ' em ', ' no ', ' na ', ' que ', ' e ', ' ou ', ' mas ', ' com ', ' sem ', ' por ', ' para ', ' se ', ' eu ', ' voce ', ' ele ', ' ela ', ' nos ', ' eles ', ' meu ', ' minha ', ' seu ', ' sua ', ' isso ', ' isto ', ' aqui ', ' ali ', ' nao ', ' sim ', ' muito ', ' mais ', ' menos ', ' tudo ', ' nada ', ' bom ', ' boa ', ' dia ', ' noite ', ' obrigado ', ' obrigada ', ' ola ', ' porque ', ' quando ', ' onde ', ' como ', ' quem ', ' qual ', ' ser ', ' estar ', ' ter ', ' fazer ', ' vou ', ' vai ', ' esta ', ' sao ', ' foi ', ' tem ', ' quero ', ' preciso ', ' gosto ', ' casa ', ' agua ', ' hoje ', ' amanha '];
const EN_WORDS = [' the ', ' an ', ' of ', ' to ', ' in ', ' on ', ' at ', ' is ', ' are ', ' was ', ' were ', ' be ', ' been ', ' and ', ' or ', ' but ', ' with ', ' without ', ' for ', ' if ', ' i ', ' you ', ' he ', ' she ', ' we ', ' they ', ' it ', ' my ', ' your ', ' his ', ' her ', ' this ', ' that ', ' these ', ' those ', ' here ', ' there ', ' no ', ' yes ', ' not ', ' very ', ' more ', ' less ', ' all ', ' nothing ', ' good ', ' day ', ' night ', ' thanks ', ' thank ', ' hello ', ' hi ', ' because ', ' when ', ' where ', ' how ', ' who ', ' which ', ' do ', ' does ', ' did ', ' have ', ' has ', ' had ', ' will ', ' would ', ' can ', ' want ', ' need ', ' like ', ' house ', ' water ', ' today ', ' tomorrow ', ' go ', ' going ', ' me ', ' please '];

function detectDirection(text) {
  const s = text.toLowerCase();
  if (/[ãõáàâêôçéíóú]/.test(s)) return 'pt-en';
  const t = ' ' + s.replace(/[^a-z'\s]/g, ' ').replace(/\s+/g, ' ').trim() + ' ';
  let pt = 0, en = 0;
  PT_WORDS.forEach(w => { if (t.includes(w)) pt++; });
  EN_WORDS.forEach(w => { if (t.includes(w)) en++; });
  pt += (s.match(/ç|lh|nh|ção|ções|mente\b|ando\b|endo\b|inho\b|inha\b/g) || []).length;
  en += (s.match(/\bth|wh|ght|ing\b|tion\b|ed\b|ly\b|'s\b|n't\b/g) || []).length;
  if (pt !== en) return pt > en ? 'pt-en' : 'en-pt';
  return /[wyk]/.test(s) ? 'en-pt' : 'pt-en';
}

/* ---------- Text-to-speech ---------- */
let voicesCache = [];
function loadVoices() { try { voicesCache = window.speechSynthesis.getVoices() || []; } catch (_) {} }
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
function speak(text, langBase) {
  if (!text || !('speechSynthesis' in window)) return;
  const u = new SpeechSynthesisUtterance(text);
  u.lang = langBase === 'pt' ? 'pt-BR' : 'en-US';
  const v = pickVoice(langBase);
  if (v) u.voice = v;
  u.rate = 1;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(u);
}

/* ---------- Fit bubble to content ---------- */
function fit() {
  requestAnimationFrame(() => {
    const h = Math.ceil(document.getElementById('bubble').getBoundingClientRect().height);
    invoke('resize_mini', { height: h }).catch(() => {});
  });
}

/* ---------- Wiring ---------- */
el('spkSrc').addEventListener('click', () => speak(el('srcText').textContent, lastDir === 'pt-en' ? 'pt' : 'en'));
el('spkOut').addEventListener('click', () => speak(el('outText').textContent, lastDir === 'pt-en' ? 'en' : 'pt'));
el('close').addEventListener('click', () => invoke('hide_mini'));

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') invoke('hide_mini');
});
// Dismiss when focus leaves the bubble (like the Google popup), but ignore the
// focus settling right after it appears.
window.addEventListener('blur', () => {
  if (Date.now() - shownAt > 500) invoke('hide_mini');
});

async function loadTheme() {
  try {
    const s = await invoke('get_settings');
    applyAppearance(s && s.appearance);
  } catch (_) {}
}

async function translateSelection(text) {
  const dir = detectDirection(text);
  lastDir = dir;
  const langpair = dir === 'pt-en' ? 'pt|en' : 'en|pt';
  el('srcLang').textContent = dir === 'pt-en' ? 'Português' : 'English';
  el('outLang').textContent = dir === 'pt-en' ? 'English' : 'Português';
  el('srcText').textContent = text;
  el('outText').textContent = '…';
  el('miniStatus').textContent = '';
  el('miniStatus').classList.remove('error');
  fit();

  try {
    const translated = await invoke('translate', { text, langpair });
    el('outText').textContent = translated;
  } catch (err) {
    const msg = String(err);
    el('outText').textContent = '';
    el('miniStatus').classList.add('error');
    if (msg.includes('no_key')) el('miniStatus').textContent = 'Add a DeepL key in Settings.';
    else if (msg.includes('deepl_auth')) el('miniStatus').textContent = 'Check the DeepL key in Settings.';
    else if (msg.includes('limit:')) el('miniStatus').textContent = msg.replace(/^.*limit:\s*/, '');
    else if (msg.includes('network_error') || msg.includes('http_error')) el('miniStatus').textContent = 'Connection failed.';
    else el('miniStatus').textContent = 'Could not translate.';
  }
  fit();
}

listen('mini-translate', (event) => {
  shownAt = Date.now();
  const text = (event.payload || '').toString().trim();
  if (!text) {
    el('srcLang').textContent = '—';
    el('outLang').textContent = '—';
    el('srcText').textContent = '';
    el('outText').textContent = '';
    el('miniStatus').classList.add('error');
    el('miniStatus').textContent = 'No text selected (nothing was copied).';
    fit();
    return;
  }
  loadTheme();
  translateSelection(text);
});

loadTheme();
