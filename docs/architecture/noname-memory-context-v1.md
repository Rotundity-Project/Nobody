# NoName Agent 记忆工程与上下文工程设计文档 V1

更新时间: 2026-04-07
状态: 设计基线
适用项目: `Nobody`
关联文档:
- `docs/architecture/noname-agent-v1.md`
- `docs/architecture/domain-model-v2.md`
- `docs/ARCHITECTURE.md`

## 1. 文档目标

本文档专门回答 `NoName Agent` 的两个核心问题：

- 记忆工程：Agent 的记忆如何分层、写入、检索、压缩、持久化。
- 上下文工程：Agent 在每一回合如何构建上下文，如何控制 token，如何兼顾一致性、相关性与长期连续性。

本文档参考了 HelloAgents 的“记忆与检索”和“上下文工程”思路，但设计目标是 `Nobody` 的游戏叙事场景，而不是通用问答场景。

## 2. 现状与差距

`Nobody` 当前已经有记忆和上下文的基础能力，但还处于“轻量版”。

### 2.1 当前已有基础

#### 记忆基础

- `src-tauri/src/memory_layers.rs`
  - `recent_events`
  - `chapter_summaries`
  - `world_facts`

#### 上下文基础

- `src-tauri/src/context_builder.rs`
  - `ContextBuildInput`
  - `ContextBundle`
  - 对 `world_facts / recent_events / chapter_summaries / recent_context / referenced_entities` 的拼装与预算裁剪

#### 世界知识基础

- `src-tauri/src/world_registry.rs`
  - `characters`
  - `map_nodes`
  - `map_edges`
  - `techniques`
  - `factions`
  - `story_state`
  - `world_facts`

### 2.2 当前差距

和真正的 Agent 运行需求相比，当前系统仍缺：

- 没有统一的记忆管理器
- 没有明确的记忆写入策略
- 没有区分工作记忆、情景记忆、语义记忆、叙事记忆
- 没有结构化笔记系统
- 没有长时程上下文压缩机制
- 没有为不同角色 Agent 构建差异化上下文
- 没有可追踪的上下文流水线

## 3. NoName Agent 的记忆工程目标

NoName Agent 的记忆工程要解决的不是“存更多文本”，而是以下 6 个问题：

1. 让 Agent 能记住近期发生了什么
2. 让 Agent 能记住章节级和世界级长期事实
3. 让 Agent 能在长时程任务中维持目标与未解决线索
4. 让记忆可检索、可压缩、可更新、可删除
5. 让记忆和现有 `GameState / PlotState / WorldRegistry` 协同工作
6. 让记忆系统适合 Tauri 桌面项目，不强依赖重量级外部服务

## 4. NoName Memory 总体架构

建议采用“4 层记忆 + 1 层检索 + 1 层压缩”的设计。

### 4.1 架构总览

建议的逻辑结构：

- `NoNameMemoryManager`
  - 统一写入、检索、压缩、清理、持久化调度
- `WorkingMemory`
  - 回合级、短 TTL、进程内快速访问
- `EpisodicMemory`
  - 时间序列事件、章节推进记录、关键遭遇
- `SemanticMemory`
  - 世界事实、实体关系、长期知识图谱
- `NarrativeMemory`
  - 章节目标、悬而未决线索、当前故事张力、角色弧线
- `MemoryRetrievalPipeline`
  - 基于任务和角色做检索、排序、去重、组装
- `MemoryCompactionPipeline`
  - 对长历史进行章节摘要、结构化笔记、压缩整合

## 5. 为什么采用这种分层

HelloAgents 里的启发是对的：把工作记忆、情景记忆、语义记忆分开，能明显降低“所有信息都塞进一个上下文窗口”的混乱度。

但 `Nobody` 还需要额外加一层 `NarrativeMemory`，因为它不是通用问答产品，而是强剧情产品。游戏叙事里有些信息并不适合被当成普通语义事实，例如：

- 本章的冲突核心是什么
- 哪条伏线还没有回收
- 玩家当前最强的短期动机是什么
- 这一章应该继续升级冲突，还是该收束

这些更像“叙事工作内存 + 结构化笔记”，所以单靠 `world_facts` 不够。

## 6. 记忆层设计

### 6.1 WorkingMemory

定义：

- 保存最近 3 到 8 个回合最重要的运行期信息
- 进程内优先
- 短 TTL
- 高优先级、低容量

适合存放：

- 最近几段 narrative segment
- 最近一次玩家输入
- 最近一次系统选项
- 最近一次工具调用摘要
- 最近一次 Agent plan
- 最近一次 guardrail reject reason

