# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A lightweight **Tauri 2** desktop app (Windows-first) that translates text PT ⇄ EN
from anywhere via global hotkeys. It lives in the system tray, not the taskbar.
The npm package is named `tradutor`; the product/binary is `Lexo` (bundle identifier
stays `com.luizveltroni.tradutor` so existing settings/keys are preserved).

- **Alt+R** — toggle the main translator popup
- **Alt+T** — translate the current selection in any app (shows a bubble at the cursor)
- **Alt+E** — flip translation direction (in-app, while the popup is focused)

Translation runs **in the Rust backend**, never in the webview — this avoids CORS and
keeps the flow out of the browser. The **only** engine is **DeepL** (best PT⇄EN quality,
fast); a DeepL API key set in Settings is required, and without one `translate` returns a
`no_key:` error the UI surfaces. No telemetry.

## Commands

```bash
npm install         # installs only @tauri-apps/cli; Rust deps come from Cargo
npm run dev         # tauri dev — hot-reloads the frontend, rebuilds Rust on change
npm run build       # tauri build — production installer (NSIS .exe / .msi on Windows)
```

Rust-only checks (run inside `src-tauri/`):

```bash
cargo check         # fast type-check of the backend
cargo clippy        # lint
cargo fmt           # format
```

**Requirements:** Rust 1.60+, Node 16+.

**There is no test suite and no JS lint/build config** — the frontend is plain files
served as-is (no bundler, no framework, no transpile step). Don't invent a test command.

## Architecture

### Two processes, two webview windows

- **Backend** (`src-tauri/src/lib.rs`) — all app logic lives here. `main.rs` is a
  one-line entry point that calls `tradutor_lib::run()`.
- **Frontend** — vanilla JS/HTML/CSS in `src/`, loaded into **two separate windows**
  declared in `src-tauri/tauri.conf.json`:
  - `popup` → `index.html` + `main.js` + `styles.css` — the full translator UI
    (input, settings, appearance panels). Hidden on launch, shown by Alt+R / tray.
  - `mini` → `mini.html` + `mini.js` + `mini.css` — the compact "translate selection"
    bubble shown at the cursor by Alt+T.

Both windows are `transparent`, `decorations: false`, `alwaysOnTop`. Closing a window
**hides** it instead of quitting (`WindowEvent::CloseRequested` → `prevent_close`); the
app only exits via the tray "Quit" item.

### Frontend ↔ backend contract

The frontend uses the global Tauri API (`withGlobalTauri: true`, so
`window.__TAURI__.core.invoke` / `.event.listen` — no npm import).

- **Commands** (Rust `#[tauri::command]`, invoked from JS): `translate`, `get_settings`,
  `save_settings`, `set_clipboard`, `hide_popup`, `hide_mini`, `resize_window`,
  `resize_mini`, `set_autostart`, `get_autostart`.
  **Any new command must be added to the `invoke_handler![...]` list in `lib.rs`** or the
  frontend call fails.
- **Events** (Rust → JS via `emit`): `popup-shown` (carries current clipboard text),
  `open-settings` (from tray), `mini-translate` (carries the captured selection text).

### Selection translation (`translate_selection` in `lib.rs`)

The trickiest flow. On the Alt+T hotkey it spawns a thread that:
1. saves the current clipboard, writes a unique **sentinel** string to it;
2. uses **enigo** to simulate Ctrl+C on whatever app is focused (falls back to
   Ctrl+Insert on Windows);
3. reads the clipboard — if it still equals the sentinel, nothing was selected;
4. positions the `mini` window near the saved cursor location and emits `mini-translate`;
5. **restores the user's original clipboard**.

The timing `sleep`s around the simulated copy are load-bearing — the OS needs time to
service the keystrokes and update the clipboard.

### Settings

`Settings`/`Appearance` structs (serde, `camelCase`) persist to `settings.json` in the
OS app-config dir. Every field has a `#[serde(default …)]`, so partial/old JSON loads
without error. Window position (`popup_x`/`popup_y`) is saved on `WindowEvent::Moved`
and restored on show.

Global hotkeys (open / selection) apply **immediately on save**: `register_global_shortcuts`
does `unregister_all` + re-register from the saved settings, exposed as the `update_shortcuts`
command that the frontend calls right after `save_settings`. `setup()` calls the same helper
at startup (falling back to `DEFAULT_HOTKEY`/`DEFAULT_SELECTION_HOTKEY` if a saved combo is
invalid). The in-app swap hotkey (Alt+E) is not a global shortcut — it's matched in JS
(`matchesHotkey` in `main.js`). On save, the settings/appearance panels return to the main
view; a shortcut that fails to register keeps you on the panel with the error.

### Language direction detection

Heuristic, **not** an API call: diacritics → word-list scoring → morphology suffixes →
`w/y/k` tie-breaker. It is **duplicated verbatim in `main.js` and `mini.js`** — keep the
two copies (`PT_WORDS`, `EN_WORDS`, `detectDirection`) in sync when editing either.

### Text-to-speech / voice input

Browser Web Speech API in the frontend (no backend involvement). `scoreVoice` prefers
"Natural"/"Online"/"neural"/"enhanced" system voices over legacy robotic ones. Speech
recognition uses `webkitSpeechRecognition`. Both degrade gracefully (buttons hidden) when
the API is missing.

### Translation engine details (`lib.rs`)

`translate` is DeepL-only: an empty `deepl_key` returns `no_key:`; otherwise it calls
`translate_deepl`. The `mini` selection bubble goes through the same command, so it
behaves identically.

- `translate_deepl` — POSTs to DeepL v2. The endpoint is chosen from the key suffix:
  keys ending in `:fx` are Free-tier and use `api-free.deepl.com`, otherwise `api.deepl.com`.
  Maps langpairs to DeepL's regional codes (`EN-US` / `PT-BR`). Error prefixes the UI keys
  off: HTTP 403 → `deepl_auth:` (bad key), 456 → `limit:` (quota), plus `network_error:` /
  `http_error:` / `decode_error:` / `api_error:`.
- `reqwest` uses native TLS on purpose (trusts OS-installed root CAs — matters behind
  corporate HTTPS interception).

## CI / release

`.github/workflows/build.yml` runs on push to `main` (Windows runner) and **auto-bumps
the patch version on every build** — no manual version edits needed. Each run:
1. reads `version` from `package.json` and increments the patch (e.g. `0.1.4` → `0.1.5`),
   writing it into `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`;
2. builds with `tauri-action` and publishes a GitHub Release tagged `v<version>` with the
   installer attached (each build = a new release, so the download link always changes);
3. commits the bumped files back to `main` with `[skip ci]`.

The bump-back commit uses the default `GITHUB_TOKEN`, whose pushes **do not** re-trigger
`on: push` — that's what prevents an infinite build loop. A `concurrency` group serializes
runs so two pushes can't race to the same version. The README's download link points at
`releases/latest`, so it never needs updating.
