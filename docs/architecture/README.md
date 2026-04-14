# Architecture Docs Index

本目录保存 `Nobody` 的架构、领域模型与 `NoName Agent` 相关正式设计文档。

## 当前状态

截至 `2026-04-09`：

- `NoName Agent V1` 基础闭环已完成
- 当前代码进度已经超过原始 `T6` 计划，进入 `assisted skeleton`
- 如果要继续推进实现，优先阅读 `noname-v1-blueprint.md`、`noname-v1-task-list.md` 与 `noname-t7-file-checklist.md`

## 推荐阅读顺序

### 基础架构

1. `../ARCHITECTURE.md`
2. `domain-model-v2.md`

### NoName Agent 主线

1. `noname-agent-v1.md`
2. `noname-memory-context-v1.md`
3. `noname-framework-protocol-v1.md`
4. `noname-v1-blueprint.md`
5. `noname-v1-task-list.md`
6. `noname-t1-file-checklist.md`
7. `noname-t7-file-checklist.md`
8. `noname-collaboration-handoff-20260413.md`
9. `noname-collaboration-cards/README.md`

### 历史补充

- `agent-framework-v1.md`

## 文件定位

- `domain-model-v2.md`
  - 领域实体、不变量、生命周期与当前模块映射。

- `noname-agent-v1.md`
  - NoName Agent 的总体框架、角色体系、状态图与模块边界。

- `noname-memory-context-v1.md`
  - 记忆工程、上下文工程、GSSC 流水线、长时程策略。

- `noname-framework-protocol-v1.md`
  - 框架分层、Capability 抽象、NNCP 协议体系。

- `noname-v1-blueprint.md`
  - 面向实现的 V1 蓝图、阶段目标、当前状态与下一阶段方向。

- `noname-v1-task-list.md`
  - 可执行的开发任务清单，现已补充 `T0-T6` 完成状态与 `T7 assisted skeleton` 说明。

- `noname-t1-file-checklist.md`
  - `T1 Core 类型与配置骨架` 的文件级实现清单，明确每个文件该放什么、先做什么、做到什么算完成。

- `noname-t7-file-checklist.md`
  - `T7 Assisted Skeleton` 的文件级实现清单，聚焦 proposal 进入受控应用分支前的模块拆解与落点。

- `noname-collaboration-handoff-20260413.md`
  - 面向协作者的任务拆分文档，明确本机已完成内容、主线进行中内容，以及“尚未开始、适合外包协作”的任务边界。

- `noname-collaboration-cards/README.md`
  - 面向协作者的任务卡片索引，一项任务一个文件，可直接转发。

- `agent-framework-v1.md`
  - 早期 Agent 接入提案，保留供追溯参考。

## 维护规则

- 与 `NoName Agent` 相关的新设计优先补充到对应专题文档。
- 如果内容跨越多个专题，先更新 `noname-v1-blueprint.md` 的执行计划，再回写细节文档。
- 如果内容进入实施层，优先更新 `noname-v1-task-list.md`。
- 如果内容已经下钻到文件级或提交级，补充到对应专项清单，如 `noname-t1-file-checklist.md` 与 `noname-t7-file-checklist.md`。
- 如果代码进度超出原始计划，先更新本索引中的“当前状态”，避免阅读者被旧计划误导。
- 废弃文档不直接删除，先在本索引中标记为“历史补充”。
