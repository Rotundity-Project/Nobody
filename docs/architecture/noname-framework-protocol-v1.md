# NoName Agent 框架搭建与通信协议设计文档 V1

更新时间: 2026-04-08
状态: 设计基线
适用项目: `Nobody`
关联文档:
- `docs/architecture/noname-agent-v1.md`
- `docs/architecture/noname-memory-context-v1.md`
- `docs/ARCHITECTURE.md`

## 1. 文档目标

本文档吸收 HelloAgents 第七章“构建你的 Agent 框架”和第十章“智能体通信协议”中适合 `Nobody` 的思想，回答两个问题：

- `Nobody` 为什么需要自建 `NoName Agent`，而不是直接依赖外部框架。
- `NoName Agent` 的通信协议应该如何设计，才能支持工具访问、Agent 协作和未来网络扩展。

## 2. 借鉴原则

参考 HelloAgents 的过程中，NoName Agent 采用“借思想，不照搬”的策略。

### 2.1 直接吸收的部分

- 轻量、可理解、可调试的自建框架思路
- 基于标准 API 的务实选择
- 渐进式构建能力，而不是一步到位做复杂全家桶
- 统一通信协议带来的标准化、互操作性和可扩展性
- 把工具通信、Agent 协作、网络发现分成不同层面来处理

### 2.2 不直接照搬的部分

HelloAgents 提出“除了 Agent 类，一切皆为工具”的极简抽象。这个思路很适合教学，但对 `Nobody` 不能完全照搬。

原因：

- `Nobody` 有强状态机
- `Nobody` 有强规则校验
- `Nobody` 有世界事实与实体约束
- `Nobody` 的记忆和上下文不是普通插件能力，而是底层运行时基础设施

因此，NoName Agent 更合适的原则是：

`除核心运行时、记忆层、上下文层、护栏层之外，一切外部能力都可抽象为 Capability。`

这比“万物皆工具”更贴合 `Nobody` 的引擎型产品结构。

## 3. HelloAgents 第七章对 Nobody 最有价值的启发

### 3.1 为什么要自建框架

HelloAgents 第七章最有价值的一点，不是“写一个新框架”，而是明确指出：

- 通用框架抽象层过多
- 版本变化快
- 黑盒严重
- 依赖复杂
- 很难针对具体领域做精细优化

对 `Nobody` 来说，这些问题都成立，而且更明显。因为 `Nobody` 是“桌面游戏引擎 + 叙事系统 + 状态系统”，不是一个通用聊天机器人。

因此，自建 `NoName Agent` 的意义在于：

- 保持对剧情、规则、世界状态的绝对控制权
- 保持高可观测性和可回放性
- 让 Agent 系统服从游戏引擎，而不是反过来让引擎迎合框架

### 3.2 轻量与教学友好

HelloAgents 强调“轻量、透明、渐进式学习”。

NoName Agent 也应该遵循这一点：

- V1 不引入重量级外部运行时
- V1 不强依赖外部网络智能体基础设施
- V1 先在 Rust 后端内部把 Runtime 跑通
- 所有关键对象都应该是可读、可测试、可追踪的

### 3.3 基于标准 API 的务实选择

HelloAgents 借 OpenAI 兼容 API 做统一 LLM 接口，这个思路对 `Nobody` 也非常适合。

现有 `Nobody` 已经有 `llm_service.rs`，后续 NoName Agent 不需要重新发明一套模型调用体系，而应当：

- 继续复用当前统一的 LLM 调用层
- 在上层加 Agent 角色、提示模板、输出契约和 Trace
- 把“模型供应商差异”隔离在 LLM adapter 下方

### 3.4 渐进式能力构建

HelloAgents 用章节逐步构建能力，这一点对 `Nobody` 非常重要。

NoName Agent 推荐的构建顺序仍然是：

1. Runtime 骨架
2. 单 Agent 角色
3. 记忆与上下文
4. 通信协议
5. 多 Agent 协作
6. 网络扩展

## 4. HelloAgents 第十章对 Nobody 最有价值的启发

HelloAgents 第十章把通信协议拆成三层：

- `MCP`：Agent 与工具通信
- `A2A`：Agent 与 Agent 协作
- `ANP`：Agent 网络中的发现与路由

这个三层拆法非常适合 `Nobody`，但要做本地化改造。

