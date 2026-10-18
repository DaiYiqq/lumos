# Lumos

**本地優先的個人知識庫：把文件匯入，用 AI 提問，每個回答都標註可點擊的引用。**

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![CI](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml/badge.svg)](https://github.com/DaiYiqq/lumos/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DaiYiqq/lumos)](https://github.com/DaiYiqq/lumos/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/DaiYiqq/lumos/total)](https://github.com/DaiYiqq/lumos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Demo](https://img.shields.io/badge/demo-online-brightgreen)](https://daiyiqq.github.io/lumos/)

Lumos 會匯入 PDF、EPUB、Word、Markdown、網頁、純文字與程式碼檔案，在**本機**完成解析、切塊、
向量化與檢索，並**只依據你的文件**回答問題。每個結論都帶 `[n]` 引用編號，點一下即可回到原文的
具體段落、頁碼與章節。文件、向量與對話全部保存在本機的一個 SQLite 資料庫裡。

> ▶ **線上 Demo：** <https://daiyiqq.github.io/lumos/>
>
> Demo 完全在瀏覽器內執行，內建少量範例文件與模擬後端；不會上傳檔案，也不需要 API Key。
> 真實文件、本機模型、檔案監看與 AI Provider 請下載桌面版體驗。

![線上 Demo](docs/assets/demo-online.png)

## 下載

安裝檔發布於 **GitHub Releases**，下載不需要註冊或登入。依作業系統與架構選擇：

| 平台        | 架構                   | 安裝檔                | 直接下載                                                                                                             |
| ----------- | ---------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Windows** | x64                    | NSIS `.exe`（推薦）   | [Lumos_0.1.0_x64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_x64-setup.exe)     |
| **Windows** | x64                    | MSI                   | [Lumos_0.1.0_x64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_x64_en-US.msi)     |
| **Windows** | arm64                  | NSIS `.exe`           | [Lumos_0.1.0_arm64-setup.exe](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_arm64-setup.exe) |
| **Windows** | arm64                  | MSI                   | [Lumos_0.1.0_arm64_en-US.msi](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_arm64_en-US.msi) |
| **macOS**   | Apple Silicon（M1–M4） | `.dmg`                | [Lumos_0.1.0_aarch64.dmg](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_aarch64.dmg)         |
| **Linux**   | x64                    | `.AppImage`（免安裝） | [Lumos_0.1.0_amd64.AppImage](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_amd64.AppImage)   |
| **Linux**   | x64（Debian/Ubuntu）   | `.deb`                | [Lumos_0.1.0_amd64.deb](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos_0.1.0_amd64.deb)             |
| **Linux**   | x64（Fedora/RHEL）     | `.rpm`                | [Lumos-0.1.0-1.x86_64.rpm](https://github.com/DaiYiqq/lumos/releases/download/v0.1.0/Lumos-0.1.0-1.x86_64.rpm)       |

→ **所有版本：** <https://github.com/DaiYiqq/lumos/releases/latest>

> 目前安裝檔**未簽章**，Windows SmartScreen 與 macOS Gatekeeper 首次執行時可能提示未知發行者，
> 下方提供安全的處理步驟。
>
> **暫不支援 macOS Intel**：本機嵌入模型依賴的 ONNX Runtime 預編譯產物沒有 x86_64 macOS 版本。

## 安裝

### Windows（x64 / arm64）

1. 一般電腦下載 `Lumos_0.1.0_x64-setup.exe`；ARM 電腦（Snapdragon 等）下載
   `Lumos_0.1.0_arm64-setup.exe`。偏好 MSI 也可使用對應的 `.msi`。
2. 執行安裝檔。若出現 SmartScreen，點選 **更多資訊 → 仍要執行**。
3. 從開始功能表啟動 **Lumos**。資料目錄為 `%APPDATA%\com.daiyi.lumos`。

### macOS（Apple Silicon）

1. 下載 `Lumos_0.1.0_aarch64.dmg` 並開啟。
2. 將 **Lumos** 拖曳到「應用程式」。
3. 首次啟動：右鍵點擊應用程式 → **打開** → **打開**；或於
   **系統設定 → 隱私權與安全性 → 仍要打開** 中允許。
4. 資料目錄為 `~/Library/Application Support/com.daiyi.lumos`。

### Linux（x64）

- **AppImage（適用所有發行版）**

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

Linux 需要 WebKitGTK 4.1 與 appindicator 函式庫；Debian/Ubuntu 可安裝：

```bash
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1 librsvg2-2
```

資料目錄為 `~/.local/share/com.daiyi.lumos`。

### 從原始碼建置（全平台）

前置條件：Rust ≥ 1.90（Windows 使用 MSVC 工具鏈）、Node.js ≥ 22、pnpm，以及
[Tauri 系統依賴](https://tauri.app/start/prerequisites/)。

```bash
node scripts/check-env.mjs   # 檢查工具鏈版本
pnpm install
pnpm tauri dev               # 帶熱更新的桌面應用程式
pnpm tauri build             # 產生目前平台的安裝檔
```

完整開發、測試與疑難排解請見 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)。

## 功能

- **本地優先**：解析、切塊、嵌入都在本機完成，不需要帳號、不上傳文件；只有你明確設定的
  AI Provider 會產生網路請求。
- **可追溯引用**：回答基於檢索到的切塊產生，`[n]` 展開右側原文面板，顯示標題、路徑、頁碼、
  章節與原始片段。
- **混合檢索**：本機 BGE-small-zh 向量（`sqlite-vec`，512 維）+ SQLite FTS5 BM25，
  使用 RRF（k = 60）融合；沒有模型時自動退化為關鍵字檢索。
- **三種 AI 模式**：DeepSeek（SSE 串流）、Ollama（本機模型）、離線抽取式備援；
  沒有依據時會明確說明，不編造答案。
- **多知識庫**：不同主題獨立索引；支援匯入檔案或整個資料夾，暫停/繼續/取消匯入，
  重新啟動後任務可恢復。
- **對話歷史**：依知識庫保存工作階段，可重新命名、重新開啟、刪除。
- **監看與匯出**：已索引目錄中的檔案新增、修改、刪除會自動重新索引；所有資料可匯出為
  Markdown + JSON。
- **桌面體驗**：深淺色主題、串流 Markdown 與語法高亮、鍵盤友善對話框、錯誤邊界、
  大型清單虛擬滾動。
- **中英雙語介面**：在設定頁切換中文 / English，選擇會被記住。

![Lumos 示範](docs/assets/demo.gif)

![帶引用的問答介面](docs/assets/p5-chat.png)

![深色主題](docs/assets/p6-dark.png)

## 使用

1. **建立知識庫**：例如「合約」「論文」「讀書筆記」，不同知識庫的索引彼此隔離。
2. **匯入檔案或資料夾**：支援 PDF、EPUB、DOCX、Markdown、HTML、TXT 與常見程式碼檔案；
   解析與切塊在背景佇列執行，可暫停、繼續或取消。
3. **提問**：開啟「問答」，選擇知識庫後輸入問題，回答會串流輸出並帶 `[n]` 引用。
4. **核對**：點擊引用編號查看原文片段、頁碼與章節，再次點擊即可取消選取。
5. **回看歷史**：工作階段保存在本機，從左欄開啟即可繼續。

### AI 設定

開啟 **設定 → DeepSeek API Key** 儲存金鑰。金鑰會寫入系統憑證儲存區
（Windows 認證管理員 / macOS 鑰匙圈 / libsecret），不會進入資料庫或日誌。也可以：

- **Ollama**：填寫位址（預設 `http://127.0.0.1:11434`）與模型名稱；未設定 DeepSeek Key 時
  Lumos 會自動偵測並優先使用。
- **離線**：兩者都不可用時，回答退化為關鍵字檢索 + 明確提示，應用程式仍可搜尋與閱讀引用。

自動化情境支援環境變數：`DEEPSEEK_API_KEY`、`OLLAMA_BASE_URL`、`OLLAMA_MODEL`、
`DEEPSEEK_MODEL`；這些值不會被寫入日誌。

## 運作原理

```text
┌──────────────────────────── React 19 + TypeScript ────────────────────────────┐
│  知識庫  │  問答（串流 Markdown + 引用面板）  │  設定                          │
│  zustand stores · Tauri IPC（invoke / Channel / events）                       │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ commands
┌───────────────▼─────────────────────────────── Rust (Tauri 2) ────────────────┐
│ ingest：解析 → 標題感知切塊（512/64）→ 持久化任務佇列                         │
│ embedding：fastembed + BGE-small-zh-v1.5（首次下載後快取）                     │
│ retrieval：sqlite-vec KNN + FTS5 trigram BM25 → RRF                           │
│ providers：DeepSeek SSE / Ollama NDJSON / 離線備援，統一 trait                 │
│ security：keyring（系統憑證儲存區）+ 環境變數備援                              │
└───────────────┬───────────────────────────────────────────────────────────────┘
                │ rusqlite
┌───────────────▼───────────────────────────────────────────────────────────────┐
│ SQLite（WAL）：libraries · documents · chunks · chunks_vec · chunks_fts        │
│                 conversations · messages · ingest_jobs · settings             │
└───────────────────────────────────────────────────────────────────────────────┘
```

設計與資料模型：[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) ·
命令參考：[docs/API.md](docs/API.md)。

### 檢索效果

在 24 篇混合文件、16 個中文查詢上評測 recall@3（評測腳本與原始結果見
[docs/RETRIEVAL.md](docs/RETRIEVAL.md)）：

| 模式                 | recall@3  |
| -------------------- | --------- |
| 關鍵字（FTS5 BM25）  | 0.750     |
| 向量（BGE-small-zh） | 0.688     |
| **混合（RRF）**      | **1.000** |

## 隱私

- 文件、切塊、向量與對話都在系統應用程式資料目錄下的一個 SQLite 檔案裡
  （Windows 為 `%APPDATA%\com.daiyi.lumos`）。Lumos 沒有伺服器，不會上傳任何內容。
- API Key 存在系統憑證儲存區；日誌中不會出現金鑰或文件正文。
- 網路請求僅限你設定的 AI Provider，以及首次下載本機嵌入模型（Hugging Face，失敗時走鏡像）。
- **匯出與備份**：設定 → 匯出全部資料，可寫出 Markdown + JSON；直接複製資料目錄即可完整備份。
- 刪除知識庫或工作階段時，對應資料會從本機資料庫移除。

## 路線圖

- ✅ P0 — 環境檢查、腳手架、品牌圖示
- ✅ P1 — SQLite schema、遷移、知識庫 CRUD
- ✅ P2 — 匯入：解析 → 切塊 → 入庫，持久化暫停/恢復佇列
- ✅ P3 — 混合檢索：sqlite-vec + BGE + BM25 → RRF
- ✅ P4 — Provider：DeepSeek / Ollama / 離線備援，串流與系統憑證儲存區
- ✅ P5 — 問答介面：串流回答、可點擊引用、對話歷史
- ✅ P6 — 打磨：日誌、檔案監看、資料匯出、錯誤邊界、虛擬清單、深色主題
- ✅ P7 — 中英雙語、文件、CI 與發布流程
- ✅ P8 — 首次公開發布：Windows（x64/arm64）、macOS Apple Silicon、Linux x64

## 文件

- [開發與疑難排解](docs/DEVELOPMENT.md)
- [架構與資料模型](docs/ARCHITECTURE.md)
- [Tauri 命令參考](docs/API.md)
- [混合檢索評測](docs/RETRIEVAL.md)
- [翻譯指南](docs/I18N.md)

## 社群

Lumos 由社群共同維護，歡迎任何規模的貢獻。

- [貢獻指南](CONTRIBUTING.md) —— 開發環境、品質關卡、PR 流程、新手任務
- [行為準則](CODE_OF_CONDUCT.md) —— 保持友善與尊重
- [治理方式](GOVERNANCE.md) · [維護者](MAINTAINERS.md) —— 角色、決策流程、如何成為維護者
- [安全策略](SECURITY.md) —— 私下回報安全漏洞
- [翻譯指南](docs/I18N.md) —— 新增介面語言或 README 翻譯
- [Discussions](https://github.com/DaiYiqq/lumos/discussions) · [Issues](https://github.com/DaiYiqq/lumos/issues)
- 新手可以從這些標籤開始：[`good first issue`](https://github.com/DaiYiqq/lumos/labels/good%20first%20issue) ·
  [`help wanted`](https://github.com/DaiYiqq/lumos/labels/help%20wanted) ·
  [`translation`](https://github.com/DaiYiqq/lumos/labels/translation)

## 參與貢獻

歡迎提交 Issue 與 Pull Request。提交前請執行：

```bash
pnpm format && pnpm typecheck && pnpm lint && pnpm test
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

請保持提交聚焦，不要提交金鑰或個人文件；修正 Bug 時請補充測試。
完整流程見 [CONTRIBUTING.md](CONTRIBUTING.md)，開發細節見
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#contributing)。

## 授權

[MIT](LICENSE) © 2026 DaiYi
