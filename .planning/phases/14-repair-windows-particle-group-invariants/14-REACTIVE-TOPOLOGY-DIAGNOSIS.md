# Reactive topology rejection classification

Recorded 2026-09-14 during Plan 14-03. This is a diagnosed rejection-contract issue, not evidence of corrupt authoritative storage.

## Reproduction

- Source base: `7b8221beb445e18e914d871065ef589432bf375f`, with the three in-progress strict property-model files copied into an isolated worktree.
- Seed: `9340296914046120310`.
- Controls: `[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5,0,199,242,0,147]`.
- Operations 19/20 create colocated `SPRING | ELASTIC | REACTIVE` groups. Step 22 encounters coincident particles 16/19 at `(4, 0)`.
- Observed chain: `ZeroLengthPairDistance` → `InvalidLaneBundle` → `StepError::ParticleLifecycleInvariant`.
- Every observed authoritative storage invariant passes before and after operations 1–21 and after the failed step. The failed step preserves the semantic snapshot, leaves 22 particles, and leaves the world unlocked and unpoisoned.
- Two identical reactive groups followed by one step reproduce; shifting the second group succeeds.

## Existing contract and correction

Phase 10's `reference/artifacts/phase10/group-topology-witnesses.json` records `zero_length_pair` as `typed_error`, with upstream NaNs. That rejection remains mandatory. No epsilon, permissive zero-rest constraint, solver-geometry change, or broader step-transaction promise is introduced.

The old public invariant-error documentation describes authoritative corruption, which does not describe this observed rejection. The narrow correction preserves a private generation-versus-storage error cause and introduces `StepError::InvalidParticleGroupTopology` for the documented zero-rest pair rejection. Actual storage errors and all other generation errors retain `ParticleLifecycleInvariant`. Private raw constraint details remain private. This is the evidence-backed need for a distinct recovery contract: callers can distinguish unsupported geometry from a storage bug.

The five-file production patch is recorded as Task 0 in Plan 14-03. Permanent public exact/minimal rejection and shifted-success regressions belong to Task 1, along with strict matching of the new domain error. `ParticleLifecycleInvariant` remains unexpected in that model.

## Evidence and limits

Retained material: `target/phase14-reactive-diagnosis/attempt-20260914/`.

- `exact.log` and `minimal.log`: original red observations.
- `classification-check.log` and `final-check-03.log`: passing isolated checks.
- `final-check.log` and `final-check-02.log`: retained failed verification attempts.
- `production.patch`: SHA-256 `cd59f16b27e7414c358e69418fb2bdac7af42bebc10591d1e2a4208e6e8077e0`.
- `HANDOFF.md`: precise commands and ownership.

The isolated Linux ARM64 Rust 1.97.0 checks passed nine reactive private tests, seven temporary public tests (including the 128-case property run), package Clippy for all targets/features, formatting, and diff checks. A real corrupt-lane control remained a storage error. The isolated full-workspace build was stopped incomplete; it is not a passing result. Main-tree combined verification and final platform acceptance remain required.

## Amendment review correction

Independent amendment review rejected a blanket generation-error classifier. The main worktree now uses `is_zero_rest_pair_rejection`, matching only `ConstraintError::ZeroLengthPairDistance`, and adds a negative test for other generation errors. The original isolated patch and its passing logs remain historical evidence; this narrowed combined candidate still requires fresh full verification. The public variant documentation now states its zero-rest-pair scope.

## Final local verification

After the explicitly authorized Docker restart, the narrowed main-tree candidate passed the ordered Rust 1.97.0 formatting, Clippy, all-target/all-feature build and all-feature test gate on 2026-09-14. The full gate records 1,012 passing tests including doctests; the separately invoked private group module records 16 passing tests. The log is `target/phase14-public/attempt-final-task0/gate.log`. The run used the same pinned Linux ARM64 image and isolated target volume as prior local checks. This closes local verification of the narrowed classifier; supported Windows, Linux and macOS final-candidate evidence remains Plan 14-04 work.
