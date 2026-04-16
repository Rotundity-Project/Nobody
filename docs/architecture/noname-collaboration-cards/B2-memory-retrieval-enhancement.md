# B2 任务卡: 记忆检索增强

状态: V1 已完成
优先级: 中
任务量: 中到大
任务联系度: 中

## 任务目标

在现有 `NoName` 记忆层基础上增强检索能力，让系统能按角色、地点、目标和关键词更稳定地召回相关记忆，并支持更合理的排序策略。

本任务重点是“检索质量”和“结构化召回”，不是引入重型外部向量系统。

## 建议范围

建议优先覆盖:

- `by_actor`
- `by_location`
- `by_goal`
- `by_keyword`

建议排序维度:

- relevance
- recency
- importance

## 建议新增或修改文件

- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_memory_store.rs`

如有必要，可新增:

- `src-tauri/src/noname_memory_query.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`

原因:

- 本任务是检索层增强
- 不应与当前主线 apply 执行耦合

## 可接受的接入方式

推荐:

- 独立 retrieval service
- query object + ranking pipeline
- 与 note store / memory store 松耦合接入

不推荐:

- 直接在 context builder 里写死检索规则
- 直接把排序逻辑散落到多个角色实现里

## 交付标准

满足以下条件即可视为完成:

- 支持至少 3 种结构化检索入口
- 有基础排序策略
- 有最小测试集验证召回与排序行为
- 检索接口可被后续 context builder 直接复用
- 附带一段说明，说明排序原则和使用限制

## 验证命令

建议最小验证:

```powershell
cargo test noname_memory_retrieval -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- retrieval service
- query / ranking 结构
- 最小测试
- 说明文档

## 备注

这项任务和 `A1/A2` 有联动，但可以单独推进。完成后会显著提升上下文工程的可用性。
