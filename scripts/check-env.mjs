#!/usr/bin/env node
/**
 * Lumos 环境自检脚本。
 *
 * 逐项检查构建 / 运行 Lumos 桌面应用所需的全部依赖，输出一张对齐的诊断表，
 * 并为每一项缺失或警告给出可直接复制执行的修复命令。
 *
 * 用法:
 *   node scripts/check-env.mjs            # 人类可读输出
 *   node scripts/check-env.mjs --no-color # 禁用颜色（便于日志存档）
 *
 * 退出码:
 *   0  全部通过（允许存在不影响构建的可选缺失）
 *   1  存在 ❌ 硬性缺失或版本过低
 *
 * 设计约束:
 *   - 只用 Node 内置模块，零 npm 依赖，克隆下来即可运行；
 *   - 兼容 Windows / macOS / Linux，Windows 是主要目标平台；
 *   - 当命令不在当前 PATH 中时，会探测常见安装位置，避免"装了却报缺失"。
 */

import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, statfsSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = path.resolve(__dirname, "..");

const IS_WIN = process.platform === "win32";
const IS_MAC = process.platform === "darwin";
const NO_COLOR =
  process.argv.includes("--no-color") || Boolean(process.env.NO_COLOR) || !process.stdout.isTTY;

/** Tauri 2 官方 MSRV（低版本会直接编译失败）。 */
const MIN_RUST = "1.77.2";
/** Lumos 要求的最低 Node.js 主版本（22 LTS）。 */
const MIN_NODE_MAJOR = 22;

const C = {
  reset: "\u001b[0m",
  dim: "\u001b[2m",
  bold: "\u001b[1m",
  green: "\u001b[32m",
  yellow: "\u001b[33m",
  red: "\u001b[31m",
  cyan: "\u001b[36m",
};

const tty = (code, text) => (NO_COLOR ? text : `${code}${text}${C.reset}`);

const ICON = { ok: "✅", warn: "⚠️ ", fail: "❌", info: "ℹ️ " };

/* ------------------------------------------------------------------ */
/* 进程执行                                                            */
/* ------------------------------------------------------------------ */

/**
 * 运行一条命令并收集输出。失败时不抛异常，由调用方判断状态。
 * @param {string} command 命令（可为绝对路径）
 * @param {string[]} args 参数
 * @returns {{ok: boolean, code: number, stdout: string, stderr: string}}
 */
