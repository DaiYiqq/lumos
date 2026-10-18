//! 检索层单元测试与「单一 vs 混合检索」效果评估。
//!
//! 效果评估需要本地嵌入模型（约 100MB），默认跳过；手动运行：
//! `LUMOS_EMBEDDING_EVAL=1 cargo test hybrid_beats_single_retrievers -- --nocapture`

use super::*;

#[test]
fn rrf_prefers_documents_hit_by_both_retrievers() {
    let fused = rrf_fuse(&[1, 2, 3], &[3, 2, 4], 10);
    let ids: Vec<i64> = fused.iter().map(|(id, _)| *id).collect();

    assert_eq!(ids.len(), 4);
    assert_eq!(&ids[..2], &[3, 2]);
    assert!(fused[0].1 > fused[1].1);
    assert!(fused[1].1 > fused[2].1);
}

#[test]
fn rrf_limits_and_handles_empty_inputs() {
    assert!(rrf_fuse(&[], &[], 5).is_empty());

    let fused = rrf_fuse(&[1, 2, 3, 4, 5], &[], 3);
    assert_eq!(fused.len(), 3);
    assert_eq!(fused[0].0, 1);
}

#[test]
fn parses_search_modes() {
    assert_eq!(SearchMode::parse(None), SearchMode::Hybrid);
    assert_eq!(SearchMode::parse(Some("VECTOR")), SearchMode::Vector);
    assert_eq!(SearchMode::parse(Some("keyword")), SearchMode::Keyword);
    assert_eq!(SearchMode::parse(Some("unknown")), SearchMode::Hybrid);
}

/// 混合检索效果评估：构造「关键词型」与「语义改写型」两类查询，
/// 对比 keyword / vector / hybrid 三种模式的 recall@3。
#[test]
fn hybrid_beats_single_retrievers() {
    if std::env::var("LUMOS_EMBEDDING_EVAL").is_err() {
        return;
    }

    use crate::db::repo::libraries;
    use crate::embedding::EmbeddingService;
    use crate::ingest::pipeline;

    let dir = std::env::temp_dir().join(format!("lumos-eval-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("临时目录应可创建");
    let models = std::env::temp_dir().join("lumos-eval-models");

    // 1. 语料：8 篇带唯一编号的「关键词型」文档 + 8 篇需要语义理解的文档
    let keyword_docs: Vec<(String, String)> = (0..8)
        .map(|index| {
            (
                format!("kw-{index}.md"),
                format!(
                    "# 工单记录\n\n工单编号 WX-7351-AB{index} 的处理结果：设备已完成检修，等待复检。"
                ),
            )
        })
        .collect();

    let semantic_docs = vec![
        (
            "离职原因.md",
            "上一年夏天我最终决定辞职。真正的原因不是待遇，而是无休止的加班让我完全失去了写作的时间。",
        ),
        (
            "合同条款.md",
            "第九条约定：任何一方反悔，需要支付的金额不得超过合同总额的百分之二十。",
        ),
        (
            "医学影像论文.md",
            "本文在编码器输出上构造实例级对比损失，从而降低对人工标注数据的依赖。",
        ),
        (
            "本地优先.md",
            "软件的索引与聊天记录都保存在用户自己的磁盘上，断网时功能不受影响。",
        ),
        (
            "混合检索.md",
            "把稠密向量与稀疏关键词两路结果按名次融合，可以同时兼顾语义相似与术语精确匹配。",
        ),
        (
            "失眠改善.md",
            "睡前一小时远离屏幕、固定起床时间，是我改善睡眠质量最有效的两个习惯。",
        ),
        (
            "咖啡因笔记.md",
            "下午三点之后摄入咖啡因会明显推迟入睡时间，深度睡眠时长也会下降。",
        ),
        (
            "徒步装备.md",
            "长途徒步需要准备防水冲锋衣、登山杖、头灯与足够两天的饮用水。",
        ),
    ];

    let mut expected: Vec<(String, String)> = Vec::new();
    let mut files = Vec::new();
    for (index, (name, content)) in keyword_docs.iter().enumerate() {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("写入 fixture 应成功");
        expected.push((format!("WX-7351-AB{index}"), name.clone()));
        files.push(path);
    }
    for (name, content) in &semantic_docs {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("写入 fixture 应成功");
        files.push(path);
    }
    // 干扰文档
    for index in 0..8 {
        let path = dir.join(format!("noise-{index}.md"));
        std::fs::write(
            &path,
            format!(
                "# 杂项记录 {index}\n\n这里是一些与所有查询都无关的日常记录，仅用于增加候选数量。"
            ),
        )
        .expect("写入 fixture 应成功");
        files.push(path);
    }

    let semantic_queries = [
        ("去年夏天为什么放弃原来的岗位", "离职原因.md"),
        ("对方反悔时最多能要求多少补偿", "合同条款.md"),
        ("有哪些研究降低了对标注数据的依赖", "医学影像论文.md"),
        ("没有网络还能正常使用吗", "本地优先.md"),
        ("如何把两路搜索结果合并排序", "混合检索.md"),
        ("怎样让自己睡得更好", "失眠改善.md"),
        ("下午喝咖啡会影响休息吗", "咖啡因笔记.md"),
        ("去远足要准备哪些东西", "徒步装备.md"),
    ];
    let mut queries: Vec<(String, String)> = expected
        .iter()
        .map(|(code, name)| (code.clone(), name.clone()))
        .collect();
    for (query, name) in semantic_queries {
        queries.push((query.to_string(), name.to_string()));
    }

    // 2. 入库
    let database = crate::db::Database::open_in_memory().expect("内存库应可创建");
    database.migrate().expect("迁移应成功");
    let library = database
        .with_connection(|connection| libraries::create_library(connection, "评估库"))
        .expect("建库应成功");
    for path in &files {
        pipeline::ingest_file(&database, &library.id, path, || false, None).expect("索引应成功");
    }

    // 3. 等待嵌入模型就绪并补写向量
    let embedder = EmbeddingService::new(models);
    embedder.warmup();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(900);
    loop {
        let status = embedder.status();
        if status.state == "ready" {
            break;
        }
        if status.state == "failed" || std::time::Instant::now() > deadline {
            panic!("嵌入模型加载失败：{status:?}");
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    while backfill(&database, &embedder, 64).expect("补写向量应成功") > 0 {}

    // 4. 三种模式各跑一遍 recall@3
    let recall = |mode: SearchMode| -> f64 {
        let mut hits = 0usize;
        for (query, expected_name) in &queries {
            let response =
                search(&database, &embedder, &library.id, query, mode, 3).expect("检索应成功");
            let found = response
                .hits
                .iter()
                .any(|hit| hit.document_path.ends_with(expected_name));
            if found {
                hits += 1;
            }
        }
        hits as f64 / queries.len() as f64
    };

    let keyword_recall = recall(SearchMode::Keyword);
    let vector_recall = recall(SearchMode::Vector);
    let hybrid_recall = recall(SearchMode::Hybrid);

    println!(
        "\n=== 检索效果对比（{} 条查询，recall@3）===",
        queries.len()
    );
    println!("keyword : {keyword_recall:.3}");
    println!("vector  : {vector_recall:.3}");
    println!("hybrid  : {hybrid_recall:.3}\n");

    assert!(
        hybrid_recall >= keyword_recall,
        "混合检索不应弱于纯关键词：hybrid={hybrid_recall:.3} keyword={keyword_recall:.3}"
    );
    assert!(
        hybrid_recall >= vector_recall,
        "混合检索不应弱于纯向量：hybrid={hybrid_recall:.3} vector={vector_recall:.3}"
    );
    assert!(
        hybrid_recall >= 0.8,
        "混合检索 recall@3 过低：{hybrid_recall:.3}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
