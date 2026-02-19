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
| 8. 角色档案系统重构 | 未完成 | 部分完成 | 角色基础结构存在（`src-tauri/src/game_state.rs`），未见完整“功法树/人格标签/成长轨迹日志”前后端闭环 |
| 9. 关系与声望系统 | 未完成 | 部分完成 | NPC 引擎存在（`src-tauri/src/npc.rs`, `src-tauri/src/npc_engine.rs`），未见完整关系维度与剧情判定联动展示 |

## 阶段 D

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 10. 功法语义模型 | 未完成 | 部分完成 | `TechniqueDef`/标签字段已存在（`src-tauri/src/entity_types.rs`），但学习/行为层联动不完整 |
| 11. 学习/突破/走火入魔链路 | 未完成 | 部分完成 | 存在突破行为（`Action::Breakthrough`），未见完整失败后果/偏差分支系统 |

## 阶段 E

| 任务 | 计划状态 | 代码审计状态 | 证据 |
|---|---|---|---|
| 12. 世界地图与区域生态 | 未完成 | 部分完成 | `MapNodeDef` 与 schema 存在（`src-tauri/schema/map_node_def.v1.json`），但运行期地图生态规则未完整接入主循环 |
| 13. 移动与遭遇机制 | 未完成 | 部分完成 | 有 location 字段和场景位置，未见完整路径/时间消耗/遭遇池机制 |

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
| 20. 成长面板与战斗复盘 | 未完成 | 部分完成 | 有信息面板（`src/components/InfoTabsDialog.vue`），但复盘与成长可视化仍不完整 |
| 21. 地图与行程 UI | 未完成 | 未完成 | 未见地图/可达区域/风险热区 UI |
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
