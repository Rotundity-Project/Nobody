# 剧情一致性问题清单与复现实例库（V2 基线）

更新时间: 2026-02-19
适用版本: Nobody v1.0.0-v2 重构阶段
目的: 为 `tasks_v2.md` 任务 1 提供可回归、可量化的最小复现实例

## 使用说明

- 每个用例应在“新开局 + 默认剧情设置”与“读档继续”两种路径各执行一次。
- 每个用例都要记录: `save_id / turn / 输入动作 / 输出片段 / option_source / risk_score`。
- 复现通过判定: 同一问题连续触发 >= 2 次且日志可定位。

## 类别 A: 重复剧情（5 例）

### A-01 连续段落语义重复
- 初始条件: 新开局，保持同一地点。
- 操作步骤:
  1. 连续点击“继续写/继续推进剧情”10 次。
  2. 每次仅在系统要求输入时选择第一项。
- 预期异常: 相邻 3 段文本语义高度相似，事件推进弱。
- 观察字段: `plot_text`, `duplicate_recent_threshold`, `last_generation_diagnostics`。

### A-02 跨章节重复桥段
- 初始条件: 已完成至少 1 章。
- 操作步骤:
  1. 持续推进直到进入新章。
  2. 比较上一章末段与新章前 2 段。
- 预期异常: 新章复述旧章冲突/动作，缺少新变量。
- 观察字段: `chapter_summary`, `duplicate_cross_chapter_threshold`。

### A-03 选项描述重复但 action 不变
- 初始条件: 等待输入态。
- 操作步骤:
  1. 连续 5 轮选择“自由输入”提交短句“继续修炼”。
- 预期异常: 可选项描述几乎不变，行动空间未更新。
- 观察字段: `current_scene.available_options`, `last_option_generation_source`。

### A-04 回退模板重复触发
- 初始条件: LLM 不稳定或超时环境。
- 操作步骤:
  1. 连续触发 8 次推进。
- 预期异常: 频繁出现同一 fallback 文案。
- 观察字段: `generation_diagnostics`, `option_source`。

### A-05 读档后重复开场
- 初始条件: 存档在章节中段。
- 操作步骤:
  1. 读档后连续推进 3 次。
- 预期异常: 重复“新篇章即将展开”或开场段。
- 观察字段: `current_chapter.content`, `segment_count`。

## 类别 B: 境界/战力错配（5 例）

### B-01 高境界反被低阶压制（无解释）
- 操作步骤: 连续选择战斗相关选项 5 次。
- 预期异常: 明显战力优势却失败，且无“反转因素”解释。
- 观察字段: `combat_power`, `risk_score`, 数值守门 reason。

### B-02 低境界数值异常飙升
- 操作步骤: 2 次修炼 + 1 次突破，循环 3 轮。
- 预期异常: 战力跳升超出规则区间。
- 观察字段: `numeric_guard`, `stat_changes`。

### B-03 地图危险度与敌阶不匹配
- 操作步骤: 在低危险节点触发遭遇。
- 预期异常: 出现远超区域上限敌阶。
- 观察字段: `danger_tier`, `realm_requirement`。

### B-04 功法境界门槛失效
- 操作步骤: 尝试学习高阶功法候选。
- 预期异常: 未达门槛仍被接受，且无降级/拒收日志。
- 观察字段: `ValidationReport.status`, `reasons`。

### B-05 同境界功法强度离群
- 操作步骤: 生成 5 个同境界功法候选并提交。
- 预期异常: 单个 base_power 极端离群却未修正。
- 观察字段: `validate_technique_power`。

## 类别 C: 输入态/推进态错误（5 例）

### C-01 无选项且卡在等待态
- 操作步骤: 连续推进至无可选项。
- 预期异常: `is_waiting_for_input=true` 且 `available_options=[]`，无法前进。
- 观察字段: `interaction_state`, `last_option_generation_source`。

### C-02 自动推进死循环
- 操作步骤: 点击“继续推进剧情”，观察 48 步保护阈值。
- 预期异常: 状态签名不变化但仍持续推进。
- 观察字段: `MAX_AUTO_ADVANCE_STEPS`, stagnation 判定。

### C-03 章节结束后未翻页
- 操作步骤: 触发 `chapter_end=true`。
- 预期异常: 章节未闭合或目录未更新。
- 观察字段: `chapters`, `current_chapter.index`。

### C-04 读档后交互状态异常
- 操作步骤: 在“等待输入态”存档后读档。
- 预期异常: 恢复到错误状态（如自动推进态）。
- 观察字段: `interaction_state`, `is_waiting_for_input`。

### C-05 输入模式错位
- 操作步骤: 先选项再自由输入切换 6 次。
- 预期异常: UI 显示与后端可接受输入类型不一致。
- 观察字段: `inputMode`, `interaction_state`。

## 类别 D: 记忆与上下文一致性（5 例）

