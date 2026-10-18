# Architecture

## Process model

Lumos is a Tauri 2 application with two processes:

- **WebView (React 19 + TypeScript)** — UI, state (zustand), Markdown rendering, streaming display.
- **Rust core** — SQLite, ingestion, embeddings, retrieval, providers, OS integration.

They communicate through Tauri IPC:

| Mechanism                  | Used for                                                           |
| -------------------------- | ------------------------------------------------------------------ |
| `invoke(command, args)`    | request/response commands (list libraries, search, save settings…) |
| `Channel<T>`               | chat token streaming (`chat_stream`)                               |
| `app.emit(event, payload)` | background progress (`ingest://progress`, `library://changed`)     |

No HTTP server is exposed; everything runs in-process.

## Modules

```text
commands/     thin IPC layer: validate input, call repository/service, map errors to strings
db/           Database wrapper (Mutex<Connection>, WAL, foreign keys, sqlite-vec registration)
              migrations/ (0001 core … 0004 settings), repo/ (libraries, documents, chunks,
              conversations, jobs, settings), models.rs (serde camelCase structs)
ingest/       extract.rs  PDF/EPUB/DOCX/HTML/Markdown/TXT/code → text
              chunk.rs    heading-aware chunker (~512 tokens, 64 overlap, page/section metadata)
              pipeline.rs parse → hash → replace document+chunks → embed → job status
              queue.rs    persistent worker queue with pause/resume/cancel and restart recovery
              watcher.rs  notify-based folder watching for indexed parents
embedding/    EmbeddingService: lazy fastembed model, background warmup, status
              model_store.rs: custom downloader HF_ENDPOINT → official → hf-mirror
retrieval/    vector KNN (sqlite-vec) + FTS5 trigram BM25 → RRF (k=60); backfill
providers/    LlmProvider trait; deepseek.rs (SSE), ollama.rs (NDJSON), none.rs (extractive),
              prompt.rs (numbered citations), detection + settings keys
security.rs   keyring (OS credential store) with DEEPSEEK_API_KEY env fallback
logging.rs    tracing subscriber + daily rolling file logs
state.rs      AppState { db, queue, embedder, watcher, log guard }
```

## Data model (SQLite, migrations 0001–0004)

| Table                        | Purpose                                                                          |
| ---------------------------- | -------------------------------------------------------------------------------- |
| `libraries`                  | id, name, embedding model, chunk config, timestamps                              |
| `documents`                  | library_id, path, title, mime, size, page_count, content_hash, status, error_msg |
| `chunks`                     | document_id, ordinal, content, page_no, section_path, char offsets, token_count  |
| `chunks_vec`                 | `vec0` virtual table, float[512] embeddings keyed by chunk rowid                 |
| `chunks_fts`                 | FTS5 trigram index over chunk content, synced by triggers                        |
| `ingest_jobs`                | persistent queue: library_id, path, status, chunk_count, message                 |
| `conversations` / `messages` | chat history; messages store citations JSON                                      |
| `settings`                   | whitelisted key/value settings (provider URLs, models)                           |
| `schema_migrations`          | applied migration versions                                                       |

Deleting a document cascades to chunks; triggers remove the corresponding `chunks_vec` and FTS rows.
Foreign keys are enabled on every connection.

## Ingestion pipeline

1. **Enqueue** — files or folder walks are expanded into `ingest_jobs` (supported extensions only);
   duplicates are skipped by path + unfinished status.
2. **Extract** — format-specific parsers produce text with page numbers (PDF) or section paths
   (Markdown/HTML/EPUB).
3. **Hash & replace** — SHA-256 of the extracted text; unchanged files are skipped, changed files
   replace the old document atomically in one transaction.
4. **Chunk** — heading-aware splitter, ~512 tokens with 64 overlap; each chunk keeps page/section and
   character offsets so citations can point back.
5. **Embed** — if the embedding model is ready, chunks are vectorized; failure is non-fatal (keyword
   search still works, background backfill catches up).
6. **Status** — job becomes `done`/`failed`; progress events update the UI.

The queue is persistent: `running` jobs are reset to `pending` on startup and resumed. Pause stops
dequeuing, cancel marks the job cancelled, and both survive a restart.

File watcher: indexed parent directories are watched (recursively, skipping drive roots); after a
700 ms debounce, new/modified files are enqueued and deleted files are removed from the index. The
directory set is refreshed every 60 s.

## Retrieval

Three modes, all local:

- **Vector** — query embedded with the BGE query instruction prefix; `sqlite-vec` KNN over
  `chunks_vec`, scoped to one library.
- **Keyword** — FTS5 trigram BM25. Chinese queries are split into overlapping 3-character windows and
  OR-ed, because trigram FTS cannot match whole Chinese phrases.
- **Hybrid** — reciprocal rank fusion: `score = Σ 1/(60 + rank)` over both lists; dedup by chunk id.

Results become numbered citations. The prompt instructs the model to cite `[n]` only from the
provided passages and to state explicitly when evidence is missing; the offline provider uses the
same rule with extractive text.

## Providers

`LlmProvider` exposes `detect`, `stream_chat` and a label. Candidate order:

1. **DeepSeek** — if an API key exists (keyring or env). SSE parser handles chunked deltas.
2. **Ollama** — if `GET /api/tags` succeeds. NDJSON parser.
3. **Offline** — always available; returns the top passages with an explicit notice, never fabricates.

Fallback happens only before the first token; a mid-stream failure surfaces an error instead of
mixing two models' output. Errors are classified (missing key, network, HTTP status, parse) for
actionable UI messages.

## Security & privacy

- API keys: keyring only (env fallback for automation); never written to SQLite, logs or frontend
  storage. `api_key_status` reports whether/where a key exists, never the value.
- Content Security Policy restricts scripts and connections; only provider endpoints and the model
  host are reachable.
- Logs contain operational metadata, not document contents or keys.
- All file access is read-only for sources; imported files are never moved or modified.

## Trade-offs

- **Single writer, one SQLite connection** wrapped in a `Mutex` — simple and deterministic; ingestion,
  search and chat are low-concurrency workloads. Heavy embedding work runs outside the lock.
- **fastembed + custom downloader** instead of a hosted embedding API — privacy and offline use;
  the model is downloaded once (~95 MB) and cached.
- **Self-built UI components** — small bundle, full control over the citation affordances; no
  component library dependency.
- **No reranker in v1** — RRF already reached 1.000 recall@3 on the evaluation set; a cross-encoder
  would add latency and another model download. It remains a candidate for later versions.
