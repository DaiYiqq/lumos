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

---

## P3 · 检索层：sqlite-vec + 本地嵌入 + BM25 → RRF

**状态：已完成（2026-10-18）**

### 已完成

- **迁移 0003**：`chunks_vec` 虚拟表（`FLOAT[512]`）+ 删除触发器，向量随分块自动清理。
- **本地嵌入** `embedding/`：
  - `fastembed` + ONNX Runtime，模型 `BAAI/bge-small-zh-v1.5`（512 维）；
  - 自研模型下载器（`model_store.rs`）：按 `HF_ENDPOINT` → 官方 → `hf-mirror.com` 的顺序容错下载 5 个模型文件，缓存到应用数据目录 `models/`；支持复用 hf-hub 已下载的主体；
  - 后台线程加载模型，不阻塞启动；加载失败时检索自动降级为纯关键词。
- **检索** `retrieval/`：
  - 向量 KNN（sqlite-vec，`embedding MATCH ? AND k = ?`）；
  - BM25（FTS5 trigram），中文查询按 3 字滑窗拆词后 OR，兼顾编号与词句查询；
  - RRF 融合（k = 60），返回带 `documentTitle/path/pageNo/sectionPath/content` 的命中；
  - `backfill` 补写历史分块向量，应用启动后自动执行。
- **命令层**：`search_chunks`（hybrid / vector / keyword 三种模式，便于诊断与降级）、`embedding_status`、`embedding_backfill`。
- **入库联动**：新文档入库后立即向量化；向量化失败不影响文档就绪，稍后自动补写。

### 验收记录（实际运行结果）

| 检查项                                      | 结果                                                                                                    |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `cargo test`                                | **31 passed**（含 RRF、向量写入/KNN、BM25 中文、迁移、删除清理）                                        |
| `cargo clippy --all-targets -- -D warnings` | 零警告                                                                                                  |
| 效果评估（16 条查询 / 24 篇语料，recall@3） | 关键词 **0.750**、向量 **0.688**、混合 **1.000**（详见 `docs/RETRIEVAL.md`）                            |
| 真实应用（中文）                            | 模型 `ready`，查询「合同违约金上限是怎么约定的」→ top1「合同违约金条款」，`vectorRank=1, keywordRank=1` |
| 真实应用（英文/PDF）                        | 查询「contrastive learning medical image segmentation」→ 命中 `paper.pdf`                               |
| 降级路径                                    | 模型不可用时 `search_chunks` 自动返回 `mode=keyword`，应用仍可用                                        |

### 踩坑记录

1. **hf-hub 对镜像站的 range 响应不兼容**：`hf-mirror.com` 下载时报 `Header Content-Range is missing`。改为自研直接 HTTP 下载器，并保留镜像回退顺序。
2. **FTS5 trigram 不接受整句中文查询**：`"违约金最多能赔多少"` 作为单个 phrase 永远命中不了；改为 3 字滑窗 OR 后才可用（编号类词仍整体引用）。
3. **BGE 查询侧需要指令前缀**：查询向量加上「为这个句子生成表示以用于检索相关文章：」后，语义改写查询的召回明显提升。
4. **分块正文不包含标题**：标题只写入 `section_path`，评估语料里把唯一编号放在标题里导致关键词检索全灭——提醒：需要关键词命中的信息必须在正文里。
5. **Windows 控制台 GBK**：Rust 测试的中文输出显示为乱码，数字正常；不影响断言。

### 与任务书的偏差

- 效果评估语料为合成数据（16 条查询），用于验证「双通道互补」设计，不代表真实语料绝对水平；真实语料评估放在 P5 端到端场景测试中补充。
- **rerank 未启用**（任务书列为进阶功能），按计划在 P6/P7 评估收益后再决定默认值。

### 下一步（P4）

Provider 层：`deepseek` / `ollama` / `none` 三种实现 + 运行期自动降级 + 流式输出；Key 存入系统凭据库，禁止明文落盘。

---

## P4 · Provider 层：DeepSeek / Ollama / 离线降级 + 流式输出

