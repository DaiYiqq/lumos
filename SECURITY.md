# Security Policy

## Supported versions

Security fixes are provided for the latest release on `main`. Older releases may be asked to upgrade.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a vulnerability

**Please do not report security vulnerabilities in public issues, discussions or pull requests.**

Use one of these private channels:

1. **GitHub Private Vulnerability Reporting** — go to the repository's **Security** tab →
   **Report a vulnerability** (preferred).
2. If that is unavailable, email **1703205607@qq.com** with the subject `[Lumos security]`.

Please include:

- affected version and platform (Windows/macOS/Linux, architecture),
- a description of the issue and its impact,
- reproduction steps or a proof of concept (no real personal documents or API keys),
- any suggested mitigation.

## What to expect

- Acknowledgement within **7 days**.
- An initial assessment within **14 days**, including severity and a fix plan.
- We will coordinate disclosure and credit you in the release notes unless you prefer to stay
  anonymous.
- Please give us reasonable time to ship a fix before public disclosure (90 days is a common upper
  bound; earlier by mutual agreement).

## Scope

Lumos is a local desktop application. Areas of interest include:

- API key handling (OS credential store, environment fallback, logs) and accidental exposure.
- Local file parsing (PDF/EPUB/DOCX/HTML) — memory safety, path traversal, zip bombs.
- SQLite / FTS5 / sqlite-vec handling and migration safety.
- Tauri IPC command validation and the Content Security Policy.
- Chat prompt/response handling, citation integrity, and export paths.
- Dependency vulnerabilities (we monitor Dependabot alerts).

Out of scope:

- Attacks requiring an already-compromised local account or physical access.
- Self-XSS or issues only reproducible with developer tools.
- Vulnerabilities in AI providers (report them to the provider).
- Denial of service from importing enormous files on your own machine (documented behavior).

## Safe development practices

- Never commit API keys, personal documents or production databases. Use environment variables
  (`DEEPSEEK_API_KEY`) for local tests.
- Logs must not contain document contents or secrets.
- Third-party dependencies are updated via Dependabot and reviewed before merge.

Thank you for helping keep Lumos and its users safe.
