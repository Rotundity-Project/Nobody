# 任务卡片 B2

任务名: 记忆检索增强
状态: 未开始
建议优先级: 中

## 目标

在当前 SQLite / 内存骨架基础上，增强 `NoName` 的记忆检索和排序能力。

## 建议范围

- 增强 `by_actor`
- 增强 `by_location`
- 增强 `by_goal`
- 增强 `by_keyword`
- 增强 relevance / recency / importance 排序

## 建议涉及文件

- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_memory_store.rs`
- `src-tauri/src/noname_memory_manager.rs`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_trace.rs`

## 交付标准

- 至少补 3 种检索维度
- 至少实现 1 套排序增强逻辑
- 提供相应 retrieval 单元测试

## 验证命令

```powershell
cargo test noname_memory_retrieval -- --nocapture
```

## 备注

- 这项任务主要在记忆层内部展开，主线耦合较低
