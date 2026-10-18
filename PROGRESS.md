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

- `gh` 尚未登录，P8 推送前需要项目负责人执行 `gh auth login`（浏览器 device flow）。
- 主题默认跟随系统；深浅色两套 token 已就绪，但尚未做视觉走查（P6 统一打磨）。
- P1 的知识库列表未做虚拟化；P6 性能阶段会处理超大列表（配合文档列表一起）。

---

## P1 · 数据层：迁移、repository、CRUD 命令

**状态：已完成（2026-10-18）**

### 已完成

- **连接层** `db/mod.rs`：`Database` 用 `Mutex<Connection>` 串行化写操作，开启 `foreign_keys=ON`、WAL、`synchronous=NORMAL`、`busy_timeout=5000`；提供 `open` / `open_in_memory` / `migrate` / `with_connection`。
- **迁移** `db/migrations.rs` + `migrations/0001_init.sql`：手写版本号迁移（`schema_migrations`），每条迁移与版本记录同事务提交，可重复执行。
- **初始结构**：`libraries`、`documents`、`chunks`、`conversations`、`messages`、`chunks_fts`（FTS5 外部内容表）+ 同步触发器；所有子表 `ON DELETE CASCADE`。
- **仓储层** `db/repo/libraries.rs`：`create_library` / `get_library` / `list_libraries`（含文档数、分块数、占用字节的聚合查询）/ `rename_library` / `delete_library`；名称去空白、长度上限 80 字符、大小写不敏感重名检测。
- **命令层** `commands/libraries.rs`：`library_list` / `library_create` / `library_rename` / `library_delete`，薄层转发 + 错误转人话；`state.rs` 持有 `AppState`，`lib.rs` 的 `setup` 里完成「数据目录 → 打开 `lumos.db` → 迁移 → manage」。
- **前端**：知识库页面完整可用（卡片列表、文档/分块/占用统计、新建、重命名、删除确认、错误横幅、骨架屏、空状态）；自研 `Dialog` / `Input` / `IconButton`；`stores/libraries.ts` 封装 zustand 状态。

### 验收记录（实际运行结果）

| 检查项                                      | 结果                                                                                                                                              |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo test`                                | **9 passed, 0 failed**（迁移幂等、建表、CRUD、重名/空名/超长名、重命名冲突、级联删除、FTS 触发器）                                                |
| `cargo clippy --all-targets -- -D warnings` | 退出码 0，零警告                                                                                                                                  |
| `cargo fmt --all --check`                   | 通过                                                                                                                                              |
| `pnpm typecheck` / `pnpm lint`              | 通过                                                                                                                                              |
| `pnpm test`                                 | **6 passed**（含创建、重命名、删除确认三个交互用例）                                                                                              |
| 真实应用 E2E                                | 通过 CDP 驱动真实窗口：创建「P1 验收库」→ 列表出现并显示 `文档 0 / 分块 0 / 占用 0 B` → 删除并确认 → 回到空状态；截图 `docs/assets/p1-window.png` |

### 踩坑记录

1. **FTS5 中文分词**：`unicode61` 会把整段中文当成一个 token，中文子串检索几乎不可用；改用 `tokenize='trigram'`，实测 `MATCH '医学图像'` 能命中「对比学习在医学图像分割中的应用」。
2. **`rusqlite` 的 bundled SQLite 默认已启用 FTS5**，无需额外 feature；但外部内容表的同步触发器必须手写（AI/AD/AU 三个）。
3. **serde `#[serde(flatten)]` 不会继承内层结构体的命名规则**：`Library` 与 `LibrarySummary` 都各自标注 `rename_all = "camelCase"`，前端才能拿到 `documentCount` 这类字段。
4. **Tauri 同步命令里直接操作 SQLite 没问题**，但后续批量嵌入必须挪到后台任务，避免阻塞 IPC。

### 下一步（P2）

摄取层：PDF / EPUB / DOCX / Markdown / TXT / HTML 解析 → 清洗 → 层级感知分块 → 本地嵌入 → 写入 `chunks` / `chunks_fts`；任务队列支持暂停/取消与重启续跑。

---

## P2 · 摄取层：解析、分块、入库、持久化任务队列

**状态：已完成（2026-10-18）**

### 已完成

