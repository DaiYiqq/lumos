# Lumos

**A local-first personal knowledge base that answers with traceable citations.**
**本地优先的个人知识库：把文档喂进去，用 AI 提问，每个回答都标注可点击的引用。**

[![CI](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml/badge.svg)](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DaiYiqq/lumos)](https://github.com/DaiYiqq/lumos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Lumos ingests PDF / EPUB / Word / Markdown / HTML / plain text and code, indexes everything locally,
and answers questions **only from your documents** — every claim carries a `[n]` citation that jumps
back to the exact passage, page and section. Your files, embeddings and chat history stay in one
SQLite database on your machine.

> 中文简介：Lumos 是一个完全本地的个人知识库。导入 PDF、EPUB、Word、Markdown、网页或代码文件，
> 用 DeepSeek / Ollama / 离线关键词模式提问，回答必须带引用编号，点一下就能回到原文。
> 文档、向量与对话都保存在本机 SQLite 中，卸载前可一键备份或导出。

![Lumos demo](docs/assets/demo.gif)

![Lumos chat with citations](docs/assets/p5-chat.png)

## Features

- **Local-first by design** — documents are parsed and indexed on-device; no account, no upload.
  The only network calls are to the AI provider you explicitly configure.
- **Traceable citations** — answers are grounded in retrieved chunks; `[n]` opens the source panel
  with title, path, page number, section path and the original text.
- **Hybrid retrieval** — local BGE-small-zh embeddings (`sqlite-vec`, 512-dim) + SQLite FTS5 BM25,
  fused with Reciprocal Rank Fusion (k = 60). Works offline with keyword-only mode.
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

![Lumos dark theme](docs/assets/p6-dark.png)

## Quick start

### Prebuilt binaries

Download the latest installer from [Releases](https://github.com/DaiYiqq/lumos/releases):

- Windows: `.exe` (NSIS) — x64 and arm64
- macOS: `.dmg` — Apple Silicon
- Linux: `.AppImage` / `.deb`

### Build from source

Prerequisites: Rust ≥ 1.90 (MSVC toolchain on Windows), Node.js ≥ 22, pnpm, and the
[Tauri system dependencies](https://tauri.app/start/prerequisites/).

```bash
node scripts/check-env.mjs   # verify toolchain versions
pnpm install
pnpm tauri dev               # desktop app with hot reload
pnpm tauri build             # installers for the current platform
```

Full development, testing and troubleshooting guide: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

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

## Documentation

- [Development guide & troubleshooting](docs/DEVELOPMENT.md)
- [Architecture and data model](docs/ARCHITECTURE.md)
- [Tauri command reference](docs/API.md)
- [Hybrid retrieval benchmark](docs/RETRIEVAL.md)

## Contributing

Issues and pull requests are welcome. Before submitting:

```bash
pnpm format && pnpm typecheck && pnpm lint && pnpm test
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

Please keep commits focused, avoid secrets/personal data, and add a test when fixing a bug.
See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#contributing) for details.

## License

[MIT](LICENSE) © 2026 DaiYi
