---
phase: 11-examples-headless-tooling-and-testbed
plan: "03"
subsystem: testing-tooling
tags: [catalog, deterministic-replay, rand-chacha, serde-json, sha256]
requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Strict exact-bit protocol IDs, bounded JSON, and SHA-256 identity types
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    provides: Complete closed rigid-world action vocabulary for shared scenario schedules
provides:
  - Versioned stable-slug catalog definitions independent of display titles
  - Bounded deterministic named and ChaCha8-seeded resolved action plans
  - Canonical JSON replay bytes with verified SHA-256 content identity
affects: [phase11-controller, phase11-headless-runner, phase11-regressions, phase11-testbed]
tech-stack:
  added: [rand_chacha 0.10.0]
  patterns:
    - Resolve private typed definitions into immutable engine-neutral plans before effects
    - Validate and re-encode persisted bytes before accepting their content-addressed identity
key-files:
  created:
    - crates/liquidfun-test-protocol/src/catalog.rs
    - crates/liquidfun-test-protocol/src/catalog/model.rs
    - crates/liquidfun-test-protocol/src/catalog/model/identity.rs
    - crates/liquidfun-test-protocol/src/catalog/resolve.rs
    - crates/liquidfun-test-protocol/tests/catalog_resolution.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/liquidfun-test-protocol/Cargo.toml
    - crates/liquidfun-test-protocol/src/lib.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs
key-decisions:
  - "Keep display titles outside canonical resolved bytes so presentation changes cannot alter replay identity."
  - "Use exact-pinned ChaCha8 with explicit seed expansion and generator version 1 for seeded choice resolution."
  - "Bind checkpoints to stable action and logical-step ordinals, never render frames or wall time."
patterns-established:
  - "Catalog resolution: validate definition and request boundaries, generate stable ordinals, then hash canonical compact JSON."
  - "Catalog replay: bound bytes and collections, verify SHA-256, validate internal identity/action consistency, and require byte-for-byte canonical re-encoding."