- **格式解析** `ingest/extract/`：
  - PDF：`pdf-extract`（纯 Rust），按分页符保留页码；扫描件无文本层时返回人话错误；
  - DOCX：自解析 zip 内 `word/document.xml`，识别 `w:pStyle` 标题样式并还原层级；
  - EPUB：自解析 `container.xml` → OPF → spine 顺序 → 章节 XHTML，解析失败回退按文件名读取；
  - Markdown / HTML：先统一转成 Markdown，再按 `#` 标题切章节；
  - 纯文本 / 代码文件：单章节直接入库。
- **层级感知分块** `ingest/chunk.rs`：512 token / 64 token 重叠，超长段落单独切片；每块保留 `page_no`、`section_path`、`char_start/char_end`、`token_count`。
- **入库流水线** `ingest/pipeline.rs`：SHA-256 内容哈希做增量索引（重复内容跳过）；解析失败不破坏已有文档；文档 + 全部分块在同一事务中原子写入。
- **持久化队列** `ingest/queue.rs` + `ingest_jobs` 迁移：任务落库，串行后台消费；支持全局暂停（文件之间生效）、单任务取消；启动时把 `running` 重置为 `pending` 并自动续跑；每完成一个文件通过 `ingest://progress` 事件通知前端。
- **前端**：知识库详情页（导入文件 / 导入文件夹、实时进度条、暂停/恢复/取消、文档列表、状态徽标、删除文档）；文件夹导入用 `walkdir` 递归展开并过滤支持格式。

### 验收记录（实际运行结果）

| 检查项                                       | 结果                                                                                                                                                 |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo test`                                 | **25 passed, 0 failed**，其中包含：100 篇 Markdown 批量索引 + 统计校验、DOCX/EPUB/HTML 三种 fixture 解析、增量索引跳过、空文件人话报错、任务重启重置 |
| `cargo clippy --all-targets -- -D warnings`  | 退出码 0，零警告                                                                                                                                     |
| `pnpm typecheck` / `pnpm lint` / `pnpm test` | 通过；前端 **7 passed**（含打开知识库详情、文档列表渲染）                                                                                            |
| 真实应用 E2E（batch1）                       | 导入 7 个文件（md / txt / docx / epub / html / pdf），全部 `ready`，共 10 个分块                                                                     |
| 真实应用 E2E（暂停）                         | `ingest_pause` 后导入 batch2，3 个任务保持 `pending`，文档数停在 7                                                                                   |
| 真实应用 E2E（重启续跑）                     | 关闭进程再启动，队列自动续跑，最终 10 篇文档 / 13 个分块全部就绪；截图 `docs/assets/p2-window.png`                                                   |

### 与任务书的偏差（已在下方说明原因）

- **嵌入尚未接入**：P2 先打通「解析 → 分块 → 入库」，向量化放到 P3 与 sqlite-vec / 混合检索一起落地。原因：嵌入模型选型与 `chunks_vec` 表结构必须在检索侧一起验证，提前写会反复改写入路径。
- **文件监听（notify）尚未接入**：依赖已加入，监听逻辑放到 P6 打磨阶段统一做（增删文件的增量同步涉及与队列、删除策略的配合）。

### 踩坑记录

1. **quick-xml 0.42 API 变化较大**：`BytesText::unescape/decode` 被移除，文本内容改用 `xml10_content()` 并手动处理常见实体；属性用 `normalized_value(XmlVersion::Implicit1_0)`；`local_name().as_ref()` 返回 `&str` 而不是字节切片。
2. **EPUB 章节路径必须相对 OPF 所在目录解析**：manifest 里的 `href="chapter1.xhtml"` 在 zip 内实际是 `OEBPS/chapter1.xhtml`，直接按 href 读取会静默跳过所有章节。
3. **DOCX 的 `<w:pStyle/>` 通常是自闭合空元素**：只匹配 `Event::Start` 会拿不到标题样式，必须同时处理 `Event::Empty`。
4. **`zip` crate 9.x 的 `file_names()` 返回 `Result<Cow<str>>` 迭代器**，需要先 `filter_map(Result::ok)`。
5. **Windows 控制台按 GBK 显示 Rust 测试输出**，中文断言失败信息会显示成乱码——是终端编码问题，不是数据问题（后续日志统一写 UTF-8 文件）。

### 下一步（P3）

检索层：sqlite-vec 向量表 + 本地嵌入（fastembed）+ BM25（FTS5）→ RRF 融合；对比单一检索与混合检索效果并记录数据。
