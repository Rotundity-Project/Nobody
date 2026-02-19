# tasks_v2 对照矩阵（代码基线审计）

更新时间: 2026-02-19
范围: `.kiro/specs/Nobody/tasks_v2.md` 顶层任务 1-28
判定口径:
- 已完成: 有明确实现与调用链
- 部分完成: 有实现骨架或局部能力，但未达到任务验收语义
- 未完成: 未见对应实现或仅停留在计划文本

## 阶段 A

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 1. 剧情一致性问题清单与复现库 | 已完成 | 已完成 | 已形成四类问题清单 + 20 条最小复现实测样例（每类≥5，`docs/qa/consistency-cases.md`），满足任务1文档交付与样例数量验收 |
| 2. 领域模型重构方案设计 | 已完成 | 已完成 | 已形成字段级实体定义、跨模块不变量与生命周期链路说明（`docs/architecture/domain-model-v2.md`） |
| 3. 数据迁移与兼容策略 | 已完成 | 已完成 | 已落地 `schema_version`/`migration_history`、旧档交互态归一化、迁移回写、批量迁移与失败报告；并通过“大样本旧档迁移成功率>=95%”与“失败可读错误”测试（`src-tauri/src/save_load.rs`, `src-tauri/src/tauri_commands.rs`, `docs/release/migration-v2.md`） |

## 阶段 B

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 4. Plot State Machine | 已完成 | 已完成 | 已实现 `AutoAdvance/WaitingForChoice/WaitingForFreeText/Resolving/Cooldown` 状态流转，并接入前端自动推进与读档恢复链路（`src-tauri/src/plot_engine.rs`, `src-tauri/src/tauri_commands.rs`, `src-tauri/src/game_engine.rs`, `src/components/GameView.vue`） |
| 5. 去重与记忆检索层 | 已完成 | 已完成 | `src-tauri/src/plot_consistency.rs`, `src-tauri/src/context_builder.rs` |
| 6. 章节目标驱动生成 | 已完成 | 已完成 | 已实现“章节目标弱命中触发重生成”闭环：首次校验命中 `chapter_goal_weak` 时自动单次重生成并按风险裁决采用结果（`src-tauri/src/tauri_commands.rs`, `src-tauri/src/plot_consistency.rs`） |
| 7. 剧情一致性验证器 V2 | 已完成 | 已完成 | `validate_and_repair_plot_update`（`src-tauri/src/plot_consistency.rs`）并接入 `execute_player_action` |
| 7A. 叙事厚度与张力增强 | 未完成 | 部分完成 | 有部分约束提示，缺少模板库/空洞表达检测完整体系 |

## 阶段 C

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 8. 角色档案系统重构 | 未完成 | 部分完成（增强中） | 已新增可持久成长轨迹日志并接入修炼/突破/战斗写入链路（`src-tauri/src/game_state.rs`, `src-tauri/src/tauri_commands.rs`）；仍待功法树/人格标签前后端展示闭环 |
| 9. 关系与声望系统 | 已完成 | 已完成 | 已接入关系维度（宗门倾向/师徒羁绊/仇恨/恩义/阵营立场）到 NPC 事件影响计算，并写入关系变动事件；同一事件在不同关系立场下反馈强度不同（`src-tauri/src/game_state.rs`, `src-tauri/src/game_engine.rs`） |

## 阶段 D

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 10. 功法语义模型 | 未完成 | 部分完成（增强中） | 已将灵根适配/高阶门槛/高风险功法语义接入战斗结果修正与后果（`src-tauri/src/tauri_commands.rs`），并保留 `TechniqueDef` 结构化字段；待补更完整学习链路与前端展示 |
| 11. 学习/突破/走火入魔链路 | 未完成 | 部分完成（增强中） | 已接入突破失败后果链路：气机紊乱累积、风险功法反噬增伤、走火入魔临界预警，并写入成长日志与事件（`src-tauri/src/game_state.rs`, `src-tauri/src/tauri_commands.rs`）；仍待完整学习条件/偏差分支体系 |

## 阶段 E

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 12. 世界地图与区域生态 | 未完成 | 部分完成（增强中） | 已将地点灵气强度接入运行期修炼收益修正（同一修炼行为因区域生态不同而收益不同），并保留 `MapNodeDef` schema；仍待更完整生态规则（资源/阵营/禁地）接入（`src-tauri/src/tauri_commands.rs`, `src-tauri/schema/map_node_def.v1.json`） |
| 13. 移动与遭遇机制 | 未完成 | 部分完成（增强中） | 已新增移动命令 `travel_to_location` + `get_reachable_locations`，接入可达性约束、时间消耗（+1日）、按灵气/仇恨/气机紊乱加权的基础遭遇触发与事件日志回写；并引入 `travel_rules_v2.json` 配置化参数。前端地图页签可触发移动并显示可达性。仍待完整路径图与遭遇池（`src-tauri/src/tauri_commands.rs`, `src-tauri/src/travel_rules.rs`, `src-tauri/config/travel_rules_v2.json`, `src/components/InfoTabsDialog.vue`, `src/components/GameView.vue`） |

