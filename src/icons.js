const GLYPHS = {
  back: '<path d="m15 18-6-6 6-6"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  close: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  copy:
    '<rect width="14" height="14" x="8" y="8" rx="2"/>' +
    '<path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  external:
    '<path d="M15 3h6v6"/><path d="M10 14 21 3"/>' +
    '<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
  eye:
    '<path d="M2.06 12.35a1 1 0 0 1 0-.7 10.75 10.75 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.75 10.75 0 0 1-19.88 0"/>' +
    '<circle cx="12" cy="12" r="3"/>',
  'eye-off':
    '<path d="M10.73 5.08A10.8 10.8 0 0 1 12 5c4.64 0 8.58 2.85 9.94 6.65a1 1 0 0 1 0 .7 11.3 11.3 0 0 1-1.44 2.49"/>' +
    '<path d="M14.08 14.16a3 3 0 0 1-4.24-4.24"/>' +
    '<path d="M17.48 17.5A10.72 10.72 0 0 1 12 19c-4.64 0-8.58-2.85-9.94-6.65a1 1 0 0 1 0-.7 11.05 11.05 0 0 1 4.45-5.14"/>' +
    '<path d="m2 2 20 20"/>',
  history:
    '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>' +
    '<path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>',
  home:
    '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/>' +
    '<path d="M3 10a2 2 0 0 1 .71-1.53l7-6a2 2 0 0 1 2.58 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  languages:
    '<path d="m5 8 6 6"/><path d="m4 14 6-6 2-3"/><path d="M2 5h12"/>' +
    '<path d="M7 2h1"/><path d="m22 22-5-10-5 10"/><path d="M14 18h6"/>',
  mic:
    '<path d="M12 19v3"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/>' +
    '<rect x="9" y="2" width="6" height="13" rx="3"/>',
  palette:
    '<path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.65-.75 1.65-1.69 0-.44-.18-.83-.44-1.12a1.6 1.6 0 0 1-.44-1.13 1.64 1.64 0 0 1 1.67-1.66h2A5.56 5.56 0 0 0 22 10.75C21.97 6.01 17.46 2 12 2z"/>' +
    '<circle cx="6.5" cy="12.5" r="1" fill="currentColor" stroke="none"/>' +
    '<circle cx="8.5" cy="7.5" r="1" fill="currentColor" stroke="none"/>' +
    '<circle cx="13.5" cy="6.5" r="1" fill="currentColor" stroke="none"/>' +
    '<circle cx="17.5" cy="10.5" r="1" fill="currentColor" stroke="none"/>',
  pause: '<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>',
  play: '<path d="m6 3 14 9-14 9z"/>',
  refresh:
    '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>' +
    '<path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  reset: '<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>',
  scan:
    '<path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/>' +
    '<path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>' +
    '<path d="M7 8h8"/><path d="M7 12h10"/><path d="M7 16h6"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
  settings:
    '<path d="M20 7h-9"/><path d="M14 17H5"/>' +
    '<circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>',
  star:
    '<path d="M11.5 2.7a.6.6 0 0 1 1 0l2.5 5.1 5.6.8a.6.6 0 0 1 .3 1l-4 3.9.9 5.6a.6.6 0 0 1-.85.6L12 17.1l-5 2.6a.6.6 0 0 1-.85-.6l.9-5.6-4-3.9a.6.6 0 0 1 .3-1l5.6-.8z"/>',
  swap: '<path d="m16 3 4 4-4 4"/><path d="M20 7H4"/><path d="m8 21-4-4 4-4"/><path d="M4 17h16"/>',
  trash:
    '<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>' +
    '<path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
  volume:
    '<path d="M11 4.7 6.4 8.9H3v6.2h3.4l4.6 4.2z"/>' +
    '<path d="M16 9a5 5 0 0 1 0 6"/><path d="M19.4 5.6a9 9 0 0 1 0 12.8"/>',
  warning:
    '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/>' +
    '<path d="M12 9v4"/><path d="M12 17h.01"/>'
};

export function iconMarkup(name) {
  const glyph = GLYPHS[name];
  if (!glyph) return '';
  return (
    '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" ' +
    'stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
    glyph +
    '</svg>'
  );
}

export function hydrate(root = document) {
  root.querySelectorAll('[data-icon]').forEach((node) => {
    node.innerHTML = iconMarkup(node.dataset.icon);
  });
}

export function setIcon(node, name) {
  node.dataset.icon = name;
  node.innerHTML = iconMarkup(name);
}
