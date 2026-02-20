# UI 视觉规范（V3）

## 1. 视觉方向

主题：玄卷墨金

- 深色基底保证沉浸感
- 暖金强调核心操作
- 冷青用于信息辅助状态

## 2. 色彩语义

- 背景：`--color-bg-base`
- 面板：`--color-panel`
- 主文字：`--color-text-primary`
- 次文字：`--color-text-secondary`
- 强调：`--color-accent`
- 成功/警告/错误：`--color-success` / `--color-warning` / `--color-danger`

## 3. 字体

- 标题：`font-display`
- 正文：`font-story`
- UI 文本：`Source Sans`

## 4. 间距与层级

- 统一间距刻度：4 / 8 / 12 / 16 / 24 / 32
- 面板圆角：12-16
- 焦点环必须可见

## 5. 动效策略

仅保留关键动效：

1. 章节切换
2. 面板开合
3. 状态切换

## 6. 约束

- 新增样式优先复用 `shared/ui`
- 禁止在业务组件中硬编码主题色
- 快照基线变更需人工确认
