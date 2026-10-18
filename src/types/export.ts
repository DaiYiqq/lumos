/** 导出结果摘要（与 Rust `ExportSummary` 对应）。 */
export interface ExportSummary {
  /** 导出根目录。 */
  root: string;
  /** 知识库数量。 */
  libraries: number;
  /** 文档数量。 */
  documents: number;
  /** 会话数量。 */
  conversations: number;
}
