## Summary

<!-- What does this PR change, and why? -->

## Related issue

<!-- e.g. Closes #123, or "No issue — small docs fix" -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Translation
- [ ] Refactor / performance
- [ ] CI / packaging
- [ ] Breaking change (describe migration below)

## How was it tested?

<!-- Commands, manual scenarios, screenshots/GIFs for UI changes. -->

```bash
pnpm format && pnpm typecheck && pnpm lint && pnpm test
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

## Checklist

- [ ] Tests added or updated (bug fixes include a regression test when possible)
- [ ] Documentation updated (`README`, `docs/API.md`, `docs/ARCHITECTURE.md`, `docs/I18N.md`, …)
- [ ] UI strings go through `src/i18n` (no hard-coded new user-facing text)
- [ ] README translations kept in sync (if user-facing docs changed)
- [ ] No API keys, personal documents, generated `dist/`, `target/` or `node_modules/`
- [ ] Screenshots/GIFs attached for visual changes
- [ ] Breaking changes documented below

## Breaking changes / migration notes

<!-- Database migrations, command signature changes, removed settings — or "None". -->
