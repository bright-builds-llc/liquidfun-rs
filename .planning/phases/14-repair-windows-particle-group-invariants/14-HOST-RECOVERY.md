---
phase: 14-repair-windows-particle-group-invariants
status: resolved
current_plan: "14-03"
updated: 2026-09-14
blocker: Host verification recovery and Docker restart decision
---

# Phase 14 execution handoff

## User objective and authority

Execute all of Phase 14 through verification. Plans 01 and 02 are complete; Plan 03 has prepared uncommitted changes; Plan 04 and final review/verification remain. Standing authorization in AGENTS.md permits ordinary repository repair, commits, main pushes and validation workflows. It does not authorize restarting unrelated running containers. A Docker Desktop restart question is pending; no approval had been received when this handoff was written. Do not infer approval from elapsed time.

## Completed and published

- Current published HEAD: `7b8221beb445e18e914d871065ef589432bf375f`.
- `72852a2`: revised four-plan execution set.
- `8b4eb56`: required formatter plugins in Cargo CI; isolated Python reproduction proved the missing GFM plugin.
- Plan 01 commits `a2759db`, `f4ef739`, `49e382f`: exact Windows diagnosis and complete probe removal.
- `030c893`: private CLI evidence test extraction; workspace Clippy restored.
- `92a7519`: diagnosis handoff.
- Plan 02 commits `e58e028`, `88643e0`, `800a51c`, `265598a`: actual-row scratch allocation, permanent original replay success, complete private rollback and candidate-only allocation-error handling.
- `7b8221b`: Wave 2 tracking handoff.
- Final Plan 02 local gate passed 998 tests with Rust 1.97.0 in the local Linux ARM64 container.
- Exact earlier sizing candidate `e58e028` passed default-feature Windows, Linux and macOS jobs in run `34800684869`. Its quality job failed three incomplete performance CLI fixtures; do not call the entire run green or attribute that platform proof to later commits.

## Prepared main-tree work — preserve all changes

Plan 03 Task 0 adds a precise zero-rest particle-topology step error through five owned files. The exact additional seed and minimal reproduction prove valid unchanged storage and deliberate Phase 10 zero-rest rejection, not corruption. The final main-tree classifier was narrowed after review to **only** `ConstraintError::ZeroLengthPairDistance`; all other generation/storage errors retain their old boundary. The latest negative classification test has not completed its fresh gate.

Plan 03 Tasks 1/2 preserve both Windows seeds, persist three newly discovered inputs, retain strict unexpected-error failures, add minimal zero-rest/shifted controls, strengthen full public snapshots and add nine focused public rejection/follow-up cases. Task 1/2 files are source-complete but uncommitted; there is no 14-03 SUMMARY. The property model accepts only the precise new topology variant, not ParticleLifecycleInvariant. PendingDelete is accepted only with a nonzero pending-row witness.

The coordinator also changed only `tools/xtask/tests/performance_cli.rs` to own isolated policy/manifest/oracle-sentinel fixtures and added missing-oracle negative tests. Its focused verification remains pending. Preserve this separate tooling change and commit it separately after gates pass.

No files are staged. Review `git status --short --untracked-files=normal`; untracked public test children and the proptest persistence file are intentional. Do not overwrite the main property files with isolated diagnostic copies.

## Evidence

- Original Windows allocation cause: 8,589,934,588-byte scratch reservation for ten rows, based on declared i32::MAX capacity. Full retained diagnosis and summaries are committed.
- `target/phase14-platform/e58e028a31ffe845c5465e8d72dd5d933c9d80f5/attempt-20260914-initial-repair/`: all three passing platform logs, identities/digests, failed quality log and final run status.
- `target/phase14-public/`: Plan 03 failed stricter-model attempts, minimized inputs and earlier focused passes.
- `target/phase14-reactive-diagnosis/attempt-20260914/`: exact/minimal red logs, isolated passing checks, original production patch, failed attempts and HANDOFF. Main adds the later narrowed classifier beyond that original patch.
- `14-REACTIVE-TOPOLOGY-DIAGNOSIS.md`: source/error-contract findings and classification correction.
- `14-CI-PERFORMANCE-REPAIR.md`: three fixture failures and pending fixture-only correction.

## Verification environment failure

Docker Desktop now returns input/output errors reading both cached image blobs and writing `io.containerd.metadata.v1.bolt/meta.db`. New containers and removal of the stopped owned diagnostic container fail. Do not factory-reset Docker or delete user images/volumes. The exact owned diagnostic container `1f8350e8a6d0` was stopped; deletion failed. Its newly created build-cache volume `liquidfun-phase14-step-debug-target` remains referenced. Source and diagnostic logs are retained outside that disposable cache.

The prior working Docker route used image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, explicit `RUSTUP_TOOLCHAIN=1.97.0`, repository at `/workspace`, `CARGO_TARGET_DIR=/cargo-target`, and volumes `liquidfun-rs-rustup` (read-only at `/usr/local/rustup`), `liquidfun-rs-cargo-registry` (at `/usr/local/cargo/registry`), and `liquidfun-rs-cargo-target` (at `/cargo-target`). Recheck toolchain/image availability after recovery.

Native macOS formatting, Clippy and build completed for the pre-review candidate, but executable startups again stalled. The partial native test/startup runs were stopped when the classification review required a new candidate. Logs remain under `target/phase14-local-verification/attempt-20260914-native-static/`; they are not a passing full gate for the latest source. Latest formatting/Markdown/managed/diff checks may pass independently; never substitute them for required tests.

## Resume sequence

1. Resolve the pending host action with the user. `docker desktop restart --timeout 180` is available, but execute it only after explicit approval because other containers are interrupted. A restart is not permission for reset/deletion.
1. Confirm Docker I/O recovery and Rust 1.97.0, or establish a reliable alternative local execution route.
1. Recheck the narrowed Task 0 amendment and run focused reactive/property/public group suites. Expected final property target has the four existing/fixed tests plus minimal rejection and shifted-success tests; require nonzero/exact selection counts.
1. Test `performance_cli` with an empty `/workspace/target` and empty native checkout overlay; keep Cargo output on its separate volume. Verify both positive fake-provider behavior and missing-oracle rejection. Do not add C++ to Cargo-only CI.
1. Run full ordered fmt → Clippy → build → tests and the managed/Markdown/diff checks. Fix actual failures; preserve failed attempts. No failing or unverified commit.
1. Coordinate staging: separate tooling fixture repair, Task 0 error classification, Task 1 property changes/plan, and Task 2 public focused tests. Existing main changes are disjoint ownership areas. Create 14-03 SUMMARY only when complete.
1. Complete Plan 04 with actual same-SHA platform and quality results, then required code review, regression/schema checks, security routing and independent phase verification. Keep Phase 15 and canonical promotion deferred.

## Recovery resumed — 2026-09-14T04:52:14.771401Z

The user explicitly approved restarting Docker Desktop in the current task. The restart completed; Docker 29.3.1 can read the pinned image again, and a container verified Rust 1.97.0 and Clippy. The prior pending-approval text above is historical. Plan 03 focused verification has resumed against the preserved worktree; no factory reset or user-data deletion occurred.

## Recovery completed

Docker restart, focused checks and the full ordered gate succeeded. Plan 03 is committed with 1,012 passing tests; the earlier blocked candidate and failed logs remain historical. This file is archived as the host-recovery record. Plan 04 now owns final platform acceptance.
