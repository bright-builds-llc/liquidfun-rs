---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-25T18:25:02.901Z
---

# Phase 13: Restore Evidence and Replay Integrity - Context

**Gathered:** 2026-07-25
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Restore exact-head Phase 9 lifecycle/contact witness provenance and the reviewed `rigid-stack-v1` catalog regression so the current native and C++ comparison surface is source-bound, independently reproducible, and accepted at one exact Linux commit. This phase repairs provenance, replay, first-divergence, regression, and Oracle CI wiring; it does not repair the separate Windows particle-group invariant or produce the final v1 release-candidate bundle.

</domain>

<decisions>
## Implementation Decisions

### Exact-head provenance authority

- **D-01:** Replace the Phase 9 witness's broad aggregate adapter digest with a mechanically enumerated target-scoped materials identity covering every source, wrapper, build rule, flag, and generated input that can affect `phase9-lifecycle-contact-witness`. Validation must fail if that dependency closure is incomplete or any declared material changes.
- **D-02:** Regenerate the witness only with the pinned C++ oracle on the canonical Linux compiler, target, and preset. Require two byte-identical generation runs, validate witness and provenance together, expose a reviewable diff, and promote the pair atomically.
- **D-03:** Do not treat a matching edited hash as sufficient evidence. The accepted record must bind the exact repository commit, upstream revision, scoped materials digest, probe digest, compiler/build identity, exact invocation, witness digest, and explicit review.

### Independent `rigid-stack-v1` replay recovery

- **D-04:** Diagnose the first changed semantic path before editing the reviewed identity. Prove whether the drift is a physics defect, resolved-scenario drift, or an intentional checkpoint/capture-schema expansion; the unchanged resolved bytes alone do not authorize re-baselining.
- **D-05:** If newly authoritative fields intentionally changed the capture contract, establish the corrected identity from the same sealed resolved bytes with two byte-identical native D0 runs and a passing pinned-oracle D1 comparison on canonical Linux. Only then may an explicit reviewed promotion update the accepted identity and provenance.
- **D-06:** If the changed fields are presentation or diagnostics that were never intended to redefine the historical physics identity, preserve the reviewed D0 identity through a versioned legacy projection and validate the expanded current checkpoint separately. Never silently remove parity-bearing fields from authority.
- **D-07:** Replace the current native-only “independent” identity check with an evidence path that is genuinely independent of the implementation being tested. No expected value may be copied from the Rust assertion output, native catalog backend, or a locally convenient D2 run.

### Exact-head Linux and Oracle CI acceptance gate

- **D-08:** Add one dedicated atomic Phase 13 acceptance gate at a single checked-out full SHA. It must compose existing commands in fail-closed order: exact-head identity, provenance validation, reviewed catalog replay, first-divergence contract tests, tracked regressions, canonical oracle configure/build, and current native/oracle comparisons.
- **D-09:** Assert the same full SHA before and after the gate, retain the exact failing request and first divergent semantic path on mismatch, and publish the gate identity only after every required step passes. A skipped or missing step is failure, not success.
- **D-10:** Keep this gate focused on Phase 13 closure. Phase 15 may consume its commit-bound evidence later, but Phase 13 must not expand into the final multi-platform release aggregation or attestation flow.

### the agent's Discretion

- Exact names and schema versions for the scoped-materials manifest, replay evidence record, thin aggregate command, and CI job.
- Whether the atomic gate is a new workflow or a clearly isolated required job, provided it has one exact-SHA authority and does not rely on an ambiguous cross-workflow join.
- Exact plan decomposition, provided diagnosis precedes fixture promotion and independently generated evidence remains a hard gate.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gap authority and phase scope

- `.planning/v1.0-MILESTONE-AUDIT.md` — Exact-head disconfirming evidence, failing CI boundaries, requirement gaps, and the three-way separation between Phase 13 provenance/replay work, Phase 14 Windows repair, and Phase 15 candidate evidence.
- `.planning/ROADMAP.md` — Fixed Phase 13 goal, requirements, dependency, and success criteria.
- `.planning/REQUIREMENTS.md` — `FND-04`, `COMP-04`, `COMP-05`, `COMP-08`, `TEST-07`, and `EXMP-03` contracts that Phase 13 must restore.

### Locked evidence and replay contracts

