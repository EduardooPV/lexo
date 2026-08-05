import { describe, it, expect } from 'vitest';
import { mountPopup, flush, el } from '../helpers/mount.js';

describe('update banner', () => {
  it('stays out of the way when there is no update', async () => {
    await mountPopup();
    expect(el('updateBanner').hidden).toBe(true);
  });

  it('shows a plain, always-non-empty label with the version as a tooltip', async () => {
    const mock = await mountPopup();
    await mock.emit('update-available', { version: '1.2.0', notes: 'Bug fixes' });
    await flush();

    expect(el('updateBanner').hidden).toBe(false);
    expect(el('updateText').textContent).toBe('Update available');
    expect(el('updateBanner').title).toBe('Version 1.2.0');
  });

  it('ignores a malformed payload rather than showing an empty banner', async () => {
    const mock = await mountPopup();
    await mock.emit('update-available', {});
    await flush();
    expect(el('updateBanner').hidden).toBe(true);

    await mock.emit('update-available', null);
    await flush();
    expect(el('updateBanner').hidden).toBe(true);
  });

  it('can be dismissed without installing anything', async () => {
    const mock = await mountPopup();
    await mock.emit('update-available', { version: '1.2.0' });
    await flush();

    el('btnDismissUpdate').click();
    await flush();

    expect(el('updateBanner').hidden).toBe(true);
    expect(mock.callsFor('install_update')).toHaveLength(0);
  });

  it('never installs anything on its own', async () => {
    const mock = await mountPopup();
    await mock.emit('update-available', { version: '1.2.0' });
    await flush();

    expect(mock.callsFor('install_update')).toHaveLength(0);
  });
});

describe('installing', () => {
  it('asks the backend to install and locks the button while it works', async () => {
    const mock = await mountPopup();
    await mock.emit('update-available', { version: '1.2.0' });
    await flush();

    el('btnInstallUpdate').click();

    expect(el('btnInstallUpdate').disabled).toBe(true);
    expect(el('btnInstallUpdate').textContent).toBe('Updating…');

    await flush();
    expect(mock.callsFor('install_update')).toHaveLength(1);
  });

  it('re-enables the button and explains itself when the install fails', async () => {
    const mock = await mountPopup({
      install_update: () => {
        throw 'updater_error: connection refused';
      }
    });
    await mock.emit('update-available', { version: '1.2.0' });
    await flush();

    el('btnInstallUpdate').click();
    await flush();

    expect(el('btnInstallUpdate').disabled).toBe(false);
    expect(el('btnInstallUpdate').textContent).toBe('Update');
    expect(el('status').textContent).toBe('Could not reach the update server. Try again later.');
    expect(el('status').classList.contains('error')).toBe(true);
  });

  it('closes the banner instead of nagging when the backend says there is nothing to install', async () => {
    const mock = await mountPopup({
      install_update: () => {
        throw 'updater_none: already on the latest version.';
      }
    });
    await mock.emit('update-available', { version: '1.2.0' });
    await flush();

    el('btnInstallUpdate').click();
    await flush();

    expect(el('updateBanner').hidden).toBe(true);
    expect(el('btnInstallUpdate').disabled).toBe(false);
    expect(el('btnInstallUpdate').textContent).toBe('Update');
    expect(el('status').classList.contains('error')).toBe(false);
  });
});
