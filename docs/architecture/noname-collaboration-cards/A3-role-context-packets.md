# A3 任务卡: 角色差异化上下文包

状态: 未开始
优先级: 高
任务量: 中
任务联系度: 高

## 任务目标

在现有 `Context Packet` 骨架基础上，支持不同 `NoName` 角色拿到不同的上下文切片，而不是所有角色共享同一种上下文包。

目标是为后续多角色扩展打基础，但不要求本次就完成多 Agent 协作主链。

## 建议范围

建议本次至少覆盖:

- `Director` 上下文包细化
- `WorldCurator` 上下文包初版
- `NpcIntent` 上下文包初版

建议关注的上下文维度:

- 角色目标
- 当前场景重点
- 世界设定片段
- 角色关系
- 可见限制与禁止越权信息

## 建议新增或修改文件

- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_roles.rs`

如果需要，也可新增:

- `src-tauri/src/noname_context_packet.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`
- `src/stores/gameStore.ts`

原因:

- 本任务是“上下文建模”任务，不是“执行主链重构”任务
- 角色上下文应先自洽，再考虑接入主链

## 可接受的接入方式

推荐:

- 新增角色专属 builder 或策略分发
- 用类型区分不同角色的上下文字段
- 优先完成纯后端 builder 和测试

不推荐:

- 一上来强行接进当前回合主链
- 直接让前端依赖这些新上下文结构

## 交付标准

满足以下条件即可视为完成:

- 至少支持 3 种角色上下文包
- 各角色上下文内容有明确差异，而不是同字段换名字
- 有统一 builder 入口或分发机制
- 有最小单元测试，验证不同角色得到的上下文不同
- 有简短说明，说明哪些信息可见、哪些信息应受限

## 验证命令

建议最小验证:

```powershell
cargo test noname_context -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- 角色上下文类型定义
- builder / dispatcher
- 单元测试
- 一页简短说明，写清角色可见边界

## 备注

这是很适合协作者做的“高价值但低冲突”任务。它会直接服务未来的多角色 Agent，但暂时不会和当前 `T7 apply` 主线打架。