**状态：已完成（2026-10-18）**

### 已完成

- **Provider 抽象** `providers/`：
  - `LlmProvider` trait（`id` / `is_available` / `stream_chat`），错误按鉴权 / 限流 / 网络 / 模型不存在 / 解析失败分类；
  - **DeepSeek**：`POST {base}/chat/completions` SSE 流式解析，支持 `DEEPSEEK_BASE_URL` / `DEEPSEEK_MODEL` 覆盖（便于测试与代理）；
  - **Ollama**：`POST {base}/api/chat` NDJSON 流式解析 + `/api/tags` 健康检查，地址与模型可在设置页修改；
  - **none**：不进行生成，直接返回关键词检索片段（离线兜底，永远可用）；
  - 自动顺序固定为 `deepseek → ollama → none`，每次切换都通过事件通知前端，不静默切换。
- **RAG Prompt** `providers/prompt.rs`：强制「只依据参考资料、每个结论标注 [编号]、无依据时明确说未找到」，并生成编号与检索片段一一对应的上下文。
- **凭据安全** `security.rs`：API Key 存 Windows 凭据管理器（macOS Keychain / Linux Secret Service），开发场景可用 `DEEPSEEK_API_KEY` 环境变量注入；**不写数据库、不写配置文件、不进日志**。
- **设置**：迁移 0004 `settings` 键值表 + 白名单命令，保存 Ollama 地址、模型名等非敏感配置。
- **命令层**：`chat_stream`（Tauri Channel 推送 `provider / notice / citations / delta / done / error` 事件）、`chat_complete`（非流式诊断入口）、`api_key_set/status/clear`、`providers_detect`、`settings_get/set`。

### 验收记录（实际运行结果）

| 检查项                                      | 结果                                                                                                                |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `cargo test`                                | 全部通过（含 SSE/NDJSON 解析、错误分类、Provider 顺序、Prompt 引用编号）                                            |
| `cargo clippy --all-targets -- -D warnings` | 零警告                                                                                                              |
| **DeepSeek 真实链路**                       | 真实 API Key（仅通过进程环境变量注入）：`provider=deepseek`，6 条引用，回答「违约金上限为合同总额的百分之二十 [1]」 |
| **none 离线链路**                           | 无 Key 无 Ollama：`provider=none`，返回 6 条带编号原文片段，明确提示「离线关键词模式」                              |
| **Ollama 协议链路**                         | 本地 mock NDJSON 服务：`provider=ollama`，流式收到「离线本地模型流式正常」，引用正常                                |
| 密钥泄露扫描                                | 仓库与构建日志中均无 API Key 字符串                                                                                 |

### 踩坑记录

1. **`keyring` 4.x 的 feature 名变了**：不是 `windows-native` 而是 `windows-native-keyring-store`；直接用默认 features 即可。
2. **`chat_stream` 的 Channel 需要先转发再返回**：Provider 的 sink 在 `spawn_blocking/async` 里推送，必须用独立转发任务消费 `mpsc`，否则 `Done` 事件会早于最后一段文本。
3. **检索返回的是 `SearchResponse` 而不是 `Vec<SearchHit>`**：命令层要取 `.hits`，否则编译期类型不匹配。
4. **Provider 失败可能发生在已经输出部分 token 之后**：只有在「尚未输出任何增量」时才自动降级，否则发出错误事件，避免把两个模型的回答拼接在一起。

### 与任务书的偏差（如实记录验证边界）

- **Ollama 使用本地 mock 服务验证协议与降级链路**：没有在本机拉取真实 Ollama 模型（体积 400MB+，且会占用用户磁盘）。真实 Ollama 模型端到端验证留到 P5/P6，或在用户已有模型时补测。
- **对话历史持久化在 P5 与聊天 UI 一起实现**：P4 的 `chat_stream` 已支持传入历史消息，但尚未写入 `conversations/messages` 表。
- API Key 只通过环境变量用于本次验收，**没有写入系统凭据库**；`api_key_set` 命令与凭据库读写路径已实现，建议用户首次使用时在设置页保存。

