# 贡献指南

感谢你愿意为 Nobody 贡献代码、文档或建议。

## 贡献方式

1. 报告问题（Bug）
2. 提出改进建议（Feature）
3. 提交代码与文档

## 提交流程

1. Fork 仓库并创建分支
2. 在分支上完成改动
3. 本地通过必要检查
4. 提交 Pull Request

示例：

```bash
git checkout -b feature/your-change
# ... make changes
git commit -m "feat: your change"
git push origin feature/your-change
```

## 开发要求

- 前端改动建议通过：
  - `npm run test`
  - `npm run build`
- Rust 改动建议通过：
  - `cargo test`（在 `src-tauri` 目录）

## 代码与文档规范

- 保持改动聚焦，避免一次 PR 混入无关修改
- 文案与文档统一使用 UTF-8（建议 BOM）
- 涉及 UI 改动时，请同步更新测试或快照

## Pull Request 建议内容

- 改动动机
- 主要变更点
- 风险与回归影响
- 测试结果

## 行为规范

请遵守 `CODE_OF_CONDUCT.md`。
