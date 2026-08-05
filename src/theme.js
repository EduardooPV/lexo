// Appearance shared by the popup and the bubble.

export const DEFAULT_APPEARANCE = {
  opacity: 1,
  bg: '#282a36',
  accent: '#bd93f9',
  text: '#f8f8f2',
  font: "'Segoe UI', system-ui, sans-serif"
};

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

// White or near-black over `hex`, whichever has more contrast.
export function readableOn(hex) {
  const value = (hex || '').replace('#', '');
  if (value.length < 6) return '#ffffff';
  const channel = (i) => parseInt(value.substr(i, 2), 16) / 255;
  const linear = (x) => (x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4));
  const luminance =
    0.2126 * linear(channel(0)) + 0.7152 * linear(channel(2)) + 0.0722 * linear(channel(4));
  return 1.05 / (luminance + 0.05) >= (luminance + 0.05) / 0.05 ? '#ffffff' : '#1b1b1f';
}

export function applyAppearance(appearance) {
  if (!appearance) return;
  const root = document.documentElement.style;
  root.setProperty('--parchment', appearance.bg);
  root.setProperty('--gold', appearance.accent);
  root.setProperty('--ink', appearance.text);
  root.setProperty('--ui-font', appearance.font);
  root.setProperty('--app-opacity', String(appearance.opacity));
  root.setProperty('--on-accent', readableOn(appearance.accent));
}
