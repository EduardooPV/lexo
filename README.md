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
  <img src="https://img.shields.io/badge/Windows-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/macOS-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
</p>

<p align="center">
  <a href="#sobre-o-projeto">Sobre o projeto</a> ·
  <a href="#stack">Stack</a> ·
  <a href="#preview">Preview</a> ·
  <a href="#atalhos">Atalhos</a> ·
  <a href="#como-rodar">Como rodar</a> ·
  <a href="#notas-de-plataforma">Notas de plataforma</a> ·
  <a href="#cicd-e-release">CI/CD e release</a>
</p>

---

## Sobre o projeto

**Lexo** é um tradutor de desktop leve (Tauri 2) que traduz texto entre **Português e Inglês** a partir de qualquer lugar, por atalhos globais. Ele roda em segundo plano na **bandeja do sistema** (não ocupa a barra de tarefas) e mostra a tradução onde você estiver.

Desenvolvido e testado no **Windows**, que segue sendo a plataforma com todas as funcionalidades (incluindo o OCR de tela, nativo do Windows). Instaladores para **macOS** e **Linux** também são publicados a cada release — veja as [notas de plataforma](#notas-de-plataforma) antes de instalar em um desses sistemas.

A tradução acontece **no backend em Rust**, nunca no webview — isso evita CORS e mantém sua chave de API fora do navegador. O único motor é o **[DeepL](https://www.deepl.com/pro-api)**, escolhido pela melhor qualidade em PT⇄EN. O plano **API Free** oferece 500.000 caracteres/mês sem custo; você adiciona sua própria chave uma vez em Configurações. Sem telemetria.

### Funcionalidades

- **Atalhos globais** que funcionam em qualquer aplicativo — todos remapeáveis por um **gravador de teclas** (você aperta a combinação, não digita o texto)
- **Tradução de seleção** com um balão compacto ao lado do cursor (estilo extensão do Google), **arrastável** e que sempre fica visível dentro da tela
- **Traduzir e substituir**: sobrescreve o texto selecionado com a tradução, no lugar — ideal para escrever e-mail em inglês
- **OCR de região da tela** *(Windows)*: arraste uma caixa sobre qualquer coisa (print, vídeo, PDF escaneado, jogo) e o Windows lê o texto. **100% nativo** — sem serviço externo, sem modelo embutido, sem peso no instalador
- **Histórico** das traduções com busca, fixar favoritos e copiar
- **Cache local** — texto repetido volta na hora e **não consome cota** da DeepL
- **Medidor de cota** da DeepL nas configurações
- **Detecção de idioma pela própria DeepL** (com repescagem automática se o alvo vier errado)
- **Pausar atalhos** pela bandeja, para não conflitar com jogos e outros apps
- **Restauração automática do clipboard** — preserva o que você copiou
- **Text-to-speech** nos dois idiomas, priorizando vozes "Natural"/"Online" do Windows
- **Entrada por voz** (ditar em vez de digitar)
- **Tema customizável** (Dracula por padrão) — cores, opacidade e fonte

---

## Stack

| Camada        | Tecnologia                                                     |
|---------------|----------------------------------------------------------------|
| Shell/Desktop | Tauri 2 (três janelas: `popup`, `mini` e `overlay`)            |
| Backend       | Rust — toda a lógica do app vive aqui                          |
| Frontend      | HTML, CSS e JavaScript vanilla em ES modules (sem bundler)     |
| Tradução      | DeepL API v2 (via `reqwest` com TLS nativo)                    |
| OCR           | GDI (`windows-sys`) + `Windows.Media.Ocr` — nativo, **Windows only** |
| Automação     | `enigo` (simula Ctrl+C / Ctrl+V), global shortcuts             |
| Ícones        | Lucide, vendorizado como path data (a CSP proíbe CDN)          |
| Voz           | Web Speech API (TTS + reconhecimento de fala)                  |
| CI/CD         | GitHub Actions → GitHub Releases (Windows + macOS + Linux)     |

---

## Preview

<img width="521" height="369" alt="image" src="https://github.com/user-attachments/assets/cd28afa1-b026-4eba-b1d6-d71c33e56897" width="100%"/>

---

## Atalhos

| Tecla           | Ação                                                        |
|-----------------|-------------------------------------------------------------|
| **Alt+R**       | Abrir/fechar o tradutor                                     |
| **Alt+T**       | Traduzir o texto selecionado (qualquer app)                 |
| **Alt+Shift+T** | Traduzir a seleção e **substituir** o texto no lugar        |
| **Alt+S**       | Selecionar uma região da tela e traduzir por OCR *(Windows)* |
| **Alt+E**       | Alternar direção da tradução (PT→EN ⇄ EN→PT)                |
| **Enter**       | Traduzir                                                    |
| **Esc**         | Fechar                                                      |

Todos os atalhos globais são remapeáveis em Configurações e valem assim que você salva.

---

## Como rodar

O desenvolvimento deste projeto acontece no Windows — os passos abaixo (e os comandos
`cargo`/`npm`) são os mesmos em qualquer sistema, mas só foram testados no Windows.
Para macOS/Linux, siga também os [pré-requisitos oficiais do Tauri](https://tauri.app/start/prerequisites/) da sua plataforma (no Linux, veja a lista exata de pacotes usada no CI em [`.github/workflows/build.yml`](.github/workflows/build.yml)).

### Pré-requisitos (Windows)

- Rust (toolchain MSVC) — `winget install Rustlang.Rustup`
- **Visual Studio C++ Build Tools** (o Tauri compila via MSVC) — workload `Microsoft.VisualStudio.Workload.VCTools`
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
npm run build       # tauri build — instalador nativo da sua plataforma
```

### Configuração inicial

1. Abra o tradutor com **Alt+R** → **Configurações**
2. Cole sua **chave da DeepL API** e salve
3. Digite ou cole um texto → **Enter** para traduzir
4. Em qualquer app, selecione um texto e pressione **Alt+T**
5. Para traduzir algo que não dá para selecionar (print, vídeo, PDF escaneado), pressione **Alt+S** e arraste uma caixa sobre o texto

> O OCR usa o motor que já vem no Windows. Se ele reclamar de pacote de idioma,
> instale Português ou Inglês em *Configurações do Windows › Hora e idioma*.

### Checagens Rust (dentro de `src-tauri/`)

```bash
cargo check         # type-check rápido do backend
cargo clippy        # lint
cargo fmt           # formatação
```

---

## Notas de plataforma

Cada [release](https://github.com/EduardooPV/lexo/releases/latest) sai com instalador para Windows, macOS e Linux, mas nem toda funcionalidade existe nos três:

| Funcionalidade | Windows | macOS | Linux |
|---|:---:|:---:|:---:|
| Tradução, histórico, cache, TTS, entrada por voz | ✅ | ✅ | ✅ |
| Tradução de seleção / traduzir e substituir | ✅ | ✅ * | ✅ * † |
| Atalhos globais | ✅ | ✅ | ✅ † |
| OCR de região da tela | ✅ | ❌ | ❌ |

**\*** No macOS, na primeira vez que você usar Alt+T/Alt+Shift+T o sistema vai pedir
permissão de **Acessibilidade** para o Lexo (necessária para simular Ctrl+C/Ctrl+V) —
conceda em *Ajustes do Sistema › Privacidade e Segurança › Acessibilidade*.

**†** No Linux essas funcionalidades dependem de X11 (via `libxdo`). Em uma sessão
**Wayland nativa** elas podem não funcionar, já que o compositor restringe entrada
sintética e atalhos globais por padrão — isso não foi testado em nenhuma distro; se
usar Wayland, prefira uma sessão XWayland/X11 se notar que os atalhos não respondem.

**Instaladores de macOS e Linux não são assinados** (sem notarização da Apple, sem
assinatura de pacote):

- **macOS**: ao abrir o `.dmg` pela primeira vez, o Gatekeeper vai dizer que o app é de
  um desenvolvedor não identificado. Clique com o botão direito no app → **Abrir**, ou
  rode `xattr -cr /Applications/Lexo.app` no Terminal.
- **Linux**: o `.AppImage` pode precisar de `chmod +x` antes de rodar; `.deb`/`.rpm` não
  passam por nenhuma verificação de assinatura, o que é normal para builds de projetos
  pequenos fora dos repositórios oficiais da distro.

---

## CI/CD e release

`.github/workflows/build.yml` roda a cada push na `main` e **incrementa o patch da versão automaticamente** a cada build — sem edição manual de versão. Três jobs:

1. **`version`** calcula a próxima versão (patch) a partir do `package.json`, sem escrever nada ainda;
2. **`build`** é uma matriz — Windows, macOS (binário universal Intel + Apple Silicon) e Linux — onde cada plataforma builda com `tauri-action` e publica sob a **mesma tag** de release;
3. **`finalize`** só roda depois que **todas** as plataformas compilaram com sucesso, e é o único job que de fato commita a versão bumpada de volta na `main` com `[skip ci]`.

Essa ordem existe para que a versão em `main` só avance quando a release inteira (as três plataformas) realmente saiu — se uma plataforma falhar, a versão não é bumpada.

O link de download do README aponta para `releases/latest`, então nunca precisa ser atualizado.

---

<p align="center">
  Feito com Tauri 2 + Rust · <a href="https://github.com/EduardooPV/lexo/releases/latest">📥 Baixar última versão</a>
</p>
