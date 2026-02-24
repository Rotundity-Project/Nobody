# Nobody Theme Tokens (v1)

## 目标
- 统一主题变量命名，避免组件内直接写颜色值。
- 保持两套主题（`theme-scroll` / `theme-night`）可独立精调。
- 让“改质感”优先在 token 层完成，而不是在组件层到处改样式。

## 分层规则
1. `base` 层：原始色与基础阴影  
- 示例：`--ink-bg-base`、`--ink-paper`、`--ink-border`、`--ink-shadow-panel`

2. `semantic` 层：语义化透明层、光泽、颗粒、抬升  
- 示例：`--ink-overlay-strong`、`--ink-elevated-glow`、`--ink-option-grain`、`--ink-elevation-soft`

3. `component` 层：组件专用组合变量  
- `runtime`：`--runtime-*`
- `script`：`--script-*`
- `info`：`--info-*`
- `modal`：`--modal-*`

## 命名规范
- 统一前缀：`--ink-*`（通用）、`--runtime-*`（运行页）、`--script-*`（剧本页）。
- 语义后缀建议：
- `*-overlay-*`：覆盖层/雾层
- `*-glow`：发光/辉光
- `*-grain`：纹理颗粒
- `*-highlight-*`：高光
- `*-shadow-*`：阴影
- `*-elevation-*`：抬升层级

## 禁止项
- 组件模板中禁止新增 `text-[#...]` / `bg-[#...]` / `border-[#...]`。
- 组件样式中禁止新增裸 `rgba(...)` 颜色值（应先落到 token）。
- 避免在组件内定义主题分支逻辑（`theme-scroll`/`theme-night` 只在 token 层分流）。

## 组件用法
- 文本：`var(--ink-text-primary)` / `var(--ink-text-muted)`
- 边框：`var(--ink-border-soft)` / `var(--ink-border-strong)` / `var(--ink-border-accent)`
- 面板背景：`var(--ink-paper)` / `var(--ink-paper-elevated)` / `var(--ink-card-bg)`
- 交互阴影：`var(--ink-action-shadow-hover)` 或 `var(--ink-elevation-*)`

## 变更流程
1. 先在 `src/styles.css` 的 `theme-scroll` 与 `theme-night` 同步增加 token。
2. 再替换组件引用，确保不直接写硬编码颜色。
3. 跑快照/视觉回归，确认两套主题一致性。
