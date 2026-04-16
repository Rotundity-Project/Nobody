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
