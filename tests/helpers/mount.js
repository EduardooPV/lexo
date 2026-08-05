import { readFileSync } from 'node:fs';
import { vi } from 'vitest';
import { installTauriMock } from './tauri.js';

function bodyOf(relativePath) {
  const html = readFileSync(new URL(relativePath, import.meta.url), 'utf8');
  const body = html.match(/<body>([\s\S]*)<\/body>/i);
  if (!body) throw new Error(`no <body> found in ${relativePath}`);
  return body[1].replace(/<script[\s\S]*?<\/script>/gi, '');
}

export async function flush(rounds = 4) {
  for (let i = 0; i < rounds; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
}

export async function mountPopup(overrides = {}) {
  document.body.innerHTML = bodyOf('../../src/index.html');
  const mock = installTauriMock(overrides);
  vi.resetModules();
  await import('../../src/main.js');
  await flush();
  return mock;
}

export async function mountMini(overrides = {}) {
  document.body.innerHTML = bodyOf('../../src/mini.html');
  const mock = installTauriMock(overrides);
  vi.resetModules();
  await import('../../src/mini.js');
  await flush();
  return mock;
}

export async function mountOverlay(overrides = {}) {
  document.body.innerHTML = bodyOf('../../src/overlay.html');
  const mock = installTauriMock(overrides);
  vi.resetModules();
  await import('../../src/overlay.js');
  await flush(1);
  return mock;
}

export const el = (id) => document.getElementById(id);
