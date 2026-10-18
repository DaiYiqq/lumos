# Governance

Lumos is a community-maintained open-source project. This document explains who can do what, how
decisions are made, and how to become a maintainer. It is intentionally lightweight — the goal is to
make collaboration predictable, not bureaucratic.

## Principles

1. **Local-first and private by default** — features that upload user content or add tracking need
   explicit discussion and maintainer consensus.
2. **Citations must remain trustworthy** — answers must stay grounded in retrieved sources and say so
   when evidence is missing.
3. **Quality gates are non-negotiable** — CI (frontend checks, Rust fmt/clippy/test on three OSes)
   must pass before merge.
4. **Respect the community** — the [Code of Conduct](CODE_OF_CONDUCT.md) applies everywhere.

## Roles

| Role                       | What they do                                                            | How to get it                                                                 |
| -------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| **Contributor**            | Reports issues, submits PRs, improves docs/translations                 | Open a PR                                                                     |
| **Reviewer**               | Triages issues, reviews PRs, guides contributors                        | Nominated by a maintainer after sustained, high-quality reviews/contributions |
| **Maintainer**             | Merges PRs, manages releases, security, roadmap and repository settings | Nominated by an existing maintainer, approved by consensus                    |
| **Translation maintainer** | Owns one language (README + UI strings), reviews translation PRs        | Same as Reviewer, scoped to a language                                        |
| **Project lead**           | Breaks ties, owns security response and release signing                 | Current lead maintainer                                                       |

## Decision making

- **Lazy consensus** is the default: a proposal (issue or PR) is accepted if no maintainer objects
  within **7 days**.
- **Contested decisions** are discussed in the issue/PR. Maintainers then vote; a simple majority
  wins and the project lead breaks ties.
- **Security reports** are handled privately by maintainers; details may be withheld until a fix is
  released.
- **Breaking changes** (data model, API commands, UX defaults) need a design comment in an issue and
  maintainer consensus before implementation.
- **Translations and docs** can be merged by any maintainer with a Reviewer's approval; they do not
  need a full design discussion.

## Becoming a maintainer

We actively want more maintainers. A contributor is typically invited after:

- 3+ months of contributions (code, docs, translations, reviews or triage),
- a track record of good judgment and respectful collaboration,
- familiarity with the quality gates and release process,
- willingness to help with reviews and issue triage.

The process: an existing maintainer nominates the person in a private maintainer channel; after
consensus (no objections within 7 days), the person is added to [MAINTAINERS.md](MAINTAINERS.md) and
`.github/CODEOWNERS`.

## Stepping down / emeritus

Maintainers may step down at any time. Inactive maintainers (no activity for 6 months) may be moved
to **emeritus** status after a friendly check-in; emeritus maintainers keep credit and can return
with a short note. Access to releases/signing keys is revoked when someone leaves the maintainer
role.

## Releases

- Maintainers tag `vX.Y.Z` on `main` after CI is green.
- The tag triggers the release workflow (Windows x64/arm64, macOS Apple Silicon, Linux x64).
- Release notes must list downloads for every platform and note unsigned installers.
- A release requires at least one maintainer review of the changelog; for security releases, the
  project lead coordinates disclosure.

## Repository settings

- `main` is protected by CI checks; direct pushes are reserved for maintainers for release/docs
  emergencies.
- Dependabot PRs are reviewed like any other PR; major toolchain upgrades need a green CI run.
- Labels: `bug`, `enhancement`, `documentation`, `translation`, `good first issue`, `help wanted`,
  `security`, `dependencies`.

## Changing this document

Open a PR with the proposed change. It takes effect after a 7-day comment period and maintainer
consensus (lazy consensus applies). Minor typo fixes can be merged directly.

## Contact

- General questions: GitHub Discussions
- Bugs/features: GitHub Issues
- Security: [SECURITY.md](SECURITY.md)
- Conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
