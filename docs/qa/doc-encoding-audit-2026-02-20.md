# 全仓文档乱码巡检清单（2026-02-20）

## 1. 处理目标

1. 清理仓库内已出现的中文乱码文档。
2. 防止 `.kiro/specs/Nobody` 再次进入 Git 跟踪。
3. 建立可重复执行的巡检流程。

## 2. 本次已执行

1. 已将 `.kiro/specs/Nobody/tasks_v2.md` 从 Git 跟踪中移除（本地文件保留）。
2. 已确认 `.gitignore` 包含 `.kiro/specs/Nobody/` 规则。
3. 已完成自动巡检与人工抽样复核。

## 3. 巡检方法

1. 自动扫描：对所有 `*.md` 执行疑似乱码特征命中统计。
2. 人工复核：逐文件查看前 20-80 行，确认是否为真实乱码。
3. 分级：按“影响范围 + 对外可见度 + 修复紧急度”分为 P0/P1/P2。

## 4. 已确认乱码文件（人工复核）

### P0（优先修复，影响对外文档/协作）

1. `CODE_OF_CONDUCT.md`
2. `SECURITY.md`
3. `CONTRIBUTING.md`
4. `RUNNING.md`
5. `.github/ISSUE_TEMPLATE/bug_report.md`
6. `.github/ISSUE_TEMPLATE/feature_request.md`
7. `.github/PULL_REQUEST_TEMPLATE.md`
状态：已完成修复

### P1（产品与研发文档，建议本周内修复）

1. `调试指南.md`
2. `测试指南.md`
3. `游戏使用说明.md`
4. `docs/ui/ui-audit-report-v3.md`
5. `docs/ui/ui-copywriting-glossary-v1.md`
6. `docs/ui/ui-visual-spec-v1.md`
7. `docs/release/v2-canary-report-sample.md`
8. `docs/release/v2-go-live-checklist.md`
9. `docs/release/v2-post-release-regression-template.md`
状态：已完成修复

### P2（建议巡检复核）

1. `docs/release/` 其余 markdown（建议抽样复查）
2. `release/` 下历史说明文档（建议抽样复查）

## 5. 已确认正常（本轮抽样）

1. `README.md`
2. `docs/USER_MANUAL.md`
3. `docs/ARCHITECTURE.md`
4. `docs/API.md`
5. `release/screenshots/README.md`
6. `UI_REDESIGN_GUIDE.md`
7. `CODE_OF_CONDUCT.md`
8. `SECURITY.md`
9. `CONTRIBUTING.md`
10. `RUNNING.md`
11. `.github/ISSUE_TEMPLATE/bug_report.md`
12. `.github/ISSUE_TEMPLATE/feature_request.md`
13. `.github/PULL_REQUEST_TEMPLATE.md`
14. `调试指南.md`
15. `测试指南.md`
16. `游戏使用说明.md`
17. `docs/ui/ui-audit-report-v3.md`
18. `docs/ui/ui-copywriting-glossary-v1.md`
19. `docs/ui/ui-visual-spec-v1.md`
20. `docs/release/v2-canary-report-sample.md`
21. `docs/release/v2-go-live-checklist.md`
22. `docs/release/v2-post-release-regression-template.md`

## 6. 后续建议顺序

1. 继续处理 P2（抽样复查后按需修复）。
2. 在 CI 增加文档编码检查步骤，避免回归。

## 7. 防复发建议

1. 继续执行：
   - `npm run specs:check-encoding`
   - `npm run specs:fix-encoding`
2. 将文档修复后统一转为 `UTF-8 BOM`（已纳入现有脚本覆盖范围）。
3. 在 CI 增加文档编码检查步骤（可后续新增独立 job）。

## 8. 备注

本清单只记录“已确认乱码”与“抽样结果”。后续如果新增/重写文档，应再次执行同样巡检流程。
