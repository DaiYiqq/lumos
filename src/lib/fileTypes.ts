/**
 * 文件类型白名单：与 Rust 端 `ingest::extract::SUPPORTED_EXTENSIONS` 保持一致。
 *
 * 修改时请两边同时更新，否则文件选择器会允许选到后端不支持的格式。
 */
export const SUPPORTED_EXTENSIONS = [
  "pdf",
  "epub",
  "docx",
  "md",
  "markdown",
  "mdx",
  "txt",
  "html",
  "htm",
  "rst",
  "csv",
  "json",
  "yaml",
  "yml",
  "toml",
  "xml",
  "rs",
  "ts",
  "tsx",
  "js",
  "jsx",
  "py",
  "go",
  "java",
  "kt",
  "c",
  "h",
  "cpp",
  "hpp",
  "cs",
  "rb",
  "php",
  "sh",
  "sql",
  "css",
  "scss",
  "vue",
  "svelte",
  "swift",
  "lua",
] as const;

/** 文件选择器使用的过滤器。 */
export const DOCUMENT_FILTER = {
  name: "文档",
  extensions: [...SUPPORTED_EXTENSIONS],
};
