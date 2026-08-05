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
  <img src="https://img.shields.io/badge/DeepL-API-0F2B46?style=flat-square&logo=deepl&logoColor=white" alt="DeepL" />
  <img src="https://img.shields.io/badge/Windows-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/macOS-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
</p>

<p align="center">
  <a href="#download">Download</a> ·
  <a href="#como-usar">Como usar</a> ·
  <a href="#atalhos">Atalhos</a> ·
  <a href="#funcionalidades">Funcionalidades</a> ·
  <a href="#notas-de-plataforma">Notas de plataforma</a> ·
  <a href="#desenvolvimento">Desenvolvimento</a>
</p>

---

## Download

📥 **[Baixar a última versão](https://github.com/EduardooPV/lexo/releases/latest)** — Windows (`.exe`/`.msi`), macOS (`.dmg`) e Linux (`.AppImage`/`.deb`/`.rpm`).

Precisa apenas de uma chave grátis da **[DeepL API](https://www.deepl.com/pro-api)** (500.000 caracteres/mês, sem custo). A tradução roda no backend em Rust — sua chave nunca passa pelo navegador. Sem telemetria.

---

## Como usar

1. Baixe e instale (link acima)
2. Abra com **Alt+R** → **Configurações** → cole a chave da DeepL → **Salvar**
3. Digite ou cole um texto → **Enter** para traduzir
4. Em qualquer app, selecione um texto e pressione **Alt+T**

Todos os atalhos são remapeáveis em Configurações (você aperta a combinação, não digita) e valem assim que salva.

## Atalhos

| Tecla           | Ação                                              |
|-----------------|----------------------------------------------------|
| **Alt+R**       | Abrir/fechar o tradutor                             |
| **Alt+T**       | Traduzir o texto selecionado (qualquer app)         |
| **Alt+Shift+T** | Traduzir a seleção e **substituir** no lugar        |
| **Alt+S**       | Selecionar uma região da tela e traduzir por OCR *(Windows)* |
| **Alt+E**       | Alternar direção da tradução (PT→EN ⇄ EN→PT)        |
| **Enter**       | Traduzir                                            |
| **Esc**         | Fechar                                              |

---

## Funcionalidades

- **Tradução de seleção** num balão compacto ao lado do cursor — arrastável e sempre visível na tela
- **Traduzir e substituir**: sobrescreve o texto selecionado com a tradução, no lugar
- **OCR de região da tela** *(Windows)*: arraste uma caixa sobre print, vídeo ou PDF escaneado. 100% nativo — sem serviço externo, sem modelo embutido
- **Histórico** com busca, favoritos e copiar, e **cache local** — texto repetido não consome cota da DeepL
- **Medidor de cota** da DeepL nas configurações
- **Direção decidida pela própria DeepL** (com repescagem automática se o alvo vier errado)
- **Pausar atalhos** pela bandeja, para não conflitar com jogos e outros apps
- **Text-to-speech** e **entrada por voz** nos dois idiomas
- **Tema customizável** — cores, opacidade e fonte

---

## Notas de plataforma

O app é desenvolvido e testado no **Windows**, onde tudo funciona, incluindo o OCR (nativo, nunca chega a mac/Linux). Em macOS e Linux, a tradução, o histórico, atalhos e a tradução de seleção funcionam, com duas ressalvas:

- **macOS** pede permissão de **Acessibilidade** na primeira vez que você usar Alt+T/Alt+Shift+T (necessária para simular Ctrl+C/Ctrl+V) — conceda em *Ajustes do Sistema › Privacidade e Segurança*.
- **Linux**: atalhos globais e simulação de teclado dependem de X11; numa sessão **Wayland nativa** podem não responder (não testado em nenhuma distro).

Os instaladores de macOS e Linux **não são assinados**: no macOS, clique com o botão direito no app → **Abrir** na primeira vez (ou `xattr -cr` nele); no Linux, dê `chmod +x` no `.AppImage`.

---

## Desenvolvimento

```bash
git clone https://github.com/EduardooPV/lexo.git
cd lexo
npm install         # instala apenas o @tauri-apps/cli; deps Rust vêm do Cargo
npm run dev          # tauri dev — hot-reload do frontend, rebuild do Rust ao alterar
npm run build        # tauri build — instalador nativo da sua plataforma
```

Checagens do backend (dentro de `src-tauri/`): `cargo check`, `cargo clippy`, `cargo fmt`.

**Pré-requisitos (Windows, plataforma de desenvolvimento):** Rust (toolchain MSVC), as [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) e Node 16+. Para macOS/Linux, siga os [pré-requisitos do Tauri](https://tauri.app/start/prerequisites/) da sua plataforma — veja os pacotes exatos usados em CI em [`.github/workflows/build.yml`](.github/workflows/build.yml).

`.github/workflows/build.yml` builda Windows, macOS e Linux sob a mesma tag de release a cada push na `main`, e **incrementa a versão automaticamente** — só depois que as três plataformas compilarem com sucesso.

---

<p align="center">
  Feito com Tauri 2 + Rust · <a href="https://github.com/EduardooPV/lexo/releases/latest">📥 Baixar última versão</a>
</p>
