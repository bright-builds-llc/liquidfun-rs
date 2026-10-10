---
phase: 35-speed-up-the-slowest-scenes
verified: 2026-10-10T01:50:41Z
status: passed
score: 13/13 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-10T01:50:41.321Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  at: 2026-10-10T09:42:49Z
  by: orchestrator agent (Claude), agent-performed objective check
  closed: "Full workspace test suite: `cargo test --workspace --all-features` at HEAD `3dac75b8a` (engine/tooling source identical to the 35-08 tree): exit 0, 165 test-result lines, 2,770 passed, 0 failed, 1 ignored (01:52Z–09:42Z 2026-10-10, run by the orchestrator agent; log kept in the session scratchpad as ws-test.log)"
optional_not_run:
  - "D-03 browser frame-drop observation (optional per locked decision D-03; recorded as Not run)"
---

# Phase 35: Speed Up the Slowest Scenes Verification Report

**Phase Goal:** The slowest scenes from the survey run measurably faster with unchanged behavior.
**Verified:** 2026-10-10T01:50:41Z
**Status:** passed
**Re-verification:** Yes, 2026-10-10T09:42:49Z: the one uncertain truth (5) was closed by an agent-run full workspace test suite.

## Goal Achievement

The goal is achieved in substance. All five survey targets are faster than the phase "before" binary in both pairs of a back-to-back ABBA run. All 25 catalog scene fingerprints match the before run. No scene module, control, preset or web source changed. The full workspace test suite, including the xtask integration targets, has since been observed passing on the phase tree (truth 5).

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | SC1: The top three to five scenes by survey cost each have a profile naming their hot path | ✓ VERIFIED | 35-PROFILES.md §Target profiles: liquid-tumbler `pressure::damping` 22.9%, tesla-valve `visit_sorted_tag_indices_in_aabb` 13.7%, stacked-drip `pressure::damping` 17.1%, washing-machine `push_fixture_particle_hit` 15.2% (full-scan CCD 29.1% inclusive), particles `pressure::damping` 22.8%. Each is a samply profile from the profiling build at BEFORE_COMMIT `809582431`, with sample counts and top-10 self and inclusive breakdowns. The five are the D-01 set; the fresh before run and its confirmation run kept the survey's top five (water-wheel's rank 5 in one run came from a single slow run, spread 34%). |
| 2 | SC1 clause: plus any scene that visibly drops frames in the browser | ✓ VERIFIED (recorded as not run per D-03) | §Browser observation (D-03) records "Not run" with the reason, which locked decision D-03 allows. Listed as an optional human item. |
| 3 | SC2: Each kept change shows a before/after survey gain beyond noise | ✓ VERIFIED | The verifier re-ran `keep_rule.py` on the saved ABBA files. A1 gains on all 3 targets in both pairs; A5 on liquid-tumbler and tesla-valve in both pairs; A6 on washing-machine (1.436 / 1.435 vs base min 2.000 / 2.006); A8 and A9 on tesla-valve in both pairs. Every kept attempt's full run: `fingerprint mismatches: none`. Cumulative `final-b1/a1/a2/b2`: `gain=True` for all five targets in both pairs, no medians above the before max. Changes: liquid-tumbler −5.8% / −8.5%, tesla-valve −19.7% / −14.6%, stacked-drip −9.9% / −6.4%, washing-machine −31.8% / −31.7%, particles −5.1% / −7.4%. |
| 4 | SC2: Unhelpful attempts are reverted and noted | ✓ VERIFIED | A0, A2, A3, A3b and A4 have §Attempts rows with numbers and reasons. Patches are saved at `target/phase35/attempts/{A2,A3,A3b,A4}.patch`. `git diff 0a2fe8cb8^ 7f0b36b18` is empty, so the A3 revert is clean. A2, A3b and A4 were never committed, and `proxy.rs` is unchanged since BEFORE_COMMIT. The re-run reproduces each revert trigger: A2 particles above the max in both pairs, A4 liquid-tumbler above the max in both pairs, A3 soup-stirrer and A3b fountain above the base max in both isolated pairs. |
| 5 | SC3: Existing Rust tests pass | ✓ VERIFIED (re-verified) | `cargo test --workspace --all-features` at HEAD `3dac75b8a` (engine/tooling source identical to the 35-08 tree): exit 0, 165 test-result lines, 2,770 passed, 0 failed, 1 ignored (01:52Z–09:42Z 2026-10-10, run by the orchestrator agent; log kept in the session scratchpad as ws-test.log). Earlier evidence:  Passing on the final tree (`checks-08.results`, exit 0): `cargo test --all-features` (liquidfun, 1,079 passed), `cargo test -p liquidfun-wasm --all-features` (316 passed), clippy `-D warnings`, the workspace all-targets build and fmt. From 35-01: `xtask --bin xtask` 127 passed and `playground_cli spot` 4/4. **Not observed:** the remaining xtask integration targets, which stalled under syspolicyd. Phase 35 changed xtask sources and the shared fake tool, and CI (`cargo test --workspace`) has not run on these commits. Static review shows the fake-tool change sits only inside `print_scene_spot_sample` and that no test pins the USAGE string, so the risk is low but unconfirmed. |
| 6 | SC3: Existing web tests pass | ✓ VERIFIED (one pre-existing failure judged unrelated) | `bun run test:unit` passed (56 files, 472 tests) and `just web-build` passed with the generated files unchanged. `just web-smoke` (optional) passed 61/62. The failing `e2e/rust-wasm-proof.spec.ts:170` expects `Loading Rust/WASM session…`; that text left `web/src` in `94a9ebafa` (2026-09-17), and the spec was last edited in `3a047bf99` (2026-09-20). Phase 35 changed nothing under `web/`, `scripts/` or `justfile` (`git diff 809582431 HEAD` is empty for those paths), so this is a pre-existing stale spec, not a phase regression. |
| 7 | SC3: Scene behavior is unchanged | ✓ VERIFIED | Re-ran `keep_rule.py before-full.jsonl final-full.jsonl` and got `fingerprint mismatches: none` for 25/25 scenes. The four final ABBA files also match. Neither file has a `timed_out` line. `spot-final` has the same SHA-256 (`bd2b2eb5…`) as `target/release/playground-scene-spot`, and the 35-07 record shows a rebuild of the committed source is byte-identical. The fingerprint hashes particle positions, velocities and colors plus body pose and velocity. After IN-05 its doc no longer claims more than that. |
| 8 | SC3: Settings and visuals are unchanged | ✓ VERIFIED | `git diff 809582431 HEAD --stat -- crates/liquidfun-wasm/src/scene/ web/ scripts/ justfile` is empty. All non-planning changes since BEFORE_COMMIT are engine internals and tests under `crates/liquidfun/src/{particle,world}/`, plus a two-line clarification of the fingerprint wording in `docs/benchmarks/scene-survey.md`. |
| 9 | 35-01: `--scene` filter and a 16-hex FNV-1a fingerprint in the survey bin and xtask | ✓ VERIFIED | `gsd-tools verify artifacts` passed 5/5. `fingerprint.rs` has `FNV_OFFSET = 0xcbf2_9ce4_8422_2325` and hashes the documented fields. `scene_spot.rs` emits `"fingerprint"` and fails when two runs of one scene give different fingerprints. The bin and `spot/args.rs` parse `--scene`. |
| 10 | D-08: The fingerprint lives only in measurement tooling | ✓ VERIFIED | The 059173843→809582431 diff touches only `scene_spot*`, the bin, `session.rs` (4 lines) and xtask. No scene module changed. |
| 11 | Kept engine changes are wired and equivalence-tested | ✓ VERIFIED | A1: `take_contact_proxies` (`particle_coupling.rs:165`) and `clear_contact_scan` (`boundary_runtime.rs:174`), then `retag_retained_order`. A5: `order_rows_with_marks` in `collect_candidate_rows`, tested by `candidate_rows_match_sorted_dedup`. A6: `moving_fixture_query_pad` called at `particle_coupling.rs:288`, with filtered-vs-full-scan tests. A8: `prepare_create` fast path with a `debug_assert_eq!` against the rebuild, plus `creation_fast_path_tests.rs`. A9: `ParticleIdHasher` and the in-place `resequence_to_storage_order`, called from `lifetime.rs:554`. The tests run in the passing liquidfun suite. |
| 12 | D-05: No unsafe, no unwrap and no authored-setting changes in kept engine code | ✓ VERIFIED | Lines added in the phase to the changed production files contain 0 `unsafe` and 0 `unwrap()`, and no TODO, FIXME or `todo!` appears in the diff. |
| 13 | 35-08: The wasm32 build passes on the final HEAD | ✓ VERIFIED (lib build; the non-lib failure predates the phase) | `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` exits 0, and so does `just web-build` (wasm-pack builds the cdylib library). The non-`--lib` build exits 101 because the native-only bins import `#[cfg(not(target_arch = "wasm32"))]` items. That gating already existed at phase start `059173843` (lib.rs lines 13-24), the bins date from 2026-09-20/21, and the Cargo.toml `[[bin]]` entries have no `required-features`. Not a phase regression. |