典型字段：

```ts
type WorkingMemoryItem = {
  memoryId: string
  turnId: string
  source: string
  category: "recent_turn" | "tool_trace" | "guardrail" | "dialogue"
  summary: string
  expiresAt?: number
  priority: number
}
```

建议策略：

- 只保存高相关摘要，不保存完整长文本
- 对超过 TTL 的数据自动淘汰
- 作为上下文构建时的最高优先级来源

### 6.2 EpisodicMemory

定义：

- 保存有时间顺序的事件序列
- 强调“发生过什么”
- 用于维持剧情连贯、回溯因果

适合存放：

- 战斗结果事件
- 行旅与遭遇事件
- 关系变化事件
- 突破、受伤、反噬、声望变化
- 章节结束事件

典型字段：

```ts
type EpisodicMemoryItem = {
  memoryId: string
  eventType: string
  timestamp: number
  chapterIndex: number
  locationId?: string
  actors: string[]
  summary: string
  detailRef?: string
  importance: "low" | "medium" | "high"
}
```

建议策略：

- 详细事件保留在存储层
- 上下文阶段只召回摘要
- 支持按 `chapter / actor / location / eventType` 查询

### 6.3 SemanticMemory

定义：

- 保存跨回合稳定成立的事实与关系
- 这是 Agent 最重要的“硬约束知识层”之一

适合存放：

- 玩家在某地
- 某角色属于某宗门
- 某功法与某灵根更契合
- 某地点风险更高
- 某 NPC 对玩家存在仇恨或好感

典型字段：

```ts
type SemanticMemoryItem = {
  factId: string
  subject: string
  predicate: string
  object: string
  confidence: number
  source: string
  updatedAt: number
  tags: string[]
}
```

建议策略：

- 与 `WorldRegistry.tables.world_facts` 对齐
- 与 `EntityStore` 配合使用
- 高置信事实优先进入上下文的 Hard Facts 区

### 6.4 NarrativeMemory

定义：

- 保存“叙事推进层”的结构化笔记
- 强调故事目标、冲突、伏线、未决事项

适合存放：

- 当前章目标
- 当前章主要冲突
- 已建立但未回收的伏线
- 角色当前内在动机
- 下一个推荐推进方向

典型字段：

```ts
type NarrativeMemoryItem = {
  noteId: string
  chapterIndex: number
  arcId?: string
  noteType: "goal" | "conflict" | "foreshadowing" | "unresolved_thread" | "character_arc"
  title: string
  summary: string
  status: "active" | "resolved" | "archived"
  relatedEntities: string[]
  updatedAt: number
}
```

建议策略：

- 每个章节结束时强制整理一次
- 长文本不直接入上下文，只入结构化摘要
- 作为 DirectorAgent 的重要输入来源

## 7. 存储后端设计

NoName Agent 不建议在 V1 就强依赖 Neo4j/Qdrant 这类外部服务。

### 7.1 V1 推荐后端

- `SQLite`
- `JSON serialized payload`
- `FTS` 或轻量全文检索
- 进程内缓存

原因：

- `Nobody` 是 Tauri 桌面项目
- 用户环境以本地单机为主
- V1 重点是把系统跑稳，而不是追求最强检索能力

### 7.2 V1 存储映射建议

- `working_memory`：内存 + 可选短期 SQLite cache
- `episodic_memory`：SQLite 表
- `semantic_memory`：优先复用 `world_registry + entity_store`，必要时补 SQLite 投影表
- `narrative_memory`：SQLite 表
- `notes`：SQLite 表

### 7.3 V2 再考虑的能力

- 向量检索
- 图查询
- 外部 lore / wiki / 设定集 RAG
- 多模态记忆

## 8. 推荐模块命名

建议新增：

- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_memory_store.rs`
- `src-tauri/src/noname_memory_compaction.rs`
- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_note_store.rs`

复用：

- `src-tauri/src/memory_layers.rs`
- `src-tauri/src/context_builder.rs`
- `src-tauri/src/world_registry.rs`
- `src-tauri/src/entity_store.rs`
- `src-tauri/src/event_log.rs`

## 9. 写入策略

记忆工程最关键的不是“存在哪里”，而是“什么时候写、写什么”。

### 9.1 回合后写入

在每次 `execute_player_action` 后：

- 写 `WorkingMemory`
- 写高价值 `EpisodicMemory`
- 必要时更新 `SemanticMemory`
- 必要时追加 `NarrativeMemory` 的活跃线索

### 9.2 章节结束时写入

在章节切换时：

