# UI 审计报告（V3 A1）

## 范围与方法
- 审计范围：`MainMenu`、`ScriptSelector`、`GameView` 及其直连子组件、全局样式与路由。
- 证据来源：代码结构、组件规模、交互事件、可访问性标记、文案与编码状态。

## 页面结构（现状）
- 路由结构清晰：`/`（主菜单）→ `/script-select`（剧本选择）→ `/game`（主界面），见 `src/router/index.ts:1`。
- `GameView` 已完成一定拆分，但仍是核心编排与状态聚合点，文件规模 558 行，见 `src/components/GameView.vue`。
- 主界面分区基本成型：
  - 顶栏动作：`GameTopBar` + `SystemCenterMenu` + `QuickActionsBar`。
  - 阅读区：`StoryViewport`。
  - 交互区：`GameInteractionPanel`。
  - 信息与系统弹层：`GameInfoCenterDialog`、`GameSystemDialogs`、`CharacterInfoModal`。

## 组件关系（现状）
- 组件数量与测试覆盖：
  - Vue 组件 33 个（`src/components`）。
  - 组件测试 26 个（`src/components/__tests__`）。
- 关键耦合点：
  - `GameView` 直接管理大量 UI 开关与流程状态（保存/加载/设置/信息面板/快捷键/自动推进），见 `src/components/GameView.vue`。
  - 全局键盘事件在页面级注册，见 `src/components/GameView.vue:591`。
- 组件目录职责仍偏“页面型”：
  - 路由页面组件位于 `src/components` 而非 `views`，增加结构识别成本。

## 信息层级（现状）
- 优点：
  - 主流程“阅读→交互”在视觉上位于中心。
  - 全局主题变量与字体体系已建立，见 `src/styles.css:1`。
- 问题：
  - 高频动作与低频设置仍同层出现，操作面分散在顶栏按钮、系统中心、信息面板、多个弹窗之间。
  - 信息面板内聚合了角色/进度/地图/复盘/导出/调试/系统提示 7 类内容，认知切换成本高，见 `src/components/InfoTabsDialog.vue:1`。

## 可用性问题（按优先级）
1. P0: 中文文案编码异常，影响核心可读性与专业感。  
证据：`src/components/InfoTabsDialog.vue:9`、`src/components/GameInfoCenterDialog.vue:4`。
2. P1: `GameView` 仍承担过多编排与流程控制职责，维护成本高。  
证据：`src/components/GameView.vue`（558 行，含自动推进、快捷键、弹层编排、策略设置、滚动控制）。
3. P1: 全局快捷键可能与输入/弹窗焦点冲突，缺少统一焦点治理。  
证据：`src/components/GameView.vue:547`、`src/components/GameView.vue:582`、`src/components/GameView.vue:591`。
4. P2: 视觉语义未完全统一，存在离散色彩与焦点样式。  
证据：`src/components/ScriptSelector.vue:68`（紫色 radio 焦点与全局琥珀主题不一致）。
5. P2: 信息面板体量过大且无任务导向分组，探索路径偏长。  
证据：`src/components/InfoTabsDialog.vue:18`（多标签并列，缺少“主流程优先”排序）。

## 结论
- 当前前端已具备 V3 重构基础（分层组件、测试覆盖、主题变量），但仍存在三项阻断体验的问题：文案编码、页面编排过重、交互焦点治理不足。
- 建议按 `A2 -> A3 -> B` 顺序推进：先冻结视觉规范与文案术语，再进行结构重构，避免返工。

## A2/A3 输入（供下一步直接使用）
- A2（视觉规范）需先统一：语义色、状态色、按钮层级、焦点环、间距刻度、动画时长。
- A3（文案规范）需先修复：全量中文编码、术语字典、错误提示分级模板。