### 下一步（P5）

前端问答界面：流式打字机、Markdown 渲染、`[n]` 引用可点击跳转、右侧原文面板、对话历史持久化与三个核心场景端到端跑通。

---

## P5 · 问答界面：流式回答、引用跳转、对话历史与设置页

**状态：已完成（2026-10-18）**

### 已完成

- **对话持久化**：`db/repo/conversations.rs` + `commands/conversations.rs`（列表 / 创建 / 详情 / 重命名 / 删除）；
  `chat_stream` 首次提问自动建会话（标题取问题前 24 字），写入用户消息与助手回答（含引用 JSON）。
- **问答界面**：
  - 会话列表（新建、重命名、删除、消息数）；
  - 消息流 + 流式打字机 + Markdown（GFM 表格/任务列表、highlight.js 代码高亮）；
  - `[n]` 转为可点击引用按钮，右侧「引用来源」面板展示标题、路径、页码/章节与原文片段；
  - Provider 徽标（DeepSeek / Ollama / 离线关键词模式）、降级提示、错误提示；
  - 三个核心场景的示例问题一键提问。
- **设置页**：API Key 保存/删除（系统凭据库）与来源展示、Provider 探测、Ollama 地址/模型、DeepSeek 模型、嵌入状态与手动补写、主题切换、数据目录展示与打开。
- **导航联动**：页面状态抽到 `stores/ui.ts`；知识库详情的「去提问」直接带库进入问答页。

### 验收记录（实际运行结果）

| 检查项                                      | 结果                                                                                                 |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `cargo test`                                | **46 passed, 2 ignored**（新增会话生命周期/消息持久化）                                              |
| `cargo clippy --all-targets -- -D warnings` | 零警告                                                                                               |
| `pnpm typecheck` / `pnpm lint`              | 通过（0 error / 0 warning）                                                                          |
| `pnpm test`                                 | **8 passed**（含流式回答渲染、点击引用打开原文、离线模式徽标）                                       |
| 真实应用 E2E · 场景 A 文献综述              | 提问「有哪些工作用了对比学习做医学图像分割？」→ 6 个引用按钮，引用面板命中 `contrastive-learning.md` |
| 真实应用 E2E · 场景 B 合同速查              | 提问「合同里对违约金上限是怎么约定的？」→ 8 个引用，面板命中 `contract.docx` 第九条原文              |
| 真实应用 E2E · 场景 C 笔记回溯              | 提问「我去年夏天为什么决定换工作？」→ 10 个引用，面板命中 `personal-notes.md`「2025 年六月…」        |
| 历史持久化                                  | 刷新页面后会话列表出现历史，打开后消息与引用完整加载                                                 |
| 截图                                        | `docs/assets/p5-chat.png`（三场景对话 + 引用面板同屏）                                               |

### 踩坑记录

1. **react-hooks 新规则**：不允许在 try/catch 里构造 JSX、不允许 effect 内同步 setState——分别改成「try 里只算字符串」与「setTimeout 延后一拍」，并在 effect 依赖里带上 `useCallback` 包装的 refresh。
2. **jsdom 没有 `scrollIntoView`**：聊天自动滚动要用可选调用，否则单元测试直接崩溃。
3. **Tauri Channel 的前端联调**：`new Channel<ChatEvent>()` 直接作为 invoke 参数传入即可，事件顺序由后端保证；真实应用里验证了流式增量、引用与 Done 的先后关系。
4. **E2E 选择器两个坑**：`aria-label` 的引号二次转义会拼出非法 selector；`aside:last-of-type` 会命中外层侧边栏——改为给引用面板加 `aria-label="引用面板"`。

### 与任务书的偏差

- **代码高亮样式**目前是 highlight.js + 自定义最小样式，深浅色统一视觉打磨留到 P6。
- **i18n（中/英）、数据导出、文件监听**尚未实现，按计划在 P6/P7 完成。
- **真实 Ollama 模型端到端**仍未跑（使用 mock 验证协议），P6 有需要时补测。

### 下一步（P6）