**Score:** 13/13 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene_spot/fingerprint.rs` | FNV-1a end-state fingerprint | ✓ VERIFIED | Wired from `scene_spot.rs:10` |
| `crates/liquidfun-wasm/src/scene_spot.rs` | Filter, fingerprint field, cross-run check | ✓ VERIFIED | |
| `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs` | `--scene` flag | ✓ VERIFIED | |
| `tools/xtask/src/playground/spot/args.rs` | SpotArgs with catalog-checked `--scene` | ✓ VERIFIED | |
| `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | Fake bin with `--scene` and fingerprint | ✓ VERIFIED | Change limited to `print_scene_spot_sample` |
| `crates/liquidfun/src/particle/contact_scan.rs` | `ProxyOrderCache`, reuse branch (A1, kept) | ✓ VERIFIED | |
| `crates/liquidfun/src/particle/storage/runtime.rs` | `proxy_order_cache` handoff (A1) | ✓ VERIFIED | |
| `crates/liquidfun/src/particle/body_contact.rs` | `maybe_child_edge` (A3) | N/A (A3 reverted) | Conditional on keep; revert is clean |
| `crates/liquidfun/src/particle/proxy/aabb_query_tests.rs` | A4 equivalence tests | N/A (A4 reverted) | Plan marks it "if A4 kept"; never committed |
| `crates/liquidfun/src/world/particle_coupling/moving_fixture_query.rs` | `moving_fixture_query_pad` (A6) | ✓ VERIFIED | |
| `crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs` | Filtered = full scan tests | ✓ VERIFIED | |
| `crates/liquidfun/src/particle/storage/creation.rs` | Ungrouped fast path (A8) | ✓ VERIFIED | |
| `crates/liquidfun/src/particle/lifetime/eviction.rs` | Cheaper eviction index (A9) | ✓ VERIFIED | |
| `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md` | Method, before run, profiles, attempts, final verification | ✓ VERIFIED | |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `scene_spot.rs` | `session.rs` | `read_particles` (cfg widened) | ✓ WIRED | Fingerprints are produced in every JSONL line |
| xtask `spot.rs` | survey bin | forwards `--scene` | ✓ WIRED | Filtered runs in 35-01 and every ABBA used `--scene` |
| `boundary_runtime.rs` | `storage/runtime.rs` | `clear_contact_scan` retains order | ✓ WIRED | line 174 |
| `particle_coupling.rs` | `contact_scan.rs` | `take_contact_proxies` → `rebuild_proxies` reuse | ✓ WIRED | line 165 |
| `body_contact::collect_candidate_rows` | `order_rows_with_marks` | row count ≥ 32 | ✓ WIRED | line 48 |
| `filtered_collision_hits` | `moving_fixture_query_pad` | moving fixtures at iteration 0 | ✓ WIRED | line 288 |
| `lifetime.rs` | `eviction.rs` | `resequence_to_storage_order` | ✓ WIRED | line 554 |
| `35-PROFILES.md` | `target/phase35/before-full.jsonl` | final fingerprint comparison | ✓ WIRED | Re-run by the verifier |

