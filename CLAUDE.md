# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A lightweight **Tauri 2** desktop app (Windows-first) that translates text PT ⇄ EN
from anywhere via global hotkeys. It lives in the system tray, not the taskbar.
The npm package is named `tradutor`; the product/binary is `Lexo` (bundle identifier
stays `com.luizveltroni.tradutor` so existing settings/keys are preserved).

- **Alt+R** — toggle the main translator popup
- **Alt+T** — translate the current selection in any app (bubble at the cursor)
- **Alt+Shift+T** — translate the selection and **paste the result over it**
- **Alt+S** — drag a box on screen; Windows OCR reads it and translates it
- **Alt+E** — flip translation direction (in-app, while the popup is focused)

All of these are remappable in Settings via a key recorder and apply on save.

Translation runs **in the Rust backend**, never in the webview — this avoids CORS and
keeps the API key out of the browser. The **only** engine is **DeepL**; a DeepL API key
set in Settings is required, and without one `translate` returns a `no_key:` error the
UI surfaces. No telemetry, no other network calls.

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

**Requirements (Windows, the primary dev target):** Rust (MSVC toolchain), the
**Visual Studio C++ Build Tools**, Node 16+, and the WebView2 runtime (preinstalled on
Windows 11). Building on macOS or Linux follows Tauri's own per-OS prerequisites —
see the CI job in `.github/workflows/build.yml` for the exact Linux system packages;
there is no local macOS/Linux dev setup documented here because this repo has only ever
been developed and tested on Windows.

**There is no test suite and no JS lint/build config** — the frontend is plain files
served as-is (no bundler, no transpile step). Don't invent a test command.

## Architecture

### Three webview windows

- **Backend** (`src-tauri/src/lib.rs`) — all app logic. `main.rs` is a one-line entry
  point calling `tradutor_lib::run()`. `src-tauri/src/ocr.rs` holds the screen-OCR code.
- **Frontend** — vanilla **ES modules** in `src/` (no bundler; `type="module"` works
  because Tauri serves the files over its own protocol), loaded into three windows
  declared in `src-tauri/tauri.conf.json`:
  - `popup` → `index.html` / `main.js` — translator, history, settings, appearance
  - `mini` → `mini.html` / `mini.js` — the compact bubble for a selection or an OCR grab
  - `overlay` → `overlay.html` / `overlay.js` — the full-screen region picker for OCR

All three are `transparent`, `decorations: false`, `alwaysOnTop`. Closing a window
**hides** it (`WindowEvent::CloseRequested` → `prevent_close`); the app only exits via
the tray "Quit" item.

### Shared frontend modules

`main.js`, `mini.js` and `overlay.js` all import from:

- `api.js` — every `invoke` wrapper **and** `describeError`, the single place that turns
  a backend error prefix (`no_key:`, `deepl_auth:`, `limit:`, `ocr_unavailable:` …) into
  human text. Add new error prefixes here, not in the callers.
- `icons.js` — the icon set (Lucide path data) vendored as strings, because the CSP
  (`default-src 'self'`) forbids any CDN. `hydrate(root)` fills every `[data-icon]`
  element; `setIcon(node, name)` swaps one. **Never hand-roll an inline `<svg>` in
  markup** — add a glyph to `GLYPHS` and reference it by name.
- `theme.js` — `THEMES`, `DEFAULT_APPEARANCE`, `applyAppearance`, `readableOn`.

`base.css` holds the design tokens and the **button system**; `styles.css`, `mini.css`
and `overlay.css` only add window-specific layout. Every clickable control is
`.btn` plus modifiers (`--primary`, `--ghost`, `--subtle`, `--danger`, `--icon`,
`--sm`, `--xs`, `--block`). **Do not introduce a new bespoke button class** — height,
radius, focus ring and transitions are defined once in `base.css`.

### Frontend ↔ backend contract

The frontend uses the global Tauri API (`withGlobalTauri: true`), wrapped by `api.js`.

**Commands** — `translate`, `get_usage`, `get_settings`, `save_settings`,
`update_shortcuts`, `get_shortcuts_paused`, `set_shortcuts_paused`, `get_history`,
`clear_history`, `delete_history_entry`, `toggle_history_pin`, `set_clipboard`,
`hide_popup`, `hide_mini`, `resize_window`, `resize_mini`, `set_autostart`,
`get_autostart`, `ocr_available`, `start_region_capture`, `ocr_region`,
`cancel_region_capture`.
**Any new command must be added to `invoke_handler![...]` in `lib.rs`** or the call fails.

**Events** (Rust → JS) — `popup-shown` (carries clipboard text, empty unless the
auto-translate setting is on), `open-view` (tray asking for `"history"` / `"settings"`),
`mini-translate` (payload `{ text, origin: "selection" | "ocr", error }`).

### Language direction

