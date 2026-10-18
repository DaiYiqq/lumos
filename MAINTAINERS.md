# Maintainers

This file lists people with repository responsibilities. See [GOVERNANCE.md](GOVERNANCE.md) for how
roles work and how to become a maintainer.

## Current maintainers

| Name  | GitHub                                 | Role                      | Areas                                     |
| ----- | -------------------------------------- | ------------------------- | ----------------------------------------- |
| DaiYi | [@DaiYiqq](https://github.com/DaiYiqq) | Project lead / maintainer | All areas; releases and security response |

## Area ownership

| Area                                         | Files / paths                                             | Owner(s)                                   |
| -------------------------------------------- | --------------------------------------------------------- | ------------------------------------------ |
| Rust backend, database, ingestion, retrieval | `src-tauri/src/**`                                        | @DaiYiqq                                   |
| Frontend UI, state, styling                  | `src/**`                                                  | @DaiYiqq                                   |
| i18n (UI + README translations)              | `src/i18n/**`, `README*.md`, `docs/I18N.md`               | @DaiYiqq — translation maintainers welcome |
| CI, release workflows, packaging             | `.github/workflows/**`, `src-tauri/tauri.conf.json`       | @DaiYiqq                                   |
| Documentation                                | `README*.md`, `docs/**`, `CONTRIBUTING.md`, `SECURITY.md` | @DaiYiqq                                   |

## Reviewers and translation maintainers

There are no additional reviewers yet. The first community reviewers/translation maintainers will be
listed here once nominated (see [GOVERNANCE.md](GOVERNANCE.md#becoming-a-maintainer)).

If you would like to maintain a translation (for example Korean, Spanish or German), open an issue
with the `translation` label — you do not need prior maintainer status to start.

## Emeritus

None yet.

## Contact

- Issues: <https://github.com/DaiYiqq/lumos/issues>
- Discussions: <https://github.com/DaiYiqq/lumos/discussions>
- Security: see [SECURITY.md](SECURITY.md)