## 阶段 F

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 14. 战斗判定引擎 V2 | 未完成 | 部分完成 | 有数值守门（`src-tauri/src/numeric_guard.rs`）与战斗分支校验，未见完整规则矩阵 4 步流程 |
| 15. 战斗结果解释器 | 已完成 | 已完成 | 已新增结构化战斗解释（主导因素/反转因素/摘要）并写入 `combat_explanation` 事件日志，供导出与复盘链路使用（`src-tauri/src/tauri_commands.rs`） |
| 16. 战斗与剧情耦合 | 已完成 | 已完成 | 已将战斗后长期状态（伤势/威望/仇恨）写入玩家状态并注入后续剧情上下文，同时记录战后状态事件（`src-tauri/src/game_state.rs`, `src-tauri/src/tauri_commands.rs`） |

## 阶段 G

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 17. LLM 输出契约（JSON Schema） | 已完成 | 已完成 | `src-tauri/schema/*.json` |
| 18. LLM-Engine 协作协议 | 已完成 | 已完成 | `tauri_commands` 中生成-校验-落库链路 |
| 19. 合理性控制算法 | 已完成 | 已完成 | 一致性修复+降级回退链路 |
| 19A. 实体结构化填充与持久化 | 已完成 | 已完成 | `entity_validator/store/query` 与命令接口 |
| 19B. 数值系统守门 | 已完成 | 已完成 | `src-tauri/config/numeric_rules_v2.json`, `numeric_guard.rs` |
| 19C. 分层记忆与上下文窗口 | 已完成 | 已完成 | `memory_layers.rs`, `context_builder.rs` 并在推进前注入 |

## 阶段 H

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 20. 成长面板与战斗复盘 | 未完成 | 部分完成（增强中） | 角色面板已展示战后状态（伤势/威望/仇恨/气机紊乱）与最近成长记录，且战斗解释事件已落日志供复盘；仍待更完整图形化复盘视图（`src/components/CharacterPanel.vue`, `src-tauri/src/tauri_commands.rs`） |
| 21. 地图与行程 UI | 未完成 | 部分完成（增强中） | 信息面板已新增地图行程页签，可展示当前位置、世界地点列表与基础风险提示；仍待可达路径、热区层和移动决策交互（`src/components/InfoTabsDialog.vue`, `src/components/GameView.vue`） |
| 22. 剧情推进控制增强 | 已完成 | 已完成 | 自动推进到交互点逻辑（`src/components/GameView.vue`） |
| 22A. UI 体验优化 | 已完成 | 已完成 | 状态反馈与信息层次已调整 |
| 22B. 小说导出重定义 | 已完成 | 已完成 | `generate_chronicle_from_plot` |
| 22C. 章节系统增强 | 已完成 | 已完成 | 章节生命周期+TOC（`plot_engine.rs`, `novel_generator.rs`） |

## 阶段 I

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 23. 属性测试与回归扩展 | 已完成 | 已完成 | 已补齐剧情一致性、地图位置、功法适配与风险、导出事实与章节目录等属性测试并稳定通过（`docs/qa/property-matrix.md`；验证命令含 `cargo test -q plot_engine::property_tests`、`novel_generator::property_tests`、`numeric_guard::tests`、`entity_validator::tests`、`test_property_map_location_consistency_after_roundtrip`） |
| 24. 性能目标 | 已完成 | 已完成 | 已形成基线报告、批量采样 P50/P95/P99 基准、运行时耗时埋点与聚合命令，并复跑验证阈值通过（`docs/qa/v2-performance-report.md`, `docs/qa/perf-history.md`, `src-tauri/src/tests.rs`, `src-tauri/src/tauri_commands.rs`, `.github/workflows/perf-benchmark.yml`） |
| 25. 发布准备 | 已完成 | 已完成 | 已完成版本号同步、迁移说明/兼容说明/已知限制/changelog 与 go-live 文档，并产出测试版安装包（`src-tauri/target/release/bundle/msi/Nobody_1.1.0_x64_en-US.msi`, `src-tauri/target/release/bundle/nsis/Nobody_1.1.0_x64-setup.exe`） |

## 阶段 J

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 26. 关键任务完成门禁 | 已完成 | 已完成 | `docs/qa/v2-test-report.md` |
| 27. 分支提交流程 | 已完成 | 已完成 | 文档记录存在（见测试报告与任务计划） |
| 28. 合并 main | 已完成 | 部分完成（待仓库历史复核） | 需结合 Git 历史/远端 PR 验证 |

## 下一步执行顺序（建议）

1. 先补阶段 A 的交付物文档（任务 1/2/3），形成稳定基线。
2. 实做阶段 B-4：显式 Plot State Machine（避免仅靠布尔字段推断状态）。
3. 实做阶段 B-6：章节目标命中校验失败时触发重生成/回退。
4. 扩展阶段 I-23：补齐导出事实覆盖率、章节目录完整性等属性测试。
