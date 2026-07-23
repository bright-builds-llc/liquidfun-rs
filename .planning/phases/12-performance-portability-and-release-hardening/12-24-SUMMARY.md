---
phase: 12-performance-portability-and-release-hardening
plan: "24"
subsystem: testing
tags: [miri, sanitizers, coverage, github-actions, evidence]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: typed non-authoritative safety and coverage evidence contract
provides:
  - bounded pure-Rust Miri and Rust address-sanitizer producers
  - separate Rust, C++, and semantic differential coverage producers
  - scheduled and manual candidate-scoped safety and coverage workflows
affects: [release-evidence-aggregation, safety-regression-validation, coverage-reporting]
tech-stack:
  added: []
  patterns:
    - exact candidate and tool identities before evidence production
    - validation-before-identity with identity written last
    - separate non-authoritative artifacts for each safety and coverage kind
key-files:
  created:
    - scripts/phase12-miri.sh
    - scripts/phase12-rust-sanitizers.sh
    - scripts/phase12-coverage.sh
    - .github/workflows/safety.yml
    - .github/workflows/coverage.yml
  modified: []
key-decisions:
  - "Keep Miri and Rust ASan on explicit pure-Rust allowlists with per-case time and log-size bounds."
  - "Keep Rust source, C++ source, and differential semantic-leaf coverage as distinct non-authoritative artifacts with no profile merging."
  - "Require exact candidate checkout, exact tool identities, typed contract validation, payload hashing, and identity-last verification before artifact upload."
patterns-established:
  - "Safety evidence scripts expose lightweight check modes and bounded candidate-scoped producer modes."
  - "Scheduled/manual evidence workflows upload only after candidate, kind, toolchain, parity authority, and payload digest assertions pass."
requirements-completed: [API-12, TEST-06, TEST-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T22:53:31Z
duration: 19min
completed: 2026-07-23
---

# Phase 12 Plan 24: Compiler and Runtime QA Automation Summary

**Candidate-bound Miri, Rust ASan, and separate Rust/C++/differential coverage producers now emit bounded, non-authoritative evidence whose payload is validated and hashed before identity-last publication.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-23T22:34:18Z
- **Completed:** 2026-07-23T22:53:31Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added safe Bash producers for seven pure-Rust Miri subsets and six Linux Rust ASan subsets, with exact nightly selection, finite per-case timeouts, bounded logs, and candidate-scoped output.
- Added independent Rust LCOV, C++ LCOV, and differential semantic-leaf modes with exact tool identities, finite command timeouts, bounded artifacts, and no cross-language profile merging.
- Required the exact typed coverage validator before every producer writes its final identity record.
- Added scheduled and manual GitHub Actions jobs that check out an exact candidate, pin every action by full commit SHA, use finite job timeouts, and keep Miri isolated from C++ setup.
- Verified every artifact's candidate, kind, toolchain, non-authoritative status, and payload digest before a distinct candidate-scoped upload.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add bounded compiler/runtime evidence scripts** - `95ee283` (feat)
2. **Task 2: Schedule candidate-scoped safety and coverage workflows** - `952ce76` (ci)

## Files Created/Modified

- `scripts/phase12-miri.sh` - Pure-Rust Miri allowlists, bounded case logs, typed validation, summary hashing, and identity-last output.
- `scripts/phase12-rust-sanitizers.sh` - Linux Rust ASan allowlists with exact nightly/target identity and bounded evidence.
- `scripts/phase12-coverage.sh` - Separate Rust, C++, and differential coverage production with exact tools and non-authoritative identities.
- `.github/workflows/safety.yml` - Scheduled/manual Miri and Rust sanitizer evidence jobs without C++ startup.
- `.github/workflows/coverage.yml` - Scheduled/manual Rust, C++, and semantic differential coverage jobs with separate uploads.

## Decisions Made

- Used explicit test allowlists for Miri and Rust ASan so expanding the repository test graph cannot silently expand expensive or unsupported safety lanes.
- Kept coverage production in three modes and directories because Rust LLVM profiles, C++ LLVM profiles, and semantic differential leaves have different meaning and must not be blended.
- Made `identity.json` the last producer write and rechecked its payload digest in CI before upload so incomplete or modified payloads cannot be mistaken for accepted evidence.
- Kept all outputs non-authoritative for physics parity; they provide safety and exercise evidence only.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- A repeated local `cargo xtask` launch through the shared default target stalled in macOS process startup. The already-required isolated target directory avoided workspace interference for the exact Rust gates; the typed validator and all script check modes completed successfully.

## Verification

- `bash -n` and `shellcheck` passed for all three scripts.
- All three script `check` modes passed, including `cargo xtask safety-evidence validate-coverage`.
- Static assertions confirmed the validator precedes identity writes, outputs remain separate and bounded, and the Miri producer does not start C++ tooling.
- `actionlint` passed for both workflows.
- Static workflow assertions confirmed ten full-SHA action uses, five finite job timeouts, exact candidate/tool identities, five distinct artifact names, and no C++ startup in the Miri job.
- Before each task commit, the exact ordered gate passed with `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and `CARGO_BUILD_JOBS=1`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Long-running Miri, sanitizer, and coverage producer modes were not executed locally; their bounded scheduled/manual workflows are the intended execution environment.

## User Setup Required

None - no credentials or repository secrets are required.

## Next Phase Readiness

- Release evidence aggregation can consume candidate-scoped safety and coverage artifacts with explicit identity and payload digests.
- The scheduled lanes can accumulate non-authoritative safety and exercise evidence without changing ordinary Cargo-only development.

## Known Stubs

None.

## Self-Check: PASSED

- All five implementation files and this summary exist.
- Task commits `95ee283` and `952ce76` exist.
- Both workflows still pass `actionlint`.

***

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
