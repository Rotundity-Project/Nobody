# 任务卡片 B4

任务名: 真实协议 / 通信层增强
状态: V1 已完成
建议优先级: 中

## 目标

把当前本地版 `NNCP-T / NNCP-A` 从对象模型推进到更真实的 task lifecycle 和消息语义层。

## 建议范围

- 扩展 task lifecycle
- 扩展 agent message 结构
- 增强 protocol trace 记录
- 不要求接入真实远程网络

## 建议涉及文件

- `src-tauri/src/noname_protocol_agent.rs`
- `src-tauri/src/noname_protocol_tool.rs`
- `src-tauri/src/noname_protocol_types.rs`
- `src-tauri/src/noname_capability_registry.rs`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`

## 交付标准

- task lifecycle 比当前对象骨架更完整
- 协议对象能表达更清晰的 request / result / state
- 有对应单元测试

## 验证命令

```powershell
cargo test noname_protocol -- --nocapture
cargo test noname_capability_registry -- --nocapture
```

## 备注

- 这项任务适合作为中期支线，不应阻塞当前 `T7`
