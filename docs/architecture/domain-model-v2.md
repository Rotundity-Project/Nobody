# Domain Model V2（字段级草案）

更新时间: 2026-02-19
对应任务: `tasks_v2.md` 任务 2

## 设计目标

- 规则优先: 世界硬约束由引擎裁决。
- 结构化优先: LLM 输出仅作为候选。
- 单一状态源: 角色/功法/地图/章节均可持久化和回放。

## 核心实体

## CharacterProfile

- identity:
  - `character_id: string`
  - `name: string`
  - `faction_id?: string`
- cultivation:
  - `realm: string`
  - `realm_level: u32`
  - `realm_sub_level: u32`
  - `combat_power: u64`
- traits:
  - `personality_tags: string[]`
  - `relationship_edges: RelationshipEdge[]`
  - `known_techniques: string[]`
- runtime:
  - `location: string`
  - `status_flags: string[]`
  - `last_updated_turn: u64`

## TechniqueDef

- identity:
  - `technique_id: string`
  - `name: string`
- semantics:
  - `tags: string[]`
  - `realm_requirement: u32`
  - `root_affinity: string[]`
  - `risk_tags: string[]`
- numeric:
  - `base_power: f64`
  - `release_cost?: f64`
- text:
  - `description: string`

## MapNodeDef

- identity:
  - `node_id: string`
  - `name: string`
  - `node_type: string`
- environment:
  - `danger_tier: u8`
  - `aura_density: f64`
  - `resource_tags: string[]`
  - `faction_control: string`
- topology:
  - `connected_nodes: string[]`

## ItemDef

- identity:
  - `item_id: string`
  - `name: string`
  - `item_type: string`
- progression:
  - `quality_tier: u8`
  - `realm_requirement?: u32`
- text:
  - `description: string`

## Plot / Chapter

- `PlotState`
  - `is_waiting_for_input: bool`
  - `interaction_state: PlotInteractionState`
  - `current_scene: Scene`
  - `current_chapter: ChapterState`
  - `chapters: ChapterState[]`
- `ChapterState`
  - `index, title, summary, content[]`
  - `interaction_count`
  - `status: in_progress|closed|exported`

## Memory Layers

- short-term: `recent_events`
- mid-term: `chapter_summaries`
- long-term: `world_facts`

## 不变量（跨模块）

- 境界压制下限:
  - 低境界战胜高境界必须有 `反转条件`（环境、克制、状态）或拒收。
- 数值区间合法:
  - `base_power` 与 `realm_requirement` 必须命中 `numeric_rules_v2.json`。
- 地图一致性:
  - 角色位置必须存在于地图节点集合。
- 章节闭环:
  - `chapter_end=true` 后必须 `finalize_chapter`，目录可追踪。
- 导出可回溯:
  - 导出文本段至少映射一个事件来源。

## 生命周期

1. LLM 生成候选实体。
2. Schema 校验。
3. 规则裁决（数值归一化/拒收）。
4. 写入 `EntityStore` 与 `MemoryLayers`。
5. 剧情推进时由 `ContextBuilder` 检索注入。
6. 章节结算后回写摘要和世界事实。

## 模块映射（当前实现）

- entity: `src-tauri/src/entity_types.rs`, `src-tauri/src/entity_validator.rs`, `src-tauri/src/entity_store.rs`
- memory: `src-tauri/src/memory_layers.rs`, `src-tauri/src/context_builder.rs`
- numeric: `src-tauri/src/numeric_guard.rs`, `src-tauri/config/numeric_rules_v2.json`
- plot/chapter: `src-tauri/src/plot_engine.rs`, `src-tauri/src/tauri_commands.rs`
