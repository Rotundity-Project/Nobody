# UI 重构指南（V3）

## 1. 目标与范围
本指南用于约束 Nobody V3 前端的结构、视觉与交互演进方式，确保在持续迭代中维持一致体验。

核心目标：
1. 玩家始终聚焦主循环：阅读剧情 -> 做出输入 -> 获取反馈。
2. 复杂系统能力按需展示：地图、复盘、导出、诊断不干扰主舞台。
3. 代码保持可拆分、可测试、可回归，避免再次出现单文件超重。

## 2. 架构总览
### 2.1 分层结构
1. `views/`：路由级页面容器，只做编排。
2. `components/`：领域组件（剧情、交互、系统、信息）。
3. `composables/`：交互流程逻辑与状态派生。
4. `shared/ui/`：基础 UI 原语（按钮、面板等）。
5. `stores/`：`gameStore` 作为领域状态单一事实源。

### 2.2 关键入口
1. `src/views/GameViewPage.vue`：游戏页路由容器。
2. `src/components/GameRuntimeView.vue`：运行时核心编排。
3. `src/router/index.ts`：菜单/剧本/游戏路由切换。

## 3. 组件地图
### 3.1 主舞台层（高频）
1. `src/components/StoryViewport.vue`：阅读视口、阅读定位、滚动状态。
2. `src/components/StoryScenePanel.vue`：章节标题、节奏标签、正文容器。
3. `src/components/VirtualStoryList.vue`：长文本段落渲染与虚拟化。
4. `src/components/GameInteractionPanel.vue`：选项输入与自由输入统一容器。
5. `src/components/ContextStatusCard.vue`：角色/章节/位置轻量状态。
6. `src/components/ChapterStatusStrip.vue`：章节进度与交互状态条。

### 3.2 世界层（按需）
1. `src/components/InfoTabsDialog.vue`：右侧抽屉，承载地图/复盘/导出/调试。
2. `src/components/GameInfoCenterDialog.vue`：信息抽屉编排与 store 映射。
3. `src/components/NovelExporter.vue`：经历导出与章节摘要。

### 3.3 系统层（低频）
1. `src/components/SystemCenterMenu.vue`：系统入口聚合。
2. `src/components/GameSystemDialogs.vue`：存档、LLM、剧情策略、一致性策略。
3. `src/components/NotificationCenter.vue`：Banner/Toast 反馈中心。

## 4. 视觉与样式规范
### 4.1 设计令牌
定义位置：`src/styles.css`
1. 基础令牌：`--ref-*`
2. 语义令牌：`--color-*`
3. 兼容别名：`--bg-*`、`--panel`、`--accent`

规则：
1. 新样式优先使用语义令牌，不直接写色值。
2. 业务组件不重复定义通用交互视觉（聚焦环、面板底色、按钮尺寸）。
3. 聚焦态必须可见，避免仅颜色区分状态。

### 4.2 间距与密度
1. 阅读区优先，章节头/摘要卡避免过高垂直占用。
2. 中等宽度下状态卡优先横向排布，减少正文被挤压。
3. 非核心信息移入抽屉或折叠区，不在主视图堆叠。

## 5. 状态与流程规范
1. 领域数据集中在 `gameStore`，不在组件树复制状态。
2. 流程逻辑放 `composables`：
   - `useStoryFlow`
   - `useInputMode`
   - `useUiPanels`
   - `useGameHotkeys`
3. `GameRuntimeView` 只做编排，不承载可复用业务细节。

## 6. 可访问性规范
1. 键盘：`Esc` 关闭遮罩层，输入控件内禁用冲突快捷键。
2. 读屏：关键动态信息使用 `aria-live` + `aria-atomic`。
3. 语义：操作分组使用 `role=group` 与 `aria-labelledby`。
4. 状态来源可观测：按钮 `aria-describedby` 关联到实时状态节点。

## 7. 文案与编码规范
1. 前台不得暴露技术 key（如 `sect_valley`），统一转用户可读标签。
2. 术语保持一致：章节、交互状态、位置、风险分等。
3. 所有文档与源码统一 UTF-8，避免 PowerShell 默认编码回写乱码。

## 8. 测试与回归规范
1. 单元/组件测试：`npm run test`
2. 视觉快照：`src/components/__tests__/VisualSnapshot.test.ts`
3. 生产构建验证：`npm run build`

变更要求：
1. 结构变更必须同步更新测试。
2. 快照变更需人工确认差异合理。
3. 无障碍语义变更需补断言，避免“看起来正常、读屏失效”。

## 9. 扩展规则
1. 先复用 `shared/ui`，再新增业务样式。
2. 新增系统入口统一放入 `SystemCenterMenu`。
3. 新增跨面板反馈优先进入 `NotificationCenter`。
4. 新增页面级流程先评估是否应抽到 composable。

## 10. 变更检查清单
1. `npm run test` 通过。
2. `npm run build` 通过。
3. 关键页面快照差异已审阅。
4. 无英文技术 key 泄露到用户文案。
5. 无编码回退（乱码）与文案断裂。
