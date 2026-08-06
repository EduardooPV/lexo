export const DEFAULT_SETTINGS = {
  hotkey: 'Alt+R',
  swapHotkey: 'Alt+E',
  selectionHotkey: 'Alt+T',
  replaceHotkey: 'Alt+Shift+T',
  ocrHotkey: 'Alt+S',
  autoTranslateClipboard: false,
  shortcutsPaused: false,
  deeplKey: 'test-key-0000:fx',
  appearance: {
    opacity: 1,
    bg: '#282a36',
    accent: '#bd93f9',
    text: '#f8f8f2',
    font: "'Segoe UI', system-ui, sans-serif"
  },
  settingsVersion: 1
};

export function sampleHistory() {
  return [
    {
      id: '1',
      source: 'Preciso revisar esse pull request',
      translated: 'I need to review this pull request',
      from: 'PT',
      to: 'EN',
      at: 3,
      pinned: true
    },
    {
      id: '2',
      source: 'The build failed on the Windows runner',
      translated: 'O build falhou no runner do Windows',
      from: 'EN',
      to: 'PT',
      at: 2,
      pinned: false
    }
  ];
}

export function createTauriMock(overrides = {}) {
  const calls = [];
  const listeners = new Map();

  const handlers = {
    get_settings: () => structuredClone(DEFAULT_SETTINGS),
    save_settings: () => null,
    update_shortcuts: () => null,
    get_autostart: () => true,
    set_autostart: () => null,
    ocr_available: () => true,
    get_history: () => [],
    clear_history: () => [],
    delete_history_entry: () => [],
    toggle_history_pin: () => [],
    get_usage: () => ({ characterCount: 412345, characterLimit: 500000 }),
    verify_deepl_key: () => ({ characterCount: 0, characterLimit: 500000 }),
    open_deepl_signup: () => null,
    translate: ({ text, target }) => ({
      text: 'translated: ' + text,
      detectedSource: 'PT',
      target: target || 'EN',
      cached: false
    }),
    check_for_update: () => null,
    pending_update: () => null,
    install_update: () => null,
    app_version: () => '9.9.9',
    set_clipboard: () => null,
    resize_window: () => null,
    resize_mini: () => null,
    hide_popup: () => null,
    hide_mini: () => null,
    start_region_capture: () => null,
    cancel_region_capture: () => null,
    ocr_region: () => null,
    ...overrides
  };

  function invoke(command, args) {
    calls.push({ command, args });
    const handler = handlers[command];
    if (handler === undefined) return Promise.resolve(null);
    if (typeof handler !== 'function') return Promise.resolve(handler);
    try {
      return Promise.resolve(handler(args || {}));
    } catch (error) {
      return Promise.reject(error);
    }
  }

  function listen(event, callback) {
    if (!listeners.has(event)) listeners.set(event, []);
    listeners.get(event).push(callback);
    return Promise.resolve(() => {});
  }

  return {
    global: { core: { invoke }, event: { listen } },
    calls,
    callsFor: (command) => calls.filter((call) => call.command === command),
    emit: (event, payload) =>
      Promise.all((listeners.get(event) || []).map((cb) => cb({ payload })))
  };
}

export function installTauriMock(overrides = {}) {
  const mock = createTauriMock(overrides);
  window.__TAURI__ = mock.global;
  return mock;
}
