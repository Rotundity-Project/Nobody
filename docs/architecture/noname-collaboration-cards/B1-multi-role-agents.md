# B1 任务卡: 多角色 Agent 扩展

状态: V1 已完成
优先级: 中
任务量: 大
任务联系度: 中

## 任务目标

把当前以 `DirectorAgent` 为核心的结构，扩展为可支持多角色协作的 `NoName Agent` 体系，为后续世界设定维护、NPC 意图生成、战斗叙事等角色分工提供基础。

本任务不要求接入当前回合执行主链，只要求完成角色模型、职责边界和最小运行骨架。

## 建议范围

建议优先落地的角色:

- `WorldCuratorAgent`
- `NpcIntentAgent`
- `CombatNarratorAgent`

建议优先明确:

- 每个角色的职责
- 输入上下文
- 输出对象
- 与 `DirectorAgent` 的边界

## 建议新增或修改文件

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_graph.rs`
- `src-tauri/src/noname_agent_registry.rs`

如有必要，可新增:

- `src-tauri/src/noname_agents/world_curator.rs`
- `src-tauri/src/noname_agents/npc_intent.rs`
- `src-tauri/src/noname_agents/combat_narrator.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`

原因:

- 本任务是角色体系扩展，不是主链 apply 执行扩展
- 新角色应先在框架层和类型层自洽

## 可接受的接入方式

推荐:

- 先扩角色定义和注册机制
- 再补最小角色执行 stub
- 最后补角色间协作草图

不推荐:

- 一步到位接入所有主线流程
- 直接让多角色参与当前 `T7 assisted apply`

## 交付标准

满足以下条件即可视为完成:

- 至少新增 2 到 3 个角色定义
- 每个角色有清晰职责与输入输出描述
- 存在最小 registry 或 role dispatch 机制
- 有最小测试或示例，证明角色可被注册和调用
- 附带一页简短设计说明

## 验证命令

建议最小验证:

```powershell
cargo test noname_roles -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- 角色定义
- 注册机制
- 最小执行 stub
- 单元测试
- 职责边界说明

## 备注

这项任务量偏大，不建议作为第一个协作任务，但很适合有 Agent 框架经验的协作者独立推进。