- 生成章节摘要
- 压缩本章 episodic history
- 更新 unresolved threads
- 标记已解决线索
- 把本章关键事实固化到 semantic memory

### 9.3 世界状态变更时写入

当 `travel / combat / breakthrough / social change / registry patch` 发生时：

- 优先更新 semantic memory
- 同步更新 narrative memory 中相关线索

## 10. 检索策略

记忆系统不应该“全量取回”，而应当按用途召回。

### 10.1 检索维度

建议至少支持：

- `by_turn`
- `by_chapter`
- `by_actor`
- `by_location`
- `by_event_type`
- `by_goal`
- `by_keyword`

### 10.2 检索排序因子

建议采用加权排序：

- `relevance` 相关性
- `recency` 新近性
- `importance` 重要度
- `authority` 来源可信度
- `narrative_priority` 叙事优先级

一个简单评分函数可以是：

```text
score = 0.35 * relevance
      + 0.20 * recency
      + 0.20 * importance
      + 0.15 * authority
      + 0.10 * narrative_priority
```

## 11. NoName Context Engineering 目标

上下文工程要解决的问题是：

- 哪些信息要放进本轮 prompt
- 哪些信息不能放进去
- 如何让 Agent 在有限 token 内拿到最需要的信息
- 如何让不同角色 Agent 拿到不同的上下文
- 如何在长时程任务中维持连续性而不被噪声淹没

## 12. NoName Context Builder

建议引入 `NoNameContextBuilder`，作为统一上下文构建入口。

### 12.1 基本职责

- 从多个来源 gather 信息
- 对候选信息进行 score / select
- 按固定结构组织上下文
- 在预算不足时进行 compress
- 输出角色专属的 Context Packet

### 12.2 推荐流程

建议采用 HelloAgents 风格的 GSSC 流水线，但做 Nobody 化改造。

NoName GSSC：

- `Gather`
- `Score`
- `Select`
- `Structure`
- `Compress`

#### Gather

收集候选信息源：

- 当前回合动作
- 当前场景
- `WorkingMemory`
- `EpisodicMemory`
- `SemanticMemory`
- `NarrativeMemory`
- `WorldRegistry`
- `EntityStore`
- 最近章节摘要
- 最近 diagnostics

#### Score

对所有候选片段打分：

- 与当前 action 是否强相关
- 是否属于硬约束事实
- 是否属于本章活跃线索
- 是否与当前地点 / 当前人物直接相关
- 是否会影响输出合法性

#### Select

根据 token budget 和角色需要，选取最有价值的片段。

#### Structure

把上下文整理成固定区块，而不是散乱拼接。

#### Compress

对低优先级历史做摘要压缩，对重复片段做合并，对长 traces 做简写。

## 13. Context Packet 结构

建议统一输出：

```ts
type NoNameContextPacket = {
  role: string
  objective: string
  hardConstraints: string[]
  sceneState: string[]
  activeThreads: string[]
  recentEpisodes: string[]
  worldFacts: string[]
  referencedEntities: string[]
  toolHints: string[]
  outputContract: string[]
  tokenBudgetUsed: number
}
```

## 14. 固定上下文区块建议

建议 Agent 上下文按照下面顺序组装：

1. `Role & Mission`
2. `Hard Constraints`
3. `Current Scene State`
4. `Active Narrative Threads`
5. `Recent High-value Episodes`
6. `Stable World Facts`
7. `Referenced Entities`
8. `Tool Hints`
9. `Output Contract`

这样做的收益是：

- 模型更容易理解优先级
- 更容易插入校验信息
- 更容易做 trace 和对比

## 15. 角色差异化上下文

不同角色不应共享同一份上下文。

### 15.1 DirectorAgent 上下文

重点：

- 当前章目标
- 活跃冲突
- 最近关键事件
- 世界硬约束
- 未回收伏线

### 15.2 WorldCuratorAgent 上下文

重点：

- 最近发生的高价值事件
- 当前世界事实
- 关系变化
- 地点变化
- 可写回的事实候选

### 15.3 NpcIntentAgent 上下文

重点：

- NPC 档案
- 与玩家关系
- 所在地点
- 最近 3 至 5 个与该 NPC 相关事件
- 当前利益冲突

## 16. 长时程上下文策略

HelloAgents 第九章提到的三件事对 NoName Agent 很重要：

- 压缩整合
- 结构化笔记
- 子代理架构

NoName Agent 建议这样落地。

### 16.1 压缩整合（Compaction）

定义：

- 当回合历史积累过长时，把高价值信息压成更短摘要，替代原始长历史进入后续上下文。

