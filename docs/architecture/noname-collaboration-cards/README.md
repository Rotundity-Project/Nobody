# NoName Agent 协作任务卡片

更新时间: 2026-04-14
状态: 可直接转发给协作者
来源: `noname-collaboration-handoff-20260413.md`

## 用途

本目录将 `NoName Agent` 的协作任务拆成“单任务单文件”的卡片格式。

适用场景:

- 需要把一项任务单独发给协作者
- 需要明确协作者的目标、边界与交付标准
- 需要避免协作者误改当前本机正在推进的主线文件

## 使用方式

建议按以下顺序分配:

1. `A1-memory-compaction.md`
2. `A2-structured-notes.md`
3. `A3-role-context-packets.md`
4. `A4-next-safe-output-interface.md`
5. `B3-agent-debug-console.md`
6. `B2-memory-retrieval-enhancement.md`
7. `B1-multi-role-agents.md`
8. `B4-protocol-communication-enhancement.md`
9. `B5-advanced-knowledge-retrieval.md`

说明:

- `A` 清单与当前 `NoName` 主线联系更高，但可以相对隔离开发
- `B` 清单更像独立支线，适合中长期并行推进
- 如果协作者不熟悉当前项目，优先从 `A1/A2/A3/B3` 开始

## 卡片列表

- [A1-memory-compaction.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/A1-memory-compaction.md)
  - 记忆压缩与长期整理模块
- [A2-structured-notes.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/A2-structured-notes.md)
  - Structured Notes / Narrative Notes 增强
- [A3-role-context-packets.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/A3-role-context-packets.md)
  - 角色差异化上下文包
- [A4-next-safe-output-interface.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/A4-next-safe-output-interface.md)
  - 更高一层受控输出接口预研
- [B1-multi-role-agents.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/B1-multi-role-agents.md)
  - 多角色 Agent 扩展
- [B2-memory-retrieval-enhancement.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/B2-memory-retrieval-enhancement.md)
  - 记忆检索增强
- [B3-agent-debug-console.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/B3-agent-debug-console.md)
  - 独立 Agent 调试台
- [B4-protocol-communication-enhancement.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/B4-protocol-communication-enhancement.md)
  - 真实协议/通信层增强
- [B5-advanced-knowledge-retrieval.md](/D:/Nobody/docs/architecture/noname-collaboration-cards/B5-advanced-knowledge-retrieval.md)
  - 外部知识与高级检索后端

## 统一协作规则

所有协作者默认遵守以下边界:

- 不直接重构 `execute_player_action` 主链
- 不直接改写当前 `T7` apply planner / executor 主入口
- 不直接调整已有 `NoName` trace 主结构，除非任务卡明确要求

默认不要碰的主线文件:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`
- `src/components/InfoTabsDialog.vue`
- `src/components/GameInfoCenterDialog.vue`
- `src/stores/gameStore.ts`
- `src/types/game.ts`

如果任务必须触碰这些文件:

- 优先通过新增小接口或新增模块接入
- 改动必须保持最小化
- 合并前应单独 review

## 统一验证建议

后端常用验证:

```powershell
cargo test noname_ -- --nocapture
```

前端常用验证:

```powershell
npm run test -- --run src/platform/webRuntime.test.ts src/components/__tests__/InfoTabsDialog.test.ts src/components/__tests__/GameInfoCenterDialog.test.ts src/stores/__tests__/gameStore.test.ts
```

如果任务只涉及文档或纯新增模块:

- 可只运行与该模块直接相关的最小测试集
- 不强制要求触发完整前后端全量回归
