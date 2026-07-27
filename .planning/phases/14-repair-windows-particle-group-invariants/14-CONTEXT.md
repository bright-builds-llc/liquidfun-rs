---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-07-27T16:17:54.200Z
---

# Phase 14: Repair Windows Particle-Group Invariants - Context

**Gathered:** 2026-07-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Repair the supported-Windows particle-group creation and mutation failure so checked public operations remain transactional and never reach the audited authoritative-storage panic. The phase preserves stable particle and group identity, one authoritative `ParticleStorage`, source-significant topology, and cross-platform semantics; it does not produce the final release-candidate evidence bundle owned by Phase 15.

</domain>

<decisions>
## Implementation Decisions

### Root cause and transaction boundary

- **D-01:** Reproduce and diagnose the first invalid candidate transition before changing error mapping or storage structure. The repair must address the root cause demonstrated by the audited Windows seed, not suppress the panic through a platform conditional, validation bypass, topology rebuild, identity substitution, or parallel storage path.
- **D-02:** Preserve the existing clone-plan-commit boundary by default: group sampling, particle identity allocation, lifetime/contact preparation, topology generation, invariant checking, shell reservation, and diagnostics advance inside an owned candidate, and authoritative world state changes only after the complete candidate is valid.
- **D-03:** Escalate to one storage-owned prepared group-creation payload only if diagnosis proves incremental candidate construction cannot maintain the locked invariants. Any such payload must remain the sole mutation authority and include every required and optional lane, identity map, group range, topology record, derived structure, lifetime state, cache, shell effect, and diagnostic allocation.

### Failure semantics and rollback proof

- **D-04:** A legitimate validation failure discovered while building an isolated group-creation candidate returns the existing typed no-effect `CreateObjectError::InvalidParticleGroupTopology` boundary. Do not add a public storage-invariant error or leak private storage vocabulary unless implementation evidence proves callers need a distinct stable recovery contract.
- **D-05:** Retain panic or debug-assert semantics only for corruption proven to exist in authoritative live storage or for a state that safe public candidate construction cannot reach. The supported public Windows sequence is already reachable and therefore cannot remain an `unreachable!` path.
- **D-06:** Prove rejection transactionality with an exact private candidate/storage comparison and a public semantic before/after snapshot. The proof covers stable particle/group IDs and next allocations; ordered membership and group metadata; required and optional lanes; pairs, triads, and rest data; proxies, contacts, weights, and solver caches; pending-delete, zombie, expiration, free, and retired identity state; group shells; diagnostic allocation; and zero lifecycle output.

### Regression and platform evidence

- **D-07:** Check in a named public-API regression containing the exact audited seed `4149329052036581951` (`0x3995_60c9_ead9_4a3f`) and its minimized controls or equivalent fully persisted operation sequence. The existing persisted test uses a different seed and does not close this gap.
- **D-08:** Preserve the pre-fix Windows failure as cited audit/CI evidence, then require the named regression, focused public particle-group integration tests, property suite, and complete relevant particle suite to pass on the supported Windows toolchain after the fix.
- **D-09:** Run the same focused and complete relevant suites on ordinary Linux and macOS boundaries to reject a Windows-only workaround or cross-platform semantic drift. Reuse the existing OS matrix unless planning finds a concrete missing boundary; do not duplicate Phase 15's candidate evidence workflow.
- **D-10:** Treat Windows, Linux, and macOS results as D2 portability and regression evidence only. They may close this implementation defect but cannot rewrite canonical fixtures, promote compatibility rows, or self-bless D1/D3 authority.

### the agent's Discretion

- Exact private type and helper names, plan decomposition, and the narrowest root-cause repair after the failing transition is demonstrated.
- Exact placement of the named regression and semantic snapshot helpers, provided the audited seed and minimized operation input remain explicit, readable, and stable.
- Exact affected-test commands and CI artifact/log retention within the existing platform workflows, provided Windows before/after identity and cross-platform results remain reviewable.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gap authority and fixed scope