建议分三级：

- `turn compaction`：把最近若干回合压成“最近局势摘要”
- `chapter compaction`：把一章压成章节摘要 + unresolved threads
- `trace compaction`：把工具调用与调试噪声压成短摘要

### 16.2 结构化笔记（Structured note-taking）

建议新增 `NoName Notes`，作为 NarrativeMemory 的外部化持久笔记层。

适合记录：

- 当前章目标
- 主线冲突
- 支线冲突
- 待回收伏线
- 世界观新事实候选
- 当前策略建议

### 16.3 子代理上下文隔离（Sub-agent clean context）

如果后续启用多 Agent，不要把所有上下文都给每个 Agent。

建议：

- 主代理拿压缩后的全局上下文
- 子代理拿干净、任务定制的局部上下文
- 子代理返回凝练摘要，而不是整段长 history

## 17. Nobody 领域特化设计

NoName Agent 的记忆和上下文工程，必须体现 `Nobody` 的游戏领域特性。

### 17.1 叙事优先级高于闲聊优先级

上下文里最重要的不是“说得像人”，而是：

- 章法是否连续
- 冲突是否推进
- 设定是否自洽

### 17.2 世界事实优先级高于生成自由度

世界事实一旦与上下文冲突，应优先保护世界事实。

### 17.3 玩家感知信息和系统真相要分层

建议区分：

- `diegetic memory`：玩家可感知的信息
- `system memory`：系统知道但玩家未必知道的信息

这样后续更容易做“信息差剧情”。

### 17.4 地点与关系是高权重召回维度

因为修仙题材的世界推进高度依赖：

- 当前身处何地
- 当前与谁有关系
- 当前资源与境界差距如何

## 18. V1 实施建议

### 18.1 先做轻量版，不要一开始就上向量数据库

V1 建议：

- WorkingMemory: 内存实现
- EpisodicMemory: SQLite
- SemanticMemory: 复用 world_registry + entity_store
- NarrativeMemory: SQLite + structured notes
- ContextBuilder: Rust 内建 GSSC pipeline

### 18.2 先接 DirectorAgent

因为它最需要：

- NarrativeMemory
- SemanticMemory
- Chapter summaries
- Hard constraints

### 18.3 前端只先做调试展示

V1 前端只需要：

- 展示 token budget 使用情况
- 展示上下文片段来源
- 展示 trace 摘要

## 19. 推荐新增数据结构

### 19.1 统一 Memory Item

```ts
type NoNameMemoryRecord = {
  recordId: string
  memoryLayer: "working" | "episodic" | "semantic" | "narrative"
  category: string
  source: string
  summary: string
  payload: Record<string, unknown>
  tags: string[]
  scoreHints?: {
    importance?: number
    authority?: number
    narrativePriority?: number
  }
  createdAt: number
  updatedAt: number
}
```

### 19.2 统一 Context Candidate

```ts
type NoNameContextCandidate = {
  candidateId: string
  sourceLayer: string
  sourceId: string
  text: string
  relevance: number
  recency: number
  importance: number
  authority: number
  narrativePriority: number
  finalScore: number
}
```

## 20. 推荐开发顺序

1. 扩展 `memory_layers.rs`，补 `NarrativeMemory`
2. 引入 `NoNameMemoryManager`
3. 把 `context_builder.rs` 升级为可插拔的 `NoNameContextBuilder`
4. 增加 GSSC 流水线与 score/select 逻辑
5. 增加 compaction 和 structured notes
6. 再考虑向量检索和外部 lore RAG

## 21. 结论

对 `Nobody` 来说，记忆工程和上下文工程不能照搬通用 Agent 框架的“聊天记录 + RAG 文档”模型，而要围绕以下四件事设计：

- 世界状态
- 叙事连续性
- 结构化笔记
- 强规则护栏

所以，NoName Agent 的最优方案是：

- 用 `Working / Episodic / Semantic / Narrative` 四层记忆管理长期状态
- 用 `GSSC` 风格上下文流水线管理每回合 prompt
- 用 `Compaction + Structured Notes + Sub-agent clean context` 支撑长时程任务
- 用 `WorldRegistry + EntityStore + MemoryManager` 形成 Nobody 自己的“记忆底盘”

## 22. 参考资料

- HelloAgents 第八章《记忆与检索》
- HelloAgents 第九章《上下文工程》
- `docs/architecture/noname-agent-v1.md`
- `src-tauri/src/memory_layers.rs`
- `src-tauri/src/context_builder.rs`
- `src-tauri/src/world_registry.rs`
