# A2 任务卡: Structured Notes / Narrative Notes 增强

状态: 未开始
优先级: 高
任务量: 中
任务联系度: 高

## 任务目标

把 `NoName` 当前的叙事记忆骨架推进成真正可用的结构化笔记层，让系统能持续记录目标、冲突、伏笔与未决线程，而不是只保留零散文本。

这项任务应服务于后续的记忆整理、上下文召回和长时程连续性。

## 建议范围

建议优先补齐:

- note type 扩展
- note 生命周期管理
- 章节结束或关键事件后的整理接口

推荐的 note 类型:

- `goal`
- `conflict`
- `foreshadowing`
- `unresolved_thread`
- `character_arc`

推荐的状态:

- `active`
- `resolved`
- `archived`

## 建议新增或修改文件

- `src-tauri/src/noname_note_store.rs`
- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_store.rs`

如有必要，可新增:

- `src-tauri/src/noname_note_types.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`
- `src/components/InfoTabsDialog.vue`

原因:

- 本任务属于记忆层与结构化数据层
- 不应通过主链 UI 或 apply 逻辑来驱动设计

## 可接受的接入方式

推荐:

- 以 `note store` 或 `note service` 方式提供接口
- 为后续章节整理、记忆压缩、上下文构建保留读取入口

不推荐:

- 把 note 状态管理写死在某个具体 Agent 角色里
- 为了演示效果直接耦合到当前调试面板

## 交付标准

满足以下条件即可视为完成:

- 支持至少 5 类结构化 note
- note 具备明确的生命周期状态
- 可以新增、更新、关闭或归档 note
- 至少提供一个“章节结束整理”或“阶段整理”接口
- 至少补充一组单元测试，验证 note 生命周期
- 代码层面能被后续 `memory compaction` 或 `context builder` 复用

## 验证命令

建议最小验证:

```powershell
cargo test noname_note -- --nocapture
```

兜底验证:

```powershell
cargo test noname_ -- --nocapture
```

## 交付物建议

建议协作者提交:

- note 数据结构
- note store / service
- 生命周期测试
- 一段 note 类型与状态设计说明

## 备注

这项任务和 `A1` 联系很强，但可以分开开发。`A2` 完成后，后续 `compaction` 和 `context recall` 会更容易做对。
