---
phase: 11-examples-headless-tooling-and-testbed
plan: "23"
subsystem: compatibility-evidence
tags: [exact-ref, inventory-validation, compatibility-ledger, deterministic-report]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "22"
    provides: Fresh same-run canonical and sanitizer Phase 11 D1 authority
provides:
  - Immutable reviewed authority for the exact approved Phase 11 run, SHA, jobs, artifacts, digests, and semantic leaves
  - Fail-closed promotion validation for four narrowly scoped headless compatibility rows
  - Deterministic generated compatibility report with UI, timing, platform, performance, packaging, and release claims excluded
affects: [phase11-corpus-closure, phase11-verification, phase12-release-readiness]
tech-stack:
  added: []
  patterns:
    - Exact authority records bind remote CI identity to local semantic and inherited proof digests
    - Compatibility promotion fails closed unless every scoped row and evidence dimension is canonical and complete
key-files:
  created:
    - reference/artifacts/phase11/exact-ref.json
    - tools/xtask/src/inventory/validation/phase11.rs
    - tools/xtask/tests/inventory_cli/phase11.rs
  modified:
    - reference/compatibility.json
    - COMPATIBILITY.md
    - tools/xtask/src/inventory.rs
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
    - crates/liquidfun-differential/tests/phase11_corpus/validation.rs
key-decisions:
  - "Promote only run 29927362730 at SHA 4ea1e1e65919619d8cd1155a5461c2cda16ab7b6 with canonical artifact 8532642627 and sanitizer artifact 8532662842."
  - "Limit promotion to four semantic headless rows and preserve every excluded UI, timing, broad-platform, performance, packaging, and release dimension as not evidenced."
  - "Recompute tracked catalog, mapping, payload, and inherited Phase 6-10 proof digests during every inventory validation instead of trusting copied authority metadata."
patterns-established:
  - "Reviewed evidence promotion: exact remote identity plus recomputed local proof graph plus canonical row/dimension allowlist."
