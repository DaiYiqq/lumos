# Command Reference

All commands are Tauri IPC commands called from the frontend via
`window.__TAURI_INTERNALS__.invoke(name, args)`. Argument names are **camelCase** (Tauri maps them
to the Rust snake_case parameters). Errors are rejected with a human-readable string.

The typed TypeScript wrappers live in [`src/lib/tauri.ts`](../src/lib/tauri.ts).

## App

| Command         | Arguments | Returns                                                  |
| --------------- | --------- | -------------------------------------------------------- |
| `app_info`      | —         | `{ name, version, dataDir, dataDirDisplay, platform }`   |
| `open_data_dir` | —         | `void` — opens the data directory in the OS file manager |

## Libraries

| Command          | Arguments      | Returns                                          |
| ---------------- | -------------- | ------------------------------------------------ |
| `library_list`   | —              | `LibrarySummary[]` (document/chunk counts, size) |
| `library_create` | `{ name }`     | `Library`                                        |
| `library_rename` | `{ id, name }` | `Library`                                        |
| `library_delete` | `{ id }`       | `void`                                           |

## Ingestion

| Command               | Arguments                        | Returns                    |
| --------------------- | -------------------------------- | -------------------------- |
| `ingest_import_paths` | `{ libraryId, paths: string[] }` | `number` — new jobs queued |
| `ingest_job_list`     | `{ libraryId }`                  | `IngestJob[]`              |
| `ingest_cancel_job`   | `{ jobId }`                      | `void`                     |
| `ingest_pause`        | —                                | `boolean` — queue paused   |
| `ingest_resume`       | —                                | `boolean` — queue running  |
| `ingest_queue_state`  | —                                | `boolean` — paused?        |
| `document_list`       | `{ libraryId }`                  | `DocumentSummary[]`        |
| `document_delete`     | `{ id }`                         | `void`                     |

Supported extensions: `pdf, epub, docx, md, markdown, html, htm, txt, rst, rs, ts, tsx, js, jsx, py,
go, java, c, h, cpp, hpp, json, yaml, yml, toml, csv, log`.

## Retrieval

| Command              | Arguments                                                                        | Returns                                                                  |
| -------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `search_chunks`      | `{ libraryId, query, mode?: "hybrid" \| "vector" \| "keyword", limit?: number }` | `SearchResponse` — `{ hits: SearchHit[], mode, notices }`                |
| `embedding_status`   | —                                                                                | `{ state: "loading" \| "ready" \| "failed", model, dimension, message }` |
| `embedding_backfill` | —                                                                                | `number` — chunks vectorized in this batch                               |

`SearchHit` contains `chunkId, documentId, documentTitle, documentPath, pageNo, sectionPath,
content, score`.

## Providers & settings

| Command            | Arguments        | Returns                                                                 |
| ------------------ | ---------------- | ----------------------------------------------------------------------- |
| `api_key_status`   | —                | `{ configured: boolean, source: "keyring" \| "environment" \| "none" }` |
| `api_key_set`      | `{ key }`        | `ApiKeyStatus` — stores in OS credential store                          |
| `api_key_clear`    | —                | `ApiKeyStatus`                                                          |
| `providers_detect` | —                | `{ statuses: ProviderStatus[], selected }`                              |
| `settings_get`     | `{ key }`        | `string \| null`                                                        |
| `settings_set`     | `{ key, value }` | `void`                                                                  |

Whitelisted settings keys: `ollama_base_url`, `ollama_model`, `deepseek_model`, `ui_language`.

## Chat

### `chat_stream`

Arguments:

```ts
{
  libraryId: string;
  question: string;
  conversationId?: string | null;   // omit to create a new conversation
  history?: Array<{ role: "user" | "assistant" | "system"; content: string }>;
  channel: Channel<ChatEvent>;
}
```

`ChatEvent` (discriminated by `type`):

| type           | payload                     | Meaning                                  |
| -------------- | --------------------------- | ---------------------------------------- |
| `conversation` | `{ id, title }`             | conversation created or reused           |
| `provider`     | `{ provider, label }`       | selected provider                        |
| `notice`       | `{ message }`               | degradation/offline notice               |
| `citations`    | `{ citations: Citation[] }` | retrieved sources, numbered              |
| `delta`        | `{ text }`                  | streamed token(s)                        |
| `done`         | `{ provider }`              | stream finished                          |
| `error`        | `{ message }`               | failure (may occur after partial output) |

`Citation`: `{ index, chunkId, documentId, documentTitle, documentPath, pageNo, sectionPath,
content, score }`.

The command persists the user message immediately and the assistant message (with citations JSON)
after a successful stream.

### `chat_complete`

`{ libraryId, question, conversationId? }` → `{ provider, answer, citations, conversationId }`.
Non-streaming variant used by diagnostics and automation.

## Conversations

| Command               | Arguments               | Returns                                       |
| --------------------- | ----------------------- | --------------------------------------------- |
| `conversation_list`   | `{ libraryId }`         | `ConversationSummary[]` (with `messageCount`) |
| `conversation_create` | `{ libraryId, title? }` | `Conversation`                                |
| `conversation_get`    | `{ id }`                | `{ conversation, messages: StoredMessage[] }` |
| `conversation_rename` | `{ id, title }`         | `Conversation`                                |
| `conversation_delete` | `{ id }`                | `void`                                        |

## Export

| Command      | Arguments       | Returns                                         |
| ------------ | --------------- | ----------------------------------------------- |
| `export_all` | `{ targetDir }` | `{ root, libraries, documents, conversations }` |

Creates `<targetDir>/lumos-export-<unix-timestamp>/` with `libraries.json`, per-library document
Markdown and per-library conversation Markdown.

## Events

| Event               | Payload                                                   | Emitted when                                   |
| ------------------- | --------------------------------------------------------- | ---------------------------------------------- |
| `ingest://progress` | `{ jobId, libraryId, path, status, chunkCount, message }` | a job starts/finishes/fails                    |
| `library://changed` | `{ libraryId }`                                           | file watcher added/modified/removed a document |

Subscribe with `listen(eventName, handler)` from `@tauri-apps/api/event`; the returned promise resolves
to an unlisten function.