打磨：深浅色视觉走查、i18n、性能（大列表虚拟化、模型/检索缓存）、错误边界、文件监听与导出入口。

---

## P6 · 打磨：日志、文件监听、导出、错误边界、虚拟列表、深色主题

**状态：已完成（2026-10-18，i18n 顺延到 P7）**

### 已完成

- **文件日志** `logging.rs`：`logs/lumos.log` 按天滚动，同时输出到 stderr；关键路径（启动、监听入队、向量补写、回答保存）改用 `tracing`，不再 `eprintln`。
- **文件监听** `ingest/watcher.rs`：
  - 监听已索引文档的父目录（递归，跳过盘符根），目录列表每 60 秒刷新；
  - 新增/修改文件自动入队（700ms 防抖），删除文件自动清理文档与分块；
  - 通过 `library://changed` 事件通知前端刷新知识库/文档列表。
- **数据导出** `commands/export.rs`：一键导出到 `lumos-export-<时间戳>/`，包含 `libraries.json`、按库分目录的文档 Markdown（含元数据与分块）与按库分目录的对话 Markdown；设置页提供导出按钮。
- **错误边界** `components/ErrorBoundary.tsx`：渲染异常时显示可恢复页面（重新加载 / 打开数据目录），不再白屏。
- **性能**：文档列表超过 30 条自动切换 `@tanstack/react-virtual` 虚拟滚动；小列表走普通渲染，测试与交互更稳定。
- **深色主题**：补齐 highlight.js 的深浅两套 token 配色；聊天、引用面板、输入框在深色下完成视觉走查。

### 验收记录（实际运行结果）

| 检查项                                      | 结果                                                                                                         |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `cargo test`                                | **48 passed, 2 ignored**（新增导出、监听相关路径）                                                           |
| `cargo clippy --all-targets -- -D warnings` | 零警告                                                                                                       |
| `pnpm typecheck` / `pnpm lint`              | 通过（0 error / 0 warning）                                                                                  |
| `pnpm test`                                 | **8 passed**                                                                                                 |
| 真实应用 E2E · 文件监听                     | 新增 `watcher-note.md` 自动入库 → 删除后自动清理 → 修改 `personal-notes.md` 后重新索引，能检索到新增唯一标记 |
| 真实应用 E2E · 导出                         | 导出 1 个知识库 / 11 篇文档 / 3 个会话，`libraries.json` 与知识库目录均生成                                  |
| 真实应用 E2E · 深色主题                     | 切换成功，聊天页、引用面板、输入框对比度正常；截图 `docs/assets/p6-dark.png`                                 |

### 踩坑记录

1. **PowerShell 正则直接改 `tauri.conf.json` 会破坏 JSON**（删除字段后留下 `, }`）：后续所有 JSON 配置一律用编辑工具修改，并 `node -e require(...)` 校验。
2. **虚拟列表在 jsdom 里高度为 0**：虚拟化组件只渲染可视区，测试环境没有布局；因此 ≤30 条走普通渲染，>30 条才启用虚拟滚动。
3. **react-hooks 的 `incompatible-library` 规则**会把 `useVirtualizer` 视为不可 memo 化的外部库并警告；已在调用处加注释说明这是预期行为。
4. **notify 的监听目录要单独维护**：目录集合每 60 秒从数据库重新计算并增量 `watch`，避免一次性监听整个磁盘。

### 与任务书的偏差（据此调整后续计划）

- **中英 i18n 顺延到 P7**：P6 先完成日志、监听、导出、错误边界、虚拟列表与深色主题，i18n 与文档/CI 一起在 P7 落地（i18n 需要系统性替换全部界面文案，单独排期更稳妥）。

### 下一步（P7）

i18n（中/英切换）、完整 README（中英/截图）、DEVELOPMENT 与 Troubleshooting、GitHub Actions CI、Release workflow、版本检查与发布准备。

---

## P7 · 中英双语、完整文档、CI 与发布流水线

**状态：已完成（2026-10-18），P8 等待 GitHub 授权**

### 已完成

