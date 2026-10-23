# Lumos

**A local-first personal knowledge base that answers with traceable citations.**

[English](README.en.md) · [简体中文](README.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![CI](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml/badge.svg)](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DaiYiqq/lumos)](https://github.com/DaiYiqq/lumos/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/DaiYiqq/lumos/total)](https://github.com/DaiYiqq/lumos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Demo](https://img.shields.io/badge/demo-online-brightgreen)](https://daiyiqq.github.io/lumos/)

Lumos ingests PDF / EPUB / Word / Markdown / HTML / plain text and code, indexes everything locally,
and answers questions **only from your documents** — every claim carries a `[n]` citation that jumps
back to the exact passage, page and section. Your files, embeddings and chat history stay in one
SQLite database on your machine.

> ▶ **Try the online demo:** <https://daiyiqq.github.io/lumos/>
>
> The demo runs entirely in your browser with a few sample documents and a mock backend — nothing is
> uploaded and no API key is needed. Install the desktop app for your own documents, local embeddings,
> file watching and real AI providers.

![Online demo](docs/assets/demo-online.png)

## Download

Installers are published on **GitHub Releases** — no account or sign-up is needed to download.
Pick the file for your OS and architecture:

| Platform    | Architecture          | Installer                 | Direct download                                                                                                      |
| ----------- | --------------------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Windows** | x64                   | NSIS `.exe` (recommended) | [Lumos_0.2.0_x64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_x64-setup.exe)     |
| **Windows** | x64                   | MSI                       | [Lumos_0.2.0_x64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_x64_en-US.msi)     |
| **Windows** | arm64                 | NSIS `.exe`               | [Lumos_0.2.0_arm64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_arm64-setup.exe) |
| **Windows** | arm64                 | MSI                       | [Lumos_0.2.0_arm64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_arm64_en-US.msi) |
| **macOS**   | Apple Silicon (M1–M4) | `.dmg`                    | [Lumos_0.2.0_aarch64.dmg](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_aarch64.dmg)         |
| **Linux**   | x64                   | `.AppImage` (portable)    | [Lumos_0.2.0_amd64.AppImage](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_amd64.AppImage)   |
| **Linux**   | x64 (Debian/Ubuntu)   | `.deb`                    | [Lumos_0.2.0_amd64.deb](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos_0.2.0_amd64.deb)             |
| **Linux**   | x64 (Fedora/RHEL)     | `.rpm`                    | [Lumos-0.2.0-1.x86_64.rpm](https://github.com/DaiYiqq/lumos/releases/download/v0.2.0/Lumos-0.2.0-1.x86_64.rpm)       |

→ **All releases:** <https://github.com/DaiYiqq/lumos/releases/latest>

> The current installers are **unsigned**. Windows SmartScreen and macOS Gatekeeper may show a
> warning the first time; the steps below explain how to proceed safely.
>
> macOS Intel is not supported yet: the ONNX Runtime prebuilt used by the local embedding model does
> not ship an x86_64 macOS binary.

## Installation

### Windows (x64 / arm64)

1. Download `Lumos_0.2.0_x64-setup.exe` (most PCs) or `Lumos_0.2.0_arm64-setup.exe` (Snapdragon /
   ARM PCs). MSI packages are available if you prefer `msiexec`.
2. Run the installer. If SmartScreen appears, click **More info → Run anyway**.
3. Launch **Lumos** from the Start menu. The app data directory is
   `%APPDATA%\com.daiyi.lumos`.

### macOS (Apple Silicon)

1. Download `Lumos_0.2.0_aarch64.dmg` and open it.
2. Drag **Lumos** into **Applications**.
3. First launch: right-click the app → **Open** → **Open** (or allow it in
   **System Settings → Privacy & Security → Open Anyway**).
4. Data lives in `~/Library/Application Support/com.daiyi.lumos`.

### Linux (x64)

- **AppImage (any distro)**

  ```bash
  chmod +x Lumos_0.2.0_amd64.AppImage
  ./Lumos_0.2.0_amd64.AppImage
  ```

- **Debian / Ubuntu**

  ```bash
  sudo apt install ./Lumos_0.2.0_amd64.deb
  ```

- **Fedora / RHEL**

  ```bash
  sudo dnf install ./Lumos-0.2.0-1.x86_64.rpm
  ```

Linux needs WebKitGTK 4.1 and an app-indicator library; on Debian/Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1 librsvg2-2
```

Data lives in `~/.local/share/com.daiyi.lumos`.

### Build from source (all platforms)

Prerequisites: Rust ≥ 1.90 (MSVC toolchain on Windows), Node.js ≥ 22, pnpm, and the
[Tauri system dependencies](https://tauri.app/start/prerequisites/).

```bash
node scripts/check-env.mjs   # verify toolchain versions
pnpm install
pnpm tauri dev               # desktop app with hot reload
pnpm tauri build             # installers for the current platform
```

Full development, testing and troubleshooting guide: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Features

- **Local-first by design** — documents are parsed and indexed on-device; no account, no upload.
  The only network calls are to the AI provider you explicitly configure.
- **Traceable citations** — answers are grounded in retrieved chunks; `[n]` opens the source panel
  with title, path, page number, section path and the original text.
- **Hybrid retrieval** — local BGE-small-zh embeddings (`sqlite-vec`, 512-dim) + SQLite FTS5 BM25,
  fused with Reciprocal Rank Fusion (k = 60). Works offline with keyword-only mode.
- **Standalone search** — search source passages with hybrid / vector / keyword modes, inspect the
  source, page/section and score, and verify citations without calling an AI model.
- **Copy in one click** — copy an answer as Markdown, or copy a citation with its source path.
- **Three AI modes** — DeepSeek (SSE streaming), Ollama (local models), and an offline extractive
  fallback that never pretends to answer without evidence.
- **Knowledge bases** — separate libraries with independent indexes; import files or whole folders,
  pause/resume/cancel ingestion, and restart safely (jobs are persisted).
- **Conversations** — chat history is saved per library; rename, reopen and delete conversations.
- **Watch & export** — files added/edited/deleted in an indexed folder are re-indexed automatically;
  export everything as Markdown + JSON.
- **Polished desktop UX** — light/dark themes, streaming Markdown with syntax highlighting,
  keyboard-friendly dialogs, error boundary, virtualized document list.
- **Bilingual UI (中文 / English)** — switch the interface language in Settings; the choice persists.

![Lumos demo](docs/assets/demo.gif)

![Lumos chat with citations](docs/assets/p5-chat.png)

![Lumos dark theme](docs/assets/p6-dark.png)

## Using Lumos

1. **Create a library** — e.g. “Contracts”, “Thesis”, “Reading notes”. Libraries keep indexes separate.
2. **Import files or a folder** — PDF, EPUB, DOCX, Markdown, HTML, TXT and common code files.
   Parsing and chunking run on a background queue; you can pause, resume or cancel.
3. **Ask a question** — open _Chat_, pick the library, and type. Answers stream in with `[n]` badges.
4. **Verify** — click any citation to see the exact passage, page and section; click again to dismiss.
5. **Reopen history** — conversations are stored locally; use the left list to revisit them.

### AI configuration

Open **Settings → DeepSeek API Key** and save a key. It is written to the OS credential store
(Windows Credential Manager / macOS Keychain / libsecret), never to the database or logs.
Alternatively:

- **Ollama** — set the endpoint (default `http://127.0.0.1:11434`) and model name; Lumos detects it
  automatically and prefers it when no DeepSeek key is present.
- **Offline** — with neither available, answers fall back to keyword retrieval plus an explicit
  notice; the app remains fully usable for searching and reading citations.

Environment variables are supported for automation: `DEEPSEEK_API_KEY`, `OLLAMA_BASE_URL`,
`OLLAMA_MODEL`, `DEEPSEEK_MODEL`. They are never logged.

## How it works

```text
┌──────────────────────────── React 19 + TypeScript ────────────────────────────┐
│  Libraries  │  Chat (streaming Markdown + citation panel)  │  Settings        │
│  zustand stores · Tauri IPC (invoke / Channel / events)                       │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ commands
┌───────────────▼─────────────────────────────── Rust (Tauri 2) ────────────────┐
│ ingest: extract → heading-aware chunk (512/64) → job queue (persisted)        │
│ embedding: BGE-small-zh-v1.5 via fastembed (downloaded once, cached)          │
│ retrieval: sqlite-vec KNN + FTS5 trigram BM25 → RRF                           │
│ providers: DeepSeek SSE / Ollama NDJSON / offline fallback behind one trait   │
│ security: keyring (OS credential store) with env fallback                     │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ rusqlite
┌───────────────▼───────────────────────────────────────────────────────────────┐
│ SQLite (WAL): libraries · documents · chunks · chunks_vec · chunks_fts        │
│               conversations · messages · ingest_jobs · settings              │
└───────────────────────────────────────────────────────────────────────────────┘
```

Design notes and schema: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) ·
Command reference: [docs/API.md](docs/API.md).

### Retrieval quality

Benchmarked on a 24-document mixed corpus with 16 Chinese queries (recall@3, see
[docs/RETRIEVAL.md](docs/RETRIEVAL.md) for the harness and raw results):

| Mode                  | recall@3  |
| --------------------- | --------- |
| Keyword (FTS5 BM25)   | 0.750     |
| Vector (BGE-small-zh) | 0.688     |
| **Hybrid (RRF)**      | **1.000** |

## Privacy

- Documents, chunks, embeddings and chat history live in one SQLite file under the OS app-data
  directory (`%APPDATA%\com.daiyi.lumos` on Windows). Nothing is uploaded to Lumos servers — there
  are none.
- The API key is stored in the OS credential store; logs never contain keys or document bodies.
- Network access is limited to the configured provider and the one-time embedding-model download
  (Hugging Face, with a mirror fallback).
- **Export & backup** — Settings → _Export all data_ writes Markdown + JSON to a folder you choose;
  copy the data directory for a full backup.
- Delete a library or conversation and its rows are removed from your local database.

## Roadmap

- ✅ P0 — environment check, scaffold, brand icon
- ✅ P1 — SQLite schema, migrations, library CRUD
- ✅ P2 — ingestion: parse → chunk → persist, persistent pause/resume queue
- ✅ P3 — hybrid retrieval: sqlite-vec + BGE embeddings + BM25 → RRF
- ✅ P4 — providers: DeepSeek / Ollama / offline fallback with streaming, keys in OS keychain
- ✅ P5 — chat UI with streaming answers, clickable citations and conversation history
- ✅ P6 — polish: logging, file watcher, data export, error boundary, virtual list, dark theme
- ✅ P7 — bilingual UI (zh/en), documentation, CI and release workflows
- ✅ P8 — first public release: Windows (x64/arm64), macOS Apple Silicon, Linux x64 installers
- ✅ v0.2.0 — standalone search (hybrid/vector/keyword), copy answers and citations

## Documentation

- [Development guide & troubleshooting](docs/DEVELOPMENT.md)
- [Architecture and data model](docs/ARCHITECTURE.md)
- [Tauri command reference](docs/API.md)
- [Hybrid retrieval benchmark](docs/RETRIEVAL.md)
- [Translation guide](docs/I18N.md)

## Community

Lumos is **community-maintained** and contributions of every size are welcome.

- [Contributing guide](CONTRIBUTING.md) — setup, quality gates, PR process, first issues
- [Code of Conduct](CODE_OF_CONDUCT.md) — be kind and respectful
- [Governance](GOVERNANCE.md) · [Maintainers](MAINTAINERS.md) — roles, decisions, how to become a maintainer
- [Security policy](SECURITY.md) — report vulnerabilities privately
- [Translation guide](docs/I18N.md) — add a UI language or README translation
- [Discussions](https://github.com/DaiYiqq/lumos/discussions) · [Issues](https://github.com/DaiYiqq/lumos/issues)
- Good starting points: [`good first issue`](https://github.com/DaiYiqq/lumos/labels/good%20first%20issue) ·
  [`help wanted`](https://github.com/DaiYiqq/lumos/labels/help%20wanted) ·
  [`translation`](https://github.com/DaiYiqq/lumos/labels/translation)

## Contributing

Issues and pull requests are welcome. Before submitting:

```bash
pnpm format && pnpm typecheck && pnpm lint && pnpm test
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

Please keep commits focused, avoid secrets/personal data, and add a test when fixing a bug.
Read [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow, or
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#contributing) for development details.

## Developer

Lumos is created and maintained by **DaiYi (戴亿)**.

| Item      | Contact                                           |
| --------- | ------------------------------------------------- |
| Developer | DaiYi (戴亿)                                      |
| QQ email  | [1703205607@qq.com](mailto:1703205607@qq.com)     |
| QQ        | 1703205607                                        |
| Gmail     | [daiy91004@gmail.com](mailto:daiy91004@gmail.com) |
| Douyin    | daiyi091004                                       |
| WeChat    | daiyi091004                                       |
| GitHub    | [@DaiYiqq](https://github.com/DaiYiqq)            |

See [DEVELOPERS.md](DEVELOPERS.md) for details. These contacts are for project collaboration and
feedback only; please report security issues privately via [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE) © 2026 DaiYi