requirements-completed: [TEST-03, EXMP-01, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T14:52:08Z
duration: 18 min
completed: 2026-07-22
---

# Phase 11 Plan 23: Reviewed Authority Promotion Summary

**The exact same-run Phase 11 canonical/sanitizer authority now drives four narrowly scoped semantic compatibility rows through a fail-closed, digest-complete validator and deterministic public projection.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-22T14:34:00Z
- **Completed:** 2026-07-22T14:52:08Z
- **Tasks:** 1
- **Tracked files modified:** 9

## Accomplishments

- Recorded the reviewed authority for run `29927362730`, approved SHA `4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`, its two successful jobs, artifacts `8532642627` and `8532662842`, locked toolchains, complete scenario leaves, and explicit failed-run denials.
- Added a bounded fail-closed validator that recomputes the authority file, catalog, mapping, three payload, and five inherited proof digests and rejects mixed identities, omitted proofs, unsupported outcomes, or out-of-scope promotion.
- Promoted only catalog execution, public semantic observation/debug draw, reviewed upstream equivalence, and semantic checkpoint/comparison rows across the four exact evidence dimensions.
- Preserved visualization pixels, wall duration, frame rate, broad platform portability, performance, packaging readiness, and release readiness as excluded claims.
- Generated `COMPATIBILITY.md` twice from the machine ledger with byte-identical output.

## TDD Evidence

- **RED:** `phase11_authority_rejects_out_of_scope_promotion` failed because inventory validation accepted an out-of-scope Phase 11 authority marker before the validator existed.
- **GREEN:** Eight focused Phase 11 inventory tests pass for exact acceptance and rejection of authority, digest, proof, outcome, denied-candidate, stale-input, and scope mutations.
- **Regression coverage:** The complete inventory CLI suite passes 35/35, Phase 11 evidence CLI passes 12/12, and the Phase 11 corpus suite passes 4/4.

No intentionally failing state was committed.

## Task Commits

1. **Task 1: bind exact authority and promote scoped compatibility rows** - `f1c88c8` (feat)

**Plan metadata:** committed separately with this summary.

## Authority Identities

- **Repository:** `bright-builds-llc/liquidfun-rs`
- **Approved implementation SHA:** `4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`
- **Upstream revision:** `7f20402173fd143a3988c921bc384459c6a858f2`
- **Authorized run:** `29927362730`, successful `workflow_dispatch`
- **Canonical job/artifact:** job `88947879161`, artifact `8532642627`
- **Sanitizer job/artifact:** job `88947879108`, artifact `8532662842`
- **Platform/toolchain:** Linux x86_64; Rust 1.97.0; CMake 4.3.3; Ninja 1.13.2; Clang 22.1.8
- **Semantic SHA-256:** `248b19ecdfa6f5cd202a5d6b07783c82e097927d78e4ad87f5a4fe4c772687eb`
- **Tracked authority SHA-256:** `251b7b58e56508ba3e4280c50c7821296d1d5d596bd6efe5f37f031a8573d3c7`
- **Denied candidate:** run `29899265024`, artifacts `8521315244` and `8521345417`

## Scoped Compatibility Promotion

- `subsystem.headless-catalog-execution`
- `subsystem.headless-public-observation-and-debug-draw`
- `subsystem.headless-reviewed-upstream-equivalence`
- `subsystem.headless-semantic-checkpoints-and-comparison`

Each row is evidenced only for implemented, unit, differential, and platform dimensions. Documented differences and intentionally unsupported dimensions remain not evidenced. The authority accepts only the `match` outcome for all debug, release, replay, and sanitizer roles across the three catalog cases.

## Files Created/Modified

- `reference/artifacts/phase11/exact-ref.json` - Immutable reviewed authority, complete proof graph, scope allowlist, excluded claims, and denied candidates.
- `tools/xtask/src/inventory/validation/phase11.rs` - Bounded exact-identity, digest, row, dimension, and scope validator.
- `tools/xtask/tests/inventory_cli/phase11.rs` - Eight mutation-oriented promotion acceptance and rejection tests.
- `reference/compatibility.json` - Four narrowly evidenced Phase 11 semantic rows.
- `COMPATIBILITY.md` - Deterministic public projection of the machine ledger.
- `tools/xtask/src/inventory.rs`, `tools/xtask/src/inventory/validation.rs`, and `tools/xtask/tests/inventory_cli.rs` - Repository-root validation wiring and focused test registration.
- `crates/liquidfun-differential/tests/phase11_corpus/validation.rs` - Closed Phase 11 artifact allowlist updated for the plan-mandated exact authority record.

## Decisions Made

- Treated the Plan 11-22 pair as the sole promotion authority. No rerun, alternate SHA, relabeled artifact, or forensic failed-run identity can substitute.
- Required complete local digest recomputation so valid-looking copied metadata cannot authorize stale catalog, mapping, payload, or inherited proof content.
- Represented promotion as four subsystem rows rather than widening existing claims, keeping the public maturity statement aligned with semantic headless evidence only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Admitted the plan-mandated authority record to the closed Phase 11 corpus topology**

- **Found during:** Task 1 full corpus verification
- **Issue:** The existing Phase 11 artifact-directory allowlist accepted only `scenario-mappings.json`, so adding the required `exact-ref.json` made corpus validation fail despite the authority file being valid.
- **Fix:** Extended the exact allowlist to those two named files only; no wildcard, extra artifact, or evidence scope was admitted.
- **File modified:** `crates/liquidfun-differential/tests/phase11_corpus/validation.rs`
- **Verification:** Phase 11 corpus suite passes 4/4 and the complete all-feature test gate passes.
- **Committed in:** `f1c88c8`

**Total deviations:** 1 auto-fixed Rule 3 blocker. **Impact:** The corpus remains closed while recognizing the exact tracked authority required by this plan.

## Issues Encountered

- `just markdown-check` remains blocked by six unrelated pre-existing files: `UPSTREAM.md`, `ARCHITECTURE.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `docs/decisions/0001-oracle-selection.md`, and `crates/liquidfun-testbed/CAPABILITY.md`. The changed `COMPATIBILITY.md` passes `mdformat --check`; unrelated files were left untouched.

## Security Verification

- The authority file must be a regular non-symlink under a 128 KiB limit and valid JSON before any evidence is considered.
- Exact run, SHA, jobs, artifacts, toolchains, review identity, semantic hashes, all case roles/leaves/policies, and failed candidates are literal fail-closed bindings.
- Local catalog, mapping, corpus payload, and inherited proof content is independently hashed during inventory validation.
- The row and evidence-dimension allowlists prevent generated public maturity from exceeding the reviewed semantic headless scope.
- No network endpoint, authentication path, production dependency, renderer capability, raw pointer, secret, raw record, or unbounded diagnostic surface was added. No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Verification

- Independent exact-ref validation passed for three cases with deny run `29899265024` and deny artifacts `8521315244` and `8521345417`, both before and after promotion.
- `cargo test -p xtask --test inventory_cli phase11` passed 8/8.
- `cargo test -p xtask --test inventory_cli` passed 35/35.
- `cargo test -p xtask --test phase11_evidence_cli` passed 12/12.
- `cargo test -p liquidfun-differential --test phase11_corpus` passed 4/4.
- `cargo xtask inventory generate` and `cargo xtask inventory check` passed twice; both report generations produced SHA-256 `2a5d3b86ba6a102b3e7c374ab9ed04ca2998045280265dfd0a8452b96b91451a` across 181 rows.
- `reference/compatibility.json` ordering and focused authority/ledger scope checks passed.
- `mdformat --check COMPATIBILITY.md`, `git diff --check`, and the pinned-submodule diff check passed. Repository-wide Markdown verification is blocked only by the six unrelated pre-existing files named above.
- The exact ordered `cargo fmt --all`, all-target/all-feature deny-warnings Clippy, all-target/all-feature build, and all-feature test gate passed in an isolated target directory before the task commit.

## Requirements Status

Plan 11-23's `TEST-03`, `EXMP-01`, and `EXMP-03` mappings are complete for the four exact semantic headless rows. This does not claim visualization pixels, timing, broad platforms, performance, packaging readiness, release readiness, or general LiquidFun parity.

## User Setup Required

None.

## Next Phase Readiness

- The reviewed D3 ledger state is ready for downstream Phase 11 corpus closure, verification, and final integration.
- The exact authority and validator make later drift or attempted scope expansion fail visibly.
- No Plan 11-23 blocker remains.

## Known Stubs

None.

## Self-Check: PASSED

- Confirmed all nine task files exist and task commit `f1c88c8` is present.
- Confirmed the modified-file stub scan found no goal-blocking placeholder or disconnected mock data.
- Confirmed no security-relevant surface escaped the plan threat model.
- Confirmed the summary contains no standalone body separator that could confuse frontmatter parsing.

***

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