## 5. NoName Agent 的协议分层

建议为 `NoName Agent` 设计三层协议系统：

- `NNCP-T`：Tool Capability 协议层
- `NNCP-A`：Agent 协作协议层
- `NNCP-N`：Network / Registry 协议层

这里的 `NNCP` 指：

`NoName Communication Protocol`

### 5.1 NNCP-T

职责：

- 标准化 Agent 对工具、资源、提示模板的访问
- 屏蔽不同工具来源的调用差异
- 统一 schema、错误、能力发现、调用方式

可以理解为 `MCP 思想` 在 `Nobody` 内部的本地化版本。

### 5.2 NNCP-A

职责：

- 标准化 Agent 之间的点对点协作
- 支持任务协商、委托、结果返回、状态跟踪
- 适合 Director 和子代理之间通信

可以理解为 `A2A 思想` 的 NoName 版本。

### 5.3 NNCP-N

职责：

- 为未来跨进程、跨实例、跨远端的 Agent 服务发现做准备
- 不作为 V1 必做项
- V1 只定义接口和元数据，不实现真正分布式网络

可以理解为 `ANP 思想` 的预留层。

## 6. NoName Agent 框架分层

在 HelloAgents 的 `core / agents / tools` 三层基础上，NoName Agent 建议扩展为 6 层。

### 6.1 Core Layer

建议文件：

- `noname_runtime.rs`
- `noname_graph.rs`
- `noname_types.rs`
- `noname_config.rs`
- `noname_errors.rs`

职责：

- 定义运行时主循环
- 驱动状态图
- 持有全局配置
- 提供统一异常体系

### 6.2 Role Layer

建议文件：

- `noname_roles.rs`
- `noname_prompts.rs`

职责：

- 定义角色类型
- 定义角色输入输出契约
- 定义系统提示与角色边界

### 6.3 Capability Layer

建议文件：

- `noname_capability_base.rs`
- `noname_capability_registry.rs`
- `noname_tools.rs`
- `noname_resources.rs`
- `noname_prompt_catalog.rs`

职责：

- 把工具、资源、提示模板统一抽象为 capability
- 支持发现、注册、调用、鉴权、限流

### 6.4 Memory & Context Layer

建议文件：

- `noname_memory_manager.rs`
- `noname_context_builder.rs`
- `noname_note_store.rs`

职责：

- 管理记忆写入与检索
- 构建角色专属上下文
- 做长期压缩与结构化笔记

### 6.5 Guardrail Layer

建议文件：

- `noname_guardrails.rs`

职责：

- 统一调用一致性、数值、实体、补丁校验
- 形成 accept / reject / repair 结果

### 6.6 Protocol Layer

建议文件：

- `noname_protocol_types.rs`
- `noname_protocol_tool.rs`
- `noname_protocol_agent.rs`
- `noname_protocol_network.rs`

职责：

- 实现 NNCP-T / NNCP-A / NNCP-N
- 定义 envelope、task、capability descriptor、route descriptor

## 7. Capability First 设计

### 7.1 为什么用 Capability 而不是纯 Tool

HelloAgents 中“万物皆工具”的思路非常简洁，但 `Nobody` 需要区分：

- 会执行操作的东西
- 只是提供上下文的东西
- 只是提供提示模板的东西

因此，NoName Agent 更适合引入 `Capability` 这一层。

建议能力对象分三类：

- `ToolCapability`
- `ResourceCapability`
- `PromptCapability`

### 7.2 三类 Capability

#### ToolCapability

可执行动作，例如：

- 生成候选剧情
- 查询实体
- 提交世界 patch 候选
- 召回相关事件

#### ResourceCapability

可读取上下文资源，例如：

- 章节摘要
- 世界事实投影
- 角色档案
- 地图节点数据

#### PromptCapability

可复用提示模板，例如：

- DirectorAgent prompt
- WorldCuratorAgent prompt
- 压缩摘要 prompt

## 8. NNCP-T 设计

NNCP-T 是 NoName Agent 的工具与资源访问协议。

它借鉴了 MCP 的三个重要思想：

- 标准化 discovery
- 标准化 schema
- 不只支持 tools，也支持 resources 和 prompts

### 8.1 Capability Descriptor