function run(command, args = []) {
  const quoted = (s) => (/[\s"]/u.test(s) ? `"${s.replaceAll('"', '\\"')}"` : s);
  const options = { encoding: "utf8", timeout: 30_000, windowsHide: true };
  // Windows 下 .cmd/.bat 与裸命令必须经由 shell 才能解析，绝对路径的 .exe 直接执行。
  const viaShell = IS_WIN && (/\.(cmd|bat)$/iu.test(command) || !/[\\/]/u.test(command));
  const result = viaShell
    ? spawnSync(`${quoted(command)} ${args.map(quoted).join(" ")}`, { ...options, shell: true })
    : spawnSync(command, args, options);

  return {
    ok: !result.error && result.status === 0,
    code: result.status ?? -1,
    stdout: (result.stdout ?? "").trim(),
    stderr: (result.stderr ?? "").trim(),
    error: result.error ? String(result.error.message) : "",
  };
}

/** 在 PATH 与常见安装位置中解析可执行文件，返回可用命令或 null。 */
function resolveCommand(name) {
  const probe = run("where", [name]);
  if (probe.ok && probe.stdout) {
    const candidates = probe.stdout
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter(Boolean)
      // `where npm` 会同时返回 npm / npm.cmd / npm.ps1，只有前两者能直接执行。
      .filter((line) => (IS_WIN ? /\.(exe|cmd|bat|com)$/iu.test(line) : true));
    if (candidates.length > 0) return candidates[0];
  }

  const home = homedir();
  const candidates = IS_WIN
    ? [
        path.join(home, ".cargo", "bin", `${name}.exe`),
        path.join(
          process.env.APPDATA ?? path.join(home, "AppData", "Roaming"),
          "npm",
          `${name}.cmd`,
        ),
        `C:\\Program Files\\GitHub CLI\\${name}.exe`,
        `C:\\Program Files\\nodejs\\${name}.exe`,
        path.join(process.env.ProgramFiles ?? "C:\\Program Files", "Git", "bin", `${name}.exe`),
      ]
    : [
        `/usr/local/bin/${name}`,
        path.join(home, ".cargo", "bin", name),
        `/opt/homebrew/bin/${name}`,
      ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

/**
 * 读取某个命令的版本号字符串。
 * @returns {{found: boolean, version: string, raw: string}}
 */
function versionOf(name, args = ["--version"]) {
  const command = resolveCommand(name);
  if (!command) return { found: false, version: "", raw: "" };
  const out = run(command, args);
  const text = out.stdout || out.stderr;
  const version = extractVersion(text);
  // 有些工具即使在异常状态下也会打印一段文字（例如 rustup 未配置默认工具链），
  // 只有真正解析到 x.y 版本号才算"可用"。
  if (!version) return { found: false, version: "", raw: text };
  return { found: true, version, raw: text };
}

/** 从任意版本输出中提取 `x.y.z`（允许两位），提取不到返回空串。 */
function extractVersion(text) {
  const match = text.match(/(\d+)\.(\d+)(?:\.(\d+))?/u);
  return match ? match[0] : "";
}

/** 语义化版本比较，返回 -1 / 0 / 1。 */
function compareVersions(a, b) {
  const parse = (v) => v.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const [x, y] = [parse(a), parse(b)];
  for (let i = 0; i < Math.max(x.length, y.length); i += 1) {
    const delta = (x[i] ?? 0) - (y[i] ?? 0);
    if (delta !== 0) return delta > 0 ? 1 : -1;
  }
  return 0;
}

/* ------------------------------------------------------------------ */
/* 平台专项探测                                                        */
/* ------------------------------------------------------------------ */

/** 找出 Visual Studio 的 MSVC 工具集版本与 link.exe 路径。 */
function detectMsvc() {
  const vsWhere = "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe";
  if (!existsSync(vsWhere)) return null;

  const query = run(vsWhere, [
    "-latest",
    "-products",
    "*",
    "-requires",
    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
    "-property",
    "installationPath",
  ]);
  if (!query.ok || !query.stdout) return null;

  const installPath = query.stdout.split(/\r?\n/u)[0].trim();
  const toolsRoot = path.join(installPath, "VC", "Tools", "MSVC");
  if (!existsSync(toolsRoot)) return null;

  const toolset = readdirSync(toolsRoot).sort((a, b) => compareVersions(b, a))[0];
  if (!toolset) return null;

  const linker = path.join(toolsRoot, toolset, "bin", "Hostx64", "x64", "link.exe");
  return existsSync(linker) ? { installPath, toolset, linker } : null;
}

/** 读取已安装的 Windows SDK 版本（取最高版本）。 */
function detectWindowsSdk() {
  const libRoot = "C:\\Program Files (x86)\\Windows Kits\\10\\Lib";
  if (!existsSync(libRoot)) return null;
  const versions = readdirSync(libRoot)
    .filter((name) => /^\d+\.\d+\.\d+\.\d+$/u.test(name))
    .sort((a, b) => compareVersions(b, a));
  return versions[0] ?? null;
}

/** 读取 WebView2 Runtime 版本，未安装返回 null。 */
function detectWebView2() {
  const reg = run("reg", [
    "query",
    "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    "/v",
    "pv",
  ]);
  if (reg.ok) {
    const match = reg.stdout.match(/pv\s+REG_SZ\s+([\d.]+)/u);
    if (match) return match[1];
  }

  const applicationRoot = "C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application";
  if (!existsSync(applicationRoot)) return null;
  const versions = readdirSync(applicationRoot)
    .filter((name) => /^\d+\.\d+\.\d+\.\d+$/u.test(name))
    .sort((a, b) => compareVersions(b, a));
  return versions[0] ?? null;
}

/** 读取 Defender 状态与排除项；Defender 未运行时返回 {running:false}。 */
function detectDefender() {
  const out = run("powershell", [
    "-NoProfile",
    "-NonInteractive",
    "-Command",
    "$svc = (Get-Service WinDefend -ErrorAction SilentlyContinue).Status; " +
      "$paths = @(); if ($svc -eq 'Running') { $paths = (Get-MpPreference -ErrorAction SilentlyContinue).ExclusionPath }; " +
      '@{ service = "$svc"; paths = $paths } | ConvertTo-Json -Compress',
  ]);
  if (!out.ok || !out.stdout) return { running: false, paths: [] };

  try {
    const parsed = JSON.parse(out.stdout);
    const running = String(parsed.service ?? "").toLowerCase() === "running";
    const paths = Array.isArray(parsed.paths) ? parsed.paths : parsed.paths ? [parsed.paths] : [];
    return { running, paths: paths.map((item) => String(item).trim()).filter(Boolean) };
  } catch {
    return { running: false, paths: [] };
  }
}

/** 检查 git 全局配置项。 */
function gitConfig(key) {
  const out = run("git", ["config", "--global", "--get", key]);
  return out.ok ? out.stdout : "";
}

/** 判断路径是否位于常见云同步目录内。 */
function isCloudSyncedPath(absolutePath) {
  return /(onedrive|dropbox|google\s*drive|googledrive|icloud|nutstore|坚果云|百度网盘|baidunetdisk)/iu.test(
    absolutePath,
  );
}

/** 磁盘剩余空间（GB），无法探测返回 null。 */
function freeDiskGb(targetPath) {
  try {
    const stats = statfsSync(targetPath);
    return (stats.bavail * stats.bsize) / 1024 ** 3;
  } catch {
    return null;
  }
}

/* ------------------------------------------------------------------ */
/* 检查项收集                                                          */
/* ------------------------------------------------------------------ */

const issues = [];

/**
 * 记录一条检查结果。
 * @param {string} label 名称
 * @param {"ok"|"warn"|"fail"|"info"} status 状态
 * @param {string} detail 展示文本
 * @param {string} [fix] 修复命令
 */
function record(label, status, detail, fix = "") {
  rows.push({ label, status, detail });
  if (status === "fail" || status === "warn") issues.push({ label, status, detail, fix });
}

const rows = [];

function checkRust() {
  const rustc = versionOf("rustc");
  if (!rustc.found) {
    record("Rust toolchain", "fail", "未安装", "winget install --id Rustlang.Rustup -e");
    record("Cargo", "fail", "未安装", "随 Rustup 一起安装");
    record("Rust MSVC target", "fail", "未安装", "rustup default stable-x86_64-pc-windows-msvc");
    return;
  }

  const toolchain = run(resolveCommand("rustup") ?? "rustup", ["show", "active-toolchain"]);
  const toolchainName = toolchain.ok ? toolchain.stdout.split(/\r?\n/u)[0].trim() : "";
  const isMsvcHost = !IS_WIN || toolchainName.includes("msvc");
  const latest = compareVersions(rustc.version, MIN_RUST) >= 0;

  record(
    "Rust toolchain",
    latest && isMsvcHost ? "ok" : "fail",
    `${rustc.version}  ${toolchainName || "(unknown toolchain)"}`,
    !latest ? "rustup update stable" : "rustup default stable-x86_64-pc-windows-msvc",
  );
  record("Cargo", "ok", versionOf("cargo").version || "已安装");
  record(
    "Rust MSVC target",
    isMsvcHost ? "ok" : "fail",
    isMsvcHost ? "已使用 *-msvc 工具链" : `当前为 ${toolchainName}`,
    "rustup default stable-x86_64-pc-windows-msvc",
  );
}

function checkWindowsNative() {
  if (!IS_WIN) {
    record("MSVC linker", "info", "非 Windows 平台，跳过");
    record("Windows SDK", "info", "非 Windows 平台，跳过");
    record(
      "WebView2 Runtime",
      "info",
      IS_MAC ? "macOS 使用系统 WebKit" : "Linux 使用系统 WebKitGTK",
    );
    return;
  }

  const onPath = run("where", ["link.exe"]);
  const msvc = detectMsvc();
  if (onPath.ok && onPath.stdout) {
    record("MSVC linker", "ok", onPath.stdout.split(/\r?\n/u)[0].trim());
  } else if (msvc) {
    record("MSVC linker", "ok", `${msvc.toolset}（cargo 自动探测，未加入 PATH）`);
  } else {
    record(
      "MSVC linker",
      "fail",
      "未找到 link.exe",
      'winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"',
    );
  }

  const sdk = detectWindowsSdk();
  record(
    "Windows SDK",
    sdk ? "ok" : "fail",
    sdk ?? "未找到 C:\\Program Files (x86)\\Windows Kits\\10\\Lib",
    'winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"',
  );

  const webview = detectWebView2();
  record(
    "WebView2 Runtime",
    webview ? "ok" : "fail",
    webview ?? "未安装（运行时缺少 WebView2 会白屏）",
    "winget install --id Microsoft.EdgeWebView2Runtime -e",
  );
}

function checkNode() {
  const node = versionOf("node");
  if (!node.found) {
    record("Node.js", "fail", "未安装", "winget install --id OpenJS.NodeJS.LTS -e");
  } else {
    const major = Number.parseInt(node.version.split(".")[0], 10);
    record(
      "Node.js",
      major >= MIN_NODE_MAJOR ? "ok" : "fail",
      `${node.version}（要求 ≥ ${MIN_NODE_MAJOR}）`,
      `winget install --id OpenJS.NodeJS.LTS -e`,
    );
  }

  const npm = versionOf("npm");
  record(
    "npm",
    npm.found ? "ok" : "fail",
    npm.found ? npm.version : "未安装（随 Node.js 一起安装）",
    "winget install --id OpenJS.NodeJS.LTS -e",
  );

  const pnpm = versionOf("pnpm");
  record(
    "pnpm",
    pnpm.found ? "ok" : "fail",
    pnpm.found ? pnpm.version : "未安装",
    "corepack enable pnpm",
  );
}

function checkGit() {
  const git = versionOf("git");
  if (!git.found) {
    record("Git", "fail", "未安装", "winget install --id Git.Git -e");
    record("Git longpaths", "fail", "Git 未安装", "winget install --id Git.Git -e");
  } else {
    const longpaths = gitConfig("core.longpaths").toLowerCase() === "true";
    record("Git", "ok", git.version);
    record(
      "Git longpaths",
      longpaths ? "ok" : "warn",
      longpaths ? "core.longpaths=true" : "未开启（深层 target/ 目录可能超 260 字符）",
      "git config --global core.longpaths true",
    );
  }

  const gh = versionOf("gh");
  const ghAuth = gh.found ? run(resolveCommand("gh") ?? "gh", ["auth", "status"]) : { ok: false };
  record(
    "GitHub CLI",
    gh.found ? "ok" : "fail",
    gh.found ? `${gh.version}${ghAuth.ok ? "，已登录" : "，未登录"}` : "未安装",
    gh.found ? "gh auth login" : "winget install --id GitHub.cli -e",
  );
}

function checkProjectPath() {
  const problems = [];
  const printable = /^[\x20-\x7E]+$/u.test(PROJECT_ROOT);
  if (!printable) problems.push("含非 ASCII 字符");
  if (PROJECT_ROOT.includes(" ")) problems.push("含空格");
  if (isCloudSyncedPath(PROJECT_ROOT)) problems.push("位于云同步目录");
  record(
    "Project path",
    problems.length === 0 ? "ok" : "fail",
    problems.length === 0 ? PROJECT_ROOT : `${PROJECT_ROOT}（${problems.join("、")}）`,
    "改用纯 ASCII、无空格、非云同步的路径，例如 C:\\dev\\lumos",
  );

  const free = freeDiskGb(PROJECT_ROOT);
  record(
    "Disk free",
    free === null ? "info" : free >= 5 ? "ok" : "warn",
    free === null ? "无法探测" : `${free.toFixed(1)} GB 可用（Rust target/ 建议 ≥ 5 GB）`,
    "清理磁盘或更换 PROJECT_DIR",
  );

  if (IS_WIN) {
    const defender = detectDefender();
    if (!defender.running) {
      record("Defender exclusion", "ok", "Defender 未运行，无需排除项");
    } else {
      const cargoDir = path.join(homedir(), ".cargo");
      const rustupDir = path.join(homedir(), ".rustup");
      const missing = [cargoDir, rustupDir, PROJECT_ROOT].filter(
        (target) =>
          !defender.paths.some(
            (entry) => path.resolve(entry).toLowerCase() === path.resolve(target).toLowerCase(),
          ),
      );
      record(
        "Defender exclusion",
        missing.length === 0 ? "ok" : "warn",
        missing.length === 0
          ? "已排除 .cargo / .rustup / 项目目录"
          : `未覆盖 ${missing.length} 个目录（显著拖慢编译）`,
        `Add-MpPreference -ExclusionPath "${cargoDir}","${rustupDir}","${PROJECT_ROOT}"`,
      );
    }
  }
}

function checkOptionalAi() {
  const ollama = versionOf("ollama");
  record(
    "Ollama（可选）",
    "info",
    ollama.found ? `${ollama.version}（本地模型增强）` : "未安装，Ollama Provider 不可用",
  );
}

/* ------------------------------------------------------------------ */
/* 输出                                                                */
/* ------------------------------------------------------------------ */

function printHeader() {
  const title = `Lumos Environment Check (${process.platform} ${process.arch})`;
  const line = "─".repeat(Math.max(title.length, 57));
  console.log(tty(C.bold, title));
  console.log(line);
}

/** 终端显示宽度：CJK 与全角字符按 2 列计算，保证表格对齐。 */
function displayWidth(text) {
  return [...text].reduce(
    (width, char) =>
      width +
      (/[\u1100-\u115F\u2E80-\uA4CF\uAC00-\uD7A3\uF900-\uFAFF\uFE30-\uFE4F\uFF00-\uFF60\uFFE0-\uFFE6]/u.test(
        char,
      )
        ? 2
        : 1),
    0,
  );
}

function printTable() {
  const labelWidth = Math.max(...rows.map((row) => displayWidth(row.label))) + 2;
  for (const row of rows) {
    const icon = ICON[row.status] ?? ICON.info;
    const pad = " ".repeat(labelWidth - displayWidth(row.label));
    const detail = row.status === "ok" ? tty(C.green, row.detail) : row.detail;
    console.log(`  ${row.label}${pad}${icon} ${detail}`);
  }
}

function printSummary() {
  const line = "─".repeat(57);
  console.log(line);
  if (issues.length === 0) {
    console.log(tty(C.green, "✅ 全部通过，可以开始构建 Lumos。"));
    console.log(tty(C.dim, "  下一步：pnpm install && pnpm tauri dev"));
    return;
  }

  const failures = issues.filter((issue) => issue.status === "fail").length;
  const warnings = issues.length - failures;
  const parts = [failures ? `${failures} 项缺失` : "", warnings ? `${warnings} 项警告` : ""].filter(
    Boolean,
  );
  console.log(tty(failures ? C.red : C.yellow, `⚠️  ${parts.join("，")}，按下面命令修复后重跑：`));
  console.log("");

  issues.forEach((issue, index) => {
    console.log(`  ${index + 1}. ${issue.label}：${issue.detail}`);
    console.log(tty(C.cyan, `     → ${issue.fix}`));
  });
  console.log("");
  console.log(`重跑：node scripts/check-env.mjs`);
}

/* ------------------------------------------------------------------ */
/* 主流程                                                              */
/* ------------------------------------------------------------------ */

checkRust();
checkWindowsNative();
checkNode();
checkGit();
checkProjectPath();
checkOptionalAi();

printHeader();
printTable();
printSummary();

process.exit(issues.some((issue) => issue.status === "fail") ? 1 : 0);