There is **no** language-detection heuristic in JavaScript any more — it lives once in
`guess_target` in `lib.rs`, and it only picks which language to translate *into*.
DeepL detects the source itself (`source_lang` is deliberately omitted), and if it
reports the text was already in the target language, `translate_text` retries once the
other way. The response carries `detectedSource`, which is what the UI badge shows.

The popup badge is accent-coloured when DeepL is deciding and pink (`is-forced`) when
the user has pinned a direction. Clicking it (`cycleDirection` in `main.js`) is a plain
2-way toggle of **whatever direction is currently shown** — pinned or auto-detected —
not a 3-way cycle back through "auto". An earlier version cycled auto → EN → PT → auto;
when the auto-detected result already happened to be `EN` (the common PT→EN case), the
first click landed back on `EN` and only changed the badge's color, so it took two
clicks to see the direction actually flip. Once pinned, a direction stays pinned; there
is no click path back to "auto" (matches the pre-rewrite app's behavior — `forcedDir`
was never reset either).

### History and cache

Both live next to `settings.json` in the OS app-config dir and are mirrored in
`AppState` behind mutexes:

- `cache.json` — last 400 `(target, text) → translation` pairs. A hit costs **zero**
  DeepL characters, which is why repeats are instant.
- `history.json` — last 200 entries plus every pinned one (pinned entries are never
  evicted and survive "Clear"). Re-translating the same text updates the existing entry
  instead of adding a duplicate.

Entry `id`s are **strings**, not numbers: they are nanosecond timestamps and would lose
precision as JSON numbers in the webview.

### Selection flows (`lib.rs`)

`capture_selection` is shared by both: it saves the clipboard, writes a unique
**sentinel**, simulates Ctrl+C with **enigo** (falling back to Ctrl+Insert on Windows),
and reads the clipboard back — if it still equals the sentinel, nothing was selected.
The `sleep`s around the simulated keystrokes are load-bearing; the OS needs time to
deliver them and update the clipboard.

- `translate_selection` restores the clipboard immediately and shows the `mini` bubble.
- `replace_selection` translates first, writes the result to the clipboard, simulates
  Ctrl+V, and **only then** restores the user's original clipboard.

### The `mini` bubble

`mini.html`'s root carries `data-tauri-drag-region="deep"`, so the whole bubble is
draggable — Tauri's drag-region logic already excludes clickable elements (buttons)
from the drag, and the two text blocks are explicitly opted back out with
`data-tauri-drag-region="false"` so they stay text-selectable instead of dragging the
window. Because dragging hands mouse capture to the OS, it can blur the webview
mid-drag; `mini.js` tracks a `grabbedAt` timestamp from a capturing `mousedown` listener
so the blur-to-dismiss handler ignores blurs that happen right after a drag starts —
without it, trying to move the bubble would immediately close it.

`resize_mini` (`lib.rs`) both sizes the window **and** re-clamps its position, using the
size it is about to have rather than the one it currently has. The bubble is first
placed at the cursor with whatever height the *previous* translation left it at, then
grows to fit the new text — clamping before that resize used stale dimensions and let
long translations hang off the bottom of the screen. `clamp_to_screen` uses each
monitor's `work_area()`, not its full `size()`, so the bubble also never slides under
the Windows taskbar.

### Screen OCR (`ocr.rs`)

Windows-only and completely native — **no service, no bundled model, nothing added to
the installer**. `start_region_capture` sizes the `overlay` window to the monitor under
the cursor; the drag rectangle comes back in CSS pixels and is scaled by the window's
DPI factor. Then:

1. GDI (`BitBlt` + `GetDIBits` via **windows-sys**) copies the region into a top-down
   BGRA buffer — alpha is forced opaque, since `BitBlt` leaves it undefined;
2. small regions are nearest-neighbour upscaled (Windows OCR rejects anything under
   40px a side and reads small text poorly);
3. `Windows.Media.Ocr` (via the **windows** crate) recognises it, and the lines are
   joined with newlines;
4. the text is handed to the `mini` bubble, which translates it like any selection.

`windows-sys` is used for the Win32/GDI half on purpose: its handles are plain type
aliases, so the code does not break when the `windows` crate reshuffles its Win32
signatures between releases. `ocr::available()` gates the UI on non-Windows builds.

### Settings

`Settings`/`Appearance` structs (serde, `camelCase`) persist to `settings.json`. Every
field has a `#[serde(default …)]`, so partial/old JSON loads without error.
`auto_translate_clipboard` defaults to **false** — it costs a DeepL call on every open,
so it is opt-in.

`save_settings` **merges** over the stored file: `popup_x`/`popup_y` and
`shortcuts_paused` are not owned by the UI and would otherwise be wiped on every save.