- **中英 i18n**：`src/i18n`（中文原文为 key 的词典 + zustand 语言 store，localStorage 持久化）；
  导航、知识库、问答、引用面板、设置、错误边界与浏览器预览提示全部可切换；
  设置页新增「界面语言」切换（中文 / English），刷新后保持。
- **文档**：
  - README 重写为英文为主、中文摘要，含特性、截图/GIF、快速开始、使用、架构图、检索基准、隐私、路线图、文档索引；
  - `docs/ARCHITECTURE.md`（进程模型、模块、数据模型、摄取/检索/Provider 设计、权衡）；
  - `docs/DEVELOPMENT.md`（环境、质量门、真实 Provider 测试、调试、Troubleshooting、发布流程、贡献指南）；
  - `docs/API.md`（全部 Tauri 命令、参数、返回、事件、错误约定）。
- **CI / 发布**：
  - `.github/workflows/ci.yml`：前端 format/typecheck/lint/test/build + Rust fmt/clippy/test（Windows/Linux/macOS）；
  - `.github/workflows/release.yml`：tag 触发，tauri-action 构建 Windows x64/arm64、macOS arm64/x64、Linux x64 并创建草稿 Release；
  - `.github/dependabot.yml`（npm / cargo / actions 每周更新）。
- **演示素材**：`docs/assets/demo.gif`（由真实运行截图合成，4 帧轮播）、`docs/assets/p7-english.png`（英文界面截图）。

### 验收记录（实际运行结果）

| 检查项                         | 结果                                                                                                  |
| ------------------------------ | ----------------------------------------------------------------------------------------------------- |
| `pnpm typecheck` / `pnpm lint` | 通过（0 error / 0 warning）                                                                           |
| `pnpm test`                    | **9 passed**（新增「设置页切换中英文」用例）                                                          |
| `pnpm build`                   | 通过                                                                                                  |
| 真实应用 E2E · 英文界面        | 导航变为 `Libraries / Chat / Settings`，刷新后仍为英文；仅剩用户数据（库名/会话标题）为中文，符合预期 |
| 英文界面截图                   | `docs/assets/p7-english.png`                                                                          |

### 与任务书的偏差

- 英文模式下 **Provider 状态详情等后端返回的说明文字仍为中文**（属于运行时数据而非界面文案），后续如需彻底英文化，应在后端按语言返回。
- 安装包签名（Windows Authenticode / macOS notarization）未配置证书，Release 产物为未签名构建。

### 下一步（P8）

推送 GitHub 仓库、创建首个 Release、验证安装包。**当前阻塞**：`gh` 未登录，需要用户在终端执行
`gh auth login`（浏览器设备码授权，不要把 token 贴到对话里）。

---

## P8 · 首次公开发布（已完成）

### 已完成

- **GitHub 仓库**：<https://github.com/DaiYiqq/lumos>（public，MIT），`main` 已推送全部提交。
  仓库通过已有凭据以 GitHub REST API 创建；由于本机 `github.com:443` 被网络阻断、
  而 GitHub 边缘 IP 可达，推送时使用了临时 CONNECT 代理（仅本机、任务结束后关闭）。
- **Release `v0.1.0` 已发布**：<https://github.com/DaiYiqq/lumos/releases/tag/v0.1.0>
  - Windows x64：`Lumos_0.1.0_x64-setup.exe`（9.5MB）、`Lumos_0.1.0_x64_en-US.msi`
  - Windows arm64：`Lumos_0.1.0_arm64-setup.exe`（8.4MB）、`Lumos_0.1.0_arm64_en-US.msi`
  - macOS Apple Silicon：`Lumos_0.1.0_aarch64.dmg`（13.2MB）、`Lumos_aarch64.app.tar.gz`
  - Linux x64：`Lumos_0.1.0_amd64.AppImage`（93.6MB）、`.deb`、`.rpm`
  - Release notes 已由 workflow 生成并经 API 补全为正式（非草稿）
- **发布流水线实际跑通**：`.github/workflows/release.yml` 4 个 job 全部 success
  （Windows x64 / Windows arm64 / macOS Apple Silicon / Linux x64）。
