import * as api from '../shared/api.js';
import { ZOOM_STEP, clampZoom } from '../shared/theme.js';
import { state } from './state.js';
import { refitAfterZoom } from './views.js';

export async function setZoom(factor) {
  const next = clampZoom(factor);
  const appearance = state.settings.appearance;
  if (next === (appearance.zoom || 1)) return;

  const cssHeight = window.innerHeight;
  appearance.zoom = next;
  try { await api.setZoom(next); } catch (_) {}
  refitAfterZoom(cssHeight);

  try { await api.saveSettings(state.settings); } catch (_) {}
}

// Ctrl + / Ctrl = zoom in, Ctrl - zoom out, Ctrl 0 reset. Returns true when the
// event was one of those so the caller can skip its own handling.
export function zoomFromKey(event) {
  if (!(event.ctrlKey || event.metaKey) || event.altKey) return false;
  const current = state.settings.appearance.zoom || 1;

  if (event.key === '+' || event.key === '=') setZoom(current + ZOOM_STEP);
  else if (event.key === '-' || event.key === '_') setZoom(current - ZOOM_STEP);
  else if (event.key === '0') setZoom(1);
  else return false;

  event.preventDefault();
  return true;
}
