# Development Guide

This document covers local development, testing, debugging and the problems we actually hit while
building Lumos (see [Troubleshooting](#troubleshooting)).

## Prerequisites

| Tool             | Version used                  | Notes                               |
| ---------------- | ----------------------------- | ----------------------------------- |
| Rust             | 1.99 (stable, MSVC)           | `x86_64-pc-windows-msvc` on Windows |
| Node.js          | 24.x                          | 22+ works                           |
| pnpm             | 12.x                          | `corepack enable` if missing        |
| MSVC Build Tools | 14.5 + Windows SDK 10.0.26100 | Windows only                        |
| WebView2         | Evergreen                     | Preinstalled on Windows 11          |
| Tauri CLI        | 2.12                          | installed via `pnpm install`        |

Run the environment check before anything else:

```bash
node scripts/check-env.mjs
```

## Setup

```bash
pnpm install
pnpm tauri dev
```

`pnpm tauri dev` starts Vite on <http://localhost:1420> and opens the desktop window. The Rust side
compiles on first run (a few minutes).

### Frontend only

```bash
pnpm dev        # browser preview; Tauri commands are unavailable and pages show an empty state
```

## Quality gates

Run the same commands CI runs:

```bash
# Frontend
pnpm format        # prettier write (use pnpm format:check in CI)
pnpm typecheck     # tsc --noEmit
pnpm lint          # eslint, zero warnings allowed
pnpm test          # vitest (jsdom)

# Backend
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Rust tests marked `#[ignore]` require real network/providers:

```bash
# Real DeepSeek streaming test (key from env, never committed)
set DEEPSEEK_API_KEY=sk-...
cargo test deepseek -- --ignored --nocapture

# Real Ollama test (needs a local model)
set OLLAMA_E2E=1
cargo test ollama -- --ignored --nocapture
```

## Project layout

```text
src/                     React 19 + TypeScript
  components/            shell, library, chat, settings, shared UI
  stores/                zustand stores (libraries, documents, chat, ui, theme, appInfo)
  lib/tauri.ts           typed wrappers around invoke()/Channel
  i18n/                  zh/en dictionaries and language store
  types/                 shared IPC types
src-tauri/src/
  commands/              #[tauri::command] entry points
  db/                    connection, migrations, repository modules, models
  ingest/                extractors, chunker, pipeline, persistent queue, watcher
  embedding/             fastembed service + model downloader
  retrieval/             vector/FTS/hybrid search and backfill
  providers/             DeepSeek, Ollama, offline provider + prompt
  security.rs            keyring-backed API key
  logging.rs             tracing + daily rolling file logs
  state.rs / lib.rs      AppState wiring and startup
docs/                    architecture, command reference, benchmark
scripts/                 check-env, icon generation
```

## Debugging

- **Logs** — `<app-data>/logs/lumos.log` (daily rolling). Increase verbosity with
  `$env:RUST_LOG="lumos_lib=debug"`.
- **App data** — Windows `%APPDATA%\com.daiyi.lumos`, macOS
  `~/Library/Application Support/com.daiyi.lumos`, Linux `~/.local/share/com.daiyi.lumos`.
- **Database** — `lumos.db` in the data directory (WAL mode). Inspect with any SQLite client while
  the app is closed.
- **WebView devtools** — right-click inside the window → _Inspect_ (debug builds only). For automated
  UI checks, temporarily add to `src-tauri/tauri.conf.json`:
  `"additionalBrowserArgs": "--remote-debugging-port=9222 --remote-allow-origins=*"` and drive it via
  the Chrome DevTools Protocol. **Remove it before committing.**
- **Embedding model** — cached under `<app-data>/models/bge-small-zh-v1.5`; delete the folder to force
  a clean re-download.

## Troubleshooting

### `cargo build` fails with linker errors on Windows

Install **Visual Studio Build Tools** with the _Desktop development with C++_ workload and the Windows
SDK, then run Rust in a **Developer PowerShell**. Verify:

```powershell
rustc -vV        # host must be x86_64-pc-windows-msvc
where.exe link   # must resolve to MSVC's link.exe
```

### `pnpm tauri dev` opens a blank window

- Check `http://localhost:1420` in a browser; if it fails, Vite is not running (port conflict?).
- A stale process may own port 1420: `Get-NetTCPConnection -LocalPort 1420` then stop the owner.
- Make sure WebView2 Runtime is installed (`winget install Microsoft.EdgeWebView2Runtime`).

### The embedding model download hangs or fails

Lumos tries `HF_ENDPOINT` → Hugging Face → `hf-mirror.com`. On restricted networks set a mirror:

```powershell
$env:HF_ENDPOINT="https://hf-mirror.com"
```

Keyword search keeps working even if embeddings are unavailable.

### Answers are slow on first question

The first request loads the embedding model (~95MB) and may download it. Subsequent questions are
fast. Watch `embedding_status` in Settings.

### `sqlite-vec` fails to load

Lumos registers the bundled extension automatically at startup. If vector search degrades to keyword
mode, check the log for `sqlite-vec`; on Linux make sure the bundled SQLite is used (default feature
`bundled`).

### `clippy` complains about `unwrap`/`expect`

That is intentional: production code must not panic. Use `?`, `ok_or`, or `unwrap_or_else` with
logging. Only program entry points and tests may `expect`.

### Tests fail with Chinese mojibake in output

The Windows console defaults to GBK. Set `chcp 65001` or run tests in Windows Terminal; source files
are UTF-8 and unaffected.

### Port 9222 is still open after E2E

The CDP argument lives only in a temporary edit of `tauri.conf.json`. Revert it and restart the app.
`git diff src-tauri/tauri.conf.json` must be empty before committing.

## Release process

```bash
# 1. bump versions (package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json)
# 2. run all quality gates
# 3. tag and push; GitHub Actions builds installers and creates a draft release
git tag v0.1.0
git push origin main --tags
# 4. review the draft release, then publish
```

The release workflow builds Windows (x64/arm64), macOS (Apple Silicon/Intel) and Linux bundles with
`tauri-apps/tauri-action`.

## Contributing

1. Open an issue for anything non-trivial.
2. Fork, branch from `main`, keep commits focused.
3. Add or update tests; run the quality gates above.
4. Do not commit API keys, personal documents or generated `target/`/`node_modules/` files.
5. Describe the change and verification in the pull request.