- `.planning/phases/01-oracle-provenance-and-repository-foundation/01-CONTEXT.md` — Immutable oracle, machine-readable provenance, canonical build identity, reviewed artifacts, and prohibition on silent regeneration.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — Provenance-first comparison, exact resolved input, first divergence, replay, minimization, atomic promotion, and failure taxonomy.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md` — Phase 9 lifecycle/contact witness purpose and upstream-derived evidence boundary.
- `.planning/phases/11-examples-headless-tooling-and-testbed/11-CONTEXT.md` — One shared resolved scenario catalog, persisted resolved bytes, replay identity, and native/oracle/regression/testbed consumers.
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md` — D0/D1 authority, canonical Linux promotion boundary, exact-commit evidence, and fail-closed release aggregation constraints.
- `.codex/tasks/lessons.md` — Repository lessons requiring explicit CI identities, canonical preset validation, Cargo/oracle isolation, and a ban on self-blessed exact bits.

### Existing implementation seams

- `tools/xtask/src/provenance/phase9_witness.rs` — Current Phase 9 witness and provenance validator, including the aggregate adapter digest that exact-head CI rejects.
- `tools/reference/src/phase9_lifecycle_contact_witness.cpp` — Pinned-oracle probe whose scoped materials and output identity must be bound.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.json` — Reviewed Phase 9 witness output.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` — Current witness provenance record to migrate and regenerate.
- `crates/liquidfun-differential/tests/catalog_regressions.rs` — Failing reviewed replay and native D0 identity tests, including the current non-independent helper.
- `scenarios/catalog/rigid-stack-v1.json` — Shared resolved scenario source whose canonical bytes remain the replay input authority.
- `crates/liquidfun-differential/src/runner/catalog.rs` — Exact resolved-byte native/oracle execution and D0 comparison seam.
- `crates/liquidfun-differential/src/fixtures/replay/catalog.rs` — Tracked catalog regression manifest, replay, and failure classification.
- `crates/liquidfun-differential/src/failure_bundle/catalog.rs` — Persisted resolved input and first-divergence evidence seam.
- `.github/workflows/ci.yml` — Existing Cargo and catalog-regression checks.
- `.github/workflows/oracle.yml` — Existing canonical Linux oracle toolchain, provenance, build, and comparison path.
- `justfile` — Thin contributor command facade that any new Phase 13 entrypoint must preserve.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- The Phase 9 validator already checks the witness and provenance pair fail-closed; it can be deepened with a scoped materials record instead of replaced.
- The catalog runner already preserves exact resolved bytes and identities across native and supervised C++ execution.
- Catalog regression replay, failure bundles, comparator first-divergence paths, and existing CI jobs provide the mechanics needed for a thin Phase 13 aggregate gate.

### Established Patterns

- Machine-readable typed records are authoritative; Markdown and console output are projections.
- Canonical scalar Linux is the only promotion authority. Local/macOS/Windows runs are D2 evidence and cannot bless fixtures.
- Provenance and schema validation happen before semantic comparison, and harness failures remain distinct from physics mismatches.
- Published Cargo-only consumers remain isolated from C++, the submodule, and reviewed reference artifacts.

### Integration Points

- Scope Phase 9 witness material identity at its CMake/xtask/probe boundary and retain the existing provenance command as the fail-closed entrypoint.
- Deepen catalog regression evidence so the current shared resolved definition feeds native repetition, oracle comparison, diagnosis, and reviewed promotion without a second scenario authority.
- Compose existing Cargo and Oracle verification surfaces into one exact-SHA Phase 13 gate instead of duplicating physics or comparator logic.

</code-context>

<specifics>
## Specific Ideas

- The current Phase 9 witness and probe hashes are stable while its broad adapter digest drifted after unrelated later additions; target-scoped materials prevent that false coupling.
- The `rigid-stack-v1` resolved bytes are unchanged while complete checkpoint JSON gained debug primitives, so diagnosis must distinguish capture-schema drift from physics drift before choosing a legacy projection or reviewed new identity.
- The gate should leave a reviewer with the exact commit, material hashes, sealed scenario bytes, native D0 repeats, oracle D1 result, and first divergent path when anything fails.

</specifics>

<deferred>
## Deferred Ideas

- Supported-Windows particle-group transactional and authoritative-storage repair — Phase 14.
- Final candidate-bound platform, performance, release aggregation, and attestation evidence — Phase 15.

</deferred>

***

*Phase: 13-restore-evidence-and-replay-integrity*
*Context gathered: 2026-07-25*
