# 任务卡片 A1

任务名: 记忆压缩与长期整理模块
状态: V1 已完成
建议优先级: 高

## 目标

为 `NoName Agent` 新增记忆压缩能力，解决长历史、章节历史和 trace 历史累积后上下文过重的问题。

## 建议范围

- 新增 `turn compaction`
- 新增 `chapter compaction`
- 新增 `trace compaction`
- 将压缩结果接入 `memory manager` 或 `note store` 可复用结构

## 建议涉及文件

- `src-tauri/src/noname_memory_compaction.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_note_store.rs`
- 如有必要，可补充 `docs/architecture/` 下说明文档

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`

## 交付标准

- 至少提供一套可调用的 compaction 接口
- 能把回合级长文本压成短摘要
- 能把章节级事件压成章节摘要或 unresolved threads
- 能把 trace 噪声压成短诊断摘要
- 至少补 3 个单元测试

## 验证命令

```powershell
cargo test noname_ -- --nocapture
```

## 备注

- 这项任务不要求直接接入当前 `T7 apply` 主线
- 优先产出独立模块和测试，后续由主线回接
