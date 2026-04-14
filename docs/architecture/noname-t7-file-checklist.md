# NoName Agent T7 文件级实现清单

更新时间: 2026-04-09
状态: 文件级执行清单
目标: 将 `T7 Assisted Skeleton 与受控应用预备` 拆到可直接实施的文件粒度
关联文档:
- `noname-v1-task-list.md`
- `noname-v1-blueprint.md`
- `noname-agent-v1.md`

## 1. T7 当前目标

`T7` 的重点不是继续扩展框架层，而是把当前已经存在的：

- `proposal`
- `guardrail`
- `assisted mode`
- `trace`

推进成一个真正可控的“辅助应用分支”。

当前阶段的约束非常重要：

- 不允许 Agent 直接接管主剧情结果
- 不允许绕过 guardrail 修改最终输出
- 不允许破坏经典链路的可回退性

## 1.1 当前进度

截至 `2026-04-09`，`T7` 第一批已经完成：

- `NoNameProposal` 已补 `status`
  - `observed / ready / blocked / applied / fallback`
- runtime 已按模式与 guardrail 结果设置 proposal 生命周期状态
- trace 已记录最终 proposal 状态
- trace 已记录 apply preflight 结果与 proposal transition log
- apply 阶段已补独立 guardrail 入口
- 调试诊断已输出 `proposal_status` 与 `apply=...`
- 前端调试文本已优先展示显式 `status`、apply preflight 与状态迁移

当前仍未完成的部分：

- proposal 已可进入“诊断层 + 章节摘要提示 + 选项偏置提示”低风险 apply，但还没有进入更高权重的剧情输出层
- trace 已有 apply transition log，并已记录诊断层 / 章节摘要提示 / 选项偏置提示 apply；但还没有更高权重输出层的 apply/reject/fallback 执行明细
- `plot_engine` 侧还没有低风险输出层应用入口

## 2. 建议文件范围

### 核心必改文件

- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_types.rs`

### 高概率配套修改文件

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/plot_engine.rs`
- `src/types/game.ts`
- `src/stores/gameStore.ts`
- `src/platform/webRuntime.ts`

### 可能新增文件

- `src-tauri/src/noname_apply.rs`
  - 如果不希望 `noname_runtime.rs` 继续膨胀，建议把 proposal 应用逻辑单独抽出去。

## 3. 文件级任务

## F1 `noname_types.rs`

### 目标

补齐 `proposal` 从“可记录对象”走向“可受控应用对象”所需的最小状态表达。

### 当前已有

- `NoNameMode`
- `NoNameProposalKind`
- `NoNameProposal`
- `applyable`

### 建议新增/调整

- 给 `NoNameProposal` 增加更明确的应用阶段字段，例如：
  - `status`: `observed / ready / applied / rejected / fallback`
- 明确 `applyable` 与 `status` 的关系
- 如果需要，可加入：
  - `applied_effects_preview`
  - `blocked_reason`

### 完成标准

- proposal 的生命周期不再只能靠 `labels` 猜测
- 前后端都能直接读取 proposal 当前所处阶段

## F2 `noname_trace.rs`

### 目标

让 trace 能完整表达“proposal 是否被尝试应用、是否被拒绝、是否 fallback”。

### 当前已有

- `capability_calls`
- `proposals`
- `guardrail_result`
- `fallback_used`

### 建议新增/调整

- 增加 proposal 级别的 trace 标记，例如：
  - `applied_proposal_id`
  - `proposal_transition_log`
- 如果继续沿用阶段图，建议补充：
  - `ApplyProposal`
  - `ApplyRejected`
  - `ApplyFallback`

### 完成标准

- 一条 trace 能看出 proposal 是否进入辅助应用分支
- fallback 是“生成 fallback”还是“应用 fallback”可以区分开

## F3 `noname_guardrails.rs`

### 目标

把 guardrail 从“判断可不可”推进到“判断可不可应用”。

### 当前已有

- `accept / repair / reject`
- `DirectorObservation` 校验
- patch / entity / map 数值校验

### 建议新增/调整

- 为 proposal 应用新增独立入口，例如：
  - `validate_proposal_for_apply()`
- 区分两类 guardrail：
  - `proposal semantic validation`
  - `proposal apply validation`
- 定义 apply 级别的失败原因分类：
  - `state_risk`
  - `plot_risk`
  - `mode_forbidden`
  - `fallback_required`

### 完成标准

- guardrail 不仅知道“这提案合理吗”，还知道“这提案现在能不能应用”
- `assisted` 模式的 apply 权限完全受 guardrail 约束

## F4 `noname_runtime.rs`

### 目标

这是 `T7` 的主战场。把当前 runtime 的 `assisted_ready`，推进成真正的“受控应用预备流程”。

### 当前已有

- `set_mode()`
- `finalize_director_proposal()`
- `assisted` 下标记 proposal 为 `applyable`

### 建议新增/调整

- 增加显式的 apply 分支，例如：
  - `maybe_apply_assisted_proposal()`
- 明确三种模式行为：
  - `disabled`: 完全跳过 NoName
  - `observe_only`: 记录 proposal，但不尝试应用
  - `assisted`: 允许进入 apply 预检与受控应用分支
- 如果应用失败：
  - 写 trace
  - 标记 proposal rejected/fallback
  - 回退到经典链路

### 推荐实现顺序

1. 先只做 `apply preflight`
2. 再做最小的 `safe apply`
3. 最后再考虑更复杂的应用策略

### 完成标准

- runtime 可以在 `assisted` 下尝试进入 apply 分支
- 失败时不会污染现有剧情结果
- trace 清楚记录 apply 尝试与回退

## F5 `noname_roles.rs`

### 目标

让 `DirectorAgent` 的输出更适合作为“可应用提案”，而不是只有观察意义。

### 当前已有

- `focus`
- `rationale`
- `proposal.title / summary / suggested_action`

### 建议新增/调整

- 让 proposal 更结构化，例如加入：
  - `target_segment`
  - `intended_effect`
  - `apply_scope`
- 约束 `DirectorAgent` 输出更适配 apply 预检

### 完成标准

- proposal 不再只是“建议观察什么”，而是“建议以什么边界影响当前回合”

## F6 `tauri_commands.rs`

### 目标

继续把它保持为编排层，而不是在这里堆具体规则。

### 当前已有

- `get_noname_mode / set_noname_mode`
- `get_noname_recent_traces / clear_noname_recent_traces`
- `execute_player_action` 中 observe-only 接入

### 建议新增/调整

- 如果进入 `assisted` 应用分支，只在这里做：
  - 模式判定
  - runtime 调用
  - diagnostics 拼接
- 避免把 apply 逻辑直接堆进 `execute_player_action`
- 统一 diagnostics 文案：
  - `NoName.observe_only`
  - `NoName.assisted.ready`
  - `NoName.assisted.applied`
  - `NoName.assisted.fallback`

### 完成标准

- `tauri_commands.rs` 只负责入口编排，不成为 apply 规则仓库

## F7 `plot_engine.rs`

### 目标

如果 proposal 真的要对剧情结果产生受控影响，最终通常会落到这里的结果拼装层。

### 建议新增/调整

- 明确 proposal 能影响的最小范围，例如：
  - `generation_diagnostics`
  - `chapter_summary hint`
  - `option bias note`
  - `non-final plot augmentation`
- 不建议第一阶段直接改：
  - `new_scene`
  - `available_options`
  - 主剧情全文主体

### 完成标准

- 第一个 apply 版本只影响“低风险输出层”
- 不直接篡改核心剧情状态机

## F8 `game.ts` / `gameStore.ts`

### 目标

让前端能明确看出 proposal 当前是：

- observe-only
- assisted-ready
- assisted-applied
- assisted-fallback

### 建议新增/调整

- 给前端类型补 proposal status 字段
- debug 文本增加：
  - 当前 mode
  - proposal status
  - apply 结果
- 如果后面有必要，再考虑单独的 proposal 调试面板

### 完成标准

- 不用翻 trace 原始 JSON，也能看出 assisted 是否真的开始生效

## F9 `webRuntime.ts`

### 目标

确保开发和测试环境不会因为新命令缺失而失真。

### 建议新增/调整

- 对新增的 assisted 相关命令补 mock
- 对 trace 返回中的 proposal status 补 mock 形状

### 完成标准

- 前端测试不会因为 assisted 模式字段新增而脆断

## 4. 推荐提交切片

建议拆成 3 个提交：

1. `feat(noname): 定义 proposal apply 状态与 assisted 预检结构`
2. `feat(noname): 接入 assisted apply preflight 与 trace 记录`
3. `feat(noname): 更新前端调试信息与 web runtime mock`

## 5. 验收清单

### 后端

- `disabled` 下 NoName 完全不影响主链路
- `observe_only` 下 proposal 仅记录不应用
- `assisted` 下 proposal 只有在 guardrail 通过时才进入 apply 预备分支
- apply 失败时可 fallback 到经典链路

### 前端

- 调试面板能显示 proposal 当前状态
- 调试文本能区分 `observe_only` 与 `assisted`
- Web mock 与测试夹具不会因新字段失效

### 验证命令

- `cargo test noname_ -- --nocapture`
- `cargo test -q`
- `npm run test -- --run src/stores/__tests__/gameStore.test.ts src/components/__tests__/GameInfoCenterDialog.test.ts src/components/__tests__/GameView.test.ts`
- `npm run build`

## 6. 推荐起手顺序

如果下一步开始写代码，建议按这个顺序推进：

1. `noname_types.rs`
2. `noname_trace.rs`
3. `noname_guardrails.rs`
4. `noname_runtime.rs`
5. `tauri_commands.rs`
6. `game.ts`
7. `gameStore.ts`
8. `webRuntime.ts`
这个顺序的好处是：先把状态机和边界讲清楚，再碰真正容易引发回归的应用分支。
