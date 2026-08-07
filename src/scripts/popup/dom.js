export const el = (id) => document.getElementById(id);

export const src = el('src');
export const out = el('out');
export const outputCard = el('outputCard');
export const status = el('status');
export const dirBadge = el('dirBadge');
export const dirLabel = el('dirLabel');

export const VIEWS = {
  main: el('mainView'),
  welcome: el('welcomeView'),
  history: el('historyView'),
  shortcuts: el('shortcutsView'),
  settings: el('settingsView'),
  appearance: el('appearanceView')
};
