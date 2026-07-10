---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "07"
subsystem: cpp-oracle-dependency
tags: [cpp, nlohmann-json, vendoring, supply-chain, licensing]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Deterministic Phase-2 protocol presentations and private Cargo/C++ isolation from Plans 02-01 through 02-04
provides:
  - Official nlohmann/json 3.12.0 single-header release artifact
  - Verbatim tag-pinned MIT license and repository-local SHA-256 verification
  - Third-party notice recording private oracle-only scope and immutable source URLs
affects: [02-08, cpp-oracle-adapter, dependency-provenance, package-isolation]

tech-stack:
  added: [nlohmann/json 3.12.0]
  patterns: [immutable release-asset vendoring, repository-local checksum verification, private oracle-only dependency scope]

key-files:
  created:
    - tools/reference/vendor/nlohmann/json.hpp
    - tools/reference/vendor/nlohmann/LICENSE.MIT
    - tools/reference/vendor/nlohmann/SHA256SUMS
  modified:
    - THIRD_PARTY_NOTICES.md

key-decisions:
  - "Use the official v3.12.0 json.hpp release asset and the same immutable tag's LICENSE.MIT rather than a package-manager or build-time fetch."
  - "Keep nlohmann/json entirely below tools/reference so no Rust manifest, published package, or ordinary Cargo path gains a dependency edge."

patterns-established:
  - "Vendor provenance: checksum manifests record immutable release/tag source URLs and verify only repository-owned bytes during builds and reviews."
  - "Notice scope: private C++ oracle dependencies are explicitly separated from published Rust consumer dependencies."

requirements-completed:
  - COMP-04
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T08:37:05Z

duration: 6 min
completed: 2026-07-10
---

# Phase 2 Plan 07: Reviewed nlohmann/json Vendoring and Notices Summary

**The private C++ oracle now has the exact official nlohmann/json 3.12.0 single header, verbatim MIT terms, immutable source provenance, and locally verifiable content hashes without any Cargo dependency or runtime fetch.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-07-10T08:31:05Z
- **Completed:** 2026-07-10T08:37:05Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Vendored the official nlohmann/json v3.12.0 `json.hpp` release asset and matched its release-published SHA-256 exactly.
- Preserved the v3.12.0 tag's MIT license verbatim and recorded repository-local hashes for both reviewed files.
- Recorded immutable release and tag-pinned source URLs in a checksum manifest accepted by `shasum --check`.
- Documented that the dependency is private C++ oracle tooling only, with no build-time download or published/default Cargo dependency edge.

## Task Commits

Each task was committed atomically:

1. **Task 1: Vendor the exact reviewed C++ JSON dependency and notices** - `97c1e39` (`chore`)

## Files Created/Modified

- `tools/reference/vendor/nlohmann/json.hpp` - Official nlohmann/json 3.12.0 single-header release artifact.
- `tools/reference/vendor/nlohmann/LICENSE.MIT` - Verbatim MIT license from the immutable v3.12.0 tag.
- `tools/reference/vendor/nlohmann/SHA256SUMS` - Version, official source URLs, and repository-local SHA-256 values for both vendored files.
- `THIRD_PARTY_NOTICES.md` - Private oracle-only dependency classification, license location, and checksum provenance.

## Decisions Made

- Used the official GitHub v3.12.0 release asset for `json.hpp`; its local SHA-256 is the same `aaf127c...c5de63` value published by the release.
- Used the same immutable v3.12.0 tag for `LICENSE.MIT`, preserving the upstream bytes instead of normalizing their whitespace.
- Kept all dependency bytes and metadata under `tools/reference/vendor/nlohmann`; no Cargo manifest, build script, package dependency, moving branch, or download step was added.

## Verification Evidence

- `cd tools/reference/vendor/nlohmann && shasum -a 256 --check SHA256SUMS` reports `json.hpp: OK` and `LICENSE.MIT: OK`.
- The vendored header SHA-256 is `aaf127c04cb31c406e5b04a63f1ae89369fccde6d8fa7cdda1ed4f32dfc5de63`, exactly matching the official v3.12.0 release record.
- Byte comparisons against the freshly downloaded immutable release/tag sources passed for both vendored files.
- `rg -n '3\.12\.0|MIT|nlohmann' tools/reference/vendor/nlohmann/SHA256SUMS THIRD_PARTY_NOTICES.md` finds the reviewed version, license, and private-tool classification.
- `rg -n 'nlohmann|json.hpp' Cargo.toml crates/liquidfun/Cargo.toml crates/liquidfun-test-protocol/Cargo.toml` returns no matches.
- The required ordered repository gate passed before the task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected stale GSD progress presentation**

- **Found during:** Plan metadata update
- **Issue:** `gsd-tools roadmap update-plan-progress 02` reported `summary_count: 7` and `updated: true` but left the tracked Phase-2 row at `6/14`; `state update-progress` updated frontmatter to 63% while leaving the body at 0%.
- **Fix:** Updated the stale roadmap row to the disk-derived `7/14` count, normalized its table spacing, and synchronized the state-body progress bar with the tool-computed 63% frontmatter value.
- **Files modified:** `.planning/ROADMAP.md`, `.planning/STATE.md`
- **Verification:** Seven Phase-2 summary files exist, the roadmap reports `7/14` with `In Progress` status, and both state progress representations report 63%.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 1 auto-fixed (1 workflow-tool bug)
**Impact on plan:** The correction keeps GSD progress metadata consistent with completed summaries; vendored dependency scope and bytes are unchanged.

## Issues Encountered

- `git diff --check` reports the trailing space in the upstream license's exact first line (`MIT License `). The byte is present in the authoritative v3.12.0 source, so it remains intentionally preserved by the verbatim-license requirement and is protected by the checked-in SHA-256 value.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-08 to compile the reviewed single header into the private C++ oracle adapter.
- No checksum, notice, immutable-source, Cargo-isolation, or build-time-fetch blocker remains.

## Self-Check: PASSED

- All four task-owned files listed in this summary exist.
- Task commit `97c1e39` exists and contains exactly the three vendor files plus `THIRD_PARTY_NOTICES.md`.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-07.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
