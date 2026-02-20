# UI 视觉规范 v1（V3 A2）

## 目标
- 建立统一视觉语言，支撑 `MainMenu`、`ScriptSelector`、`GameView` 三个主场景的一致体验。
- 以“玄卷墨金”作为基调：深色背景、暖金强调、冷青辅助。

## 色板（Semantic Tokens）
- `--color-bg-base`: `#0b0f14`
- `--color-bg-elevated`: `#151a24`
- `--color-panel`: `rgba(20, 24, 34, 0.9)`
- `--color-panel-strong`: `#1d2332`
- `--color-border`: `rgba(148, 163, 184, 0.2)`
- `--color-text-primary`: `#f8fafc`
- `--color-text-secondary`: `#cbd5e1`
- `--color-text-muted`: `#94a3b8`
- `--color-accent`: `#c0a77a`
- `--color-accent-strong`: `#f2c86b`
- `--color-accent-cool`: `#6dd0b2`
- `--color-success`: `#34d399`
- `--color-warning`: `#f59e0b`
- `--color-danger`: `#f87171`
- `--color-info`: `#38bdf8`

## 字体规范
- 标题字体：`Cinzel` + `Noto Serif SC`（用于章节标题、面板标题）。
- 正文字体：`Noto Serif SC`（用于剧情正文、摘要、长文本）。
- UI 字体：`Source Sans 3`（用于按钮、表单、状态信息）。
- 数字与技术信息：`Source Sans 3`（必要时可加 `font-mono`）。

## 字号与层级
- `Display`: 32/40，章节主标题。
- `H1`: 24/32，页面级标题。
- `H2`: 20/28，区块标题。
- `Body`: 16/26，正文。
- `Body-sm`: 14/22，辅助说明。
- `Caption`: 12/18，标签与元信息。

## 间距与圆角
- 间距刻度：`4, 8, 12, 16, 24, 32, 40, 48`。
- 页面主边距：`24`（桌面）/ `16`（平板）/ `12`（移动）。
- 面板间距：`16`。
- 圆角：
  - 卡片：`12`
  - 主要容器：`16`
  - 弹层：`16`
  - 按钮：`8`

## 动效规范
- 允许动效类型：
  - 章节切换（淡入上移）
  - 面板开合（透明度 + 位移）
  - 状态切换（颜色与阴影过渡）
- 时长：
  - 快速：`120ms`
  - 标准：`200ms`
  - 强调：`320ms`
- 缓动：`ease-out`（入场）/ `ease-in-out`（状态切换）。
- 禁止持续噪声型动画（闪烁、循环脉冲）。

## 状态色与组件反馈
- 默认按钮：深灰底 + 浅字；悬浮加亮一级。
- 主按钮：`accent`；悬浮切换 `accent-strong`。
- 成功：`success`（完成、可达、通过）。
- 警告：`warning`（风险、回退、降级）。
- 错误：`danger`（失败、阻断）。
- 信息：`info`（提示、系统说明）。

## 交互可见性
- 焦点环：统一 `2px` 外描边，颜色使用 `--color-accent-strong`。
- 禁用态：透明度不低于 `0.55`，且保留清晰文本对比。
- 键盘可达：所有主操作按钮需可通过 `Tab` 到达并有焦点反馈。

## 落地约束（用于后续重构）
- 禁止新增未注册颜色常量，统一走 semantic tokens。
- 新组件默认使用 `Source Sans 3`，剧情正文显式使用 `font-story`。
- 交互组件不得混用独立主题色（例如局部紫色焦点），需对齐规范。
