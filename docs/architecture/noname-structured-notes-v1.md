# NoName Structured Notes V1

更新时间: 2026-04-16
对应任务: `A2-structured-notes`

## 目标

把 `NoName` 的叙事记忆从简单文本列表推进为可维护的结构化 notes 层，让目标、冲突、伏笔、未决线索和人物弧光可以持续被记录、关闭、归档和章节整理。

这版仍保持在记忆层，不接入 `tauri_commands`、`noname_runtime` 或前端调试面板。

## Note 类型

当前 `NoNameNarrativeNoteType` 支持:

- `goal`
- `conflict`
- `foreshadowing`
- `unresolvedThread`
- `characterArc`

## Note 状态

当前 `NoNameNarrativeStatus` 支持:

- `active`
- `resolved`
- `archived`

默认检索仍优先返回 active notes，避免已解决线索反复污染运行上下文。章节整理和长期记忆整理可以通过专门接口读取归档结果。

## Store 能力

`NoNameNoteStore` 当前提供:

- `upsert`
- `get`
- `list_all`
- `list_active`
- `list_by_chapter`
- `list_by_status`
- `update`
- `resolve`
- `close`
- `archive`
- `review_chapter`
- `organize_chapter_end`

其中 `organize_chapter_end` 会把当前章节中 `resolved` 的 note 自动转为 `archived`，并返回 `NoNameChapterNoteReview`。

## 章节整理输出

`NoNameChapterNoteReview` 会返回:

- `chapterIndex`
- `activeNoteIds`
- `archivedNoteIds`
- `goalNoteIds`
- `conflictNoteIds`
- `foreshadowingNoteIds`
- `unresolvedThreadNoteIds`
- `characterArcNoteIds`
- `carriedForwardCount`
- `archivedFromResolvedCount`
- `updatedAt`

这让章节结束时可以清楚知道哪些线索继续带入下一章，哪些线索已经归档。

## Memory Manager 接入

`NoNameMemoryManager` 当前提供最小 facade:

- `update_note`
- `resolve_note`
- `archive_note`
- `organize_chapter_notes`
- `notes_by_chapter`

这些接口会保持 `NoNameNoteStore` 与 narrative memory store 的同步，方便后续 `memory compaction` 和 `context builder` 复用。

## 当前限制

- 当前没有接 UI，也没有直接接游戏主线。
- `close` 目前等价于 `resolve`，后续如需要可以拆成更细状态。
- 章节整理只做生命周期归档和结构化 review，不自动生成新剧情内容。

## 后续建议

下一步可以继续:

1. 让 A1 compaction 在生成 summary 时自动创建或更新相关 note。
2. 让 A3 role context packets 针对不同角色读取不同 note 类型。
3. 视需要补 note merge / split / supersede 语义，避免长线剧情里 note 过多。
