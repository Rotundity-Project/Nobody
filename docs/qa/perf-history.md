# V2 Performance History

Date: 2026-02-19  
Scope: Task 24 trend tracking

## Records

| Date | Commit | Method | Plot P95 (ms) | Combat Parse P95 (ms) | Notes |
|---|---|---|---:|---:|---|
| 2026-02-19 | `30dcded` | initial ignored benchmark | 0.001 | 0.000 | baseline, engine-only |
| 2026-02-19 | `3981c1b` | batched per-op sampling + P50/P95/P99 | 0.002 | 0.000 | reduced timer jitter |
| 2026-02-19 | `bfdc0d5` | diagnostics summary command | n/a | n/a | added runtime diagnostics percentile aggregation |
| 2026-02-19 | `TBD` | task24 closure rerun | 0.002 | 0.000 | thresholds revalidated before marking task 24 complete |

## Notes

- These numbers exclude LLM network latency by design.
- Current values are lower-bound latency from local test harness.
- CI runner cross-platform trend rows should be appended after workflow integration.
