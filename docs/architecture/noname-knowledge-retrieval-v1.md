# NoName Knowledge Retrieval V1

## 2026-04-16 Retrieval Report

B5 now also exposes a structured retrieval report for debugging and future adapter evaluation:

- `NoNameKnowledgeQuery.sources` filters snippets by source after provider retrieval.
- `NoNameKnowledgeQuery.minScore` drops weak snippets before final ranking/limit.
- `retrieve_with_report(...)` returns snippets plus provider-level counts.
- `NoNameKnowledgeProviderReport` records returned and accepted counts per provider.
- `NoNameKnowledgeRetrievalReport` records total returned count, accepted-before-limit count, requested limit, provider count, and final snippets.
- Retrieval service deduplicates by `source + documentId`, keeps the highest-scoring snippet, and reports `duplicateDroppedCount`.
- `droppedByLimitCount` records how many deduplicated snippets were removed by final `limit`.

This keeps observability inside the retrieval service instead of scattering provider metrics through future runtime code.

## 2026-04-16 Graph Lore Provider

B5 now includes a lightweight graph retrieval prototype:

- `GraphKnowledgeProvider`
- Input: `NoNameKnowledgeGraphNode` and `NoNameKnowledgeGraphEdge`
- Retrieval: node title/body/tag matches plus relation and neighbor boosts
- Output: the same `NoNameKnowledgeSnippet` shape used by other providers

This does not introduce a vector database or external graph database. It keeps B5 isolated while giving future lore/wiki adapters a concrete provider boundary.

更新时间: 2026-04-14
对应任务: `B5-advanced-knowledge-retrieval`

## 目标

先提供一个轻量、可替换的知识检索后端原型，为未来 lore / wiki / 外部设定集接入留出统一接口。

## 当前原型

当前提供两层:

- `NoNameKnowledgeProvider`
  - 统一 provider 接口
  - 负责按 query 返回 snippet

- `NoNameKnowledgeRetrievalService`
  - 管理 provider
  - 聚合多 provider 检索结果
  - 统一排序和 limit 截断

## 当前实现

当前已落地两个最小 provider:

- `InMemoryKnowledgeProvider`
  - 适合本地 demo / 测试 / lore 原型
  - 不依赖向量库或外部服务

- `GraphKnowledgeProvider`
  - 适合轻量图谱 lore / wiki 原型
  - 通过节点、边和邻接关系提供关系增强召回

## 查询能力

`NoNameKnowledgeQuery` 当前支持:

- `keyword`
- `tags`
- `limit`

## 当前限制

- 不是向量检索
- 已有轻量图检索原型，但还不是外部图数据库
- 没有接入 runtime 主链
- 主要价值在于接口和 provider 边界已经稳定下来

## 后续建议

后面如果继续推进，建议顺序是:

1. 新增 `vector provider`
2. 视需要补 `graph provider`
3. 再评估如何与 `memory/context` 系统拼接成混合检索
