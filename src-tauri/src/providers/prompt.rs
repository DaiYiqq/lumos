//! RAG Prompt 组装：强制模型只依据检索到的资料回答，并标注可溯源的引用编号。

use std::fmt::Write as _;

use crate::providers::types::ChatMessage;
use crate::retrieval::SearchHit;

/// 系统提示词。
#[must_use]
pub fn system_prompt() -> String {
    concat!(
        "你是 Lumos，一个严格基于用户本地文档回答问题的助手。\n\n",
        "必须遵守以下规则：\n",
        "1. 只能使用「参考资料」中的内容回答，不得使用你自己的常识或编造信息。\n",
        "2. 每个结论后必须用 [编号] 标注来源，编号必须来自参考资料，不得编造编号。\n",
        "3. 如果参考资料不足以回答问题，直接回答「未在文档中找到相关内容」，并简要说明缺少什么。\n",
        "4. 使用用户提问的语言回答，保持简洁、分点清晰。\n",
        "5. 不要输出参考资料以外的引用编号，也不要假装引用了不存在的文档。"
    )
    .to_string()
}

/// 把检索命中拼成带编号的参考资料。
#[must_use]
pub fn build_context(hits: &[SearchHit]) -> String {
    let mut context = String::new();
    for (index, hit) in hits.iter().enumerate() {
        let location = describe_location(hit);
        let _ = write!(
            context,
            "[{}] 《{}》{}\n{}\n\n",
            index + 1,
            hit.document_title,
            location,
            hit.content.trim()
        );
    }
    context
}

/// 描述引用的位置（页码 / 章节 / 路径）。
fn describe_location(hit: &SearchHit) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(page) = hit.page_no {
        parts.push(format!("第 {page} 页"));
    }
    if let Some(section) = &hit.section_path
        && !section.trim().is_empty()
    {
        parts.push(section.clone());
    }
    parts.push(hit.document_path.clone());
    format!("（{}）", parts.join(" · "))
}

/// 组装最终发给模型的用户消息。
#[must_use]
pub fn build_user_prompt(context: &str, question: &str) -> String {
    format!(
        "参考资料：\n\n{context}用户问题：{question}\n\n\
         请只依据以上参考资料回答；每个结论用 [编号] 标注来源；\
         如果资料不足，请回答「未在文档中找到相关内容」。"
    )
}

/// 组装完整的消息列表（system + 历史 + 当前问题）。
#[must_use]
pub fn build_messages(
    context: &str,
    history: Option<&[ChatMessage]>,
    question: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::system(system_prompt())];

    if let Some(history) = history {
        // 只保留最近 8 条，避免上下文过长
        let start = history.len().saturating_sub(8);
        messages.extend(history[start..].iter().cloned());
    }

    messages.push(ChatMessage::user(build_user_prompt(context, question)));
    messages
}

/// 无模型时的离线回答：直接展示检索到的原文片段。
#[must_use]
pub fn offline_answer(hits: &[SearchHit]) -> String {
    if hits.is_empty() {
        return "当前为离线关键词模式（未配置 AI 模型）。没有找到与问题相关的原文片段。"
            .to_string();
    }

    let mut answer =
        String::from("当前为离线关键词模式（未配置 AI 模型），以下是检索到的最相关原文片段：\n\n");
    for (index, hit) in hits.iter().enumerate() {
        let location = describe_location(hit);
        let _ = writeln!(
            answer,
            "[{}] 《{}》{}\n{}\n",
            index + 1,
            hit.document_title,
            location,
            hit.content.trim()
        );
    }
    answer.push_str("配置 DeepSeek API Key 或启动 Ollama 后，可以获得带归纳的完整回答。");
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(index: usize) -> SearchHit {
        SearchHit {
            chunk_id: index as i64,
            document_id: format!("doc-{index}"),
            document_title: format!("文档 {index}"),
            document_path: format!("/tmp/doc-{index}.md"),
            page_no: Some(3),
            section_path: Some("第三章".to_string()),
            content: "违约金上限为合同总额的百分之二十。".to_string(),
            score: 0.5,
            vector_rank: Some(1),
            keyword_rank: Some(2),
        }
    }

    #[test]
    fn context_contains_numbered_citations() {
        let context = build_context(&[hit(0), hit(1)]);
        assert!(context.contains("[1] 《文档 0》"));
        assert!(context.contains("[2] 《文档 1》"));
        assert!(context.contains("第 3 页"));
        assert!(context.contains("第三章"));
    }

    #[test]
    fn prompt_instructs_grounded_answers() {
        let prompt = build_user_prompt("资料", "问题");
        assert!(prompt.contains("未在文档中找到相关内容"));
        assert!(prompt.contains("参考资料"));
    }

    #[test]
    fn messages_include_system_history_and_question() {
        let history = vec![
            ChatMessage::user("上一个问题"),
            ChatMessage::assistant("上一个回答"),
        ];
        let messages = build_messages("资料", Some(&history), "新问题");
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, crate::providers::types::Role::System);
        assert_eq!(messages[3].role, crate::providers::types::Role::User);
        assert!(messages[3].content.contains("新问题"));
    }

    #[test]
    fn offline_answer_lists_snippets() {
        let answer = offline_answer(&[hit(0)]);
        assert!(answer.contains("离线关键词模式"));
        assert!(answer.contains("[1] 《文档 0》"));
    }
}