```ts
type NoNameCapabilityDescriptor = {
  capabilityId: string
  kind: "tool" | "resource" | "prompt"
  name: string
  version: string
  description: string
  inputSchema?: Record<string, unknown>
  outputSchema?: Record<string, unknown>
  tags: string[]
  trusted: boolean
  localOnly: boolean
}
```

### 8.2 Tool Call

```ts
type NoNameToolCall = {
  callId: string
  capabilityId: string
  arguments: Record<string, unknown>
  traceId: string
  timeoutMs?: number
}
```

### 8.3 Tool Result

```ts
type NoNameToolResult = {
  callId: string
  ok: boolean
  content: string
  payload?: Record<string, unknown>
  errorCode?: string
  errorMessage?: string
}
```

### 8.4 Resource Read

```ts
type NoNameResourceRead = {
  capabilityId: string
  selector?: Record<string, unknown>
  traceId: string
}
```

### 8.5 Prompt Resolve

```ts
type NoNamePromptResolve = {
  capabilityId: string
  variables?: Record<string, unknown>
  traceId: string
}
```

### 8.6 Nobody 中的落地方式

V1 不需要实现完整远程 MCP 客户端，但建议把本地内部调用统一成 MCP 风格：

- `list_capabilities()`
- `call_capability()`
- `read_resource()`
- `resolve_prompt()`

这样后面要接真正的 MCP server，就不会推翻内部设计。

## 9. NNCP-A 设计

NNCP-A 是 NoName Agent 的 Agent-to-Agent 协议层。

它借鉴 A2A 的两个关键点：

- 对等通信思维
- 任务生命周期管理

但在 `Nobody` 中，我们不建议一开始做完全对等网络，而是做“运行时受控协作”。

### 9.1 核心对象

```ts
type NoNameAgentAddress = {
  role: string
  runtimeScope: "local" | "session" | "future-remote"
  agentId: string
}
```

```ts
type NoNameAgentMessage = {
  messageId: string
  traceId: string
  sessionId: string
  chapterIndex?: number
  sceneId?: string
  from: NoNameAgentAddress
  to: NoNameAgentAddress
  intent: string
  content: string
  payload?: Record<string, unknown>
  createdAt: number
}
```

### 9.2 Task 对象

```ts
type NoNameTask = {
  taskId: string
  traceId: string
  kind: string
  requester: string
  assignee: string
  objective: string
  input: Record<string, unknown>
  status: "created" | "negotiating" | "delegated" | "running" | "completed" | "failed" | "cancelled"
  result?: Record<string, unknown>
  error?: string
}
```

### 9.3 为什么需要任务生命周期

HelloAgents 在 A2A 里强调标准任务状态，这是非常有价值的。对于 `Nobody`，它可以直接解决：

- 子代理是否已接单
- 当前是否还在生成
- 是否被 guardrail 拒绝
- 是否已经 fallback
- 某个子任务有没有卡住

### 9.4 V1 推荐协作方式

V1 不建议开放自由群聊式多 Agent，而建议：

- `DirectorAgent -> WorldCuratorAgent`
- `DirectorAgent -> NpcIntentAgent`
- `DirectorAgent -> CombatNarratorAgent`

也就是：

- 主代理负责规划
- 子代理负责局部深挖
- 子代理只返回凝练摘要或提案

## 10. NNCP-N 设计

NNCP-N 是未来的网络协议层。

V1 不实现完整网络，但建议先定义以下概念：

- `AgentRegistryEntry`
- `CapabilityRoute`
- `DiscoveryQuery`
- `RemoteTrustLevel`

### 10.1 Registry Entry

```ts
type NoNameRegistryEntry = {
  agentId: string
  role: string
  endpoint?: string
  supportedCapabilities: string[]
  protocolVersions: string[]
  trustLevel: "local" | "trusted" | "external"
}
```

### 10.2 设计意义

这样做的目的不是 V1 就上分布式，而是提前保证：

- 本地协议对象不会把未来远程扩展堵死
- 后续若要接外部服务发现，不需要重写消息模型

## 11. 统一消息信封

为了把第七章的“消息系统”思想和第十章的“通信协议”思想合并，NoName Agent 建议统一使用一个总信封结构。

