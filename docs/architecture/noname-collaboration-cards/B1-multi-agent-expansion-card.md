# 任务卡片 B1

任务名: 多角色 Agent 扩展
状态: 未开始
建议优先级: 中

## 目标

在 `DirectorAgent` 之外，扩展更多 `NoName Agent` 角色，为后续多角色协作打基础。

## 建议范围

- 新增 `WorldCuratorAgent`
- 新增 `NpcIntentAgent`
- 可选新增 `CombatNarratorAgent`
- 保持本地骨架风格，不要求立即接入主链

## 建议涉及文件

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_prompts.rs`
- `src-tauri/src/noname_tools.rs`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`

## 交付标准

- 至少新增 2 个可构造角色
- 每个角色都有最小 prompt 或输出结构
- 每个角色有最小单元测试
- 不破坏当前 `DirectorAgent`

## 验证命令

```powershell
cargo test noname_roles -- --nocapture
```

## 备注

- 当前主线尚未正式启动多角色协作，这项很适合独立支线
