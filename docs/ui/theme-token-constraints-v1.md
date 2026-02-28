# 主题 Token 约束（V1）

## 目标
- 保证浅色古风（`theme-scroll`）与深色风格（`theme-night`）可并行维护。
- 降低组件级硬编码背景导致的视觉回归风险。

## 核心 Token 层级
- 基础层（不可在组件内覆写）
- `--paper-bg`
- `--paper-cloud`
- `--panel-bg`
- `--panel-bg-soft`
- 页面语义层（允许在主题层统一调整）
- `--runtime-shell-bg`
- `--runtime-shell-overlay`
- `--runtime-main-panel-bg`
- `--runtime-story-bg-color`
- `--runtime-story-bg-image`
- 组件交互层（允许组件按状态覆写）
- `--runtime-btn-*`
- `--runtime-dock-btn-*`
- `--runtime-option-btn-*`

## 覆写规则
- 组件内禁止直接写固定背景色值（例如 `#f5f0e8`、`#26393b`），必须通过 Token 引用。
- 同一主题下，运行页主背景、卡片背景、底栏背景必须来自同一套基础层 token。
- 新增主题变量时，`theme-scroll` 与 `theme-night` 必须同时定义，避免单主题缺失回退。

## 回归检查清单
- 背景纹理：无意外横纹/竖纹、无未定义变量导致的透明块。
- 标题分隔线：浅色与深色下对比均可读。
- 按钮层次：普通/悬停/主按钮状态清晰。
- 滚动区域：故事区底色、纹理与滚动条颜色一致。
