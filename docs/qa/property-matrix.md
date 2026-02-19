# Property Matrix (Task 23)

Date: 2026-02-19
Scope: `tasks_v2.md` Task 23

## Purpose
- Provide a single executable map from invariants to concrete tests.
- Make CI and local regression checks traceable.

## Matrix

| Invariant | Test Name | File | Command |
|---|---|---|---|
| Plot pauses at decision points | `test_property_18_plot_pauses_at_decision_points` | `src-tauri/src/plot_engine.rs` | `cargo test -q test_property_18_plot_pauses_at_decision_points` |
| Option count bound (2..5) | `test_property_19_option_count_constraint` | `src-tauri/src/plot_engine.rs` | `cargo test -q test_property_19_option_count_constraint` |
| Free-text parsing robustness | `test_property_20_free_text_intent_parsing` | `src-tauri/src/plot_engine.rs` | `cargo test -q test_property_20_free_text_intent_parsing` |
| Unreasonable action rejection | `test_property_21_unreasonable_actions_are_rejected` | `src-tauri/src/plot_engine.rs` | `cargo test -q test_property_21_unreasonable_actions_are_rejected` |
| TOC/chapter integrity | `prop_toc_integrity_matches_chapters` | `src-tauri/src/novel_generator.rs` | `cargo test -q prop_toc_integrity_matches_chapters` |
| Export fact coverage | `prop_export_source_event_coverage_is_complete` | `src-tauri/src/novel_generator.rs` | `cargo test -q prop_export_source_event_coverage_is_complete` |
| Map aura normalization bounds | `prop_map_aura_normalization_is_bounded` | `src-tauri/src/numeric_guard.rs` | `cargo test -q prop_map_aura_normalization_is_bounded` |
| Technique power band bounds | `prop_technique_power_normalization_hits_band` | `src-tauri/src/numeric_guard.rs` | `cargo test -q prop_technique_power_normalization_hits_band` |
| Technique root affinity invariant | `prop_technique_root_affinity_never_contains_blank_or_duplicate` | `src-tauri/src/entity_validator.rs` | `cargo test -q prop_technique_root_affinity_never_contains_blank_or_duplicate` |

## Suggested Bundles

- Quick smoke:
  - `cargo test -q prop_toc_integrity_matches_chapters`
  - `cargo test -q prop_export_source_event_coverage_is_complete`
  - `cargo test -q prop_map_aura_normalization_is_bounded`
  - `cargo test -q prop_technique_root_affinity_never_contains_blank_or_duplicate`

- Full Task-23 bundle:
  - `cargo test -q plot_engine::property_tests`
  - `cargo test -q novel_generator::property_tests`
  - `cargo test -q numeric_guard::tests`
  - `cargo test -q entity_validator::tests`

## Remaining Gaps
- Map position consistency invariant in end-to-end state transitions.
- Technique risk-tag invariant linked with chapter events.
- Chronicle sentence-level coverage ratio (currently event-id coverage level).