- `.planning/v1.0-MILESTONE-AUDIT.md` — Exact-head Windows panic, failing property/regression flow, audited seed, requirement gaps, and Phase 14/15 boundary.
- `.planning/ROADMAP.md` — Fixed Phase 14 goal, dependency, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` — `PART-03`, `PART-04`, `PART-09`, `PART-10`, `TEST-02`, and `TEST-04` acceptance contracts.
- `.planning/PROJECT.md` — Native-Rust, safe ownership, deterministic behavior, stable public identity, compatibility, and evidence-truthfulness constraints.

### Locked particle and platform contracts

- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md` — Single authoritative storage, validate-then-commit permutations, optional-lane completeness, identity lifecycle, and transactional failure rules.
- `.planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-CONTEXT.md` — Group-creation, append, join, split, topology, stable identity, source-order mutation, and public group-view contracts.
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md` — Supported-platform verification, D0-D3 evidence authority, and separation between portability evidence and compatibility promotion.
- `.codex/tasks/lessons.md` — Explicit CI identity, exact canonical-boundary validation, and prohibition on self-blessed compatibility evidence.

### Existing implementation and regression seams

- `crates/liquidfun/src/world/particle_object.rs` — Clone-plan-commit group creation, shell/diagnostic preflight, authoritative system publication, and the reachable `storage_object_creation_error` panic mapping.
- `crates/liquidfun/src/particle/storage.rs` and `crates/liquidfun/src/particle/storage/` — Authoritative lanes, stable identity state, mutation candidates, group records, topology, derived structures, and invariant checks.
- `crates/liquidfun/src/world/particle_object/group_lifecycle.rs` — Transactional group destruction and compaction patterns that the repair must not weaken.
- `crates/liquidfun/tests/particle_group_properties.rs` and `crates/liquidfun/tests/particle_group_properties/` — Versioned public operation generator, semantic snapshots, rejection rollback assertions, and the currently different persisted regression seed.
- `crates/liquidfun/tests/particle_groups.rs` and `crates/liquidfun/tests/particle_group_mutation.rs` — Focused public creation, append, failure, join, split, and topology transaction coverage.
- `.github/workflows/ci.yml` — Existing Windows, Linux, and macOS Cargo verification boundaries.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `World::plan_particle_group` and `World::commit_particle_group`: already isolate group construction in a cloned `ParticleSystem` and publish only after storage/topology validation and shell preflight.
- `ParticleStorage::plan_group` and mutation candidates: existing storage-owned pure preparation and explicit commit seams for create, join, split, compaction, regeneration, flag changes, and rotations.
- `SemanticSnapshot` plus the particle-group property model: existing public rollback and invariant harness covering versioned operation sequences.
- `stable_snapshot` group integration helpers: focused public no-effect checks for wrong-system, stale-target, capacity, and topology failures.

### Established Patterns

- Public operations validate and prepare complete owned candidates before a short commit.
- Stable world/system/group/particle IDs remain public authority while dense rows are private and ephemeral.
- Typed no-effect errors represent legitimate public rejection; impossible-state panics are reserved for internally unreachable corruption.
- Exact input identity and evidence tier are explicit; supported-platform success does not authorize canonical fixture promotion.

### Integration Points

- The failure crosses `plan_particle_group` → particle/lifetime/contact preparation → `ParticleStorage::plan_group` → `storage_object_creation_error`.
- The regression belongs in the public particle-group property/integration surface and must run through the existing multi-platform Cargo CI matrix.
- Phase 15 consumes a repaired exact head later; Phase 14 must leave release-candidate aggregation and final attestation untouched.

</code-context>

<specifics>
## Specific Ideas

- Prefer a minimal root-cause repair inside the existing candidate boundary when evidence permits; widen the storage transaction only when the diagnosed invalid transition requires it.
- Persist the exact audited seed and minimized controls together so a generator or framework change cannot silently reinterpret the regression.
- Keep the no-effect assertion semantic and comprehensive enough to catch identity, topology, optional-lane, derived-state, or lifecycle drift even when the public call returns an error rather than panicking.

</specifics>

<deferred>
## Deferred Ideas

- Final exact-head multi-platform release-candidate evidence, aggregation, and frozen-source attestation — Phase 15.

</deferred>

***

*Phase: 14-repair-windows-particle-group-invariants*
*Context gathered: 2026-07-27*
