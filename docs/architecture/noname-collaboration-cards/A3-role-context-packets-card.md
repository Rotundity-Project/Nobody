# 任务卡片 A3

任务名: 角色差异化上下文包
状态: 未开始
建议优先级: 高

## 目标

在现有 `Context Packet` 基础上，支持不同角色 Agent 获得不同结构和权重的上下文。

## 建议范围

- 细化 `Director` 上下文
- 新增 `WorldCurator` 上下文初版
- 新增 `NpcIntent` 上下文初版
- context builder 支持按 role 分支构建

## 建议涉及文件

- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_roles.rs`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`

## 交付标准

- 至少支持 3 种 role 的差异化上下文
- 不同 role 的上下文字段或取数策略能看出差异
- token budget 不被破坏
- 有对应 context builder 单元测试

## 验证命令

```powershell
cargo test noname_context_builder -- --nocapture
cargo test noname_roles -- --nocapture
```

## 备注

- 这项任务目前还未正式启动，非常适合协作者独立推进
