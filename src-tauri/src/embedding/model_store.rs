//! 嵌入模型文件的下载与缓存。
//!
//! 不走 HuggingFace 的 hf-hub（它对镜像站的 range 响应很挑剔，国内网络容易失败），
//! 而是直接用阻塞 HTTP 下载 5 个模型文件并缓存到应用数据目录：
//!   - `onnx/model.onnx`（约 95MB）
//!   - `tokenizer.json`
//!   - `config.json`
//!   - `special_tokens_map.json`
//!   - `tokenizer_config.json`
//!
//! 下载顺序：`HF_ENDPOINT`（若设置）→ 官方 huggingface.co → hf-mirror.com 镜像。
//! 任何一个站点成功即返回；全部失败时返回面向用户的中文错误。

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{AppError, AppResult};

/// 模型在 Hub 上的仓库名。
const MODEL_REPO: &str = "Xenova/bge-small-zh-v1.5";

/// 需要下载的文件。
const MODEL_FILES: &[&str] = &[
    "onnx/model.onnx",
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

/// 模型文件的最小合理体积（用于判断缓存是否完整）。
fn min_file_size(file: &str) -> u64 {
    if file == "onnx/model.onnx" {
        40 * 1024 * 1024
    } else {
        64
    }
}

/// 下载完成后的本地模型文件集合。
pub struct ModelFiles {
    /// ONNX 模型文件路径。
    pub onnx: PathBuf,
    /// tokenizer.json 路径。
    pub tokenizer: PathBuf,
    /// config.json 路径。
    pub config: PathBuf,
    /// special_tokens_map.json 路径。
    pub special_tokens_map: PathBuf,
    /// tokenizer_config.json 路径。
    pub tokenizer_config: PathBuf,
}

/// 确保模型文件存在于 `cache_dir/bge-small-zh-v1.5/` 下，返回本地路径。
///
/// # Errors
///
/// 所有候选站点都下载失败或缓存目录不可写时返回面向用户的错误。
pub fn ensure_model_files(cache_dir: &Path) -> AppResult<ModelFiles> {
    let model_dir = cache_dir.join("bge-small-zh-v1.5");
    std::fs::create_dir_all(&model_dir)?;

    if model_dir_is_complete(&model_dir) {
        return Ok(model_files(&model_dir));
    }

    // 复用 hf-hub 之前下载到一半的模型主体，避免重复下载 95MB
    if !model_dir.join("onnx/model.onnx").is_file()
        && let Some(blob) = find_existing_model_blob(cache_dir)
    {
        let target = model_dir.join("onnx/model.onnx");
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::copy(&blob, &target);
    }

    let bases = candidate_bases();
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|error| AppError::Internal(format!("HTTP 客户端初始化失败：{error}")))?;

    let mut last_error = String::new();
    for base in &bases {
        tracing::info!(base = %base, "正在下载本地嵌入模型（首次约 100MB）");
        match download_from_base(&client, base, &model_dir) {
            Ok(()) => return Ok(model_files(&model_dir)),
            Err(error) => last_error = error.to_string(),
        }
    }

    Err(AppError::InvalidInput(format!(
        "本地嵌入模型下载失败（已尝试官方与镜像站点）：{last_error}。\
         可设置环境变量 HF_ENDPOINT=https://hf-mirror.com 后重启 Lumos 重试；\
         在模型下载完成前，检索会自动使用离线关键词模式"
    )))
}

/// 候选下载站点（去重、按优先级排序）。
fn candidate_bases() -> Vec<String> {
    let mut bases = Vec::new();
    if let Ok(value) = std::env::var("HF_ENDPOINT") {
        let trimmed = value.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            bases.push(trimmed.to_string());
        }
    }
    for fallback in ["https://huggingface.co", "https://hf-mirror.com"] {
        if !bases.iter().any(|base| base == fallback) {
            bases.push(fallback.to_string());
        }
    }
    bases
}

/// 从单个站点下载缺失文件。
fn download_from_base(
    client: &reqwest::blocking::Client,
    base: &str,
    model_dir: &Path,
) -> AppResult<()> {
    for file in MODEL_FILES {
        let target = model_dir.join(file);
        if is_valid_file(&target, min_file_size(file)) {
            continue;
        }
        let url = format!("{base}/{MODEL_REPO}/resolve/main/{file}");
        let response = client
            .get(&url)
            .send()
            .map_err(|error| AppError::InvalidInput(format!("下载 {file} 失败：{error}")))?;
        if !response.status().is_success() {
            return Err(AppError::InvalidInput(format!(
                "下载 {file} 失败：HTTP {}",
                response.status()
            )));
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temp = target.with_extension("part");
        let mut reader = response;
        let mut writer = std::fs::File::create(&temp)?;
        std::io::copy(&mut reader, &mut writer)?;
        std::fs::rename(&temp, &target)?;
    }
    Ok(())
}

/// 缓存是否完整。
fn model_dir_is_complete(model_dir: &Path) -> bool {
    MODEL_FILES
        .iter()
        .all(|file| is_valid_file(&model_dir.join(file), min_file_size(file)))
}

/// 组装本地路径。
fn model_files(model_dir: &Path) -> ModelFiles {
    ModelFiles {
        onnx: model_dir.join("onnx/model.onnx"),
        tokenizer: model_dir.join("tokenizer.json"),
        config: model_dir.join("config.json"),
        special_tokens_map: model_dir.join("special_tokens_map.json"),
        tokenizer_config: model_dir.join("tokenizer_config.json"),
    }
}

/// 文件存在且体积达到下限。
fn is_valid_file(path: &Path, min_size: u64) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() >= min_size)
        .unwrap_or(false)
}

/// 在 hf-hub 缓存结构里寻找已下载的模型主体。
fn find_existing_model_blob(cache_dir: &Path) -> Option<PathBuf> {
    let hub_cache = cache_dir.join("models--Xenova--bge-small-zh-v1.5/blobs");
    std::fs::read_dir(hub_cache)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .max_by_key(|path| path.metadata().map(|metadata| metadata.len()).unwrap_or(0))
        .filter(|path| is_valid_file(path, 40 * 1024 * 1024))
}
