export const DEFAULT_APPEARANCE = {
  opacity: 1,
  bg: '#282a36',
  accent: '#bd93f9',
  text: '#f8f8f2',
  font: "'Segoe UI', system-ui, sans-serif",
  fontWeight: 400,
  zoom: 1
};

export const ZOOM_MIN = 0.7;
export const ZOOM_MAX = 2;
export const ZOOM_STEP = 0.1;

export function clampZoom(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return 1;
  return Math.round(Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, number)) * 100) / 100;
}

export const THEMES = [
  { name: 'Dracula (default)', bg: '#282a36', accent: '#bd93f9', text: '#f8f8f2' },
  { name: 'Nord', bg: '#2e3440', accent: '#88c0d0', text: '#eceff4' },
  { name: 'One Dark', bg: '#282c34', accent: '#61afef', text: '#abb2bf' },
  { name: 'Tokyo Night', bg: '#1a1b26', accent: '#7aa2f7', text: '#c0caf5' },
  { name: 'Monokai', bg: '#272822', accent: '#a6e22e', text: '#f8f8f2' },
  { name: 'Gruvbox', bg: '#282828', accent: '#fabd2f', text: '#ebdbb2' },
  { name: 'Catppuccin', bg: '#1e1e2e', accent: '#cba6f7', text: '#cdd6f4' },
  { name: 'Night Owl', bg: '#011627', accent: '#82aaff', text: '#d6deeb' },
  { name: 'Solarized Dark', bg: '#002b36', accent: '#268bd2', text: '#93a1a1' },
  { name: 'Solarized Light', bg: '#fdf6e3', accent: '#268bd2', text: '#586e75' },
  { name: 'GitHub Light', bg: '#ffffff', accent: '#0969da', text: '#1f2328' }
];

function channels(hex) {
  const value = (hex || '').replace(/^#/, '');
  if (!/^[\da-f]{6}$/i.test(value)) return [];
  return [0, 2, 4].map((i) => parseInt(value.slice(i, i + 2), 16));
}

function relativeLuminance(hex) {
  const [r, g, b] = channels(hex).map((channel) => {
    const x = channel / 255;
    return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function readableOn(hex) {
  const luminance = relativeLuminance(hex);
  if (!Number.isFinite(luminance)) return '#ffffff';
  return 1.05 / (luminance + 0.05) >= (luminance + 0.05) / 0.05 ? '#ffffff' : '#1b1b1f';
}

export function contrast(hexA, hexB) {
  const a = relativeLuminance(hexA);
  const b = relativeLuminance(hexB);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

export function secondaryOn(bg, ink) {
  const background = channels(bg);
  const foreground = channels(ink);
  if (!background.length || !foreground.length) return ink;
  for (let percent = 50; percent <= 100; percent += 5) {
    const mixed = background.map((channel, i) =>
      Math.round(channel + (foreground[i] - channel) * percent / 100).toString(16).padStart(2, '0')
    );
    const color = '#' + mixed.join('');
    if (contrast(bg, color) >= 4.5) return color;
  }
  return ink;
}

export function errorOn(bg) {
  const colors = ['#ff79c6', '#ff5555', '#d6336c', '#b00040', '#8b0030'];
  return colors.find((color) => contrast(bg, color) >= 4.5) ||
    (contrast(bg, '#000000') >= 4.5 ? '#000000' : '#ffffff');
}

export function focusOn(bg, accent, ink) {
  return contrast(bg, accent) >= 3 ? accent : ink;
}

export function applyAppearance(appearance) {
  if (!appearance) return;
  const root = document.documentElement.style;
  root.setProperty('--parchment', appearance.bg);
  root.setProperty('--gold', appearance.accent);
  root.setProperty('--ink', appearance.text);
  root.setProperty('--ui-font', appearance.font);
  root.setProperty('--ui-weight', String(appearance.fontWeight || 400));
  root.setProperty('--app-opacity', String(appearance.opacity));
  root.setProperty('--on-accent', readableOn(appearance.accent));
  root.setProperty('--gray', secondaryOn(appearance.bg, appearance.text));
  const error = errorOn(appearance.bg);
  root.setProperty('--red', error);
  root.setProperty('--on-error', readableOn(error));
  root.setProperty('--focus', focusOn(appearance.bg, appearance.accent, appearance.text));
}
