# V2 Performance Baseline Report

Date: 2026-02-19
Scope: Task 24 (performance targets)
Environment: local dev machine, no LLM network calls, Rust test harness

## Targets
- Plot progression P95 < 2.5s (with cache)
- Combat parse P95 < 1.0s (excluding LLM network time)

## Method
- Added manual benchmark tests in `src-tauri/src/tests.rs`:
  - `perf_plot_advance_p95_under_target` (`#[ignore]`)
  - `perf_combat_parse_p95_under_target` (`#[ignore]`)
- Sampling strategy:
  - batch sampling (`outer_rounds * inner_iters`) to reduce timer jitter
  - report per-op `P50/P95/P99` (ms)
  - use `std::hint::black_box` to avoid unrealistic compiler optimization effects
- Added runtime timing diagnostics in `execute_player_action`:
  - `total`
  - `plot_gen`
  - `option_gen`
  - location: `src-tauri/src/tauri_commands.rs` (`last_generation_diagnostics`)
- Added diagnostics summarizer command:
  - `summarize_generation_diagnostics(diagnostics: Vec<String>)`
  - output: `sampleCount / totalP50/P95/P99 / plotGenP95 / optionGenP95`
- Commands:
  - `cargo test -q perf_plot_advance_p95_under_target -- --ignored --nocapture`
  - `cargo test -q perf_combat_parse_p95_under_target -- --ignored --nocapture`

## Results
- Plot advance per-op: `P50=0.001 ms`, `P95=0.002 ms`, `P99=0.003 ms`
- Combat parse per-op: `P50=0.000 ms`, `P95=0.000 ms`, `P99=0.001 ms`

## Interpretation
- Both metrics are far below the target thresholds.
- These are baseline engine-only numbers (no network, no UI, no external I/O), so they represent lower-bound latency.

## Limitations
- Current benchmark does not include end-to-end Tauri command overhead.
- Current benchmark excludes LLM request latency by design.
- Current benchmark runs in test harness, not production load profile.
- Diagnostics summarizer currently aggregates caller-provided diagnostics only; no persistent ring buffer yet.

## Next
1. Add diagnostics exporter to aggregate `耗时(ms)` fields into percentile stats.
2. Add repeat runs on CI runners for cross-platform comparability.
3. Track trend by commit in `docs/qa/perf-history.md`.
