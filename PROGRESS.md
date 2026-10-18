# Lumos 开发进度日志

> 本文件是项目的「施工日志」：每个阶段做了什么、踩了什么坑、当前已知问题、下一步计划。
> 所有阶段完成都必须更新此文件并 commit。

---

## 项目配置（已与项目负责人确认）

| 配置项          | 值                |
| --------------- | ----------------- |
| GITHUB_USERNAME | DaiYiqq           |
| REPO_NAME       | lumos             |
| REPO_VISIBILITY | public            |
| LICENSE         | MIT               |
| AUTHOR_NAME     | DaiYi             |
| AUTHOR_EMAIL    | 1703205607@qq.com |
| DEFAULT_BRANCH  | main              |
| PROJECT_DIR     | `D:\dev\lumos`    |

---

## P0 · 环境核实 + 脚手架 + 品牌图标

**状态：已完成（2026-10-18）**

### 环境核实结果（2026-10-18，实际运行）

| 项目             | 结果                                                                         |
| ---------------- | ---------------------------------------------------------------------------- |
| 操作系统         | Windows 11（10.0.26300），x86_64                                             |
| Rust             | 1.99.1，`stable-x86_64-pc-windows-msvc`                                      |
| Cargo            | 1.99.1                                                                       |
| MSVC             | Visual Studio 生成工具 2026，MSVC 14.51.36231                                |
| Windows SDK      | 10.0.26100.0                                                                 |
| WebView2 Runtime | 155.0.4283.60                                                                |
| Node.js          | v24.21.0（≥ 22 满足要求）                                                    |
| npm / pnpm       | 11.19.0 / 12.14.0                                                            |
| Git              | 2.56.0，`core.longpaths=true`、`core.autocrlf=false`                         |
| GitHub CLI       | 2.102.0（尚未 `gh auth login`）                                              |
| Defender         | 服务未运行，无需配置排除项                                                   |
| 网络             | 直连 crates.io / npm / github 可用（无镜像）；若后续构建超时再配置用户级镜像 |

环境自检脚本：`scripts/check-env.mjs`（跨平台 Node 脚本，任一 ❌ 时以非零码退出）。
首次运行发现缺失 Rust / gh / pnpm，已分别用 `rustup-init`、`winget`、`npm i -g` 安装，复跑全绿。

### 联网核实的版本（2026-10-18，来自官方 registry / Tauri 官方 schema）

| 依赖                        | 选定版本                                                | 说明                                                                                                              |
| --------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Tauri                       | 2.12.4（CLI 2.12.3）                                    | `crates.io` / `npm` 当前稳定版                                                                                    |
| Tauri MSRV                  | Rust 1.90                                               | 取自 `tauri-v2.12.4` 仓库 `Cargo.toml` 的 `rust-version`                                                          |
| tauri.conf.json schema      | `https://schema.tauri.app/config/2`（已下载并逐项核对） | 字段 `security.csp/devCsp`、`window.width/minWidth/center`、`bundle.targets/icon` 均确认存在                      |
| 脚手架                      | `create-tauri-app` 4.7.5，`--template react-ts`         | 官方脚手架，非手写配置                                                                                            |
| React / React DOM           | 19.3.1                                                  |                                                                                                                   |
| TypeScript                  | 6.0.3                                                   | ⚠️ 与任务书写的「5.x」不同：官方模板锁 `~6.0.3`，且 `typescript-eslint@8.71.2` 的 peer 要求 `<6.1.0`，TS 7 不兼容 |
| Vite                        | 8.3.4                                                   | ⚠️ 与任务书写的「6.x」不同：官方模板已是 `^8`，插件 peer 均为 Vite 8                                              |
| Tailwind CSS                | 4.3.3（`@tailwindcss/vite`）                            | 符合 v4 要求                                                                                                      |
| zustand                     | 5.0.15                                                  |                                                                                                                   |
| react-markdown / remark-gfm | 10.1.0 / 4.0.1                                          |                                                                                                                   |
| highlight.js                | 11.12.0                                                 |                                                                                                                   |
| lucide-react                | 1.59.0                                                  |                                                                                                                   |
| @tanstack/react-virtual     | 3.14.21                                                 |                                                                                                                   |
| eslint / prettier / vitest  | 10.13.0 / 3.9.10 / 5.0.5                                |                                                                                                                   |

> 所有「版本与任务书不一致」的地方都以**官方脚手架与官方 registry 的实际版本**为准，原因见上表。

### 已完成

