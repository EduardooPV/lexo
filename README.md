<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="Lexo" width="96" />
</p>

<h1 align="center">Lexo</h1>

<p align="center">
  Tradutor de desktop PT ⇄ EN por atalho global. Vive na bandeja do sistema e traduz texto de qualquer aplicativo, na hora.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-backend-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/JavaScript-vanilla-F7DF1E?style=flat-square&logo=javascript&logoColor=black" alt="JavaScript" />
  <img src="https://img.shields.io/badge/DeepL-API-0F2B46?style=flat-square&logo=deepl&logoColor=white" alt="DeepL" />
  <img src="https://img.shields.io/badge/Windows-first-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
</p>

<p align="center">
  <a href="#sobre-o-projeto">Sobre o projeto</a> ·
  <a href="#stack">Stack</a> ·
  <a href="#preview">Preview</a> ·
  <a href="#atalhos">Atalhos</a> ·
  <a href="#como-rodar">Como rodar</a> ·
  <a href="#cicd-e-release">CI/CD e release</a>
</p>

---

## Sobre o projeto

**Lexo** é um tradutor de desktop leve (Tauri 2, Windows-first) que traduz texto entre **Português e Inglês** a partir de qualquer lugar, por atalhos globais. Ele roda em segundo plano na **bandeja do sistema** (não ocupa a barra de tarefas) e mostra a tradução onde você estiver.

A tradução acontece **no backend em Rust**, nunca no webview — isso evita CORS e mantém sua chave de API fora do navegador. O único motor é o **[DeepL](https://www.deepl.com/pro-api)**, escolhido pela melhor qualidade em PT⇄EN. O plano **API Free** oferece 500.000 caracteres/mês sem custo; você adiciona sua própria chave uma vez em Configurações. Sem telemetria.

### Funcionalidades

- **Atalhos globais** que funcionam em qualquer aplicativo (abrir, traduzir seleção, inverter direção — todos remapeáveis)
- **Tradução de seleção** com um balão compacto ao lado do cursor (estilo extensão do Google)
- **Restauração automática do clipboard** — preserva o que você copiou
- **Text-to-speech** nos dois idiomas, priorizando vozes "Natural"/"Online" do Windows
- **Entrada por voz** (ditar em vez de digitar)
- **Tema customizável** (Dracula por padrão) — cores, opacidade e fonte
- Integração com **system tray** e memória da posição da janela
- **Detecção heurística de direção** PT↔EN (sem chamada de API)

---

## Stack

| Camada        | Tecnologia                                             |
|---------------|--------------------------------------------------------|
| Shell/Desktop | Tauri 2 (duas janelas: `popup` e `mini`)               |
| Backend       | Rust — toda a lógica do app vive aqui                  |
| Frontend      | HTML, CSS e JavaScript vanilla (sem bundler/framework) |
| Tradução      | DeepL API v2 (via `reqwest` com TLS nativo)            |
| Automação     | `enigo` (simula Ctrl+C na seleção), global shortcuts   |
| Voz           | Web Speech API (TTS + reconhecimento de fala)          |
| CI/CD         | GitHub Actions → GitHub Releases (installer Windows)   |

---

## Preview

> Prints do app rodando. Substitua os arquivos em `docs/screenshots/` (ou arraste as imagens direto no editor do GitHub) pelos seus próprios.

<!-- Dica: você pode arrastar imagens direto na caixa de edição do README no GitHub —
     ele hospeda em github.com/user-attachments/assets/... e gera o link automaticamente. -->

![Popup principal do tradutor (Alt+R)](./docs/screenshots/popup.png)

![Tradução de seleção com balão no cursor (Alt+T)](./docs/screenshots/selection.png)

![Configurações e aparência](./docs/screenshots/settings.png)

---

## Atalhos

| Tecla     | Ação                                        |
|-----------|---------------------------------------------|
| **Alt+R** | Abrir/fechar o tradutor                     |
| **Alt+T** | Traduzir o texto selecionado (qualquer app) |
| **Alt+E** | Inverter direção PT↔EN                       |
| **Enter** | Traduzir                                    |
| **Esc**   | Fechar                                      |

---

## Como rodar

### Pré-requisitos

- Rust 1.60+
- Node.js 16+
- Uma chave da **[DeepL API](https://www.deepl.com/pro-api)** (o plano API Free basta)

### 1. Clone o repositório

```bash
git clone https://github.com/EduardooPV/lexo.git
cd lexo
```

### 2. Instale as dependências

```bash
npm install         # instala apenas o @tauri-apps/cli; deps Rust vêm do Cargo
```

### 3. Rode em desenvolvimento

```bash
npm run dev         # tauri dev — hot-reload do frontend, rebuild do Rust ao alterar
```

### 4. Gere o instalador de produção

```bash
npm run build       # tauri build — instalador NSIS (.exe) / .msi no Windows
```

### Configuração inicial

1. Abra o tradutor com **Alt+R** → **⚙ Configurações**
2. Cole sua **chave da DeepL API** e salve
3. Digite ou cole um texto → **Enter** para traduzir
4. Em qualquer app, selecione um texto e pressione **Alt+T**

### Checagens Rust (dentro de `src-tauri/`)

```bash
cargo check         # type-check rápido do backend
cargo clippy        # lint
cargo fmt           # formatação
```

---

## CI/CD e release

`.github/workflows/build.yml` roda a cada push na `main` (runner Windows) e **incrementa o patch da versão automaticamente** a cada build — sem edição manual de versão. Cada execução:

1. lê a `version` do `package.json` e incrementa o patch, propagando para `package.json`, `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml`;
2. builda com `tauri-action` e publica um GitHub Release `v<version>` com o instalador anexado;
3. commita os arquivos com a versão bumpada de volta na `main` com `[skip ci]`.

O link de download do README aponta para `releases/latest`, então nunca precisa ser atualizado.

---

<p align="center">
  Feito com Tauri 2 + Rust · <a href="https://github.com/EduardooPV/lexo/releases/latest">📥 Baixar última versão (Windows)</a>
</p>