```ts
type NoNameEnvelope<T = Record<string, unknown>> = {
  envelopeId: string
  protocol: "NNCP-T" | "NNCP-A" | "NNCP-N"
  version: string
  traceId: string
  sessionId: string
  chapterIndex?: number
  sceneId?: string
  from: string
  to: string
  kind: string
  content: string
  payload: T
  meta: {
    createdAt: number
    priority?: number
    timeoutMs?: number
    requiresAck?: boolean
  }
}
```

### 11.1 为什么必须统一 Envelope

收益有 4 个：

- 方便追踪
- 方便测试
- 方便协议演进
- 方便前端做调试展示

## 12. 错误与异常体系

HelloAgents 第七章里把 `exceptions.py` 作为核心层的一部分，这一点很值得吸收。

NoName Agent 建议建立统一异常体系：

- `NoNameProtocolError`
- `NoNameCapabilityError`
- `NoNameGuardrailError`
- `NoNameContextError`
- `NoNameMemoryError`
- `NoNameNegotiationError`

并统一映射到：

```ts
type NoNameErrorPayload = {
  code: string
  message: string
  retryable: boolean
  stage: string
}
```

## 13. 配置体系

HelloAgents 第七章强调 `config` 是核心层的一部分，这对 `Nobody` 非常适用。

NoName Agent 建议新增 `noname_config.rs`，统一管理：

- agent mode
- protocol mode
- capability whitelist
- local / trusted / external trust policy
- memory write policy
- context token budgets
- timeout policy

## 14. 与 Nobody 现有结构的融合点

### 14.1 现有能力继续复用

- `llm_service.rs` 继续做模型调用中枢
- `memory_layers.rs` 和 `context_builder.rs` 继续作为记忆/上下文底层
- `plot_consistency.rs`、`state_patch_validator.rs`、`numeric_guard.rs` 继续做 guardrail
- `tauri_commands.rs` 继续做入口编排层

### 14.2 建议新增的 NoName 模块

- `noname_runtime.rs`
- `noname_graph.rs`
- `noname_protocol_types.rs`
- `noname_protocol_tool.rs`
- `noname_protocol_agent.rs`
- `noname_protocol_network.rs`
- `noname_capability_registry.rs`
- `noname_config.rs`
- `noname_errors.rs`

## 15. Nobody V1 推荐实现范围

### 15.1 必做

- Core Layer
- Role Layer
- Capability Registry
- NNCP-T 本地版
- NNCP-A 本地版
- Trace 与统一 Envelope

### 15.2 暂缓

- 真正远程 MCP client
- 真正远程 A2A server
- 完整 NNCP-N 网络发现
- 分布式 Agent 注册中心

## 16. 设计决策总结

### 16.1 从 HelloAgents 第七章吸收的内容

- 为什么要自建框架
- 轻量与透明优先
- 标准 API 优先
- 渐进式建设
- Core / Agent / Tool 分层思维

### 16.2 从 HelloAgents 第十章吸收的内容

- 工具通信与 Agent 协作要分层
- Tools / Resources / Prompts 都需要标准接口
- 任务生命周期需要标准化
- 协议应该先定义对象与状态，再逐步扩展传输方式

### 16.3 对 Nobody 的本地化改造

- 把 Tool 扩展成 Capability
- 把 A2A 改成 Runtime 受控协作
- 把 ANP 改成 V1 预留接口
- 把一切协议都纳入统一 Envelope + Trace + Guardrail 体系

## 17. 结论

HelloAgents 第七章和第十章给 `Nobody` 最有价值的不是某个具体 Python 类，而是两个核心思想：

- 自建框架要保持轻量、透明、渐进式演化
- 通信协议要按“工具访问、Agent 协作、网络发现”分层

因此，融合后的 `NoName Agent` 在 `Nobody` 中的最优形态是：

- 以 `Core + Role + Capability + Memory/Context + Guardrail + Protocol` 六层架构为骨架
- 以 `NNCP-T / NNCP-A / NNCP-N` 三层协议为通信体系
- 以统一 `Envelope + Trace + Task Lifecycle` 为运行时基础设施

这样设计，既保留了 `Nobody` 的引擎控制力，也吸收了 HelloAgents 在框架透明性和协议分层上的长处。

## 18. 参考资料

- HelloAgents 第七章《构建你的 Agent 框架》
- HelloAgents 第十章《智能体通信协议》
- `docs/architecture/noname-agent-v1.md`
- `docs/architecture/noname-memory-context-v1.md`
