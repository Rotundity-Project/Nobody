# V2 存档迁移与兼容策略（草案）

更新时间: 2026-02-19
对应任务: `tasks_v2.md` 任务 3

## 目标

- 保障旧存档可加载。
- 对缺失字段自动填充默认值。
- 迁移失败可回退，且给出可读错误。

## 版本策略

- 存档新增字段:
  - `schema_version: string`
  - `migration_history: string[]`
- 推荐版本:
  - `v1_legacy` -> 历史存档
  - `v2_0` -> 引入 interaction_state 与章节生命周期增强

## 迁移路径

1. 读取存档 JSON。
2. 检测 `schema_version`。
3. 按版本执行迁移函数链。
4. 写入新字段并记录 `migration_history`。
5. 执行结构校验后进入运行时。

## 关键迁移规则（v1 -> v2_0）

- PlotState:
  - 若缺少 `interaction_state`:
    - `is_waiting_for_input=false` -> `auto_advance`
    - `is_waiting_for_input=true` 且 `available_options` 非空 -> `waiting_for_choice`
    - 否则 -> `waiting_for_free_text`
- ChapterState:
  - 若缺少 `status` -> `in_progress`（当前章）/`closed`（历史章）
- Diagnostics:
  - 缺失 `last_generation_diagnostics` 时填 `null`

## 回退策略

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

## 当前实现现状

- 已有读档兼容: 缺省字段通过 serde default 处理。
- 已补字段示例: `interaction_state`、`chapter.status`。
- 待补: 显式 `schema_version` 与 `migration_history` 落盘逻辑。
