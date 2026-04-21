# NoName Protocol Runtime V1

更新时间: 2026-04-14
对应任务: `B4-protocol-communication-enhancement`

## 目标

把当前 `NoName` 协议从“只有对象模型”推进到“能表达行为协议和本地生命周期”的阶段。

## 当前补充内容

- 扩展 `NoNameTaskStatus`
  - `pending`
  - `queued`
  - `running`
  - `completed`
  - `failed`
  - `cancelled`
  - `timed_out`

- 扩展 `NoNameTaskLifecycle`
  - `attempt`
  - `timeout_ms`
  - `last_error`
  - `cancellation_reason`

- 扩展 agent message kind
  - `taskRequest`
  - `delegation`
  - `status`
  - `result`
  - `cancel`
  - `timeout`
  - `error`

- 新增 tool envelope
  - `request`
  - `result`
  - `error`
  - `timeout`

- 新增本地 `NoNameProtocolRuntime`
  - 记录 task state
  - 接受 agent message
  - 执行本地 tool roundtrip
  - 支持 cancel / timeout

## 生命周期边界

当前允许的主路径:

1. `pending -> queued`
2. `queued -> running`
3. `running -> completed / failed / cancelled / timed_out`

当前不允许:

- `pending -> completed`
- 终态重新回到运行态
- 任意跳过 `queued/running` 直接进入完成态

## 当前限制

- 仍然是本地 stub，不是实际跨进程通信
- 已最小接入 `NoNameRuntime` 的 observe fan-out；尚未接入 assisted apply 主执行链
- tool envelope 目前优先覆盖 request / result / error 语义，资源和 prompt 仍保持原有轻量接口

## 后续建议

后面如果继续推进，建议顺序是:

1. 用 protocol runtime 驱动多角色 observe fan-out
2. 再考虑把 tool/resource/prompt 都统一挂到协议 runtime
3. 最后再评估是否需要真实 transport 或跨 runtime 适配

## 2026-04-19 T8-1 Update

- `NoNameRuntime` 的多角色 observe fan-out 已开始使用 role context specialization：每个非 Director 角色会先生成 `NoNameRoleContextPacket`，再压回现有 agent registry 可读取的 `NoNameContextPacket`。
- protocol delegation payload 现在会携带 `roleGoal / sceneFocus / forbiddenScopes`，让协议层 task lifecycle 能看见本次分发使用的角色目标与边界。
- assisted apply 主执行链仍保持隔离；本切片只增强 observe fan-out 的上下文分发质量。

## 2026-04-19 T8-2 Update

- multi-role observe prompt templates now require `roleGoal` and `forbiddenScopes`, so role boundaries are visible to the local prompt pipeline rather than only to protocol delegation payloads.
- observe tool calls also receive the same role boundary fields in args, preparing the later move toward protocol-runtime-managed tool envelopes.
- The current implementation still keeps tool/resource/prompt execution local and lightweight; assisted apply authority is unchanged.

## 2026-04-19 T8-3 Update

- Runtime fan-out now passes `NoNameRoleContextPacket` directly into `NoNameAgentRegistry`, moving flattened-context compatibility out of `NoNameRuntime`.
- This keeps protocol lifecycle orchestration focused on task dispatch while the registry owns role-agent compatibility.

## 2026-04-19 T8-4 Update

- Role context summaries now flow from observe fan-out into trace related observations, making protocol/debug views show role goals, scene focus, and forbidden scopes alongside each role's proposal.

## 2026-04-19 T8-6 Update

- The observe fan-out trace path now also carries `noteTypeHits`, so protocol-facing debug output can show which structured note types were selected for each role context.
- This keeps protocol runtime observability aligned with the role-context ranking that now happens before dispatch.

## 2026-04-19 T8-7 Update

- Protocol-facing related observations now include compact context-source stats and token budget usage alongside role goals and note hits.
- This extends the observe fan-out audit trail without moving tool/resource/prompt execution authority into the protocol runtime.

## 2026-04-20 T8-8 Update

- Protocol-facing related observations now also include compact role-context slice deltas, so debug output can show per-section clipping such as `recentSignals:5->3`.
- This remains an observe-only audit enhancement; protocol runtime authority boundaries are unchanged.
