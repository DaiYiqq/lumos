//! 应用级命令：版本、数据目录等元信息。

use serde::Serialize;
use tauri::{AppHandle, Manager};

/// 应用元信息，供前端状态栏与“关于”页面展示。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// 产品名，例如 `Lumos`。
    pub name: String,
    /// 语义化版本号，例如 `0.1.0`。
    pub version: String,
    /// 应用数据目录（数据库、日志、设置都存放于此）。
    pub data_dir: String,
    /// 数据目录的展示形式：家目录前缀替换为 `~`，用于界面与截图。
    pub data_dir_display: String,
    /// 运行平台标识，例如 `windows`。
    pub platform: String,
}

/// 读取应用元信息。
///
/// # Errors
///
/// 当系统无法解析应用数据目录时，返回人类可读的中文错误信息，
/// 由前端直接展示，不暴露底层错误码。
#[tauri::command]
pub fn app_info(app: AppHandle) -> Result<AppInfo, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;

    let home_dir = app.path().home_dir().ok();
    let display = shorten_home(&data_dir, home_dir.as_deref());

    Ok(AppInfo {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        data_dir: data_dir.display().to_string(),
        data_dir_display: display,
        platform: std::env::consts::OS.to_string(),
    })
}

/// 将路径中的家目录前缀替换为 `~`，避免界面上暴露用户名。
fn shorten_home(path: &std::path::Path, home: Option<&std::path::Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) => {
            let rest = rest.display().to_string();
            if rest.is_empty() {
                "~".to_string()
            } else {
                format!("~{}{rest}", std::path::MAIN_SEPARATOR)
            }
        }
        None => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::shorten_home;
    use std::path::Path;

    #[test]
    fn replaces_home_prefix_with_tilde() {
        let path = Path::new(r"C:\Users\demo\AppData\Roaming\com.daiyi.lumos");
        let home = Path::new(r"C:\Users\demo");
        assert_eq!(
            shorten_home(path, Some(home)),
            r"~\AppData\Roaming\com.daiyi.lumos"
        );
    }

    #[test]
    fn keeps_path_when_home_is_unknown() {
        let path = Path::new(r"D:\data\lumos");
        assert_eq!(shorten_home(path, None), r"D:\data\lumos");
    }
}
