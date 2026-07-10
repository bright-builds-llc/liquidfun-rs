---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T02:00:42.312Z
---

# Phase 1: Oracle, Provenance, and Repository Foundation - Context

**Gathered:** 2026-07-09
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Freeze the final upstream LiquidFun oracle and establish the licensed, reproducible, Cargo-first repository foundation, architecture evidence, and exhaustive compatibility inventory needed before broad physics implementation. Phase 1 may build and validate the C++ oracle and a native Rust skeleton, but it does not define the semantic differential protocol or implement broad physics behavior.

</domain>

<decisions>
## Implementation Decisions

### Oracle selection, provenance, and licensing

- **D-01:** Resolve the official `v1.1.0` tag to its exact commit and use that release commit as the preferred canonical oracle. Before finalizing the pin, run a bounded delta audit against official post-tag candidate `7f20402173fd143a3988c921bc384459c6a858f2`; select the candidate only if an ADR shows material behavioral, build, API, test, or example benefits.
- **D-02:** Pin the selected commit as a read-only `third_party/liquidfun` Git submodule. Do not follow a moving branch and do not edit the submodule in place.
- **D-03:** Prefer repository-owned wrapper configuration for legacy CMake compatibility. If a source patch is unavoidable, store it outside the submodule with preimage and patch hashes, alteration notes, and an explicit build-only or behavior-affecting classification.
- **D-04:** Make `UPSTREAM.md`, an oracle-selection ADR, a machine-readable provenance lock, and a source/alteration map the canonical records for repository URL, exact revision, release context, verified Box2D ancestry, build identity, applicable notices, derived artifacts, and intentional update procedure.
- **D-05:** Reconcile the existing root MIT `LICENSE` with the still-pending upstream and derivative-work analysis. Preserve applicable LiquidFun/Box2D notices and record altered-source obligations before translating code, tests, scenarios, or reference data.

### Cargo-first repository and oracle build foundation

- **D-06:** Use Rust 1.97.0 for the pinned development toolchain, Rust 2024 Edition with resolver 3, and Rust 1.92.0 as the provisional publishable-crate MSRV. Validate the Phase 1 skeleton at both versions without treating the provisional MSRV as immutable release policy.
- **D-07:** Start with one deep publishable crate, `crates/liquidfun`, plus the minimum private `tools/xtask` crate. Set `default-members = ["crates/liquidfun"]`, keep tooling unpublished, and defer protocol/differential crates to Phase 2.
- **D-08:** Keep C++, CMake, Ninja, the upstream submodule, reference data, and tooling path dependencies out of `liquidfun`, its features, and its `build.rs`. Plain Cargo build, test, documentation, and package workflows must work with the submodule absent.
- **D-09:** Build the contributor-only oracle through repository-owned CMake wrapper files and checked-in presets, executed by private `xtask`; keep root `justfile` recipes as thin, visible aliases. Do not introduce Bazel in Phase 1.
- **D-10:** Validate the recommended CMake 4.3.3, Ninja 1.13.2, and canonical Linux Clang/LLVM 22.1.8 pins during the foundation spike. Allow documented local floors where the checked-in wrapper proves compatibility, while canonical reference artifacts record exact compiler, target, preset, and flags.
- **D-11:** Separate fast Cargo-only CI from opt-in oracle lanes. Prove consumer isolation with `cargo package --list` plus an unpacked packaged-crate build/test performed without the submodule or reference assets.

### Compatibility inventory and evidence governance

- **D-12:** Use a manifest-led hybrid: a curated machine-readable compatibility inventory is authoritative, an upstream discovery snapshot detects omissions, and `COMPATIBILITY.md` is deterministically generated for humans.
- **D-13:** Give every inventory row a stable ID and explicit upstream kind, path or symbol, applicability or reviewed-exclusion rationale, Rust target, provenance/license reference, and independent evidence fields for investigated, planned, implemented, unit tested, differentially validated, platform validated, documented difference, and intentionally unsupported. Do not collapse evidence into one linear status.
- **D-14:** Add fast CI checks for inventory coverage, submodule/provenance agreement, deterministic generated-file integrity, and packaged-crate isolation. Expensive artifact regeneration belongs in oracle or scheduled lanes; fast checks still validate hashes and manifests.
- **D-15:** Reference artifacts must carry content hash, generator revision, oracle revision, build preset/compiler/target/flags, and required notices. Tests must never silently regenerate checked-in evidence.

### Agent's Discretion

