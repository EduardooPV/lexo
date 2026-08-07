import { THEMES, applyAppearance } from '../shared/theme.js';
import { el } from './dom.js';

export function appearanceFromControls() {
  return {
    opacity: parseFloat(el('opacity').value),
    bg: el('colBg').value,
    accent: el('colAccent').value,
    text: el('colText').value,
    font: el('font').value
  };
}

export function appearanceToControls(appearance) {
  el('opacity').value = appearance.opacity;
  el('opacityVal').textContent = Math.round(appearance.opacity * 100) + '%';
  el('colBg').value = appearance.bg;
  el('colAccent').value = appearance.accent;
  el('colText').value = appearance.text;

  const select = el('font');
  if (![...select.options].some((option) => option.value === appearance.font)) {
    const custom = document.createElement('option');
    custom.value = appearance.font;
    custom.textContent = 'Custom';
    select.append(custom);
  }
  select.value = appearance.font;
}

export function syncThemeSelect() {
  const index = THEMES.findIndex(
    (theme) =>
      theme.bg === el('colBg').value.toLowerCase() &&
      theme.accent === el('colAccent').value.toLowerCase() &&
      theme.text === el('colText').value.toLowerCase()
  );
  el('themeSelect').value = index >= 0 ? String(index) : 'custom';
}

export function previewAppearance() {
  const appearance = appearanceFromControls();
  el('opacityVal').textContent = Math.round(appearance.opacity * 100) + '%';
  applyAppearance(appearance);
  syncThemeSelect();
}

export function buildThemeSelect() {
  const select = el('themeSelect');
  THEMES.forEach((theme, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = theme.name;
    select.append(option);
  });
  const custom = document.createElement('option');
  custom.value = 'custom';
  custom.textContent = 'Custom';
  select.append(custom);

  select.addEventListener('change', () => {
    if (select.value === 'custom') return;
    const theme = THEMES[Number(select.value)];
    el('colBg').value = theme.bg;
    el('colAccent').value = theme.accent;
    el('colText').value = theme.text;
    previewAppearance();
  });
}