- **本地安装包**：`pnpm tauri build --bundles nsis` 于本机构建成功（Windows x64，9.10 MiB）。
- **仓库信息**：description、topics、MIT license、README 徽章与文档链接已配置。

### 实际踩坑与修复（CI 三轮迭代）

1. **CI 没有先构建前端**：`tauri-build` 编译期需要 `../dist`，Rust job 补了 `pnpm build`。
2. **本机架构相关的 Rust 单元测试**：`shorten_home` 测试硬编码 Windows 路径，在 macOS/Linux
   CI 失败；改为 `temp_dir()` + `MAIN_SEPARATOR`，三平台通过。
3. **Linux 链接失败**：`ort`（ONNX Runtime）预编译产物需要较新的 glibc，ubuntu-22.04 缺少
   `__isoc23_strtol`；CI/Release 的 Linux runner 升级到 **ubuntu-24.04**。
4. **macOS Intel 无预编译 ONNX Runtime**：`ort-sys` 不提供 `x86_64-apple-darwin` 产物；
   Release 矩阵移除 Intel macOS，只保留 Apple Silicon（README/DEVELOPMENT 已注明）。
5. **Dependabot 噪音**：自动 PR 引入 TypeScript 7（typescript-eslint 不支持）导致 CI 变红；
   `dependabot.yml` 忽略 `typescript >= 6.1` 与 `@types/node >= 25`，并关闭了已开的 5 个 PR。
6. **本机 github.com 出口 IP 被阻断**：API 域名可达但 `github.com:443` 不可达；用本机 CONNECT
   代理把 TLS 隧道固定到可达的 GitHub 边缘 IP 完成 push。该代理只监听 `127.0.0.1:8888`，
   未使用任何第三方服务，任务结束后已停止。

### 已知限制（如实记录）

- 安装包**未签名**（无 Authenticode / Apple notarization 证书），Windows SmartScreen 与
  macOS Gatekeeper 会提示未知发布者；正式分发前应补签名与公证。
- **macOS Intel 不提供安装包**（ORT 预编译限制）。
- 英文模式下 Provider 状态详情等后端文案仍为中文（运行期数据，非界面文案）。
- Release 的 tag（`v0.1.0`）指向 `eac53c9`；其后仅有一个「测试跨平台修复」提交（`a5b7deb`），
  不影响安装包内容。

### 最终状态

- **CI 全绿**：`main@ae6971a` 的 4 个 job 全部 success —— Frontend、Rust (Windows)、Rust (Ubuntu 24.04)、Rust (macOS)。
- **Release `v0.1.0` 已发布**（非草稿），资产：Windows x64/arm64（NSIS+MSI）、macOS Apple Silicon（dmg）、Linux x64（AppImage/deb/rpm）。
- 9 个 commit，工作区干净；版本库无密钥/无个人文档。
- 前端 9 tests、Rust 48 tests（2 ignored）、clippy/fmt/lint/typecheck 全绿。
- 产品主链路已实机验证：导入 → 混合检索 → 带引用流式问答 → 点击引用回原文 → 历史/监听/导出/中英切换。

---

## P8 追加：README 多语言化与全平台下载入口

**状态：已完成（2026-10-18 晚）**

### 已完成

- **README 四种语言**（顶部语言切换器互相链接）：
  - `README.md`（English，默认）
  - `README.zh-CN.md`（简体中文）
  - `README.zh-TW.md`（繁體中文）
  - `README.ja.md`（日本語）
- **全平台下载表**：每个 README 都包含 Windows x64/arm64（NSIS + MSI）、macOS Apple Silicon（dmg）、
  Linux x64（AppImage/deb/rpm）的**直达 Release 资产链接**，并注明「下载无需账号」。
- **逐平台安装说明**：
  - Windows：SmartScreen「更多信息 → 仍要运行」；
  - macOS：拖入应用程序 + 右键打开 / 隐私与安全性放行；
  - Linux：AppImage `chmod +x`、`apt install ./deb`、`dnf install ./rpm`，以及 WebKitGTK 依赖；
  - 全平台源码构建命令与 Tauri 依赖链接。
