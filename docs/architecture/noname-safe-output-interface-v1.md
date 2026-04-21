# NoName Safe Output Interface V1

更新时间: 2026-04-16
对应任务: `A4-next-safe-output-interface`

## 目标

为 `NoName Agent` 从“低风险提示层”走向“更高一层受控输出层”预留安全接口，但不直接接管最终剧情状态机，也不修改 `T7 apply` 主链。

这版实现的是独立接口草案和本地 review stub，不做真实剧情应用。

## 允许的受控输出类型

当前 `NoNameControlledOutputKind` 支持:

- `recapNote`
  - 用于章节摘要、回顾提示、上下文续接。
  - 默认映射到 `ChapterSummaryHint`。

- `sceneAugmentation`
  - 用于场景氛围、感官细节、低风险补写建议。
  - 默认映射到 `PlotTextHint`，但 V1 要求人审。

- `narrativeNote`
  - 用于诊断层或结构化 note 层的叙事标记。
  - 默认映射到 `Diagnostics`。

- `intermediateNarrativeHint`
  - 用于下一轮选项倾向、叙事方向提示。
  - 默认映射到 `OptionBiasHint`。

## 禁止触碰范围

当前 `NoNameForbiddenOutputScope` 明确禁止:

- `finalPlotState`
- `canonWorldFact`
- `characterStats`
- `inventoryOrResource`
- `mapTopology`
- `chapterLifecycle`
- `playerChoice`
- `combatOutcome`

任何 request 标记触碰这些范围时，`NoNameControlledOutputInterface` 都会返回 `Reject`。

## 接口草案

当前新增 `src-tauri/src/noname_output_interface.rs`，核心对象包括:

- `NoNameControlledOutputPolicy`
- `NoNameControlledOutputRequest`
- `NoNameControlledOutputReview`
- `NoNameControlledOutputInterface`

核心流程:

1. 上游 Agent 或 adapter 构造 `NoNameControlledOutputRequest`。
2. `NoNameControlledOutputInterface.review(...)` 检查输出类型、内容长度、禁区和是否需要人审。
3. 返回 `Allow / Reject / NeedsReview`。
4. V1 不直接执行 apply，只作为后续接入前的安全门面。

当前 V2 切片已把 review 结果接入 `NoNameRuntime` trace：`assisted` 预检通过后会记录每个 apply scope 的 controlled output review，其中 `PlotTextHint` 仍保持 `NeedsReview`，不会自动接管最终剧情正文。

当前 V2 后续切片已把 A3 角色上下文中的 `forbiddenScopes` 映射到 `NoNameControlledOutputPolicy.forbiddenScopes`，并在 trace 的 `controlledOutputReviews[].policyForbiddenScopes` 中显式记录当次 review 使用的策略禁区。

当前前端调试台已为 `NeedsReview` 增加人工复核入口：开发者可以把单条 review 标记为“可进入高层 apply 设计”或“暂不应用”。该标记会通过 `mark_noname_controlled_output_review` 写回最近 trace，形成可追踪的 review intent；当人工批准 `PlotTextHint` 时，后端会记录 `review_intent_ready / awaiting_second_guardrail` 计划与执行日志，但不会直接触发剧情正文写入。

当前后续切片已新增 `resolve_noname_second_guardrail`：只处理已人工批准且处于 `awaiting_second_guardrail` 的 review，输出 `allow / reject / fallback` 决策，并写入 trace 的 apply plan、execution 与 transition log。`allow` 也只是进入“等待显式人工 apply 命令”的下一站，不会自动写最终剧情正文。

当前 T7-0.6 已新增 `apply_noname_manual_plot_text_hint`：只有在 `PlotTextHint` 已人工批准、二次护栏 `allow`、并且调用方再次提交 `chapterIndex / segmentIndex / expectedSegmentText` 快照一致时，才会把正文提示写入当前剧情段落；写入后 trace 会记录 `manual_apply / manual_plot_text_applied / manual_apply:plot_text_hint`，用于区分“人工显式写入”和普通低风险自动 apply。

当前 T7-0.7 已补前端显式人工 apply 预览：调试台会展示当前段落的“写入前 / 写入后”，在段落已经包含 NoName 标记或 trace 已记录 `manual_plot_text_applied` 时禁用按钮；后端返回 stale snapshot / 重复写入 / 章节变化等错误时，前端会转成更明确的中文提示。

当前 T7-3 已补 apply lifecycle 可视化基线：前端会从 trace 的 plan、execution、review、transition 与 fallback 字段推导“提案阶段 / Apply 预检 / 低风险输出 / 人工复核 / 二次护栏 / 人工写入 / 回退”进度，并统一展示到调试台、复制摘要和 Info 调试文本。

当前 T7-1 第七切片已新增 `noname_apply.rs` reviewed apply runtime：人工批准、二次护栏、scope 匹配、快照校验、剧情写入和 trace 留痕已从 `tauri_commands.rs` 抽出；前端显式写入改走通用命令 `apply_noname_reviewed_output(scope=plotTextHint)`，旧命令保留兼容；`ChapterSummaryHint`、`OptionBiasHint` 与 `PlotAugmentationHint` 也已接入同一 runtime。前两者分别要求提交章节摘要快照、诊断提示快照一致后才写入对应低风险提示层；`PlotAugmentationHint` 要求提交 pending augmentation 列表快照一致后，只写入 `pending_plot_augmentation_hints`。下一轮生成时，pending augmentation 会以“非最终、可忽略、不得直接改写状态”的安全上下文进入 plot prompt；只有 plot_engine 结果真正落地、没有预设回退且未被双通道叙事覆盖时才按快照清空。

## V1 决策规则

- 空 request id、空 title、空 content 会被拒绝。
- 不在 allowlist 内的输出类型会被拒绝。
- 内容超过 `maxContentChars` 会被拒绝。
- 触碰禁区会被拒绝。
- `sceneAugmentation -> PlotTextHint` 在 V1 中返回 `NeedsReview`。
- 安全的 `recapNote` 可以返回 `Allow`。

## V1 不做什么

- 不改 `execute_player_action`。
- 不改 `tauri_commands.rs`。
- 不改 `noname_runtime.rs`。
- 不直接写入最终剧情正文。
- 不修改角色属性、物品、地图、章节生命周期或战斗结算。

## 验证场景

当前最小测试覆盖:

- 安全的 recap note 可以通过 review。
- 触碰 canon world fact 的 request 会被拒绝。
- scene augmentation 映射到 plot text hint 时需要人工复核。
- policy 明确列出 allowlist 和 forbidden scopes。
- A3 role context 的 `forbiddenScopes` 可以映射为 controlled output policy，并随 review trace 输出。
- 前端调试台可以对 `NeedsReview` review 做人工标记，并在复制摘要里体现待复核/通过/拒绝数量。
- 后端可记录 `humanReviewDecision / humanReviewedAt / humanReviewNote`，并把人工复核 intent 写入 trace 状态迁移日志。
- 人工批准后的 `PlotTextHint` 会进入二次 guardrail / apply planner 等待队列，trace 会记录 `review_intent_ready` 与 `awaiting_second_guardrail`，但不会直接 apply 到正文。
- `resolve_noname_second_guardrail` 可记录 `second_guardrail_allow / second_guardrail_reject / second_guardrail_fallback`，用于完整表达二次护栏决策。
- `apply_noname_manual_plot_text_hint` 可在人工批准 + 二次护栏 allow + 正文快照一致时显式写入 `PlotTextHint`，并拒绝陈旧段落、重复 NoName 标记和不匹配的章节。
- 前端调试台可在写入前展示差异预览，并对重复应用 / stale snapshot 给出明确提示。
- apply lifecycle 可视化可以稳定表达 apply / review / guardrail / manual apply / fallback 当前状态。
- reviewed apply runtime 已提供统一入口，当前覆盖 `PlotTextHint`、`ChapterSummaryHint`、`OptionBiasHint` 与 `PlotAugmentationHint`；其中低风险提示层已通过 `plot_engine` 入口落地，non-final plot augmentation 已进入 pending 提示列表并可被下一轮 plot prompt 安全消费。

## 后续建议

下一步可以继续:

1. 继续观察 pending augmentation 被消费后的生成质量，必要时增加更细的“已消费/保留原因”trace 记录。
2. 将更多输出类型接入现有 lifecycle 可视化与测试基线，继续保持高权重输出与最终剧情正文写入之间的人工/护栏边界。

## 2026-04-18 T7-1.5 Safe Output Update

- `PlotAugmentationHint` consumption is now observable without expanding its authority.
- Safe boundary remains unchanged: pending augmentation is only prompt context for the next generation and must not directly mutate final plot state, canon world facts, options, or scene structure.
- New trace outcomes: `pending_plot_augmentation_consumed` and `pending_plot_augmentation_retained`.
- Frontend lifecycle now surfaces this safe-output boundary as `剧情增强消费`, making it clear whether the non-final hint was consumed or deliberately retained.
