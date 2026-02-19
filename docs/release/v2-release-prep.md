# V2 发布准备说明（草案）

更新时间: 2026-02-19  
建议版本: `1.0.0 -> 1.1.0`

## 发布目标

- 交付系统级能力扩展（结构化实体、分层记忆、数值守门、章节导出增强）。
- 保证 `1.x` 存档可读、可迁移、可继续游玩。
- 提供可追溯的质量与性能证据（属性测试矩阵 + P95 基线）。

## 版本与配置同步清单

- `package.json` -> `version: 1.1.0`
- `src-tauri/tauri.conf.json` -> `version: 1.1.0`
- `src-tauri/Cargo.toml` -> `version: 1.1.0`
- 更新发布说明: `docs/release/v2-changelog-draft.md`

执行状态:
- 已完成（2026-02-19）：三端版本号同步到 `1.1.0`。

## 变更摘要

- 剧情推进增强:
  - 引入显式 `interaction_state` 状态流转，并接入读档恢复链路。
  - 一致性修复链路增强，支持交互态兜底与诊断输出。
- 存档迁移增强:
  - `schema_version` + `migration_history` 已接入，支持旧档加载归一化与迁移回写。
- 导出与回归增强:
  - 章节目录完整性、导出事实覆盖率、句子级事实覆盖率、风险标签追溯等属性测试已补齐。
- 性能可观测性增强:
  - P50/P95/P99 批量采样基准。
  - 运行时耗时诊断聚合命令与手动 CI 性能工作流。

## 迁移与兼容结论

- 迁移说明: `docs/release/migration-v2.md`
- 兼容范围: `1.x` 存档可加载并迁移到 `v2_0` 运行态。
- 失败策略:
  - 不覆盖原文件。
  - 返回可读错误，保留回滚路径。

## 发布门禁（建议）

1. 前端测试通过: `npm test`
2. 后端关键属性通过:
   - `cargo test -q novel_generator::property_tests`
   - `cargo test -q test_property_map_location_consistency_in_save_load_plot_recovery`
3. 性能基线通过:
   - `cargo test -q perf_plot_advance_p95_under_target -- --ignored --nocapture`
   - `cargo test -q perf_combat_parse_p95_under_target -- --ignored --nocapture`
4. CI 绿灯:
   - `.github/workflows/ci.yml`
   - `.github/workflows/perf-benchmark.yml`（手动触发）

## 已知限制

- Plot State Machine 仍未完全独立化（当前已是增强版显式状态，但未形成单独模块边界）。
- 章节目标驱动尚未形成“失败即重生成”的闭环执行器。
- 地图与行程 UI（任务21）仍未落地。
- 战斗解释器（任务15）仍为文本拼接级别，缺结构化主导/反转因子。

## 灰度建议

1. 内部包回归（存档读写/剧情推进/导出/性能基线）。
2. 小范围灰度（10%-20% 用户）。
3. 观察 72 小时错误日志与耗时分位数，再全量发布。

## 回滚方案

- 回滚目标版本: `1.0.0`
- 对异常存档优先走只读加载与导出保底。
- 保留 `migration_history` 与运行时诊断用于根因定位。