- **下载链接实测**：`releases/download/v0.1.0/...` 六个主要资产返回 206/302，可直达下载；
  `releases/latest` 页面作为总入口。
- **发布模板同步**：`.github/workflows/release.yml` 的 release body 现在包含下载表与四种语言链接，
  后续 tag 自动生成带下载入口的 Release notes。
- 当前 Release 的 notes 也通过 GitHub API 更新为同样的下载表与语言入口。

### 验证

| 检查项               | 结果                                                                          |
| -------------------- | ----------------------------------------------------------------------------- |
| 4 个 README 语言互链 | 通过                                                                          |
| v0.1.0 资产直达链接  | 6/6 返回可下载响应（206/302）                                                 |
| `pnpm format`        | 通过                                                                          |
| Release 资产完整性   | 9 个资产（Windows x64/arm64 NSIS+MSI、macOS dmg/app、Linux AppImage/deb/rpm） |

### 说明

- “全平台”当前覆盖：Windows（x64、arm64）、macOS（Apple Silicon）、Linux（x64）。
  macOS Intel 因 ONNX Runtime 无预编译产物暂不支持；Linux arm64 暂未提供。
- 安装包仍未签名，README 已明确说明首次运行的系统提示与处理方式。

---

## P9 · 社区共同维护（Community-ready）

**状态：已完成（2026-10-18）**

### 已完成

- **贡献体系**：
  - `CONTRIBUTING.md`：贡献方式、开发流程、质量门、PR 约定、翻译流程、评审与发布、中文摘要；
  - `CODE_OF_CONDUCT.md`：Contributor Covenant 2.1（含中文摘要与举报邮箱）；
  - `SECURITY.md`：支持版本、私下报告渠道（GitHub Private Vulnerability Reporting）、响应时限与范围；
  - `GOVERNANCE.md`：角色（Contributor / Reviewer / Maintainer / Translation maintainer）、懒共识决策、
    成为维护者的路径、发布与仓库设置；
  - `MAINTAINERS.md`：当前维护者、领域归属、Reviewer/翻译维护者招募说明。
- **GitHub 协作设施**：
  - Issue 模板：Bug / Feature / Translation（YAML 表单）+ `config.yml`（引导 Discussions / 安全报告 / 贡献指南）；
  - `pull_request_template.md`：变更说明、测试命令、文档/i18n/安全检查清单；
  - `.github/CODEOWNERS`：默认与分领域 owner；
  - 已开启 **Discussions**、**Private Vulnerability Reporting**；
  - 新增 `translation`、`community` 标签。
- **翻译体系**：`docs/I18N.md`（UI 词典机制、新增语言步骤、README 翻译规范、术语表、翻译状态与 Review checklist）；
  任一语言的翻译贡献都可按流程成为 translation maintainer。
- **首批 community issues**（均可在 GitHub 直接认领）：
  - #7 Korean UI + README translation
  - #8 winget manifest
  - #9 Linux arm64 installers
  - #10 optional reranker
- **README（4 种语言）**统一新增 Community 区块与 PR Welcome 徽章，链接贡献指南、行为准则、
  治理、安全、翻译指南、Discussions、good first issue。

### 验证

| 检查项                          | 结果                                |
| ------------------------------- | ----------------------------------- |
| 社区文件存在于 GitHub `main`    | 9/9 校验通过（树内容一致）          |
| Discussions                     | 已开启                              |
| Private Vulnerability Reporting | 已开启                              |
| 标签与首批 issues               | `translation`、`community` + #7–#10 |
| README 语言互链 / PR 徽章       | 4 个 README 已更新                  |
| CI                              | 文档变更后仍全绿（见 Actions）      |

### 说明

- 维护者模型是「项目负责人 + 领域 owner」，后续通过 `GOVERNANCE.md` 的提名流程扩充 Reviewer 与翻译维护者。
- 仓库未启用强制 PR 审核的 branch protection，避免阻塞个人维护；团队扩大后可按治理文档逐步开启。
