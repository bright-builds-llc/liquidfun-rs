---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 15-hobby-2026-09-17T00-43-37
generated_at: 2026-09-17T00:43:37.323921+00:00
---

# Phase 15: Hobby-project Wrap-up — Revised Context

**Gathered:** 2026-09-16 (local date)
**Status:** Ready for replanning; previous strict campaign remains deferred
**Mode:** Yolo — recommended defaults auto-selected, not a claim of individual human answers

<domain>
## Phase Boundary

Capture and implement a small wrap-up for the existing experimental Rust physics library: consistent maturity/support documentation, a short experimental-release checklist, a useful native example check, and an accurate completion record. This applies the owner's explicit hobby scope change and the immediately preceding proposal to re-discuss Phase 15. It does not restart the original strict qualification campaign or add physics features.

Existing plans 15-01 through 15-12 belong to the earlier lifecycle. Replan before executing; preserve historical completed work and label unexecuted strict work deferred rather than passed. The roadmap still contains the original optional strict goal; reconcile its active routing during replanning. No plan execution or publication is authorized by this discuss-only command.
</domain>

<decisions>
## Carried-forward owner decisions

- **D-10:** Independent review may be performed by a separate identified AI reviewer under AGENTS.md; do not represent AI review as human approval.
- **D-11:** A fun, useful experimental library is the goal. Linux x64 qualification and dedicated benchmark infrastructure are not ordinary completion requirements.
- **D-12:** Keep local checks plus one macOS Cargo CI job. Cross-platform, C++ comparisons, Miri/sanitizers, fuzz, coverage and benchmarks are optional manual checks. Managed Bright Builds automation remains separate.

## Auto-selected decisions for the revised phase

### Experimental completion and release preparation
- **D-13:** Adopt a short experimental-release checklist: required local Rust checks, relevant known-bug regressions, one representative native/headless example, a passing macOS Cargo CI result for the implementation being closed, accurate limitations/change notes, license/notices, and independent review. Record the checked source and actual results briefly. Do not require a full evidence registry, frozen C/A attestation, Linux, C++ campaigns or controlled benchmarking for this checklist.
- **D-14:** Reuse `cargo xtask package verify` before preparing a publishable package: it validates contents/notices and builds/tests an extracted consumer package outside the repository. No new release framework. Checklist completion is preparation, not a package publication or release-tag instruction; do not choose a release version in this discussion.
- **D-15:** Keep strict qualification separate and truthfully non-ready until its original validators pass. Preserve those validators, historical records and source-bound claims. Describe experimental usability independently; do not make strict audit accept a reduced evidence bundle.

### Compatibility promises
- **D-16:** Treat non-macOS platforms as best effort with optional manual testing. Keep existing Windows fixes and regressions. Describe macOS CI as tested coverage, not a blanket platform warranty. No new platform matrix or long-term platform guarantee.
- **D-17:** Improve feature/behavior parity incrementally, document known differences and supported scenarios, and retain existing regressions. Exhaustive upstream closure is not a hobby completion gate. Do not claim complete parity from local checks or omit known failures.
- **D-18:** Public APIs are experimental and may evolve; document incompatible changes before an eventual release. Preserve safe handles, checked mutation and native Cargo-only consumption. No stable 1.x API commitment is introduced.
- **D-19:** Keep the pinned Rust 1.97.0 development toolchain and existing Cargo `rust-version = "1.92"` metadata. Defer a durable MSRV promise, not the meaning of the declared minimum: verify it before publication, or deliberately revise manifest and docs together with a documented consumer impact. No mandatory recurring MSRV matrix for ordinary hobby work.

### Bounded handoff
- **D-20:** Replan this phase as documentation/checklist reconciliation plus focused verification and a completion record. Explicitly supersede incompatible old plan objectives while retaining their provenance. Reconcile PROJECT-SCOPE.md's formerly pending proposals and stale release/support wording during execution. No new solver work, sweeping tooling deletion or new certification framework.
- **D-21:** Use on-demand local benchmarks for diagnosis; published comparisons identify workload, hardware, compiler and limitations. Controlled-host strict evidence remains optional. Retain local precommit checks required by AGENTS.md; do not advertise `just check` as their equivalent.

### Historical decisions
D-01 through D-09 described the original strict campaign and remain historical in git and the earlier discussion log. D-10 through D-21 govern this revised hobby wrap-up; strict evidence and publication boundaries still apply when their specific operations are explicitly selected.

### Agent discretion
Choose a concise checklist layout, one existing representative native example, documentation organization and a small plan decomposition. Preserve reusable tooling and known-bug tests. No pending GSD todos matched this phase.
</decisions>

<canonical-refs>
## Canonical References

**Downstream agents must read these before planning or implementing.**

- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` — standing authority, independent review, local checks and hobby exceptions.
- `PROJECT-SCOPE.md`, `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md` — current hobby scope and retained strict history; previously pending proposals are resolved by D-13 through D-21 for planning.
- `README.md`, `CONTRIBUTING.md`, `RELEASE.md`, `TESTING.md`, `BENCHMARKING.md`, `COMPATIBILITY.md` — maturity, contributor commands, release and support wording to reconcile.
- `Cargo.toml`, `crates/liquidfun/Cargo.toml`, `rust-toolchain.toml`, `justfile` — current package version, minimum compiler, development pin and command facade.
- `tools/xtask/src/package.rs`, `tools/xtask/src/main.rs` — isolated package verification and the distinct aggregate check command.
- `.github/workflows/ci.yml`, `reference/compatibility.json` — actual hosted baseline and scoped compatibility facts.
- `.planning/phases/14-repair-windows-particle-group-invariants/14-CONTEXT.md` — preserve transactional regression protection.
- `reference/release/required-evidence.toml`, `tools/xtask/src/release/attestation.rs` — optional strict semantics to preserve, not ordinary wrap-up prerequisites.
</canonical-refs>

<code-context>
## Existing Code Insights

- `just fmt`, `just clippy`, `just build` and `just test` expose existing checks; precommit still requires `cargo fmt --all` followed by Clippy, build and tests in order.
- `just package-verify` calls `cargo xtask package verify`; reuse its isolated consumer checks and notices inspection.
- `just check` calls the xtask package/protocol/docs/provenance aggregate and may additionally inspect upstream when the submodule exists. It does not replace the Rust verification sequence.
- Existing native/headless examples and private testbed supply a representative demonstration without requiring cross-platform GUI certification.
- Strict release auditing and documentation projections are coupled. Keep strict records unchanged and label experimental preparation separately rather than weakening the projection validator.
</code-context>

<specifics>
## Specific Ideas

The user wants "more of a fun project, not a super high quality implementation" and selected local checks plus one CI job with optional manual expensive checks. Prefer using the engine and fixing concrete bugs over recurring certification work. This pass captures recommended choices automatically under gsd-yolo-discuss.
</specifics>

<deferred>
## Deferred Ideas

Package publication and release tags; complete-parity certification; durable API/MSRV or platform guarantees; controlled performance claims; new physics features and broad tooling removal. Inspect older debug records during wrap-up only to distinguish actual current defects from stale historical bookkeeping; do not re-run old campaigns merely to close their records.
</deferred>
