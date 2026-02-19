# V2 Go-Live 清单（最终）

更新日期：2026-02-19  
版本：1.1.0

## 1. 构建与测试门禁

- [x] 前端测试通过：`npm test`（14 文件 / 42 用例）
- [x] 后端测试通过：`cargo test -q`（286 passed / 0 failed / 2 ignored）
- [x] 静态检查通过：`cargo clippy -- -D warnings`
- [ ] CI 工作流复核：`.github/workflows/ci.yml`
- [ ] 性能工作流复核：`.github/workflows/perf-benchmark.yml`（手动触发）

## 2. 迁移与兼容验收

- [x] 旧存档迁移链路可用（见 `docs/release/migration-v2.md`）
- [x] `schema_version` / `migration_history` 迁移写回策略确认
- [x] 迁移失败不覆盖原文件（保留回滚路径）

## 3. 灰度发布记录（待执行）

- [ ] 灰度开始时间
- [ ] 灰度范围（建议 10%-20%）
- [ ] 灰度用户规模
- [ ] P0/P1 错误统计
- [ ] 性能 P95 统计
- [ ] 72h 观察结论

## 4. 回滚预案

- [x] 回滚目标版本：1.0.0
- [x] 回滚触发条件已定义（关键错误率/迁移失败率/性能劣化）
- [x] 回滚后验收路径已定义（存档可读、剧情可推进、导出可用）

## 5. 关联文档

- 发布说明：`docs/release/v2-release-notes.md`
- 发布准备：`docs/release/v2-release-prep.md`
- 结项清单：`docs/release/v2_closure_checklist.md`
- 灰度报告模板：`docs/release/v2-canary-report-sample.md`
- 发布后回归模板：`docs/release/v2-post-release-regression-template.md`
