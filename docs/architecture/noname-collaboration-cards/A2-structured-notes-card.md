# 任务卡片 A2

任务名: Structured Notes / Narrative Notes 增强
状态: 未开始
建议优先级: 高

## 目标

把当前 `NarrativeMemory` 从基础骨架推进成真正可用的结构化笔记系统。

## 建议范围

- note type 扩展
- note 生命周期管理
- 章节结束时的 note 整理
- unresolved thread 与 character arc 的最小支持

## 建议涉及文件

- `src-tauri/src/noname_note_store.rs`
- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_store.rs`
- `src-tauri/src/noname_memory_manager.rs`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src/components/InfoTabsDialog.vue`
- `src/stores/gameStore.ts`

## 交付标准

- 至少支持以下 note type
  - `goal`
  - `conflict`
  - `foreshadowing`
  - `unresolved_thread`
  - `character_arc`
- note 至少支持以下状态
  - `active`
  - `resolved`
  - `archived`
- 提供章节结束时整理 note 的最小接口
- 提供相应单元测试

## 验证命令

```powershell
cargo test noname_ -- --nocapture
```

## 备注

- 这项任务与 `NoName` 主线高度互补
- 但应避免直接改当前 apply / planner 主线
