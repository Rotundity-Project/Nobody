# B4 任务卡: 真实协议/通信层增强

状态: V1 已完成
优先级: 中
任务量: 大
任务联系度: 中

## 任务目标

把当前本地版 `NNCP-T / NNCP-A` 从对象模型和骨架定义，推进到更真实的通信层与任务生命周期层，为后续多 Agent 协作提供更稳定的协议基础。

这项任务不要求连到真实外部网络，但应让本地协议从“数据结构”走向“行为协议”。

## 建议范围

建议优先推进:

- task lifecycle
- agent-to-agent message envelope
- tool request / tool result envelope
- error / timeout / cancellation 基础语义

## 建议新增或修改文件

- `src-tauri/src/noname_protocol_agent.rs`
- `src-tauri/src/noname_protocol_tool.rs`
- `src-tauri/src/noname_protocol_types.rs`

如有必要，可新增:

- `src-tauri/src/noname_protocol_runtime.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`

原因:

- 当前主线重点不是通信层
- 通信协议应先在框架层独立演进

## 可接受的接入方式

推荐:

- 先补协议类型和状态机
- 再补本地 runtime stub
- 最后做最小 agent-to-agent 示例

不推荐:

- 一开始就耦合到当前 `DirectorAgent` 主链调用
- 把通信语义散落到现有 runtime 逻辑里

## 交付标准

满足以下条件即可视为完成:

- 有明确的 message envelope 定义
- 有最小 task lifecycle 定义
- 支持 tool request / tool result 基础往返
- 有至少一组协议级单元测试
- 有一页简短说明，描述协议边界和未来扩展点

## 验证命令

建议最小验证:

```powershell
cargo test noname_protocol -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- 协议类型
- 本地 runtime stub
- 生命周期测试
- 协议说明文档

## 备注

这项任务适合对 Agent 框架和协议设计比较熟悉的协作者，不建议作为第一项入门任务。
