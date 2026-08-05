<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="Lexo" width="96" />
</p>

<h1 align="center">Lexo</h1>

<p align="center">
  A global-hotkey PT ⇄ EN desktop translator. Lives in the system tray and translates text from any app, instantly.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-backend-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/DeepL-API-0F2B46?style=flat-square&logo=deepl&logoColor=white" alt="DeepL" />
  <img src="https://img.shields.io/badge/Windows-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/macOS-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
</p>

<p align="center">
  <a href="#download">Download</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#shortcuts">Shortcuts</a> ·
  <a href="#features">Features</a> ·
  <a href="#platform-notes">Platform notes</a> ·
  <a href="#development">Development</a>
</p>

---

## Download

📥 **[Download the latest release](https://github.com/EduardooPV/lexo/releases/latest)** — Windows, macOS and Linux.

All you need is a free **[DeepL API](https://www.deepl.com/pro-api)** key (500,000 characters/month, no cost). Translation runs in the Rust backend — your key never touches the browser. No telemetry.

---

## Usage

1. Download and install (link above)
2. Open with **Alt+R** → **Settings** → paste your DeepL key → **Save**
3. Type or paste text → **Enter** to translate
4. In any app, select text and press **Alt+T**

Every shortcut is remappable in Settings (you press the combination, not type it) and applies as soon as you save.

## Shortcuts

| Key             | Action                                             |
|-----------------|-----------------------------------------------------|
| **Alt+R**       | Open/close the translator                            |
| **Alt+T**       | Translate the selected text (any app)                |
| **Alt+Shift+T** | Translate the selection and **replace** it in place  |
| **Alt+S**       | Select a screen region and translate it via OCR *(Windows)* |
| **Alt+E**       | Toggle translation direction (PT→EN ⇄ EN→PT)         |
| **Enter**       | Translate                                            |
| **Esc**         | Close                                                |

---

## Features

- **Selection translation** in a compact bubble next to the cursor — draggable and always kept on screen
- **Translate and replace**: overwrites the selected text with its translation, in place
- **Screen-region OCR** *(Windows)*: drag a box over a screenshot, video, or scanned PDF. Fully native — no external service, no bundled model
- **History** with search, favorites, and copy, plus **local caching** — repeated text costs zero DeepL characters
- **Usage quota meter** in Settings
- **Direction decided by DeepL itself** (with an automatic retry if the guessed target is wrong)
- **Pause shortcuts** from the tray, to avoid conflicts with games and other apps
- **Text-to-speech** and **voice input** in both languages
- **Customizable theme** — colors, opacity, and font

## Development

```bash
git clone https://github.com/EduardooPV/lexo.git
cd lexo
npm install
npm run dev
npm run build
```

`npm install` only installs `@tauri-apps/cli`; Rust dependencies come from Cargo. `npm run dev` runs `tauri dev` (hot-reloads the frontend, rebuilds Rust on change); `npm run build` produces your platform's native installer.

Backend checks (inside `src-tauri/`): `cargo check`, `cargo clippy`, `cargo fmt`.

**Prerequisites (Windows, the primary dev platform):** Rust (MSVC toolchain), the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022), and Node 16+. For macOS/Linux, follow Tauri's own [prerequisites](https://tauri.app/start/prerequisites/) for your platform — see the exact packages used in CI in [`.github/workflows/build.yml`](.github/workflows/build.yml).

`.github/workflows/build.yml` builds Windows, macOS, and Linux under the same release tag on every push to `main`, and **bumps the version automatically** — only after all three platforms build successfully.

---

<p align="center">
  Built with Tauri 2 + Rust · <a href="https://github.com/EduardooPV/lexo/releases/latest">📥 Download the latest release</a>
</p>
