# Changelog

All notable changes to Lumos are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-10-23

### Added

- **Standalone search page** — search passages inside a library with three modes:
  - hybrid (vector + BM25 fused with RRF),
  - vector (local BGE-small-zh embeddings via `sqlite-vec`),
  - keyword (SQLite FTS5 trigram BM25).
    Results show document title, path, page/section, fused score and the original passage, so a
    citation can be verified without asking the AI.
- **Copy actions** — copy an assistant answer as Markdown, or copy a citation (title, location,
  passage, source path) from the citation panel and search results.
- **Navigation** — a new “Search / 搜索” entry in the sidebar; the online demo includes the same page.

### Changed

- Bumped app version to `0.2.0` (`package.json`, `Cargo.toml`, `tauri.conf.json`).
- README download tables now point to `v0.2.0` installers.
- The online demo mock backend implements the search API so the new page works at
  <https://daiyiqq.github.io/lumos/>.

### Fixed

- Search UI reports when the vector model is unavailable and the request fell back to keyword mode.

## [0.1.0] - 2026-10-18

### Added

- Local-first ingestion for PDF / EPUB / Word / Markdown / HTML / plain text and code.
- Heading-aware chunking with page/section metadata and a persistent pause/resume queue.
- Hybrid retrieval: local BGE-small-zh embeddings (`sqlite-vec`) + FTS5 BM25 + RRF.
- DeepSeek (SSE), Ollama (NDJSON) and offline providers; API keys in the OS credential store.
- Streaming chat UI with clickable `[n]` citations, citation panel and conversation history.
- File watcher, Markdown/JSON export, dark theme, zh/en UI, error boundary and virtualized lists.
- Linux x64, Windows x64/arm64 and macOS Apple Silicon installers; GitHub Actions CI and releases.