### Data-Flow Trace (Level 4)

Not applicable. The phase changes engine internals and measurement tooling, not rendered UI. Behavior equivalence is covered by the fingerprint comparison in truth 7.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Final fingerprints equal the before run | `python3 target/phase35/keep_rule.py before-full.jsonl final-full.jsonl <5 targets>` | `fingerprint mismatches: none`; all 5 `gain=True`; none above the max | ✓ PASS |
| Cumulative ABBA gain in both pairs | `keep_rule.py final-b1 final-a1` and `final-b2 final-a2` | All 5 targets `gain=True` in both pairs; no mismatches | ✓ PASS |
| Each kept and reverted decision is reproducible from the saved data | `keep_rule.py <A>-b{1,2} <A>-a{1,2}` for A1-A9 plus the iso files for A3/A3b | Matches every §Attempts decision. A9 pair 2 flags sparky (one pair only, recorded) | ✓ PASS |
| The timed binary equals the committed source build | `shasum -a 256 bin/spot-final target/release/playground-scene-spot` | Both `bd2b2eb5…` | ✓ PASS |
| Full workspace suite incl. xtask integration | `cargo test --workspace --all-features` | exit 0; 2,770 passed, 0 failed, 1 ignored at `3dac75b8a` | ✓ PASS (re-verification) |

The verifier did not re-run timing, following the instruction not to time on a busy host.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PERF-08 | 35-01..35-08 | Maintainers can inspect profile-guided fixes for the slowest scenes, each kept only with a before/after survey gain beyond run-to-run noise | ✓ SATISFIED | Truths 1-4 and 11. 35-PROFILES.md is the inspectable record. Each kept change (A1, A5, A6, A8, A9) names its hot path and passes D-11. |
| PERF-09 | 35-01, 35-03..35-08 | Visitors see unchanged scene behavior, settings, controls and visuals after the fixes; existing Rust and web tests stay green | ✓ SATISFIED | Behavior, settings, controls and visuals are verified (truths 6-8); Rust workspace and web unit tests pass (truths 5-6). |

Orphaned requirements: none. REQUIREMENTS.md maps only PERF-08 and PERF-09 to Phase 35, and the plans claim both.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `tools/xtask/src/playground/spot.rs` | 1, 25, 218, 281 | Filtered stamp still says "every playground catalog scene" (REVIEW WR-01) | ⚠️ Warning | Tooling label only. Phase 36 should not read filtered stamps as full surveys; fix before Phase 36 builds its table. |
| `crates/liquidfun/src/particle/contact_scan.rs` | 159-178 | Permutation precondition is not asserted (IN-01) | ℹ️ Info | Holds today; future-proofing |
| `crates/liquidfun/src/particle/body_contact.rs` | 60-82 | Undocumented `row < row_count` and non-empty preconditions (IN-02) | ℹ️ Info | Holds today |
| `crates/liquidfun/src/world/particle_coupling.rs` | 268-314 | Call order of the FIXTURE_CONTACT_FILTER hook changed for moving fixtures at iteration 0 (IN-03) | ℹ️ Info | Hit list unchanged. Only an order-dependent user hook could notice, and no catalog scene uses one. |
| `creation.rs`, `eviction.rs` | — | Fast paths skip the rebuild's incidental validation of corrupt state (IN-04) | ℹ️ Info | Equivalent while the invariants hold |

No blockers: no TODO or FIXME, no stubs, no `unsafe`, no `unwrap()` in the changed production code.

### Notes on Reverted Attempts

A3 (chain child-edge hoist, −9.4% / −10.4% on liquid-tumbler) and A3b (no-branch `Shape::Edge` variant, −7.6% / −7.3%) were reverted under the locked D-11 rule. In each case, a scene without a chain fixture (soup-stirrer for A3, fountain for A3b) was above its base max in both isolated pairs. Later diagnostic pairs did not reproduce either regression, so it may have been host noise. Applying the locked keep rule as written is correct, so this is not a gap. The retry is recorded in §Notes for Phase 36 with the saved patches.

### Human Verification

None required. The full workspace test suite item was closed by an agent-run check (see truth 5). The optional D-03 browser observation remains recorded as Not run; locked decision D-03 makes it optional, and bit-identical fingerprints plus an empty scene/web diff make a visual change very unlikely.

### Gaps Summary

There are no gaps. The kept changes are wired, equivalence-tested, bit-identical on all 25 fingerprints, and faster beyond noise on all five targets, and the full workspace test suite passes. Two pre-existing failures predate the phase and do not block SC3:

- The non-`--lib` wasm32 build: the native-only bins have been cfg-gated since Phases 22 and 24.
- The stale `web-smoke` spec at `rust-wasm-proof.spec.ts:170`: its loading text left `web/src` on 2026-09-17.

Both are tracked in `deferred-items.md`.

---

_Verified: 2026-10-10T01:50:41Z_
_Verifier: Claude (gsd-verifier)_
_Re-verified: 2026-10-10T09:42:49Z by the orchestrator agent (truth 5 only)_
