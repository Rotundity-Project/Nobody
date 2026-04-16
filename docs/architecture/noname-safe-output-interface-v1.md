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

## 后续建议

下一步可以继续:

1. 把 A3 的 `forbiddenScopes` 映射到本接口的 forbidden scopes。
2. 为 `NeedsReview` 设计更完整的前端确认入口，让开发者手动确认是否进入更高层 apply。
3. 继续保持 `PlotTextHint` 与最终剧情正文写入之间的人工/护栏边界。
