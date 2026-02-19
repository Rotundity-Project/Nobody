# V2 发布验收清单（Go-Live）

更新时间: 2026-02-19
版本: `1.1.0`

## 1. 构建与测试门禁

- [ ] 前端测试通过: `npm run test`
- [ ] 后端关键属性通过:
  - [ ] `cargo test -q novel_generator::property_tests`
  - [ ] `cargo test -q test_property_map_location_consistency_in_save_load_plot_recovery`
- [ ] 性能基线通过:
  - [ ] `cargo test -q perf_plot_advance_p95_under_target -- --ignored --nocapture`
  - [ ] `cargo test -q perf_combat_parse_p95_under_target -- --ignored --nocapture`
- [ ] CI 工作流通过:
  - [ ] `.github/workflows/ci.yml`
  - [ ] `.github/workflows/perf-benchmark.yml`（手动触发）

## 2. 迁移与兼容验收

- [ ] 使用 `1.x` 历史存档完成加载与继续游玩
- [ ] 校验 `schema_version` 与 `migration_history` 写入
- [ ] 迁移失败案例返回可读错误并保持原文件不覆盖

## 3. 灰度发布记录

- 灰度开始时间:
- 灰度范围（目标 10%-20%）:
- 灰度用户数:
- 关键错误数（P0/P1）:
- 关键性能指标（P95）:
- 72h 观察结论:
- 是否全量:

## 4. 回滚决策

- 回滚触发条件:
- 回滚版本:
- 回滚执行人:
- 回滚后验证结论:
