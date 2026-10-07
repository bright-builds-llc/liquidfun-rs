# Requirements: liquidfun-rs

**Milestone:** v1.4 Scenario Performance
**Defined:** 2026-10-03 (simplified 2026-10-06)
**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## Milestone Requirements

Make the playground scenes that actually feel slow faster, without changing their physics, settings, controls or visuals. Increased memory use is allowed. Numbers are local reproducible observations, not universal public speed claims. PERF numbering continues after historical PERF-01–06.

The original 2026-10-03 definition (PERF-07–17 plus SCN-01–25, one phase per scene) was shelved on 2026-10-06 because its measurement-harness phase grew to ~150k lines of tooling and ~3.8 GB of evidence without producing a speedup. It remains in git history at commit `011e80e`; the harness work is on branch `wip/v1.4-phase34-harness`.

- [ ] **PERF-07**: Maintainers can time every playground catalog scene natively with one command and see the scenes ranked by median ms/step, with a check that fails when a catalog scene is missing.
- [ ] **PERF-08**: Maintainers can inspect profile-guided fixes for the slowest scenes, each kept only with a before/after survey gain beyond run-to-run noise.
- [ ] **PERF-09**: Visitors see unchanged scene behavior, settings, controls and visuals after the fixes; existing Rust and web tests stay green.
- [ ] **PERF-10**: Maintainers can read a whole-catalog before/after timing table for the final v1.4 source.

## Acceptance

- Measure before changing; keep a change only with a profile-named hot path and a before/after gain (median of at least three runs, same machine).
- Keep measurement tooling out of production scene code.
- Normal local checks gate every commit; expensive or cross-platform qualification stays optional.

## Future Requirements

- Optional controlled-host and cross-platform campaigns, allocator deep dives, and additional control combinations when the local profiles justify them.
- Independently scoped opt-in experimental acceleration only if later authorized behavior/fidelity policy permits it.

## Out of Scope

| Item | Reason |
| --- | --- |
| Lower-quality defaults, fewer particles/iterations, changed physics/geometry, reduced resolution or approximated rendering | User explicitly selected preserved behavior and fidelity. |
| Default nondeterministic solver/SIMD changes or relaxing safe-Rust policy | Speed priority does not waive determinism or safety constraints. |
| A per-scene evidence campaign for all 25 scenes | Shelved 2026-10-06 in favor of targeting the scenes that are actually slow. |
| Required C++ ratio, sealed parity matrix, strict Linux qualification or a new benchmark dependency stack | Current milestone is local scenario performance under experimental scope. |
| Unrelated new scenarios, Drawing material feature expansion or general preset redesign | These do not establish the requested performance improvements. |
| Package publication, release tags or universal public speed claims | Separately authorized work; milestone label is planning metadata. |

## Traceability

| Requirement | Phase | Status |
| --- | --- | --- |
| PERF-07 | Phase 34 | Pending |
| PERF-08 | Phase 35 | Pending |
| PERF-09 | Phase 35 | Pending |
| PERF-10 | Phase 36 | Pending |

**Coverage:** 4/4 milestone requirements mapped exactly once.

*Last updated: 2026-10-06 after simplifying v1.4 to a survey plus targeted fixes.*
