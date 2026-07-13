---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "09"
subsystem: rigid-continuous-collision
tags: [rust, ccd, toi, sub-stepping, transactional-solver, work-budget]
requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "04"
    provides: Transactional source-ordered discrete islands, complete BodyState staging, and checked proxy synchronization
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "08"
    provides: Deterministic CCD candidate selection, contact TOI cache lifecycle, and exact pending-step identity
provides:
  - Pinned 64-body and 32-contact source-ordered TOI islands with complete per-event rollback
  - TOI-specific seed position solving and cold velocity solving without discrete impulse-cache pollution
  - Automatic discrete-then-continuous World::step orchestration with tokenless sub-step resume
  - Checked aggregate continuous-work budgets with bounded semantic partial-progress evidence
  - Bullet anti-tunneling witnesses across static, kinematic, and permitted dynamic targets
affects: [07-11, phase-7-rigid-evidence, world-step-contract]
tech-stack:
  added: []
  patterns: [whole-event transaction, cold TOI constraint solve, private resume checkpoint, semantic budget evidence]
key-files:
  created:
    - crates/liquidfun/src/world/contact_solver/toi.rs
    - crates/liquidfun/src/world/continuous/event.rs
    - crates/liquidfun/src/world/island/toi.rs
    - crates/liquidfun/src/world/step/continuous.rs
  modified:
    - crates/liquidfun/src/world/continuous.rs
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/rigid_differential.rs
    - crates/liquidfun/tests/rigid_ccd.rs
key-decisions:
  - "Use the pinned 64-body and 32-contact TOI capacities and expand only through seed-body adjacency in source order, stopping cleanly at capacity rather than exposing dynamic solver allocation."
  - "Back up complete participating world state before each accepted TOI event and restore it on every scan, island, solve, or proxy failure so no partial continuous event can escape."
  - "Resume continuous work only for an exact matching StepConfiguration and skip pair discovery, callbacks, force integration, and the discrete solver on that continuation."
  - "Represent aggregate budget exhaustion as a typed semantic checkpoint containing only discrete completion and committed-event count; retain all cache, sweep, and contact resume state privately."
patterns-established:
  - "TOI solve isolation: seed bodies use the pinned position tolerance and pass count, velocity constraints always cold-start, and transient continuous impulses never enter the discrete warm-start cache."
  - "Continuous step lifecycle: a fresh call runs one discrete stage, then commits whole TOI events until complete, one-event sub-step pending, or a coherent checked budget boundary."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T07:07:56Z
duration: 41 min
completed: 2026-07-13
---

# Phase 7 Plan 09: TOI Islands, Sub-Stepping, and Work Budgets Summary

**Rigid stepping now resolves eligible fast contacts through bounded transactional TOI islands, supports tokenless one-event sub-stepping, and exposes coherent resumable budget exhaustion without leaking CCD internals.**

## Performance

- **Duration:** 41 min
- **Started:** 2026-07-13T06:26:55Z
- **Completed:** 2026-07-13T07:07:56Z
- **Tasks:** 2
- **Files modified:** 22

## Accomplishments

- Added hard-pinned TOI island construction that starts from one accepted seed, expands seed-body contact adjacency in source order, honors dynamic/static traversal and sensor/bullet exclusions, and stops within 64 bodies and 32 contacts.
- Added a TOI-specific solver with the pinned 20-pass seed-body position correction, TOI Baumgarte/tolerance behavior, cold velocity constraints, transient semantic impulse evidence, and no discrete warm-start cache writes.
- Made each continuous event transactional across body transforms, sweeps, velocities, forces, sleep state, contact cache/count state, contact topology, and proxy synchronization; injected late failure proves exact rollback.
- Integrated continuous work into `World::step`: the discrete lifecycle runs once, complete mode drains accepted events, sub-stepping returns `ContinuousPending` after one event, and a matching continuation skips all discrete work.
- Added one checked per-call continuous-work budget. Exhaustion occurs only between complete events and returns public `ContinuousProgress` with bounded semantic classification while the same world remains resumable.
- Added static, kinematic, and dynamic bullet anti-tunneling witnesses, force-clear-on-pending coverage, and explicit discrete-only setup for legacy solver/lifecycle tests whose contracts intentionally exclude the newly active CCD stage.

## Task Commits

Each task was committed atomically after the exact ordered Rust gate passed:

1. **Task 1: Build and solve bounded TOI islands transactionally** - `f4859a4` (`feat`)
2. **Task 2: Integrate complete, pending, and budget-limited CCD stepping** - `2d9e62d` (`feat`)

## Files Created/Modified

- `crates/liquidfun/src/world/island/toi.rs` - Fixed-capacity source-ordered TOI island scratch and complete body/contact inputs.
- `crates/liquidfun/src/world/contact_solver/toi.rs` - Cold TOI position/velocity constraint kernel with seed-only position correction.
- `crates/liquidfun/src/world/continuous/event.rs` - Whole-event backup, validation, solve, proxy preparation, commit, and rollback orchestration.
- `crates/liquidfun/src/world/step/continuous.rs` - Continuous completion/pending/budget coordinator and typed error translation.
- `crates/liquidfun/src/world/continuous.rs` - Production candidate consumption and exact fresh-versus-resumed checkpoint state.
- `crates/liquidfun/src/world/step.rs` - Checked aggregate work limit, public semantic progress, and discrete-once world-step integration.
- `crates/liquidfun/src/world/body.rs` - Checked TOI body-state commit preserving initial sweep anchors.
- `crates/liquidfun/src/world/contact_manager.rs` - Transactional manager state, TOI invalidation, and continuous refresh integration.
- `crates/liquidfun/src/rigid_differential.rs` - Feature-gated semantic TOI island/order/impulse/failure witnesses.
- `crates/liquidfun/tests/rigid_ccd.rs` - Island bounds, cold-impulse isolation, rollback, resume, budget, and anti-tunneling tests.
- `crates/liquidfun/tests/rigid_world_config.rs` - Successful continuous-pending automatic force-clear witness.
- `crates/liquidfun/tests/rigid_ccd_selection.rs`, `rigid_contact_solver.rs`, `rigid_contacts.rs`, and `rigid_island_solver.rs` - Explicit discrete-only fixture setup where the asserted contract intentionally predates automatic CCD orchestration.

## Decisions Made

- Retained the upstream fixed 64-body/32-contact island bounds as hard private production limits. The builder preserves seed and adjacency order and treats capacity as a deterministic traversal stop rather than growing an attacker-controlled work graph.
- Used a complete event transaction instead of a field-by-field undo path. Body storage and the contact manager are copied before candidate selection, while all fallible synchronizations are prepared before live proxy replacement.
- Kept the continuous velocity solve entirely cold and transient. Cached discrete impulses are neither applied nor overwritten, so a later discrete step sees only discrete-solver history.
- Made zero a valid continuous-work budget because the boundary immediately after a committed discrete stage is coherent and useful. A matching later call consumes the private checkpoint without repeating integration or callbacks.
- Kept the public checkpoint intentionally small: whether the discrete stage completed and how many whole continuous events this call committed. Candidate indices, queues, contact counts, alphas, and sweep state remain private.
- Invalidated a pending continuation on continuous/sub-step configuration changes and existing body/fixture mutations. Exact matching step inputs are required before the discrete stage may be skipped.

## Test Evidence

- Task 1 RED exited 101 on the absent TOI event diagnostic and island/solver types; all three planned transactional island witnesses then passed GREEN.
- Task 2 RED exited 101 on the absent continuous-work builder and typed budget error; all four planned step/resume/anti-tunneling/force-clear witnesses then passed GREEN.
- Focused final checks passed:
  - `cargo test -p liquidfun --all-features --test rigid_ccd` - 6/6
  - `cargo test -p liquidfun --all-features --test rigid_world_config` - 13/13
  - `cargo test --all-features --test rigid_ccd_selection` - 3/3
  - `cargo test --all-features --test rigid_contact_solver` - 8/8
  - `cargo test --all-features --test rigid_contacts` - 10/10
  - `cargo test --all-features --test rigid_island_solver` - 11/11
- Before both retained task commits, the exact ordered gate passed with authoritative exit code 0:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- The final full gate included 150 library tests, every integration target including all new CCD witnesses, and all 12 doctests.
- GSD artifact verification passed 2/2. Its textual key-link heuristic reported 0/2 because the cohesive implementation is split through Rust submodules rather than source-path string references; manual symbol tracing verified `continuous/event.rs -> solve_toi_island -> contact_solver/toi.rs` and `step.rs -> step/continuous.rs -> solve_next_continuous_event`.