requirements-completed: [EXMP-02, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-21T23:02:52Z
duration: 14 min
completed: 2026-07-21
---

# Phase 11 Plan 03: Canonical Scenario Catalog Resolution Summary

**Stable named and `ChaCha8`-seeded catalog definitions now resolve into bounded exact-bit action schedules whose canonical JSON bytes and SHA-256 identity replay without backend, filesystem, clock, or renderer effects.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-21T22:49:06Z
- **Completed:** 2026-07-21T23:02:52Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- Added validated catalog slugs, schema/scenario/generator versions, generator IDs, exact run settings, eligibility, semantic entity/action identities, ordered schedules, checkpoint declarations, run identities, and immutable resolved scenarios.
- Added pure deterministic named and `ChaCha8`-seeded resolution with reviewed definition, entity, action, checkpoint, choice, iteration, and canonical-byte limits.
- Added strict content-addressed replay that rejects oversized or malformed records, unknown fields, noncanonical encoding, hash mismatch, unstable ordinals, invalid checkpoint references, and action/settings disagreement.
- Added eight focused integration tests covering named and seeded goldens, repeated byte identity, mutable-title independence, exact run identity, decode replay, N/N+1 bounds, seed/settings rejection, and tampering.

## TDD Evidence

- **RED:** The first catalog-resolution target run failed both named and seeded golden SHA-256 expectations, establishing fixed canonical-byte identities before acceptance.
- **GREEN:** The two reviewed golden hashes were recorded and all seven original acceptance tests passed.
- **REFACTOR:** Identity types moved into `model/identity.rs` to keep the main model below the repository's deep-module size trigger; replay consistency coverage then increased the target to eight passing tests.

## Task Commits

Each task was committed atomically:

1. **Rule 3 blocker: Borrow Phase 10 schema values for strict Clippy** - `b1747e4` (fix)
1. **Task 1: Implement catalog contracts and pure deterministic resolution** - `ad3610c` (feat)

**Plan metadata:** committed with this summary.

## Files Created/Modified

- `Cargo.toml` and `Cargo.lock` - Pin and resolve private `rand_chacha` 0.10.0 without changing `liquidfun` dependencies.
- `crates/liquidfun-test-protocol/Cargo.toml` - Adds the deterministic generator dependency only to the unpublished protocol crate.
- `crates/liquidfun-test-protocol/src/lib.rs` and `src/catalog.rs` - Export the new catalog contracts and resolver through the existing private protocol surface.
- `crates/liquidfun-test-protocol/src/catalog/model.rs` - Owns catalog definitions, programs, schedules, checkpoints, entities, run identities, resolved plans, bounds, and bounded errors.
- `crates/liquidfun-test-protocol/src/catalog/model/identity.rs` - Owns invariant-bearing slugs, versions, generator/action IDs, eligibility, and exact validated run settings.
- `crates/liquidfun-test-protocol/src/catalog/resolve.rs` - Selects definitions, generates deterministic plans, encodes canonical JSON, hashes exact bytes, and validates replay.
- `crates/liquidfun-test-protocol/tests/catalog_resolution.rs` - Proves golden identities, deterministic replay, bounds, validation, title independence, and tamper rejection.
- `crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs` - Borrows schema values at the helper boundary to clear a pre-existing strict-Clippy blocker without changing output.

## Decisions Made

- Display titles remain definition metadata only. Stable slug, versions, generator, seed, exact settings, entities, actions, and checkpoints form canonical replay bytes.
- The seeded generator is explicitly `ChaCha8` from exact-pinned `rand_chacha` 0.10.0; a versioned deterministic 32-byte expansion makes the `u64` seed input unambiguous.
- Canonical replay verifies an externally carried SHA-256, parses under fixed byte and collection bounds, checks semantic ordinal and settings consistency, and rejects any encoding that does not re-encode byte-identically.
- Setup gravity is action ordinal zero; each later action executes exactly one logical step and owns the checkpoint with the same one-based ordinal.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Borrowed Phase 10 schema helper inputs to satisfy strict workspace Clippy**

- **Found during:** Task 1 pre-commit workspace gate
- **Issue:** Existing `array_schema(item: Value, ...)` triggered `clippy::needless_pass_by_value`, blocking the mandatory warning-denied gate independently of Plan 11-03 behavior.
- **Fix:** Changed the helper to accept `&Value` and borrowed only its nine direct temporary call sites, preserving generated schema semantics.
- **Files modified:** `crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs`
- **Verification:** The exact ordered format, full-workspace Clippy, all-target build, and all-feature test gate passes.
- **Committed in:** `b1747e4`

**Total deviations:** 1 auto-fixed (1 Rule 3 blocking issue).
**Impact on plan:** The isolated semantics-preserving fix only restored the required verification gate; no catalog scope or Phase 10 behavior changed.

## Issues Encountered

- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and were not committed or reverted.

## Security Verification

- Resolver input is parsed into validated slug, version, generator, exact-bit setting, eligibility, and bounded collection types before generation.
- Replay verifies exact SHA-256 bytes, rejects oversized and noncanonical records, and checks action/settings, ordinal, and checkpoint consistency after strict deserialization.
- Work and allocation are bounded by 256 definitions, 4,096 entities, 128 actions/checkpoints/choices, 1,024 solver iterations, and 1 MiB canonical bytes.
- Errors expose only bounded semantic categories and never raw records, pointers, storage indices, secrets, filesystem paths, or stderr.
- `liquidfun` remains dependency-unchanged and free of protocol, renderer, process, C++, and foreign runtime dependencies.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

The frontmatter preserves Plan 11-03's `EXMP-02` and `EXMP-03` mappings. Their global requirement checkboxes remain pending until later Phase 11 plans connect controllers, native/oracle adapters, regression and benchmark consumers, headless commands, and end-to-end verification to these canonical resolved bytes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Renderer-neutral controllers and backend adapters can now consume one immutable resolved plan with stable action/checkpoint identities.
- Regression and failure pipelines can persist canonical bytes plus SHA-256 instead of relying on a seed alone.
- Global `EXMP-02` and `EXMP-03` completion remains deferred to Phase 11 integration and verification.

## Self-Check: PASSED

- Confirmed all five created catalog/test files and the summary exist.
- Confirmed blocker commit `b1747e4` and task commit `ad3610c` exist; the task commit contains only its nine planned catalog/dependency artifacts.
- Confirmed focused tests pass 8/8 and the exact ordered format, full-workspace Clippy, all-target build, and all-feature test gate passes with the required temporary target directory.
- Confirmed no known stub or unplanned threat surface was introduced and all four unrelated shared-tree edits remain unstaged.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
