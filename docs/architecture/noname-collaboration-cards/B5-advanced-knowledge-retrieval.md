# B5 任务卡: 外部知识与高级检索后端

状态: 未开始
优先级: 中低
任务量: 大
任务联系度: 低到中

## 任务目标

为未来 `Nobody` 的 lore / wiki / 外设定集接入准备外部知识检索能力，包括向量检索、图检索或外部 lore RAG 的预留接口与最小原型。

本任务属于中长期支线，不要求立即服务当前 `T7` 主线。

## 建议范围

建议任选其一先做原型:

- 向量检索
- 图检索
- 外部 lore RAG

建议优先完成:

- 接口定义
- 数据源抽象
- 最小召回流程
- 本地 mock 或轻量 demo

## 建议新增或修改文件

优先新增:

- `src-tauri/src/noname_knowledge_store.rs`
- `src-tauri/src/noname_knowledge_retrieval.rs`

如有必要，可新增:

- `docs/architecture/noname-knowledge-retrieval-v1.md`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`

原因:

- 这项任务距离当前主线较远
- 应以独立原型方式推进，而不是插入执行主链

## 可接受的接入方式

推荐:

- 先定义 retrieval provider 接口
- 再接一个最小 demo backend
- 用文档说明未来如何与 `memory/context` 系统接轨

不推荐:

- 直接引入重型依赖并强耦合到主链
- 直接改变当前本地记忆系统的核心模型

## 交付标准

满足以下条件即可视为完成:

- 有至少一类高级检索后端原型
- 有统一 provider 或 adapter 接口
- 有最小 demo 或测试
- 有文档说明使用方式、限制和未来接入点

## 验证命令

建议最小验证:

```powershell
cargo test noname_knowledge -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- provider 接口
- 原型实现
- 最小测试或 demo
- 设计说明文档

## 备注

这项任务不急，但很适合独立研究型协作者并行推进，不会影响当前主线稳定性。