- [x] 安装缺失依赖：Rust 1.99.1、GitHub CLI 2.102.0、pnpm 12.14.0
- [x] `git config --global core.longpaths true` + `core.autocrlf false`
- [x] 编写并实测 `scripts/check-env.mjs`（含非 ASCII/空格路径的错误分支实测）
- [x] 用官方 `create-tauri-app` 生成 Tauri 2 + React + TS 脚手架
- [x] `tauri.conf.json` 按官方 schema 定制（窗口尺寸、CSP/devCsp、bundle 元信息、图标清单）
- [x] 自研品牌图标：`scripts/generate-icon.py`（Pillow 纯代码绘制）→ `docs/assets/lumos-icon.png` → `pnpm tauri icon` 生成全套尺寸（ico/icns/png/Square\*/StoreLogo）
- [x] Rust 侧 `commands/app.rs` 提供 `app_info` 命令，前端状态栏显示版本与数据目录
- [x] 极简前端骨架：自研 Button / EmptyState / Logo，zustand 主题（跟随系统/浅色/深色），三个一级页面导航

### P0 验收记录（实际运行结果）

| 检查项          | 命令                                        | 结果                                                                                                                              |
| --------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| 环境自检        | `node scripts/check-env.mjs`                | 退出码 0，全部 ✅                                                                                                                 |
| 类型检查        | `pnpm typecheck`                            | 通过                                                                                                                              |
| 单元测试        | `pnpm test`                                 | 3 个用例全部通过（Vitest 5 + jsdom）                                                                                              |
| 前端 Lint       | `pnpm lint`                                 | 退出码 0                                                                                                                          |
| 前端构建        | `pnpm build`                                | 通过，`index.js` 234.56 kB / `index.css` 13.45 kB                                                                                 |
| Rust 格式       | `cargo fmt --all --check`                   | 通过                                                                                                                              |
| Rust Lint       | `cargo clippy --all-targets -- -D warnings` | 通过（零警告）                                                                                                                    |
| 调试构建        | `cargo build`                               | 通过                                                                                                                              |
| 发布构建 + 打包 | `pnpm tauri build --bundles nsis`           | 通过，产物 `src-tauri/target/release/bundle/nsis/Lumos_0.1.0_x64-setup.exe`（1.62 MiB）                                           |
| 窗口启动        | 分别启动 dev 与 release 构建                | 两种构建都能弹出 `Lumos` 窗口                                                                                                     |
| IPC 连通        | 前端调用 `app_info` 命令                    | 状态栏显示 `~\AppData\Roaming\com.daiyi.lumos`、`Lumos v0.1.0 · windows`（家目录显示为 `~`，不暴露用户名，完整路径在 tooltip 中） |
| 生产 CSP        | release 构建实际渲染                        | 无白屏、样式与中文字体正常                                                                                                        |

窗口截图证据：`docs/assets/p0-window.png`（通过 WebView2 CDP 截取的完整视口，含底部状态栏）。

### 踩坑记录

1. **winget 安装 Rustup 失败（退出码 1）**：winget 的 rustup manifest 调用的安装器在当前机器上交互动失败。
   解决：直接下载官方 `rustup-init.exe`，用 `-y --default-host x86_64-pc-windows-msvc` 安装。
2. **PowerShell 5.1 下 `Set-Content -Encoding utf8` 会把中文源码写成乱码**：曾把 `generate-icon.py` 写坏。
   解决：改用编辑工具写入 UTF-8；后续所有含中文文件不再经过 PowerShell 转码。
3. **TypeScript 6 弃用 `baseUrl`**（error TS5101）：`paths` 现在直接相对 tsconfig 解析，删除 `baseUrl` 即可。
4. **`where link.exe` 找不到属正常现象**：cargo 通过 vswhere 自动定位 MSVC 链接器，自检脚本已对这种情况显示「自动探测」。
5. **`generate_handler!` 不能引用 `pub use` 重导出的命令**：宏会在命令定义的模块里生成 `__cmd__*` 隐藏项，必须写完整路径 `commands::app::app_info`。
6. **WebView2 窗口用 `PrintWindow` 截图会漏掉底部状态栏**（合成层未完整绘制），且 `SetForegroundWindow` 常被前台窗口抢回。
   解决：开发期给窗口加 `additionalBrowserArgs` 打开 CDP（仅本地验证用，已从配置移除），用 CDP 截取完整视口；release 包另用 PrintWindow 验证不白屏。
7. **release 首次构建 5 分钟**（LTO + 465 个 crate）；NSIS 与 nsis_tauri_utils 首次会自动从 GitHub 下载并校验哈希，国内网络下这一步可能较慢，但本次直连成功。

### 已知问题

- 前端一级页面暂时是空状态（知识库/问答/设置），数据层与检索层接入后逐个落地；这属于阶段计划内，不是遗留 TODO。
- `gh` 尚未登录，P8 推送前需要项目负责人执行 `gh auth login`（浏览器 device flow）。
- 主题默认跟随系统；深浅色两套 token 已就绪，但尚未做视觉走查（P6 统一打磨）。

### 下一步（P1）

数据层：SQLite 连接与迁移（`schema_migrations`）、library CRUD、单元测试。
