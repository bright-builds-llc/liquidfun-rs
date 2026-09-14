# Phase 14: Repair Windows Particle-Group Invariants - Research

**Researched:** 2026-09-13
**Domain:** Native Rust particle-group transactions and Windows regression diagnosis
**Confidence:** HIGH for repository/evidence inventory; LOW for unproven root cause

<user-constraints>
## User Constraints (from CONTEXT.md)

The following decisions, discretion, and deferred scope are copied verbatim from the phase context. [VERIFIED: 14-CONTEXT.md]

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


## Deferred Ideas

- Final exact-head multi-platform release-candidate evidence, aggregation, and frozen-source attestation — Phase 15.

</user-constraints>

## Project Constraints (from AGENTS.md)

- Preserve native Rust production behavior, Cargo-only consumer isolation, one authoritative storage owner, stable public identity, pinned upstream provenance, deterministic source ordering, and truthful compatibility claims. No new dependency, public error, unsafe code, topology approximation, or parallel storage authority is justified by this research. [VERIFIED: AGENTS.md; 14-CONTEXT.md]
- Standing authorization dated 2026-09-13 permits ordinary non-force main publication, relevant workflow inspection/dispatch/rerun, diagnosis, fixes, and fresh attempts without another approval token. Recheck repository/ref/candidate/workflow identity before effects. Preserve failed evidence and use separate attempt directories; never silently duplicate uncertain dispatches. Acceptance criteria remain mandatory. [VERIFIED: AGENTS.md; standards-overrides.md]
- Replace old Plan 14-03's human-action/no-push-before-platform-evidence deadlock with local gates → commit → ordinary authorized push → existing matrix evidence → diagnose/retry. This changes workflow authorization, not the requirement for real passing platform evidence before phase closure. [VERIFIED: AGENTS.md; existing 14-03-PLAN.md]
- Before each Rust commit run in order: cargo fmt --all; cargo clippy --all-targets --all-features -- -D warnings; cargo build --all-targets --all-features; cargo test --all-features. Run bun scripts/bright-builds-check.ts all. Keep files within the established 628-line managed boundary and functions cohesive; use foo.rs plus foo/, early returns, maybe_ internal Option names, no unwrap(), and Arrange/Act/Assert tests. [VERIFIED: supplied global instructions; AGENTS.bright-builds.md; standards/core/code-shape.md; standards/core/testing.md; standards/languages/rust.md]
- Read local guidance and standards before work; sync before implementation; preserve append-only task/lesson blocks. Use mdformat 1.0.0 with Python 3.13 and just markdown-check for non-GSD Markdown; never mdformat .planning/**. No standalone body frontmatter delimiters. [VERIFIED: AGENTS.md; standards/core/verification.md; supplied global instructions]
- Functional core/imperative shell and invariant-bearing types inform the narrow candidate repair. Root orchestrator loaded active lessons within budget, with no audit trigger. [VERIFIED: standards/core/architecture.md; root task delegation]

<phase-requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| PART-03 | Dense particle indices may change during sorting, rotation, and compaction while stable public particle IDs continue to resolve correctly until destruction. | Existing identity maps and structural storage equality; add next-allocation proof. |
| PART-04 | Every particle storage permutation updates required and optional SoA lanes, ID maps, proxies, contacts, pairs, triads, lifetimes, and group ranges atomically. | Owned mutation/permutation candidates and complete private rollback comparisons. |
| PART-09 | Consumers can create particle groups from shapes, strokes, explicit positions, or existing groups and can inspect their ranges, flags, transforms, velocities, mass, and inertia. | Existing public recipe/model and semantic snapshots. |
| PART-10 | Group creation, destruction, joining, splitting, connectivity, can-be-empty behavior, solid depth updates, rigid motion, and contiguous membership preserve upstream semantics. | Localize candidate failure before touching topology or ordering. |
| TEST-02 | Public world, rigid-body, joint, particle, callback, query, and destruction workflows have integration tests through supported APIs. | Existing three focused integration targets and supported OS matrix. |
| TEST-04 | Property tests cover geometry invariants, broad-phase behavior, handle validity, particle permutation/group invariants, query correctness, and reproducible world operation sequences. | Preserve explicit audited replay plus random property and vocabulary tests. |

Descriptions are verbatim requirement text; support refers to inspected implementation seams below. [VERIFIED: .planning/REQUIREMENTS.md; inspected particle sources]
</phase-requirements>

## Summary

This is a continuation after partial diagnostic work and a broad structural refactor. Commit 3d93f13 already added the audited seed, exact minimized controls, explicit 14-operation expansion, and a temporary platform-specific panic expectation. There is no corresponding completed Phase 14 summary. Do not spend another plan rediscovering or overwriting that work. The latest Windows job still fails all three property-target tests. [VERIFIED: git log for particle_group_properties.rs; existing Phase 14 directory; CI run 34777296141/job 103777632638]

The first invalid transition and exact storage error have NOT been established by this research. The existing diagnostic assumes the final append is the failing operation, but only that append is inside catch_unwind; earlier setup and the first 13 operations remain outside it. Latest CI proves the named diagnostic itself fails with the production panic, so the executor must instrument the complete trace and cannot lock the final append as root cause. [VERIFIED: particle_group_properties.rs; latest Windows job log]

**Primary recommendation:** Plan diagnosis on actual Windows first, then the smallest demonstrated candidate repair and complete rollback proof, then supported-platform D2 closure using the existing CI matrix. [VERIFIED: 14-CONTEXT.md D-01–D-10; .github/workflows/ci.yml]

## Standard Stack

| Component | Existing version | Purpose |
| --- | --- | --- |
| Rust/Cargo | 1.97.0; aarch64-apple-darwin locally | Pinned native implementation and tests. |
| proptest | 1.11.0 | Existing bounded public and storage state machines. |
| bitflags | 2.13.0 in production dependency baseline | Existing particle/group flag types. |
| thiserror | 2.0.18 in production dependency baseline | Existing typed errors. |
| Built-in test harness / std | Toolchain supplied | Named regressions, owned candidates, structural comparisons. |
| GitHub Actions Cargo CI | Checked-in ci.yml | ubuntu-24.04, macos-15, windows-2025 default-feature matrix. |

These are installed/locked versions, not claims about newest releases. Retain them; no package installation or version upgrade is proposed, so registry publication dates are not needed for this repair. [VERIFIED: rust-toolchain.toml; rustc --version; cargo --version; Cargo.lock; .github/workflows/ci.yml]

## Architecture Patterns

### Current paths after Phase 13.1

| Responsibility | Actual file |
| --- | --- |
| Public create/plan/commit group operations | crates/liquidfun/src/world/particle_object/group.rs |
| append_group_particle, contact preparation, shared error mapping | crates/liquidfun/src/world/particle_object.rs |
| Authoritative storage fields and PartialEq | crates/liquidfun/src/particle/storage.rs |
| Single-particle creation preparation/publication | crates/liquidfun/src/particle/storage/creation.rs |
| check_invariants, lane lengths, identity map, derived references | crates/liquidfun/src/particle/storage/lifecycle.rs |
| Group membership rebuild/validation | crates/liquidfun/src/particle/storage/validation.rs |
| GroupPlan, plan_group, mutation payload | crates/liquidfun/src/particle/storage/mutation.rs |
| Join/split preparation | crates/liquidfun/src/particle/storage/mutation/join.rs and split.rs |
| Atomic permutation | crates/liquidfun/src/particle/storage/permutation.rs and permutation/ |
| Contact replacement | crates/liquidfun/src/particle/storage/runtime.rs |
| Topology finite-value/reference validation | crates/liquidfun/src/particle/storage/lanes.rs |
| Group depth | crates/liquidfun/src/particle/storage/group/depth.rs |
| Existing private lifecycle tests | crates/liquidfun/src/world/particle_object/group_lifecycle_tests.rs |
| Public property replay and snapshots | crates/liquidfun/tests/particle_group_properties.rs and particle_group_properties/ |

All paths and symbols were inspected with repository reads/searches. Old plan references to validation.rs as the full invariant checker and particle_object.rs as the plan implementation are incomplete after refactoring. [VERIFIED: listed source files]

### Preserve the owned candidate

World::plan_particle_group validates owner/append target, samples, reserves diagnostic identity arithmetic, clones the source ParticleSystem, appends samples/lifetimes, refreshes contacts, then calls storage.plan_group. That method checks invariants, clones storage, prepares topology, optionally joins, computes depth, and validates. World::commit_particle_group publishes shell, system candidate, and diagnostic counter. [VERIFIED: world/particle_object/group.rs; particle/storage/mutation.rs]

ParticleStorage already derives Clone and PartialEq over its full field inventory. Prefer structural comparison to unsafe raw-memory snapshots or production Debug exposure. Additional private snapshot fields are necessary for lifetime state, group arena/free generations, next diagnostic allocation, and lifecycle journals outside ParticleStorage. Tests needing storage-private invariant access belong under storage or a narrow cfg(test) helper; world tests cannot simply call a pub(super) storage method. [VERIFIED: particle/storage.rs; particle/storage/lifecycle.rs; world/particle_object.rs]

The private-test ownership decision is resolved: complete world transaction tests belong in world/object/tests/particle_group_transactions.rs, registered by world/object/tests.rs, whose parent owns next_diagnostic_id. The temporary diagnostic replay belongs beside them in particle_group_diagnostics.rs. A cfg(test), pub(crate) helper in particle/storage.rs can invoke its child lifecycle module's invariant checker and expose bounded observations to lib tests. Test-only hooks in world/particle_object.rs, world/particle_object/group.rs and storage/mutation.rs observe the actual call path. An integration-test executable links the ordinary library without these cfg(test) hooks; it must run separately and its exact inputs must be mapped to the lib replay. No public debug API or production visibility widening is required. [VERIFIED: world/object.rs; world/object/tests.rs; particle/storage/lifecycle.rs; PLANNED: 14-01 Task 2 and 14-02 Task 2]

### Hypotheses to test, not locked fixes

1. Instrument full Model initialization and each operation; find the first public call and raw error before shared mapping. The final-append assumption is unsupported. [VERIFIED: named diagnostic body; current failing job]
1. Distinguish append/lifetime preparation, contact refresh, initial plan_group invariant validation, generated topology validation, join remap, solid depth, and final validation. The same mapper collapses four private categories into one panic. [VERIFIED: world/particle_object.rs; mutation.rs]
1. Only after stage localization test whether the invalid state is lane/identity maintenance, stale derived indices, or non-finite topology data. Those are possible paths in inspected validators, not a demonstrated cause of this seed. [VERIFIED: lifecycle.rs; lanes.rs; mutation.rs]
1. Floating arithmetic/platform divergence causing non-finite generated values remains a hypothesis requiring Windows raw values and input identity; do not prescribe clamping or tolerances from it. [ASSUMED]

## Don't Hand-Roll

| Problem | Use existing mechanism |
| --- | --- |
| Atomic mutation | GroupPlan and MutationCandidate/PreparedPermutation. |
| Stable identity | Existing storage identities/free/retired slots and world arenas. |
| Deterministic operation replay | VersionedGenerator plus explicit persisted operations. |
| Public rollback | RollbackSnapshot and SemanticSnapshot, extended only for missing observables. |
| Platform execution | Existing Cargo CI default-feature matrix. |

These mechanisms are already implemented and align with locked decisions. Do not introduce a second storage owner, alternate topology reconstruction, custom serialization as the compatibility oracle, or a Phase 15 evidence producer. [VERIFIED: inspected source files; 14-CONTEXT.md]

## Evidence and Current Platform Status

| Evidence | Exact identity | Observed result |
| --- | --- | --- |
| Original audit Cargo CI | run 30070539790; source 4b6b15ba562cedcad3f37e379d01be9e9cb2e949; Windows job 89410277623 | Vocabulary/property tests panic at old particle_object.rs:1171. |
| Current inspected Cargo CI | run 34777296141; source 5aadb6c105b98ae09443f74e44c57f8ce7eae19d; Windows job 103777632638 | All three property target tests fail with same message at particle_object.rs:344. |
| Same current run, Linux defaults | job 103777632709 | Success. |
| Same current run, macOS defaults | job 103777632737 | Success. |
| Same current run, Linux quality | job 103777632760 | Failure; detailed cause not inspected in this scoped research. |
| Phase 13.1 terminal source | d3e70780a984ba25096f288d7c603ffa8153a748 | Formal verification 36/36, complete 72-command structural matrix; not Phase 14 Windows closure. |

Run identities/conclusions were read through gh run view; original/latest Windows logs were inspected. Phase 13.1 findings come from its September 13 formal verification, not older Plan 25 readiness prose. [VERIFIED: GitHub run/job records above; 13.1-VERIFICATION.md]

Original minimized input is seed 4149329052036581951, controls [0,0,0,0,0,0,0,0,0,0,0,0,0,59]. Original framework persistence hash is ca6e3bac494f125289898c2abc649305ee64e5d673f60339b47915ff6342257b. The checked-in constants match the controls. Latest random failure additionally reports seed 190752942043209832 with controls [0,0,0,0,0,0,0,0,0,0,0,0,0,41,225]. Retain this as an additional diagnostic input; it does not replace the audited seed. [VERIFIED: original/current Windows job logs; particle_group_properties.rs]

A local exact-regression command was started during research, emitted compilation output, and was deliberately terminated (exit 143) without test results to avoid overlapping the root agent's ordered verification. No fresh local reproduction or passing result is claimed here; no Windows execution was dispatched during research. [VERIFIED: research exec session 92260]

## Common Pitfalls

- **False-red localization:** catching only the final operation misses earlier panics. Label before and after every operation and initialization during diagnosis; emit raw category before the unchanged mapper and retain the backtrace. Do not add catches or change the actual property's failure semantics to manufacture passing CI. [VERIFIED: current diagnostic and failing Windows log; PLANNED: 14-01 Task 2]
- **False-green exact filters:** old private command omitted world:: from the complete test path while using --exact. List the selected lib target first, require a nonzero inventory, then use the exact returned name. Existing module path is world::particle_object::group_lifecycle_tests. [VERIFIED: lib/world module wiring; old 14-01-PLAN.md]
- **Normalized identity hides rollback:** SemanticSnapshot uses ordinals for cross-world replay, while RollbackSnapshot adds actual IDs for same-world rejection. Preserve both roles and add private next-allocation/lifetime/arena proofs. [VERIFIED: snapshot.rs; model.rs]
- **Overbroad mapper change:** storage_object_creation_error also serves ordinary particle creation and lifecycle errors. Candidate-specific mapping must not silently convert authoritative corruption elsewhere into a public rejection. [VERIFIED: world/particle_object.rs; particle_object/particle.rs]
- **Weak success criterion:** simply returning any error for all group creation could make panic tests green. Keep vocabulary behavior, successful creates/appends, and topology/mutation tests; assert the diagnosed valid operation applies where warranted. [VERIFIED: existing model Outcome and focused integration contracts]
- **CI wiring mistaken for evidence:** cargo test -p liquidfun covers these targets, but earlier failing targets prevent later targets completing. Require actual named and complete-suite outcomes and matching SHA; static workflow inspection is insufficient. [VERIFIED: ci.yml; latest job's failing target]
- **Old authorization rules reintroduced:** do not copy Plan 14-03's temporary-ref human checkpoint or Plan 13.1's historical one-attempt prohibitions. [VERIFIED: AGENTS.md standing authorization; standards-overrides.md]

## Code Examples

Existing candidate publication pattern; retain the boundary rather than replacing it. [VERIFIED: world/particle_object/group.rs]

```rust
let plan = self.plan_particle_group(system, recipe)?;
Ok(self.commit_particle_group(plan))
```

Use explicit Cargo targets and inspect their test counts. These are proposed execution commands based on existing target files; this research did not complete their execution. [VERIFIED: crates/liquidfun/tests target files; CI commands]

```bash
cargo test -p liquidfun --test particle_group_properties -- --list
cargo test -p liquidfun --test particle_group_properties persisted_audited_windows_seed -- --exact --nocapture
cargo test -p liquidfun --test particle_group_properties --test particle_groups --test particle_group_mutation
cargo test -p liquidfun --lib -- --list
cargo test -p liquidfun --lib world::particle_object::group_lifecycle_tests
cargo test -p liquidfun
cargo test -p liquidfun --all-features
```

Use the complete package as the unambiguous relevant-particle superset; this also includes adjacent regression surfaces. During diagnosis run the cfg(test) lib replay and unchanged public regression as separate steps before the existing full default-feature suite. Temporary `if: ${{ !cancelled() }}` conditions on the public and full-suite steps keep them reachable after a diagnostic failure without hiding any failed exit. Record source/input lineage between the two executables. Locally gated diagnostic publication, exact-SHA remote collection and separately gated probe/CI removal are owned by 14-01 Task 2. Final acceptance must still run the full package. [VERIFIED: ci.yml; existing target structure; 14-CONTEXT.md D-08/D-09; PLANNED: 14-01 Task 2]

## State of the Art

| Older artifact | Current implementation/evidence | Planning impact |
| --- | --- | --- |
| Input recovery is unfinished | Seed/controls/expanded operations committed in 3d93f13 | Verify/preserve, do not recreate. |
| Monolithic particle_object.rs and storage internals | Stable facades plus focused child files | Use actual paths and privacy boundaries. |
| Phase 13.1 pending fourth-candidate approval | September 13 passed report and standing authorization | Do not reopen completed structural recovery. |
| Human platform-evidence checkpoint before any push | Ordinary authorized publication and workflow iteration | Keep evidence gate without approval deadlock. |

[VERIFIED: git log; source tree; 13.1-VERIFICATION.md; AGENTS.md; old plans]

## Environment Availability

| Dependency | Available | Version/boundary | Fallback |
| --- | --- | --- | --- |
| Rust/Cargo | Yes | 1.97.0, local macOS ARM64 | Existing supported CI runners for other OS boundaries. |
| gh read access | Yes | 2.87.3; original/current logs readable | No fallback needed for historical evidence. |
| Bun managed checker | Yes | 1.3.14 | Repo documented checker command. |
| Native Windows execution locally | Not established | Host is aarch64-apple-darwin | Existing windows-2025 matrix job. |
| Native Linux execution locally | Not established | Local host is macOS | Existing ubuntu-24.04 matrix job. |

[VERIFIED: tool version probes; rustc host toolchain path; gh calls; ci.yml]

Windows/Linux runner execution is required during implementation. Existing remote workflow access and standing authorization provide the route; no new service provision or credential request was identified. No C++ build is needed for the focused native regression. [VERIFIED: ci.yml; AGENTS.md; native test dependency boundary]

## Security Domain

Security enforcement is not explicitly disabled; Nyquist validation is explicitly false, so a separate Validation Architecture section is omitted. [VERIFIED: .planning/config.json]

This is a headless native library mutation boundary. Web authentication, session, access-control, and cryptographic controls are not exercised by these changes; do not add web-security packages. Relevant controls are bounded input, typed validation, atomic mutation, stable ownership, and evidence identity. The inherited ASVS V2–V6 labels in the GSD template should not be presented as a claim of compliance with a particular ASVS release. [VERIFIED: phase scope; source APIs; GSD researcher template]

| Boundary/threat | STRIDE | Standard control for this phase |
| --- | --- | --- |
| Safe public input reaches panic | Denial of Service | Bounded sampling/work; eliminate demonstrated reachable panic; explicit valid/invalid regressions. |
| Candidate partially publishes identity/topology | Tampering | Complete structural and semantic before/after comparison; commit only validated candidate. |
| Historical/other-SHA run presented as passing repair | Spoofing / Repudiation | Persist full source SHA, run/job, runner/toolchain, test counts, and outcomes. |
| Platform success promoted to canonical authority | Elevation of Privilege | Keep all phase evidence D2; preserve Phase 15 and canonical fixture boundaries. |

Threat/control mapping is a phase-specific assessment grounded in the locked contracts and inspected boundaries, not an external certification claim. [VERIFIED: 14-CONTEXT.md; source and CI boundaries]

## Assumptions Log

| # | Claim | Section | Risk if wrong |
| --- | --- | --- | --- |
| A1 | Platform numerical divergence could produce non-finite topology values. | Architecture Patterns | A guessed numerical repair could alter valid physics; require observed first failure before choosing it. |

This hypothesis is not a locked implementation decision and requires experimental diagnosis, not user speculation. [ASSUMED]

## Resolved Planning Dispositions

There are no unresolved research choices preventing executable planning. The runtime cause remains UNPROVEN: the first two rows are required execution investigations, not claims that the physics diagnosis is resolved. Plan 14-02 depends on completed 14-01 and cannot begin a repair until both findings are observed and recorded. [PLANNED: 14-01 Task 2; 14-02 depends_on; 14-CONTEXT.md D-01]

| Question | Resolved disposition and owner | Required evidence / gate |
| --- | --- | --- |
| First failing initialization/operation/stage and raw mapped category | Execution investigation in 14-01 Task 2. Replay full setup and all original operations in the private lib harness; observe real stages and the raw error before unchanged mapping on supported Windows. | 14-DIAGNOSIS.md must bind a first failing operation, invariant predicate, raw category, state values and exact input/source/run lineage. Existing panic logs alone do not satisfy this gate; 14-02 remains blocked until observed. |
| Invalid live source versus invalid owned candidate | Execution investigation in 14-01 Task 2. A storage-owned cfg(test) helper checks live storage before cloning/public operations and captures candidate progression through the actual planner. | A before/after stage comparison proves the ownership and origin of invalidity. Do not infer candidate-only corruption from the panic text or predetermine the mapping change. Required before 14-02. |
| Private snapshot ownership and state outside ParticleStorage equality | Resolved implementation placement in 14-02 Task 2: world/object/tests/particle_group_transactions.rs plus its registration in world/object/tests.rs; narrow cfg(test) helpers in the owning module for otherwise inaccessible state, with exact ownership amended before any extra file edit. | Compare complete ParticleStorage and ParticleLifetimeState; separately cover all remaining ParticleSystem fields, ordered group shells/diagnostics, arena allocation/free/generation state, world next_diagnostic_id, and immediate/deferred lifecycle journals. Prove next-allocation continuity and map every D-06 category to equality or an explicit assertion. No public/debug exposure or raw-memory comparison. |

The temporary instrumentation ownership and publication/removal sequence are fixed in 14-01 Task 2; valid local gates authorize the diagnostic commit/push under standing authorization even while Windows still fails. Preserve the failing Windows command and original property semantics, then remove probes with separately passing local gates. This resolves the execution route without weakening D-01, asserting a cause, or moving Phase 15 evidence work into this phase. [VERIFIED: AGENTS.md standing authorization; PLANNED: 14-01 Task 2]

## Sources

- [Original audited Cargo CI](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/30070539790), [Windows job](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/30070539790/job/89410277623): exact historical failure and controls. [VERIFIED: gh run view --log]
- [Current inspected Cargo CI](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34777296141), [Windows job](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34777296141/job/103777632638): all three failures at current inspected source and new minimized input. [VERIFIED: gh run view --log]
- Local AGENTS.md, AGENTS.bright-builds.md, standards-overrides.md, standards/index.md, architecture/code-shape/verification/testing/Rust pages. [VERIFIED: repository reads]
- Local phase contexts 09/10/12/14, milestone audit, ROADMAP, REQUIREMENTS, PROJECT, Phase 13.1 verification and Plan 25 summary. [VERIFIED: repository reads]
- Current source files in the architecture map, CI workflow, Cargo.lock, and rust-toolchain.toml. [VERIFIED: repository reads]

## Metadata

- Standard stack: HIGH; installed and lockfile-backed, no dependency change. [VERIFIED: probes and Cargo.lock]
- Architecture: HIGH; current code paths and transaction boundary inspected. [VERIFIED: sources above]
- Pitfalls: HIGH for existing broken diagnostic/filter/authorization; LOW for root-cause hypotheses. [VERIFIED: code, logs, old plans]
- Validity: recheck code/CI identity at execution; root cause remains intentionally unproven. Research is ready to inform diagnosis-first plans, not a claimed repair. [VERIFIED: current research scope]
