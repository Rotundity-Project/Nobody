# NoName Memory Retrieval V1

更新时间: 2026-04-14
对应任务: `B2-memory-retrieval-enhancement`

## 目标

为 `NoName` 记忆层提供可复用的结构化检索接口，让后续 `context builder` 和多角色 Agent 都能按相同规则召回记忆。

## 当前查询能力

`NoNameMemoryQuery` 当前支持:

- `keyword`
- `actor`
- `location`
- `goal`
- 兼容旧字段 `search_term`

推荐用法:

- 通用召回: `keyword`
- 人物相关: `actor`
- 场景相关: `location`
- 目标相关: `goal`
- 多维组合: `keyword + actor + location`

## 排序原则

每个 memory section 内部采用统一三段式排序:

1. `relevance`
   - 先看结构化过滤条件是否命中
   - 命中 `actor / location / goal / keyword` 越多，分越高
   - 额外加入少量 role section boost

2. `recency`
   - `episodic` 用 `timestamp`
   - `semantic` / `narrative` 用 `updated_at`
   - `working` 用写入顺序近似最近性

3. `importance`
   - `working` 用 `priority`
   - `episodic` 用 `importance`
   - `semantic` 用 `confidence`
   - `narrative` 用 `note_type` 的启发式权重

## 当前限制

- 还不是向量检索，不适合处理高度模糊语义相似
- 相关性仍是规则式打分，不是学习排序
- `working memory` 缺少显式时间戳，只能用插入顺序近似 recency
- role boost 目前是轻量启发式，后续可继续微调

## 后续建议

如果继续往下做，优先顺序建议是:

1. 为 query 增加更多显式标签入口
2. 在 `context builder` 按角色精细化组装查询
3. 视需要再接更重的外部知识或高级检索后端
