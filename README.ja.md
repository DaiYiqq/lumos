# Lumos

**ローカルファーストの個人ナレッジベース。文書を取り込み、AI に質問すると、すべての回答に
クリック可能な引用が付きます。**

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![CI](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml/badge.svg)](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DaiYiqq/lumos)](https://github.com/DaiYiqq/lumos/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/DaiYiqq/lumos/total)](https://github.com/DaiYiqq/lumos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Demo](https://img.shields.io/badge/demo-online-brightgreen)](https://daiyiqq.github.io/lumos/)

Lumos は PDF / EPUB / Word / Markdown / HTML / プレーンテキスト / コードを取り込み、解析・
チャンク分割・ベクトル化・検索を**すべてローカル**で行います。回答は**あなたの文書だけ**に
基づいて生成され、各主張には `[n]` の引用が付きます。引用をクリックすると、元の段落・
ページ・セクションに戻れます。文書・ベクトル・会話履歴は 1 つの SQLite データベースに
保存されます。

> ▶ **オンライン Demo:** <https://daiyiqq.github.io/lumos/>
>
> Demo はブラウザ内だけで動作し、サンプル文書とモックバックエンドを使用します。ファイルは
> アップロードされず、API キーも不要です。実際の文書・ローカル埋め込み・ファイル監視・
> AI Provider を試すにはデスクトップ版をインストールしてください。

![オンライン Demo](docs/assets/demo-online.png)

## ダウンロード

インストーラーは **GitHub Releases** で公開しています。ダウンロードにアカウントは不要です。
OS とアーキテクチャに合わせて選択してください。

| プラットフォーム | アーキテクチャ         | インストーラー            | 直接ダウンロード                                                                                                     |
| ---------------- | ---------------------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Windows**      | x64                    | NSIS `.exe`（推奨）       | [Lumos_0.1.0_x64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_x64-setup.exe)     |
| **Windows**      | x64                    | MSI                       | [Lumos_0.1.0_x64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_x64_en-US.msi)     |
| **Windows**      | arm64                  | NSIS `.exe`               | [Lumos_0.1.0_arm64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_arm64-setup.exe) |
| **Windows**      | arm64                  | MSI                       | [Lumos_0.1.0_arm64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_arm64_en-US.msi) |
| **macOS**        | Apple Silicon（M1–M4） | `.dmg`                    | [Lumos_0.1.0_aarch64.dmg](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_aarch64.dmg)         |
| **Linux**        | x64                    | `.AppImage`（ポータブル） | [Lumos_0.1.0_amd64.AppImage](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_amd64.AppImage)   |
| **Linux**        | x64（Debian/Ubuntu）   | `.deb`                    | [Lumos_0.1.0_amd64.deb](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_amd64.deb)             |
| **Linux**        | x64（Fedora/RHEL）     | `.rpm`                    | [Lumos-0.1.0-1.x86_64.rpm](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos-0.1.0-1.x86_64.rpm)       |

→ **すべてのリリース:** <https://github.com/DaiYiqq/lumos/releases/latest>

> 現在のインストーラーは**未署名**です。Windows SmartScreen と macOS Gatekeeper が初回起動時に
> 警告を表示する場合があります。以下の手順で安全に進めてください。
>
> **macOS Intel は未対応**です。ローカル埋め込みモデルが使う ONNX Runtime のビルド済み
> バイナリに x86_64 macOS 版が存在しないためです。

## インストール

### Windows（x64 / arm64）

1. 通常の PC では `Lumos_0.1.0_x64-setup.exe`、ARM PC（Snapdragon など）では
   `Lumos_0.1.0_arm64-setup.exe` をダウンロードします。MSI 版も利用できます。
2. インストーラーを実行します。SmartScreen が表示されたら **詳細情報 → 実行** をクリックします。
3. スタートメニューから **Lumos** を起動します。データは
   `%APPDATA%\com.daiyi.lumos` に保存されます。

### macOS（Apple Silicon）

1. `Lumos_0.1.0_aarch64.dmg` をダウンロードして開きます。
2. **Lumos** を「アプリケーション」にドラッグします。
3. 初回起動時はアプリを右クリック → **開く** → **開く**。または
   **システム設定 → プライバシーとセキュリティ → このまま開く** で許可します。
4. データは `~/Library/Application Support/com.daiyi.lumos` に保存されます。

### Linux（x64）

- **AppImage（すべてのディストリビューション）**

  ```bash
  chmod +x Lumos_0.1.0_amd64.AppImage
  ./Lumos_0.1.0_amd64.AppImage
  ```

- **Debian / Ubuntu**

  ```bash
  sudo apt install ./Lumos_0.1.0_amd64.deb
  ```

- **Fedora / RHEL**

  ```bash
  sudo dnf install ./Lumos-0.1.0-1.x86_64.rpm
  ```

Linux では WebKitGTK 4.1 と appindicator ライブラリが必要です。Debian/Ubuntu の場合:

```bash
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1 librsvg2-2
```

データは `~/.local/share/com.daiyi.lumos` に保存されます。

### ソースからビルド（全プラットフォーム）

前提: Rust ≥ 1.90（Windows は MSVC ツールチェーン）、Node.js ≥ 22、pnpm、および
[Tauri のシステム依存関係](https://tauri.app/start/prerequisites/)。

```bash
node scripts/check-env.mjs   # ツールチェーンのバージョンを確認
pnpm install
pnpm tauri dev               # ホットリロード付きデスクトップアプリ
pnpm tauri build             # 現在のプラットフォーム向けインストーラーを生成
```

開発・テスト・トラブルシューティングの詳細: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)。

## 機能

- **ローカルファースト** — 解析・チャンク分割・埋め込みはすべて端末内で実行。アカウント不要、
  アップロードなし。ネットワーク通信は、明示的に設定した AI Provider のみです。
- **追跡可能な引用** — 回答は検索されたチャンクに基づき、`[n]` をクリックすると右側パネルに
  タイトル・パス・ページ番号・セクション・原文が表示されます。
- **ハイブリッド検索** — ローカルの BGE-small-zh 埋め込み（`sqlite-vec`、512 次元）と
  SQLite FTS5 BM25 を RRF（k = 60）で融合。モデルがなくてもキーワード検索で動作します。
- **3 つの AI モード** — DeepSeek（SSE ストリーミング）、Ollama（ローカルモデル）、
  オフラインの抽出的フォールバック。根拠がない場合は明確に説明し、回答を捏造しません。
- **複数ナレッジベース** — テーマごとに独立したインデックス。ファイルやフォルダーを
  インポートでき、取り込みの一時停止・再開・キャンセル、再起動後の復元に対応します。
- **会話履歴** — ナレッジベースごとに会話を保存。名前変更・再開・削除ができます。
- **監視とエクスポート** — インデックス済みフォルダー内のファイル追加・変更・削除を検知して
  自動で再インデックス。すべてのデータを Markdown + JSON でエクスポートできます。
- **デスクトップ UX** — ライト/ダークテーマ、ストリーミング Markdown とシンタックスハイライト、
  キーボード対応ダイアログ、エラーバウンダリ、大きなリストの仮想スクロール。
- **中国語 / 英語 UI** — 設定画面で切り替え可能。選択は保存されます。

![Lumos デモ](docs/assets/demo.gif)

![引用付きチャット](docs/assets/p5-chat.png)

![ダークテーマ](docs/assets/p6-dark.png)

## 使い方

1. **ナレッジベースを作成** — 「契約書」「論文」「読書メモ」など。インデックスは分離されます。
2. **ファイルまたはフォルダーをインポート** — PDF、EPUB、DOCX、Markdown、HTML、TXT、
   一般的なコードファイルに対応。解析とチャンク分割はバックグラウンドキューで実行されます。
3. **質問する** — 「チャット」を開き、ナレッジベースを選んで入力。回答は `[n]` 引用付きで
   ストリーミング表示されます。
4. **確認する** — 引用番号をクリックすると原文・ページ・セクションを確認できます。
5. **履歴を開く** — 会話はローカルに保存され、左側の一覧から再開できます。

### AI の設定

**設定 → DeepSeek API Key** でキーを保存します。キーは OS の資格情報ストア
（Windows 資格情報マネージャー / macOS キーチェーン / libsecret）に保存され、データベースや
ログには書き込まれません。代わりに:

- **Ollama** — エンドポイント（既定 `http://127.0.0.1:11434`）とモデル名を設定。
  DeepSeek キーがない場合は自動検出して優先します。
- **オフライン** — どちらも利用できない場合、キーワード検索 + 明示的な通知にフォールバック。
  アプリは引き続き検索と引用の閲覧に使えます。

自動化向けに環境変数も利用できます: `DEEPSEEK_API_KEY`、`OLLAMA_BASE_URL`、
`OLLAMA_MODEL`、`DEEPSEEK_MODEL`。ログには出力されません。

## 仕組み

```text
┌──────────────────────────── React 19 + TypeScript ────────────────────────────┐
│  ライブラリ  │  チャット（ストリーミング Markdown + 引用パネル）  │  設定        │
│  zustand stores · Tauri IPC（invoke / Channel / events）                       │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ commands
┌───────────────▼─────────────────────────────── Rust (Tauri 2) ────────────────┐
│ ingest: 抽出 → 見出し対応チャンク（512/64）→ 永続化ジョブキュー               │
│ embedding: fastembed + BGE-small-zh-v1.5（初回ダウンロード後キャッシュ）       │
│ retrieval: sqlite-vec KNN + FTS5 trigram BM25 → RRF                           │
│ providers: DeepSeek SSE / Ollama NDJSON / オフライン、単一 trait で抽象化       │
│ security: keyring（OS 資格情報ストア）+ 環境変数フォールバック                 │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ rusqlite
┌───────────────▼───────────────────────────────────────────────────────────────┐
│ SQLite（WAL）: libraries · documents · chunks · chunks_vec · chunks_fts        │
│                conversations · messages · ingest_jobs · settings             │
└───────────────────────────────────────────────────────────────────────────────┘
```

設計とデータモデル: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) ·
コマンドリファレンス: [docs/API.md](docs/API.md)。

