# V2 存档迁移与兼容策略

更新时间: 2026-02-19  
对应任务: `tasks_v2.md` 任务 3 / 25

## 目标

- 保障旧存档可加载。
- 对缺失字段自动填充默认值并记录迁移轨迹。
- 迁移失败可回退，且给出可读错误。

## 版本策略

- 存档关键字段:
  - `schema_version: string`
  - `migration_history: string[]`
- 版本定义:
  - `v1_legacy` -> 历史存档
  - `v2_0` -> 引入 interaction_state 与章节生命周期增强后的运行态

## 迁移路径

1. 读取存档 JSON。
2. 检测 `schema_version`（缺省视为 `v1_legacy`）。
3. 执行迁移函数链（包含交互状态归一化）。
4. 写入/更新 `schema_version` 与 `migration_history`。
5. 执行结构校验后进入运行时。

## 关键迁移规则（v1_legacy -> v2_0）

- PlotState:
  - 若缺少 `interaction_state`:
    - `is_waiting_for_input=false` -> `auto_advance`
    - `is_waiting_for_input=true` 且 `available_options` 非空 -> `waiting_for_choice`
    - 否则 -> `waiting_for_free_text`
- ChapterState:
  - 若缺少 `status` -> `in_progress`（当前章）/`closed`（历史章）
- Diagnostics:
  - 缺失 `last_generation_diagnostics` 时填 `null`

## 兼容矩阵

| 输入存档版本 | 处理方式 | 结果 |
|---|---|---|
| 无 `schema_version` | 视为 `v1_legacy` 并迁移 | 可加载（写回 `v2_0` 运行态） |
| `v1_legacy` | 执行迁移链 | 可加载 |
| `v2_0` | 直载 + 必要归一化 | 可加载 |
| `2.x+`（未来版本） | 拒绝加载 | 返回版本不兼容错误 |

## 失败与回退策略

- 迁移失败:
  - 不覆盖原文件。
  - 输出错误: 失败字段、失败原因、建议修复步骤。
- 加载失败兜底:
  - 保持旧存档只读。
  - 提供“尝试最小兼容加载”入口（仅恢复玩家核心状态）。

## 验收指标

- 旧存档加载成功率 >= 95%。
- 迁移失败用例 100% 返回可读错误。
- 迁移后的剧情与章节状态可继续推进，不出现死锁。

## 当前实现状态

- 已实现:
  - `schema_version`/`migration_history` 字段持久化。
  - 旧档加载迁移与交互态归一化。
  - 读档恢复链路与剧情状态重建/对齐。
  - 批量迁移能力与失败报告（`migrate_all_saves`）。
- 待增强:
  - 迁移失败样本库与自动修复建议模板。
