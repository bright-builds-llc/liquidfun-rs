---
phase: 23-baseline-pair-and-named-audit
plan: "03"
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:03:59Z
subsystem: observability-tooling
tags: [xtask, playground-cli, audit-bundle, fake-tools]

requires:
  - phase: 22-observability-shell
    provides: fake-tool pair and profile CLI, exclusive stamps, LIQUIDFUN_XTASK_STAMP_UNIX
  - phase: 23-baseline-pair-and-named-audit
    provides: copy-only dam-break-audit-bundle command and playground_cli/bundle.rs stub
provides:
  - fake-stamp CLI success coverage for copy-only audit-bundle including *syms* sidecar
  - fail-closed CLI coverage for HEAD mismatch, missing gzip, path traversal, and samply-tainted pair.json
affects:
  - 23-07 live pair/profile/bundle stamps for the named audit

tech-stack:
  added: []
  patterns:
    - fake pair then profile with bumped LIQUIDFUN_XTASK_STAMP_UNIX then dam-break-audit-bundle
    - fail-closed CLI asserts nonzero, no skip wording, no audit-bundle-identity.json

key-files:
  created: []
  modified:
    - tools/xtask/tests/playground_cli/bundle.rs

key-decisions:
  - "Keep THIRD_STAMP local in playground_cli/bundle.rs so 23-01 support.rs stays untouched."
  - "Do not patch playground/bundle.rs; 23-02 already copies rust.json.syms.json and fails closed."
  - "Do not mark PERF-AUDIT complete in this plan; fake cmake/samply still cannot name dominating functions."

patterns-established:
  - "Audit-bundle CLI tests mint FIRST_STAMP pair, SECOND_STAMP profile, THIRD_STAMP bundle via 1000000000/1/2 unix seconds."
  - "Traversal tests use --pair-stamp ../etc and 2001-09-09T01:46:40Z and assert no evidence-root etc directory."

requirements-completed: []

duration: 4min
completed: 2026-09-21
---

# Phase 23 Plan 03: Fake-Stamp Audit-Bundle CLI Summary

**Fake-stamp `dam-break-audit-bundle` CLI tests copy `pair.json`, `pair.md`, `rust.json.gz`, and `rust.json.syms.json` into a third exclusive stamp, and fail closed on HEAD mismatch, missing gzip, path traversal, and samply-tainted pair.json.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-21T04:00:27Z
- **Completed:** 2026-09-21T04:03:59Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Proved copy-only bundling through the real CLI: destination stamp is `2001-09-09T01-46-42Z`, source bytes stay unchanged, and `audit-bundle-identity.json` records `kind=audit_bundle`, `not_timing_authority=true`, `timing_authority=unprofiled_wall_clock`, `profile_blob=rust.json.gz`, plus copied `rust.json.syms.json`.
- Locked fail-closed CLI cases: rewritten profile `git_head`, deleted `rust.json.gz`, `--pair-stamp ../etc` / `2001-09-09T01:46:40Z`, and `"samply": true` in `pair.json` all exit nonzero with no identity file.
- Confirmed `just playground-dam-break-audit-bundle` is exactly `cargo xtask playground dam-break-audit-bundle`.

## Task Commits

Each task was committed atomically:

1. **Task 1: CLI success copies pair, gzip, and *syms* into a new stamp** - `667be95` (test)
2. **Task 2: CLI fail-closed for mismatch, missing gzip, traversal, and profile-tainted pair.json** - `dcca660` (test)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## RED excerpt

Production copy and fail-closed behavior already shipped in 23-02. Task 1 and Task 2 tests passed on first run against `cargo xtask playground dam-break-audit-bundle`; there was no RED `todo!()` gap and no production patch.

```text
test bundle::dam_break_audit_bundle_copies_pair_gzip_and_syms_into_a_new_stamp ... ok
test bundle::dam_break_audit_bundle_rejects_git_head_mismatch ... ok
test bundle::dam_break_audit_bundle_rejects_missing_rust_json_gz ... ok
test bundle::dam_break_audit_bundle_rejects_pair_stamp_path_traversal ... ok
test bundle::dam_break_audit_bundle_rejects_samply_in_pair_json ... ok
test result: ok. 14 passed; 0 failed
```

GREEN: `cargo test -p xtask --test playground_cli -- --test-threads=1` exits 0. Fake samply bytes remain `fake-samply-json-gz`; tests assert copy/identity only, not named functions.

## Files Created/Modified

- `tools/xtask/tests/playground_cli/bundle.rs` — fake-stamp success, sidecar copy, justfile alias, and fail-closed CLI tests

## Physical line counts

Counted with Python `sum(1 for _ in path.open())` including blanks and comments:

| File | Lines | Cap |
| --- | ---: | ---: |
| `tools/xtask/tests/playground_cli/bundle.rs` | 297 | 628 |

## Decisions Made

- Keep `THIRD_STAMP` (`2001-09-09T01-46-42Z`) in `bundle.rs` rather than `support.rs`.
- Leave `tools/xtask/src/playground/bundle.rs` and `pair.rs` untouched; 23-02 already copies `*syms*` and rejects forbidden pair keys.
- Do not mark REQUIREMENTS `PERF-AUDIT` complete: this plan only covers D-04 plumbing. Named-function notes remain 23-07 (D-01).
- Do not `git add` any `*.json.gz`; fixtures stay under `target/`. `git ls-files '*.json.gz'` is empty.

## Deviations from Plan

None - plan executed exactly as written. RED was conditional on a 23-02 sidecar/fail-closed gap; none was found.

***

**Total deviations:** 0 auto-fixed
**Impact on plan:** None.

## Issues Encountered

None. The first Task 1 commit used `git commit -F` after the heredoc wrapper rejected parentheses in the hook-injected command string.

## Known Stubs

None. The previous `//! Audit-bundle CLI tests land in 23-03.` stub was replaced with real tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-05/23-06 heap command and CLI tests, and 23-07 live pair/profile/bundle stamps.
- Do not run live Dam Break, cmake, or host samply until 23-07/23-09.
- Do not treat fake gzip bytes or copied `alloc::vec::Vec` sidecar JSON as named-function evidence.

## Self-Check: PASSED

***
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*
