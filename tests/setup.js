import { beforeEach } from 'vitest';
import { installTauriMock } from './helpers/tauri.js';

beforeEach(() => {
  installTauriMock();
});