## Simplification Review

- Split cohesive kernels at their domain boundaries: island construction in `island/toi.rs`, constraint solving in `contact_solver/toi.rs`, event transaction in `continuous/event.rs`, and public-step coordination in `step/continuous.rs`.
- The resulting new modules are approximately 204, 167, 298, and 102 lines respectively; `continuous.rs` remains about 620 lines and the established `step.rs` discrete lifecycle remains large but gained only narrow checked-limit and routing changes.
- Reused the existing `BodyState`, `ContactManager`, `PreparedSynchronization`, `Sweep`, and contact constraint representations. No new dependency, unsafe code, continuation token, queue abstraction, or alternate collision kernel was introduced.
- Whole-state backup plus prepare-before-commit is smaller and more auditable than compensating mutation logic across body, contact, and proxy lanes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exported semantic budget progress and invalidated configuration changes**

- **Found during:** Task 2 public-step integration
- **Issue:** The planned typed budget error required a crate-root export, and changing continuous/sub-step flags while pending had to invalidate the private checkpoint.
- **Fix:** Updated `lib.rs`, `world.rs`, and `world/config.rs` with the semantic export and no-token invalidation boundary.
- **Verification:** Full Clippy, build, integration, and doctest gate passed.
- **Committed in:** `2d9e62d`

**2. [Rule 3 - Blocking] Made legacy discrete test contracts explicit**

- **Found during:** Task 2 full-suite verification
- **Issue:** CCD selection setup and discrete contact/island suites relied on the former absence of automatic CCD in `World::step`; the newly correct default continuous stage added TOI correction and same-call contact refresh evidence.
- **Fix:** Disabled continuous physics only in helpers whose documented contract is discrete setup or discrete solver/lifecycle behavior, restoring it after CCD selection setup where needed.
- **Verification:** All affected targets and the final full suite passed without weakening production CCD behavior.
- **Committed in:** `2d9e62d`

**3. [Rule 2 - Missing Critical] Separated force-clear evidence from CCD motion**

- **Found during:** Task 2 force-clear witness
- **Issue:** Applying `f32::MAX` force to the swept bullet changed the collision witness itself, so the test completed without a pending TOI event.
- **Fix:** Accumulated the sentinel force on an inactive dynamic body in the same world, proving the global successful-step clear policy without perturbing the CCD scenario.
- **Verification:** The focused pending force-clear test and all 13 world-configuration tests passed.
- **Committed in:** `2d9e62d`

***

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** All changes were necessary to expose the planned semantic contract, preserve checkpoint correctness, or keep pre-existing discrete tests scoped to their declared behavior. Production CCD scope did not expand beyond Plan 07-09.

### Process adjustment: RED evidence was not committed

- Repository policy requires the complete ordered Rust gate before every commit. Both intentionally failing RED states were run and recorded but not retained; only verified GREEN task states were committed.

## Issues Encountered

- Long full-suite output exceeded the first retained desktop session's direct result window. The suite was rerun with stdout suppressed and an outer wait budget large enough to capture the authoritative process exit before any commit.
- The authoritative full suite exposed three groups of legacy fixtures that were correctly exercising discrete-only behavior but had relied on CCD not yet being integrated. Each fixture now declares its discrete boundary explicitly; production remains continuous-enabled by default.
- `roadmap update-plan-progress 07` reported success without changing the numeric Phase 7 row because the CLI row matcher does not normalize a leading zero. Re-running the same GSD command with phase `7` updated the row to 10/13.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-11 can consume real complete/pending/budget-limited world-step outcomes and semantic continuous contact/body evidence without depending on private caches or resume internals.
- The Phase 7 evidence matrix can now include bullet-versus-static, kinematic, and dynamic anti-tunneling scenarios backed by complete transactional rollback and deterministic work bounds.
- No production blocker remains. The remaining Phase 7 work is differential/evidence completion rather than TOI island or world-step plumbing.

## Self-Check: PASSED

- Task commits `f4859a4` and `2d9e62d` exist; all four declared created modules and every key modified file exist.
- The 22-file implementation/test diff is represented above, and every focused and exact full verification command passed.
- Diff review found no partial event commit, warm-start cache pollution, repeated discrete work on resume, public CCD storage/token, unsafe code, new dependency, unbounded TOI island, or unrelated production change.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*
