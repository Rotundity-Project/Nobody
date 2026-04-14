# 任务卡片 B3

任务名: 独立 Agent 调试台
状态: 未开始
建议优先级: 高

## 目标

为 `NoName Agent` 新增独立调试台，而不是继续堆在当前信息抽屉里。

## 建议范围

- 新增独立调试面板或控制台
- 支持查看 trace 列表
- 支持查看 proposal / apply plan / apply execution
- 支持开发模式入口

## 建议涉及文件

- `src/components/AgentTracePanel.vue`
- `src/components/NoNameDebugConsole.vue`
- `src/components/GameInfoCenterDialog.vue`
- `src/stores/gameStore.ts`

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`

## 交付标准

- 至少有一个独立组件承载 NoName 调试内容
- 不破坏当前 `GameInfoCenterDialog`
- 至少补 2 个前端测试

## 验证命令

```powershell
npm run test -- --run src/components/__tests__/InfoTabsDialog.test.ts src/components/__tests__/GameInfoCenterDialog.test.ts src/stores/__tests__/gameStore.test.ts
```

## 备注

- 如果需要触碰 `gameStore.ts`，请只做只读展示增强，避免改主链状态同步逻辑
