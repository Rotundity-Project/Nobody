# V2 Test Report

Date: 2026-02-17
Branch: fix/ts-build-compat-20260215
Scope: V2 core (hybrid memory, entity pipeline, chronicle export, chapter lifecycle, status UX)

## Backend
- Command: `cargo test -q`
- Result: PASS
- Summary: 210 passed, 0 failed
- Note: one long-running test observed (`test_generate_random_script_fallback_when_llm_missing` > 60s), but passed.

## Frontend
- Command: `npm run test`
- Result: PASS
- Summary: 12 files, 33 tests passed, 0 failed

## Build / Packaging
- Command: `npm run tauri build`
- Result: PASS
- Output bundles:
  - `src-tauri/target/release/bundle/msi/Nobody_1.0.0_x64_en-US.msi`
  - `src-tauri/target/release/bundle/nsis/Nobody_1.0.0_x64-setup.exe`

## Key Regression Checks
- Plot progression and input state: PASS
- Chronicle export with TOC: PASS
- Hybrid memory context builder: PASS
- Entity validation/store/query command path: PASS

## Blockers
- None.