Global hotkeys apply **immediately on save**: `register_global_shortcuts` does
`unregister_all` + re-register, exposed as `update_shortcuts`, which the frontend calls
right after `save_settings`. A combo that fails to register keeps you on the panel with
the error. The tray's "Pause shortcuts" flips `shortcuts_paused`, which makes
`register_global_shortcuts` unregister everything and return early, and rebuilds the
tray menu so the label flips to "Resume shortcuts".

The in-app swap hotkey (Alt+E) is **not** a global shortcut — it is matched in JS
(`matchesHotkey` in `main.js`), which shares `keyFromEvent` with the settings key
recorder so both agree on how a combo is spelled.

**Migrations**: `Settings.settings_version` (bumped as `SETTINGS_VERSION` in `lib.rs`)
tracks the settings schema. `migrate_settings`, called once at startup before anything
else reads settings, one-time-fixes stored files older than the current version. It
exists because `auto_translate_clipboard` used to default to `true` back when the
setting didn't actually do anything; once it started spending a DeepL call on every
open, every pre-existing settings file still said `true` from the old default, which
would have silently turned the feature on for upgrading users even though the new
default is opt-in. **If you change what a default means** (not just its literal
default value), bump `SETTINGS_VERSION` and add the fixup here — don't rely on
`#[serde(default)]` alone, since that only helps fields that were *absent*, not ones
that were present with a now-stale value.

### Translation engine details (`lib.rs`)

`translate_deepl` POSTs to DeepL v2. The endpoint comes from the key suffix: keys ending
in `:fx` are Free-tier and use `api-free.deepl.com`, otherwise `api.deepl.com`. Targets
map to DeepL's regional codes (`EN-US` / `PT-BR`). `get_usage` hits `/v2/usage` for the
quota bar in Settings. Errors are prefixed so the UI can key off them: HTTP 403 →
`deepl_auth:`, 456 → `limit:`, plus `network_error:` / `http_error:` / `decode_error:` /
`api_error:`.

`reqwest` uses native TLS on purpose (trusts OS-installed root CAs — matters behind
corporate HTTPS interception).

## CI / release

`.github/workflows/build.yml` runs on push to `main` and **auto-bumps the patch version
on every build** — no manual version edits needed. It builds Windows, macOS and Linux
installers under one release, in three jobs:

1. **`version`** (ubuntu-latest) reads `version` from `package.json`, computes the next
   patch, and only outputs it — it does not write to any file or commit anything.
2. **`build`** is a matrix (`windows-latest`, `macos-latest` with
   `--target universal-apple-darwin`, `ubuntu-22.04`) that all depend on `version`'s
   output, so every platform releases under the **identical** tag. Each job bakes the
   computed version into its own throwaway checkout (never committed there), then builds
   with `tauri-action`, which publishes/updates a single GitHub Release tagged
   `v<version>` and appends that platform's installer to it. `fail-fast: false` so one
   platform's failure doesn't cancel the others mid-build.
3. **`finalize`** runs only after `build` succeeds **on every platform**, applies the
   same version bump to `package.json` / `tauri.conf.json` / `Cargo.toml`, runs
   `cargo check` so `Cargo.lock`'s own recorded version for the `tradutor` package stays
   in sync, and commits+pushes all four files back to `main` with `[skip ci]`.

Splitting it this way means a version is only ever persisted to `main` once every
platform has actually shipped — if e.g. the Linux build fails, `main` stays on the old
version instead of drifting ahead of what got released. The commit uses the default
`GITHUB_TOKEN`, whose pushes **do not** re-trigger `on: push` (prevents an infinite
loop); the `concurrency` group serializes whole workflow runs so two pushes can't race.
The README's download link points at `releases/latest`, so it never needs updating.

**Ubuntu is pinned to `22.04`**, not `-latest`, for `libwebkit2gtk-4.1-dev` availability
— this is Tauri's own documented recommendation, not an arbitrary choice.
`libxdo-dev` in the Linux dependency list is for **enigo** (selection translate /
replace-in-place simulate Ctrl+C/Ctrl+V via libxdo on X11) — without it those two
features fail to build, not just fail at runtime.

**Platform parity is not equal.** OCR (`ocr.rs`) is Windows-only by design — `ocr::available()`
returns `false` elsewhere and the frontend hides the button; the `overlay` window still
gets declared cross-platform but sits unused. Selection translate / replace-in-place
rely on `enigo`, which uses X11 (`libxdo`) on Linux — **under a native Wayland session,
both those features and the global shortcuts plugin may not work at all**, since Wayland
compositors deliberately restrict synthetic input and global key grabs; this has not
been tested on any Linux machine, only inferred from how the dependencies work. macOS
and Linux builds are **unsigned** (no Apple notarization, no package signing) — see the
README's platform notes for what that means for a first launch.

There is **no auto-updater yet**: users still download each release manually. Adding
`tauri-plugin-updater` needs a signing keypair (`TAURI_SIGNING_PRIVATE_KEY` as a repo
secret, public key in `tauri.conf.json`) before it can work.