### 検索品質

24 文書・16 個の中国語クエリで recall@3 を評価（評価スクリプトと生データは
[docs/RETRIEVAL.md](docs/RETRIEVAL.md)）:

| モード                   | recall@3  |
| ------------------------ | --------- |
| キーワード（FTS5 BM25）  | 0.750     |
| ベクトル（BGE-small-zh） | 0.688     |
| **ハイブリッド（RRF）**  | **1.000** |

## プライバシー

- 文書・チャンク・埋め込み・会話履歴は、OS のアプリデータディレクトリにある 1 つの SQLite
  ファイルに保存されます（Windows は `%APPDATA%\com.daiyi.lumos`）。Lumos のサーバーは
  存在せず、何もアップロードされません。
- API キーは OS の資格情報ストアに保存され、ログにキーや文書本文は含まれません。
- ネットワークアクセスは、設定した AI Provider と埋め込みモデルの初回ダウンロード
  （Hugging Face、失敗時はミラー）に限定されます。
- **エクスポートとバックアップ** — 設定 → すべてのデータをエクスポートで Markdown + JSON を
  書き出せます。データディレクトリをコピーすれば完全バックアップになります。
- ナレッジベースや会話を削除すると、対応する行はローカルデータベースから削除されます。

## ロードマップ

- ✅ P0 — 環境チェック、スキャフォールド、ブランドアイコン
- ✅ P1 — SQLite スキーマ、マイグレーション、ナレッジベース CRUD
- ✅ P2 — 取り込み: 解析 → チャンク → 保存、永続化された一時停止/再開キュー
- ✅ P3 — ハイブリッド検索: sqlite-vec + BGE + BM25 → RRF
- ✅ P4 — Provider: DeepSeek / Ollama / オフライン、ストリーミングと OS キーチェーン
- ✅ P5 — チャット UI: ストリーミング回答、クリック可能な引用、会話履歴
- ✅ P6 — 仕上げ: ログ、ファイル監視、エクスポート、エラーバウンダリ、仮想リスト、ダークテーマ
- ✅ P7 — 中国語/英語 UI、ドキュメント、CI、リリースワークフロー
- ✅ P8 — 初回公開リリース: Windows（x64/arm64）、macOS Apple Silicon、Linux x64

