// Region picker for screen OCR: drag a box, hand the rectangle to the backend.
// Coordinates stay in CSS pixels — Rust scales them by the window's DPI factor.

import * as api from './api.js';

const backdrop = document.getElementById('backdrop');
const selection = document.getElementById('selection');
const hint = document.getElementById('hint');

const MIN_SIDE = 8;

let origin = null;

const label = document.createElement('div');
label.id = 'size';
selection.append(label);

function reset() {
  origin = null;
  selection.hidden = true;
  backdrop.style.opacity = '1';
  hint.hidden = false;
}

function rectFrom(event) {
  const x = Math.min(origin.x, event.clientX);
  const y = Math.min(origin.y, event.clientY);
  return {
    x,
    y,
    width: Math.abs(event.clientX - origin.x),
    height: Math.abs(event.clientY - origin.y)
  };
}

function draw(rect) {
  selection.style.left = rect.x + 'px';
  selection.style.top = rect.y + 'px';
  selection.style.width = rect.width + 'px';
  selection.style.height = rect.height + 'px';
  label.textContent = `${Math.round(rect.width)} × ${Math.round(rect.height)}`;
}

window.addEventListener('mousedown', (event) => {
  if (event.button !== 0) {
    api.cancelRegionCapture().catch(() => {});
    return;
  }
  origin = { x: event.clientX, y: event.clientY };
  selection.hidden = false;
  backdrop.style.opacity = '0';
  hint.hidden = true;
  draw({ x: origin.x, y: origin.y, width: 0, height: 0 });
});

window.addEventListener('mousemove', (event) => {
  if (!origin) return;
  draw(rectFrom(event));
});

window.addEventListener('mouseup', (event) => {
  if (!origin) return;
  const rect = rectFrom(event);
  reset();

  if (rect.width < MIN_SIDE || rect.height < MIN_SIDE) {
    api.cancelRegionCapture().catch(() => {});
    return;
  }
  api.ocrRegion(rect).catch(() => {});
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    reset();
    api.cancelRegionCapture().catch(() => {});
  }
});

window.addEventListener('contextmenu', (event) => event.preventDefault());

// Each capture re-focuses the window, so this clears any leftover drag state.
window.addEventListener('focus', reset);
window.addEventListener('blur', reset);
