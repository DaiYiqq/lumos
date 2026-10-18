# Lumos

**A local-first personal knowledge base that answers with traceable citations.**

Lumos ingests your PDF / EPUB / Word / Markdown / web pages / plain-text files and lets you ask
questions about them. Every answer must cite the original passage — click a citation and jump back
to the exact spot. Indexes, vectors and chat history live in a single SQLite file on your machine.

**本地优先的个人知识库：把你的 PDF、EPUB、Word、Markdown、网页和纯文本全部喂进去，用 AI 提问，每个回答都标注可点击的引用来源。**

> Current status: P0–P2 complete — app shell, SQLite data layer, and the document ingestion
> pipeline (parse → chunk → persist, with a persistent pause/resume queue) are working and covered
> by tests. Retrieval, AI providers and the full chat UI are next. The roadmap below tracks what
> actually works today — this README stays honest until the first release.

## Why Lumos

|                                        | Lumos                      | Upload files to a chat app | Obsidian   | Commercial RAG tools |
| -------------------------------------- | -------------------------- | -------------------------- | ---------- | -------------------- |
| Data stays on your machine             | ✅                         | ❌                         | ✅         | ❌                   |
| Traceable citations back to the source | ✅                         | ⚠️ often hallucinated      | ❌         | ⚠️ depends           |
| Works without an API key               | ✅ keyword search fallback | ❌                         | ✅ (no AI) | ❌                   |
| Zero-config desktop install            | ✅                         | ✅                         | ✅         | ⚠️                   |
| Open source                            | ✅ MIT                     | ❌                         | ⚠️ partial | ❌                   |

## Tech stack

- **Shell**: Tauri 2 (Rust) + React 19 + TypeScript 6 + Vite 8
- **State**: zustand, self-built minimal UI components (no heavy component library)
- **Styling**: Tailwind CSS v4
- **Backend**: Rust, rusqlite (bundled) with sqlite-vec + FTS5, fastembed (BGE-small-zh), tokio
- **AI**: DeepSeek / Ollama / offline keyword fallback behind one provider trait
- **Retrieval**: hybrid vector + BM25 with RRF fusion (see [docs/RETRIEVAL.md](docs/RETRIEVAL.md))
- **Storage**: SQLite in the OS application-data directory

## Development

Prerequisites: Rust ≥ 1.90 (MSVC toolchain on Windows), Node.js ≥ 22, pnpm, and the Tauri
[system dependencies](https://tauri.app/start/prerequisites/).

```bash
node scripts/check-env.mjs   # verify your environment first
pnpm install
pnpm tauri dev               # start the desktop app with hot reload
```

Quality gates:

```bash
pnpm typecheck   # tsc --noEmit
pnpm lint        # eslint
pnpm test        # vitest
pnpm tauri build --bundles nsis   # Windows installer (use dmg/appimage/deb on other platforms)
```

## Roadmap

- ✅ P0 — environment check, scaffold, brand icon
- ✅ P1 — SQLite schema, migrations, library CRUD
- ✅ P2 — ingestion: parse → chunk → persist, persistent pause/resume queue
- ✅ P3 — hybrid retrieval: sqlite-vec + BGE embeddings + BM25 → RRF
- 🚧 P4 — providers: DeepSeek / Ollama / offline fallback with streaming
- 📋 P5 — full UI with clickable citations
- 📋 P6 — polish: themes, i18n, performance, accessibility
- 📋 P7 — docs, README polish, demo GIF, CI/release workflows
- 📋 P8 — first public release

## License

[MIT](LICENSE) © 2026 DaiYi
