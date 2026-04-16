# A1 任务卡: 记忆压缩与长期整理模块

状态: V1 已完成
优先级: 高
任务量: 中
任务联系度: 高

## 任务目标

为 `NoName Agent` 新增可独立演化的记忆压缩模块，用于处理长历史、章节历史与 trace 历史的结构化压缩，降低上下文膨胀风险，并为后续长时程一致性提供基础。

这项任务的目标不是直接改当前 `T7 apply` 主线，而是补上记忆工程里尚未落地的 `compaction` 能力。

## 建议范围

建议优先覆盖以下三类压缩:

- turn compaction
- chapter compaction
- trace compaction

建议输出形式:

- 压缩摘要对象
- 保留关键实体、冲突、未决线索、角色关系
- 支持后续被上下文构建器再次读取

## 建议新增文件

- `src-tauri/src/noname_memory_compaction.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_note_store.rs`

如果实际实现需要，也可以补充:

- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_store.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`
- `src/stores/gameStore.ts`

原因:

- 这些文件属于当前本机主线推进中的 apply 编排层
- 本任务应停留在记忆层，不应反向耦合主链入口

## 可接受的接入方式

推荐通过以下方式接入:

- 新增独立 compaction service
- 通过现有 memory manager 或新增 facade 暴露只读/只写接口
- 先提供最小集成点，不要求直接进入回合主链

不推荐:

- 直接把 compaction 逻辑塞进 `tauri_commands.rs`
- 直接在执行主链里做大量历史清洗

## 交付标准

满足以下条件即可视为完成:

- 存在独立的 compaction 模块，不依赖 `T7 apply` 主链
- 至少支持 turn / chapter / trace 三类压缩中的两类落地
- 压缩结果是结构化对象，而不是只有纯字符串摘要
- 压缩结果保留关键字段，例如角色、地点、目标、冲突、未解决事项
- 至少补充一组单元测试，验证压缩输出稳定
- 提供简短文档或模块注释，说明输入、输出与后续接入点

## 验证命令

建议最小验证:

```powershell
cargo test noname_memory -- --nocapture
```

如果测试命名尚未成型，可接受:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交以下内容:

- 代码实现
- 单元测试
- 一段简短说明，描述压缩策略和后续可接入点

## 备注

这是最适合优先分给协作者的任务之一，因为它:

- 边界清晰
- 与主线互补
- 不容易和当前本机主链发生冲突
