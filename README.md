<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="Lexo" width="96" />
</p>

<h1 align="center">Lexo</h1>

<p align="center">
  A global-hotkey desktop translator. Lives in the system tray and translates text from any app, instantly.
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

## Download

📥 **[Download the latest release](https://github.com/EduardooPV/lexo/releases/latest)** — Windows, macOS and Linux.

All you need is a free **[DeepL API](https://www.deepl.com/pro-api)** key (500,000 characters/month, no cost). The Rust backend sends your key to DeepL for translation and usage checks. Lexo stores it in your system credential store when available and warns you if it has to use the local settings file. No telemetry.

## Usage

1. Download and install (link above)
2. Open with **Alt+R** → the welcome screen links to DeepL, paste the key → **Connect**
3. Type or paste text → **Enter** to translate
4. In any app, select text and press **Alt+T**

Every shortcut is remappable in the **Shortcuts** panel (you press the combination, not type it) and applies as soon as you save.

## Preview

https://github.com/user-attachments/assets/6274bf59-a5e6-40f6-883b-4eace1fb39b8

https://github.com/user-attachments/assets/c490ebe1-7e29-4074-b444-5d663b5d007e

## Shortcuts

| Key             | Action                                             |
|-----------------|-----------------------------------------------------|
| **Alt+R**       | Open/close the translator                            |
| **Alt+T**       | Translate the selected text (any app)                |
| **Alt+Shift+T** | Translate the selection and **replace** it in place  |
| **Alt+S**       | Select a screen region and translate it via OCR *(Windows)* |
| **Alt+E**       | Switch between your two preferred target languages         |
| **Enter**       | Translate                                            |
| **Esc**         | Close                                                |

---

## Features

- **Selection translation** in a compact bubble next to the cursor — draggable and always kept on screen
- **Translate and replace**: overwrites the selected text with its translation, in place
- **Screen-region OCR** *(Windows)*: drag a box over a screenshot, video, or scanned PDF. Fully native — no external service, no bundled model
- **Automatic source detection** and over 100 DeepL target languages. The default pair is Brazilian Portuguese and American English. Select a different pair in Settings or choose a target for a single translation.
- **Formality** preferences for languages that support them
- **Optional translation history** with search, favorites, copy, and JSON or CSV export. Local caching reuses matching translations.
- **Monthly quota** in the popup and Settings, with a system notification at 80% and confirmation before translating more than 5,000 characters
- **Pause shortcuts** from the tray, to avoid conflicts with games and other apps
- **Text-to-speech and voice input** using the voices and speech recognition available on your system
- **Customizable theme** — colors, opacity, and font

## Platform notes

None of the installers are code-signed yet, so the OS will warn you on first run:

- **Windows**: SmartScreen shows "Windows protected your PC" — click **More info** → **Run anyway**.
- **macOS**: Gatekeeper blocks the unsigned app — right-click it → **Open** the first time (or run `xattr -cr` on it).
- **Linux**: the `.AppImage` needs `chmod +x` before it will run.

This is normal for a small open-source project without a paid code-signing certificate — nothing is wrong with the download.

## Development

```bash
git clone https://github.com/EduardooPV/lexo.git
cd lexo
npm ci
npm run dev
npm run build
```

`npm ci` installs the Tauri CLI and frontend test tools. Rust dependencies come from Cargo. `npm run dev` runs `tauri dev` (hot-reloads the frontend, rebuilds Rust on change); `npm run build` produces your platform's native installer.

Run `npm test` from the root. In `src-tauri/`, run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Prerequisites (Windows, the primary dev platform):** Rust (MSVC toolchain), the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022), and Node 20.19+ on the 20.x line or a supported newer release. For macOS/Linux, follow Tauri's own [prerequisites](https://tauri.app/start/prerequisites/) for your platform — see the exact packages used in CI in [`.github/workflows/build.yml`](.github/workflows/build.yml).

`.github/workflows/build.yml` builds Windows, macOS, and Linux under the same release tag on every push to `main`, and **bumps the version automatically** — only after all three platforms build successfully.

---

<p align="center">
  Built with Tauri 2 + Rust · <a href="https://github.com/EduardooPV/lexo/releases/latest">📥 Download the latest release</a>
</p>