## ドキュメント

- [開発ガイドとトラブルシューティング](docs/DEVELOPMENT.md)
- [アーキテクチャとデータモデル](docs/ARCHITECTURE.md)
- [Tauri コマンドリファレンス](docs/API.md)
- [ハイブリッド検索ベンチマーク](docs/RETRIEVAL.md)
- [翻訳ガイド](docs/I18N.md)

## コミュニティ

Lumos は**コミュニティによって維持**されており、あらゆる規模の貢献を歓迎します。

- [コントリビューションガイド](CONTRIBUTING.md) — 開発環境、品質ゲート、PR の流れ、最初のタスク
- [行動規範](CODE_OF_CONDUCT.md) — 敬意と寛容さを大切に
- [ガバナンス](GOVERNANCE.md) · [メンテナー](MAINTAINERS.md) — 役割、意思決定、メンテナーになる方法
- [セキュリティポリシー](SECURITY.md) — 脆弱性は非公開で報告
- [翻訳ガイド](docs/I18N.md) — UI 言語や README 翻訳の追加
- [Discussions](https://github.com/DaiYiqq/lumos/discussions) · [Issues](https://github.com/DaiYiqq/lumos/issues)
- はじめやすいラベル: [`good first issue`](https://github.com/DaiYiqq/lumos/labels/good%20first%20issue) ·
  [`help wanted`](https://github.com/DaiYiqq/lumos/labels/help%20wanted) ·
  [`translation`](https://github.com/DaiYiqq/lumos/labels/translation)

## コントリビュート

Issue と Pull Request を歓迎します。提出前に以下を実行してください:

```bash
pnpm format && pnpm typecheck && pnpm lint && pnpm test
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

コミットは焦点を絞り、秘密情報や個人文書を含めないでください。バグ修正にはテストを追加して
ください。詳しい流れは [CONTRIBUTING.md](CONTRIBUTING.md)、開発の詳細は
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#contributing) を参照してください。

## 開発者

Lumos は **戴亿（DaiYi）** が作成し、維持しています。

| 項目           | 連絡先                                            |
| -------------- | ------------------------------------------------- |
| 開発者         | 戴亿（DaiYi）                                     |
| QQ メール      | [1703205607@qq.com](mailto:1703205607@qq.com)     |
| QQ             | 1703205607                                        |
| Gmail          | [daiy91004@gmail.com](mailto:daiy91004@gmail.com) |
| 抖音（Douyin） | daiyi091004                                       |
| WeChat         | daiyi091004                                       |
| GitHub         | [@DaiYiqq](https://github.com/DaiYiqq)            |

詳細は [DEVELOPERS.md](DEVELOPERS.md) を参照してください。連絡先はプロジェクトに関する
連絡のみに使用します。セキュリティ問題は [SECURITY.md](SECURITY.md) に従って非公開で報告してください。

## ライセンス

[MIT](LICENSE) © 2026 DaiYi