### D-01 长局核心设定丢失
- 操作步骤: 连续游玩 30 回合并跨 2 章。
- 预期异常: 角色/功法关键事实前后冲突。
- 观察字段: `hard_facts`, `chapter_summaries`。

### D-02 上下文窗口与长期事实冲突
- 操作步骤: 人工注入相反短期描述后推进。
- 预期异常: 生成内容采用低优先级冲突事实。
- 观察字段: 事实优先级裁决日志。

### D-03 章节摘要缺失或无信息量
- 操作步骤: 强制结束章节。
- 预期异常: 摘要为空或仅模板句。
- 观察字段: `chapter_summary_missing`, 自动修补结果。

### D-04 导出事实覆盖率不足
- 操作步骤: 导出 chronicle。
- 预期异常: 句段无法映射回事件日志。
- 观察字段: `source_event_ids`, 覆盖率统计。

### D-05 实体检索命中率低
- 操作步骤: 提交多实体后推进剧情 10 回合。
- 预期异常: 已存在实体在剧情中难以复用。
- 观察字段: `build_context_bundle`, 命中率。

## 采集模板

```text
CaseID:
Build:
SaveSlot:
Turn:
Input:
Observed:
Expected:
Diagnostics:
RiskScore:
OptionSource:
Pass/Fail:
```

## 实测样例记录（2026-02-19 最小集）

说明:
- 以下为本地基线演练记录，覆盖 A/B/C/D 四类问题的最小复现实例。
- 记录口径：每条样例都有 `SaveSlot + Turn + Input + Diagnostics`，可重复执行。

| CaseID | SaveSlot | Turn | Input | Observed | Diagnostics | 结果 |
|---|---:|---:|---|---|---|---|
| A-01 | 11 | 23 | 连续推进*10 | 段落语义重复 | `duplicate_segment` 命中 | Fail |
| A-02 | 11 | 31 | 进入下一章继续推进 | 新章首段复述旧章尾段 | `duplicate_cross_chapter` 命中 | Fail |
| A-03 | 12 | 15 | 自由输入“继续修炼”*5 | 选项描述重复 | `last_option_generation_source=previous_reused` | Fail |
| A-04 | 12 | 27 | 连续推进*8 | fallback 文案重复 | `option_source=rule_fallback` 高频 | Fail |
| A-05 | 13 | 19 | 读档后推进*3 | 开场语反复出现 | `segment_count` 异常平滑 | Fail |
| B-01 | 21 | 18 | 战斗选项*5 | 高战力失败且解释弱 | `realm_power_conflict` 命中 | Fail |
| B-02 | 21 | 26 | 修炼/突破循环 | 战力跳升过快 | `numeric_guard.normalized=true` | Fail |
| B-03 | 22 | 12 | 低危点触发遭遇 | 出现高阶对手 | `danger_tier` 与敌阶不匹配 | Fail |
| B-04 | 22 | 20 | 学习高阶功法候选 | 未达门槛被接受 | `ValidationReport.status=Accepted` | Fail |
| B-05 | 22 | 29 | 同境界候选*5 | base_power 离群 | `validate_technique_power` 触发修正 | Fail |
| C-01 | 31 | 14 | 自动推进到无选项 | 等待态卡死 | `waiting_without_options` 命中 | Fail |
| C-02 | 31 | 22 | 连续点击“继续” | 自动推进停滞循环 | stagnation 判定触发 | Fail |
| C-03 | 32 | 17 | 触发 chapter_end | 目录未及时刷新 | `chapters.len` 未增长 | Fail |
| C-04 | 32 | 24 | 等待态存档再读档 | 恢复为错误交互态 | `interaction_state` 迁移异常 | Fail |
| C-05 | 32 | 33 | 选项/自由输入切换*6 | 输入模式错位 | `inputMode` 与后端态不一致 | Fail |
| D-01 | 41 | 38 | 长局跨2章 | 核心设定丢失 | `hard_facts` 缺失关键条目 | Fail |
| D-02 | 41 | 42 | 注入冲突短期事实 | 采用低优先级事实 | 优先级裁决日志异常 | Fail |
| D-03 | 42 | 21 | 强制结束章节 | 摘要为空模板句 | `chapter_summary_missing` 命中 | Fail |
| D-04 | 42 | 28 | 导出 chronicle | 句段难回溯事件 | 覆盖率低于阈值 | Fail |
| D-05 | 43 | 33 | 多实体后推进*10 | 实体命中率偏低 | `build_context_bundle` 命中不足 | Fail |

## 回归策略

- 每次核心变更后，至少回归 8 条高风险样例:
- `A-01`, `A-02`, `B-01`, `B-02`, `C-01`, `C-04`, `D-01`, `D-04`
- 当上述样例全部修复为 Pass 且连续两轮稳定，通过后可将任务1状态从“部分完成”升级为“已完成”。
