# B3 任务卡: 独立 Agent 调试台

状态: V1 已完成
优先级: 中高
任务量: 中
任务联系度: 中

## 任务目标

新增一个更独立、更面向 `NoName Agent` 的调试控制台，而不是继续把所有信息都塞在当前信息抽屉中。

这项任务的重点是提升可观测性和开发效率，不是修改 `NoName` 主链逻辑。

## 建议范围

建议优先支持以下信息展示:

- mode
- proposal 状态
- apply scope
- target segment
- intended effect
- apply plan
- apply execution
- fallback / guardrail 结果

建议优先形态:

- 独立面板
- 独立对话框
- 可切换 tab 的开发者工具视图

## 建议新增或修改文件

- `src/components/AgentTracePanel.vue`
- `src/components/NoNameDebugConsole.vue`

如有必要，可少量接入:

- `src/components/GameInfoCenterDialog.vue`
- `src/stores/gameStore.ts`

## 不要碰的文件

默认不要重构:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/components/InfoTabsDialog.vue`

原因:

- 当前已有最小 debug 面板，正在作为主线稳定入口
- 本任务应通过新增 UI 形成独立调试台，而不是推翻现有面板

## 可接受的接入方式

推荐:

- 复用现有 store 数据
- 仅新增开发者视图
- 通过 props/store 读取，不要求改 trace 结构

不推荐:

- 为了 UI 方便去重写 trace 数据模型
- 大改已有游戏主界面布局

## 交付标准

满足以下条件即可视为完成:

- 存在独立 `NoName` 调试台入口
- 至少能展示 proposal、apply plan、apply execution、fallback 信息
- 与现有调试抽屉共存，不互相替代
- 有最小组件测试或交互测试
- UI 在开发模式下可稳定打开和关闭

## 验证命令

建议最小验证:

```powershell
npm run test -- --run src/components/__tests__/InfoTabsDialog.test.ts src/components/__tests__/GameInfoCenterDialog.test.ts
```

如果新增了独立测试:

```powershell
npm run test -- --run src/components/__tests__/NoNameDebugConsole.test.ts
```

## 交付物建议

建议协作者提交:

- 新调试组件
- 最小入口接入
- 组件测试
- 截图或简短说明

## 备注

这是非常适合前端协作者的任务。它对主线侵入低，但能明显提升后续所有 `NoName` 开发体验。
