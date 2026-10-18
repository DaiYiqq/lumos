# Contributing to Lumos

Thanks for helping make Lumos better! This guide covers the workflow, quality gates and review
expectations. Everything here is also a good place to start if you are new to open source.

[简体中文摘要](#简体中文) · [Code of Conduct](CODE_OF_CONDUCT.md) · [Security](SECURITY.md) ·
[Governance](GOVERNANCE.md) · [Translation guide](docs/I18N.md)

## Ways to contribute

You do not need to write Rust to be useful:

- **Bug reports** — reproduction steps, OS/version, logs (`<data-dir>/logs/lumos.log`).
- **Translations** — UI strings (`src/i18n/index.ts`) and READMEs (4 languages today).
- **Documentation** — tutorials, screenshots, troubleshooting entries, benchmark results.
- **Code** — features and fixes from the roadmap or issues labelled `good first issue`.
- **Triage / review** — reproduce reports, review PRs, keep issues tidy.
- **Design / UX** — citation panel, onboarding, accessibility, empty states.

Look for labels: [`good first issue`](https://github.com/DaiYiqq/lumos/labels/good%20first%20issue),
[`help wanted`](https://github.com/DaiYiqq/lumos/labels/help%20wanted),
[`translation`](https://github.com/DaiYiqq/lumos/labels/translation).

## Development setup

Prerequisites: Rust ≥ 1.90 (MSVC on Windows), Node.js ≥ 22, pnpm, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/).

```bash
node scripts/check-env.mjs
pnpm install
pnpm tauri dev
```

Full setup, debugging and troubleshooting: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).
Architecture and data model: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Making a change

1. **Open or find an issue.** For anything non-trivial, discuss first so we agree on the approach.
2. **Fork and branch** from `main`:

   ```bash
   git switch -c feat/short-description
   ```

   Prefixes we use: `feat/`, `fix/`, `docs/`, `i18n/`, `refactor/`, `test/`, `chore/`.

3. **Keep commits focused.** One logical change per PR. Conventional-commit style messages are
   welcome (`feat(chat): …`, `fix(watcher): …`), not mandatory.
4. **Add tests.**
   - Rust logic → unit test in the same module.
   - UI behavior → `src/App.test.tsx` or a new `*.test.tsx` with Testing Library.
   - Bug fixes should include a regression test whenever practical.
5. **Run the quality gates** (same commands as CI):

   ```bash
   pnpm format && pnpm typecheck && pnpm lint && pnpm test
   (cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test)
   ```

6. **Update documentation** when behavior, commands, settings or limits change. API changes require
   `docs/API.md`; schema changes require `docs/ARCHITECTURE.md` and a migration.
7. **Open a pull request** using the template. Fill in _what_ and _why_, include screenshots/GIFs for
   UI changes, and mention the issue (`Closes #123`).

### UI and translation changes

- UI strings live in `src/i18n/index.ts` (Chinese source text → English translation).
  Add the key to the dictionary and use `useT()` in components; do not hard-code new user-facing
  Chinese or English text in JSX.
- README translations are `README.zh-CN.md`, `README.zh-TW.md`, `README.ja.md`. Keep download tables,
  links and version numbers in sync.
- See [docs/I18N.md](docs/I18N.md) for adding a new language end to end.

### Code style

- **Rust**: `cargo fmt`; `clippy -D warnings`; no `unwrap()`/`expect()` in production code (only
  program entry points and tests); errors use `AppError` / user-friendly messages.
- **TypeScript/React**: strict types, function components, zustand stores for shared state,
  self-built UI primitives; avoid `any` and `@ts-ignore`.
- **SQL**: migrations are append-only (`0001…`); each new migration gets a version and tests.
- **Privacy**: never log API keys or document contents; never commit real personal documents.
- **Dependencies**: prefer existing crates/packages; justify new heavy dependencies in the PR.

## Review and merge

- CI must be green (frontend checks + Rust on Windows/macOS/Ubuntu).
- One maintainer approval is required (see [GOVERNANCE.md](GOVERNANCE.md)).
- Maintainers may ask for changes; stale PRs (no response for 30 days) may be closed, but can be
  reopened at any time.
- Squash merge is the default; the PR title becomes the commit subject.
- Security-sensitive changes get an additional review from a maintainer listed in
  [MAINTAINERS.md](MAINTAINERS.md).

## Releases

Maintainers tag `vX.Y.Z` on `main` after the quality gates pass. The tag triggers
`.github/workflows/release.yml`, which builds Windows (x64/arm64), macOS Apple Silicon and Linux x64
installers and creates a draft GitHub Release. Release notes must list downloads for every platform.

Versioning follows semantic versioning for the app: `MAJOR` for breaking UX/data changes, `MINOR` for
features, `PATCH` for fixes. Breaking database migrations require a backup note in the release.

## Reporting bugs and security issues

- Functional bugs: use the **Bug report** issue template. Include OS, app version (status bar),
  steps, expected/actual behavior, and relevant log lines.
- Security vulnerabilities: **do not open a public issue** — follow [SECURITY.md](SECURITY.md).
- Feature ideas: use the **Feature request** template; describe the problem before the solution.

## Community standards

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Be respectful,
assume good intent, and help newcomers. Harassment or discrimination is not tolerated.

## 简体中文

感谢参与 Lumos！你不需要会写 Rust 也能帮上忙：

- **反馈 Bug**：附带系统、版本（状态栏）、复现步骤与日志（`<数据目录>/logs/lumos.log`）。
- **翻译**：界面文案在 `src/i18n/index.ts`；README 已有 4 种语言；新增语言见
  [docs/I18N.md](docs/I18N.md)。
- **文档**：教程、截图、Troubleshooting、评测数据。
- **代码**：从 `good first issue` / `help wanted` 标签开始；较大改动请先开 Issue 讨论。
- **评审与整理**：复现问题、Review PR、维护标签。

开发流程：Fork → 从 `main` 建分支 → 小步提交 → 补测试 → 跑本地质量门
（`pnpm format/typecheck/lint/test` 与 `cargo fmt/clippy/test`）→ 按模板开 PR。
合并要求：CI 全绿 + 至少一位维护者 Approve；默认 squash merge。
提交前请更新文档；安全漏洞请按 [SECURITY.md](SECURITY.md) 私下报告，不要公开 Issue。

行为准则见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。请保持友善、尊重与耐心。
