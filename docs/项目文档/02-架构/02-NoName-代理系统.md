# NoName 代理系统

## 1. 目标

`NoName` 是 `Nobody` 的 Agent 增强子系统，用来在不破坏主状态机和规则边界的前提下，提供：

- 剧情方向观察与建议
- 角色/世界/战斗等差异化视角补充
- 低风险提示注入
- 可 review、可追踪、可回退的受控应用流程

## 2. 当前已经实现的能力

截至当前 `main`，以下能力已经落地：

### 运行与模式

- `disabled`
- `observeOnly`
- `assisted`

### 主体模块

- `runtime`
- `trace`
- `graph`
- `guardrail`
- `memory manager`
- `memory compaction`
- `memory retrieval`
- `context builder`
- `role context packet`
- `controlled output interface`
- `reviewed apply`

### 角色与协作

- `Director`
- `WorldCurator`
- `NpcIntent`
- `CombatNarrator`
- `System`

### 前端支持

- trace 历史查看
- proposal、apply plan、apply execution 展示
- controlled output review 操作
- second guardrail 决策
- `NoName` 调试台

## 3. 当前真正的运行边界

`NoName` 现在并不能直接改写最终剧情结果。当前主线是：

1. 产出 proposal
2. 经过 apply preflight
3. 对受控输出做 review
4. 对高一层 apply 做人工批准
5. 经过 second guardrail
6. 在快照校验成功后显式写入安全范围

这说明 `NoName` 当前是“受控协作增强”，不是“自主改写剧情”。

## 4. 已知强项

- 结构清晰，模块边界已经成型
- trace 能力强，便于调试和审计
- 记忆、上下文、角色差异化都已经有实现骨架
- reviewed apply 已经不是文档概念，而是可运行链路

## 5. 当前主要问题

### 5.1 命令层仍然过重

`reviewed apply` 的核心逻辑虽已下沉到 `noname_apply.rs`，但命令编排仍强依赖 `tauri_commands.rs`。

### 5.2 Web mock 与后端容易漂移

当前 `webRuntime.ts` 仍然是简化 runtime，但第一轮关键对齐已经完成：

- `policyForbiddenScopes` 不再只保留极小子集
- controlled output review 已保留 `proposalId` 精确绑定语义

后续仍要继续跟随后端真实行为演进，避免新的 review / apply 字段再次漂移。

### 5.3 文档曾经明显落后于代码

旧版协作文档把多个已落地模块标成“未开始”，已不适合作为当前任务基线。

## 6. 当前结论

`NoName` 已经完成 `V1` 骨架并进入 `T7 reviewed apply runtime` 阶段。  
后续不建议立即扩大它对正文和主剧情状态的控制权，应该先完成：

- 命令层减压
- 前后端语义对齐
- 记忆与 structured notes 串联
- 协作文档重建