- Exact machine-readable formats for the provenance lock, compatibility inventory, discovery snapshot, and artifact manifests.
- Exact module and command layout inside `xtask`, provided orchestration stays thin and errors remain actionable.
- Exact CMake preset names and validated local version floors.
- Exact documentation generator presentation, provided the machine-readable inventory remains authoritative and generated output is checked for drift.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and acceptance

- `.planning/ROADMAP.md` § Phase 1 — fixed goal, requirements, success criteria, and research/ADR flags.
- `.planning/PROJECT.md` — native Rust, Cargo-only consumer, provenance, licensing, safety, determinism, and transparency constraints.
- `.planning/REQUIREMENTS.md` — FND-01 through FND-05, FND-07, FND-08, COMP-01, COMP-02, TEST-09, and DOCS-03.

### Reconciled research

- `.planning/research/SUMMARY.md` — prescriptive stack, Phase 1 research flags, critical pitfalls, and dependency-driven ordering.
- `.planning/research/STACK.md` — toolchain/workspace strategy, CMake/Ninja wrapper, compiler identity, CI, packaging, and dependency/license policy.
- `.planning/research/ARCHITECTURE.md` — oracle isolation, component responsibilities, reference-data flow, dependency direction, and anti-patterns.
- `.planning/research/FEATURES.md` — candidate-tree feature inventory and truthfulness requirements.
- `.planning/research/PITFALLS.md` — wrong-upstream, provenance-loss, workspace-fragmentation, C++-leakage, and false-parity prevention.

### Repository standards and current license state

- `AGENTS.md` — repo-local project constraints, Rust rules, GSD workflow requirements, and verification contract.
- `AGENTS.bright-builds.md` — managed Bright Builds workflow and cross-cutting defaults.
- `standards-overrides.md` — local exception registry; no substantive active override currently replaces the defaults.
- `standards/core/architecture.md` — cohesive functional-core/imperative-shell and domain-boundary guidance.
- `standards/core/verification.md` — sync-first and pre-commit verification requirements.
- `standards/core/testing.md` — focused behavior tests and Arrange/Act/Assert expectations.
- `standards/languages/rust.md` — Rust module, guard, optional naming, invariant, and verification guidance.
- `LICENSE` — current MIT project license text that Phase 1 must reconcile with upstream notices and derivative-work policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `.planning/research/`: current stack, feature, architecture, and pitfall research already converges on a Cargo/CMake process-isolated design and supplies candidate pins for validation.
- Root Bright Builds files and `standards/`: ready-made repository policy and verification guidance for the new workspace.
- Existing `README.md`, `CONTRIBUTING.md`, and CI-managed repository metadata: documentation surfaces to update once commands and status are real.

### Established Patterns

- The repository is greenfield and currently has no Cargo manifest or Rust implementation, so Phase 1 can establish clean boundaries without compatibility migration.
- Planning artifacts treat compatibility status as multidimensional evidence and explicitly prohibit unsupported parity claims.
- Managed Bright Builds standards favor cohesive deep modules, thin orchestration, deterministic verification, and tested pure decision logic.

### Integration Points

- Root workspace/toolchain/lint configuration will anchor ordinary Cargo use.
- `tools/xtask` and `tools/reference` will own contributor-only submodule validation and CMake/Ninja orchestration.
- CI and `justfile` will expose separate Cargo-only and oracle workflows.
- `UPSTREAM.md`, the oracle ADR/provenance records, and the compatibility inventory will become mandatory inputs to every later subsystem plan.

</code_context>

<specifics>
## Specific Ideas

- Treat the official release tag as the default and the already-researched `7f204…` tree as a candidate that must earn selection through an explicit delta audit.
- Prove consumer independence by testing the packaged crate from an unpacked archive with upstream/reference inputs unavailable, not merely by observing that a workspace build succeeds.
- Let curated maintainer judgments and conservative mechanical discovery reinforce each other; do not claim that a lightweight C++ scanner alone proves semantic completeness.

</specifics>

<deferred>
## Deferred Ideas

- Semantic JSON Lines protocol, comparator, and differential-runner crates — Phase 2.
- In-process C ABI or FFI — only after later profiling proves process IPC is a material bottleneck.
- Bazel/Bzlmod — only after a measured orchestration problem justifies a separate ADR.
- Distributed per-subsystem evidence ledgers — defer until inventory size or parallel contributor pressure demonstrates a need.
- Broad physics implementation, object-model decisions, and numerical/tolerance policy — later roadmap phases.

</deferred>

***

*Phase: 01-oracle-provenance-and-repository-foundation*
*Context gathered: 2026-07-09*
