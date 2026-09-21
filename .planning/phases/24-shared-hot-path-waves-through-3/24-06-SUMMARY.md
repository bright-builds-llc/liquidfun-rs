---
phase: 24-shared-hot-path-waves-through-3
plan: "06"
subsystem: isolation-honesty
tags: [PERF-BASELINE, package-verify, cargo-tree, bright-builds, independent-review]

requires:
  - phase: 24-shared-hot-path-waves-through-3
    provides: exclusive native_scene_spot stamp 2026-09-21T21-10-50Z with not_timing_authority
  - phase: 24-shared-hot-path-waves-through-3
    provides: unprofiled Dam Break Medium pair.json rust_over_cpp_ratio <= 3.0 on this host
provides:
  - BENCHMARKING.md pointer to unreviewed playground-scene-spot-check.md
  - isolation evidence that liquidfun stays bitflags-only with unsafe_code forbid
  - explicit record that independent AI review is not self-approved
affects:
  - /gsd-verify-work 24
  - independent AI review
  - PERF-NOTES / PERF-WASM in Phase 25

tech-stack:
  added: []
  patterns:
    - package verify plus cargo tree bitflags-only isolation
    - just remains a one-line cargo xtask printer including playground-scene-spot
    - implementer does not write a review acknowledgment

key-files:
  created: []
  modified:
    - BENCHMARKING.md
    - crates/liquidfun/src/particle/storage/runtime.rs
    - crates/liquidfun/src/particle/solver/constraints.rs
    - crates/liquidfun/src/world/particle_coupling.rs

key-decisions:
  - "Independent AI review is not performed by this implementing agent; /gsd-verify-work 24 and a separate AI reviewer remain required."
  - "BENCHMARKING.md points at docs/playground-scene-spot-check.md as unreviewed native scene spot-checks; that sample is not the Dam Break 3x gate and must not enter the empty manifest."
  - "Clippy unnecessary_wraps on always-Ok particle storage Results was dropped rather than allowed so cargo clippy -p liquidfun -D warnings could pass."

patterns-established:
  - "Isolation gates: cargo xtask package verify plus cargo tree -p liquidfun --edges normal showing only bitflags."
  - "The implementing agent must not approve Phase 24; independent AI review is a later separate-AI step."

requirements-completed: [PERF-BASELINE]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T21:39:00Z

duration: 17min
completed: 2026-09-21
---

# Phase 24 Plan 06: Isolation and Honesty Summary

**Package isolation, scalar `unsafe_code = "forbid"`, empty `reviewed_reports`, and a BENCHMARKING.md pointer to unreviewed five-scene spot-checks; independent AI review remains a later separate-AI step.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-09-21T21:21:15Z
- **Completed:** 2026-09-21T21:38:25Z
- **Tasks:** 2
- **Files modified:** 4

## Independent review (D-13) — not satisfied here

This plan does **not** write a REVIEW.md acknowledgment bound to the implementing agent's identity. Independent AI review (D-13, owner 2026-09-16) is **not** performed by this implementer and is **not** satisfied by passing automated checks.

The implementing agent must not approve the phase. `/gsd-verify-work 24` plus a **separate** AI reviewer remain required.

No `24-REVIEW.md` (or similar) acknowledgment file was authored by this implementing agent claiming phase approval.

## Accomplishments

- Pointed `BENCHMARKING.md` Exploratory local diagnosis at `[docs/playground-scene-spot-check.md](docs/playground-scene-spot-check.md)` as an unreviewed local sample that is not the Dam Break 3× gate and must not be copied into `reference/performance/manifest.toml`. No wall-ms and no “Rust is N×” claim were pasted.
- Proved published `liquidfun` stays bitflags-only (`package verify` + `cargo tree --edges normal`). Workspace `unsafe_code = "forbid"` is unchanged. `reviewed_reports = []`. No committed `.json.gz` / `.trace` / `dhat-heap.json`.
- Confirmed `just` playground recipes remain one-line `cargo xtask` printers, including `playground-scene-spot`. README has no universal Rust-is-N× claim.
- Dropped two always-`Ok` particle-storage `Result` wrappers so `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings` exits 0.

## Task Commits

Each task was committed atomically:

1. **Task 1: Point BENCHMARKING.md at unreviewed spot-check notes** - `ceda4cb` (docs)
1. **Task 2: Isolation, Bright Builds, and no self-approval** - `5715c2b` (fix: clippy `unnecessary_wraps`)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Isolation evidence (Task 2)

| Check | Result |
| --- | --- |
| `cargo xtask package verify` | exit 0; `package verified: 238 entries built and tested outside the repository` (re-run after the wrap fix) |
| `cargo tree -p liquidfun --edges normal` | `liquidfun` → `bitflags v2.13.0` only; no `dhat`, `samply`, `serde`, `serde_json`, `flate2`, `cmake`, or `rayon` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings` | exit 0 after dropping always-Ok wraps |
| `cargo test -p liquidfun` | exit 0; lib tests and 23 doctests passed |
| `cargo deny --locked check` | advisories ok, bans ok, licenses ok, sources ok |
| `bun scripts/bright-builds-check.ts all` | `file-lengths` findings=0, all findings=0 |
| `just markdown-check` | exit 0 |

Assertions:

- `rg -n 'unsafe_code = "forbid"' Cargo.toml` matches; no `[profile.release]` debug override; `[profile.profiling]` remains.
- `rg -n "std::simd|rayon" crates/liquidfun/src` is empty.
- `rg -n "samply|cmake|dhat" justfile` is empty.
- Playground recipes are one-line `cargo xtask playground …` aliases: `dam-break-bench`, `dam-break-profile`, `dam-break-timers`, `dam-break-audit-bundle`, `dam-break-heap`, `scene-spot`.
- `crates/liquidfun/Cargo.toml` production deps are `bitflags` only.
- `reference/performance/manifest.toml` has `reviewed_reports = []`; `git diff -- reference/performance/manifest.toml` is empty.
- `git ls-files '*.json.gz' '*.trace' 'dhat-heap.json'` is empty.
- `rg -n "Rust is .*×" README.md` is empty.
- No `mod.rs` under `crates/liquidfun/src/particle` or `tools/xtask/src/playground`.
- No implementer-authored `24-REVIEW.md`.

## Files Created/Modified

- `BENCHMARKING.md` — unreviewed pointer to `docs/playground-scene-spot-check.md`; still forbids copying playground numbers into the empty manifest
- `crates/liquidfun/src/particle/storage/runtime.rs` — `limit_solver_speeds` and `replace_indexed_particle_contacts` return `()`
- `crates/liquidfun/src/particle/solver/constraints.rs` — LimitVelocity calls the void speed clamp
- `crates/liquidfun/src/world/particle_coupling.rs` — indexed contact commit no longer maps a never-Err Result

## Decisions Made

- Do not self-approve. Independent AI review remains after `/gsd-verify-work 24` and must be a separate AI reviewer (owner policy 2026-09-16).
- Keep `BENCHMARKING.md` as a path pointer to the five-scene spot-check notes, not a public speed claim or manifest row.
- Fix clippy `unnecessary_wraps` by dropping always-Ok internal Results rather than adding an allow or a TSV exception.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Clippy `unnecessary_wraps` on particle storage**
- **Found during:** Task 2 (isolation commands)
- **Issue:** `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings` failed on always-`Ok` `Result<(), ParticleStorageError>` for `limit_solver_speeds` and `replace_indexed_particle_contacts`. 24-05 had logged this as a pre-existing out-of-scope finding; this plan requires the clippy command to exit 0.
- **Fix:** Return `()` from those internal methods and update LimitVelocity plus indexed contact commit call sites.
- **Files modified:** `crates/liquidfun/src/particle/storage/runtime.rs`, `crates/liquidfun/src/particle/solver/constraints.rs`, `crates/liquidfun/src/world/particle_coupling.rs`
- **Verification:** clippy exit 0; `cargo test -p liquidfun` exit 0; `cargo xtask package verify` exit 0; `cargo tree -p liquidfun --edges normal` still bitflags-only
- **Committed in:** `5715c2b`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for the written clippy gate. No SIMD, Rayon, `unsafe`, playground tooling leak, or review self-approval.

## Issues Encountered

Clippy `-D warnings` on `liquidfun` failed until the always-Ok wraps were dropped. Isolation and honesty gates then passed as specified.

## Authentication Gates

None.

## Known Stubs

None.

## Threat Flags

None. Task 1 only added a docs pointer. Task 2 ran isolation gates and dropped two never-Err internal Results. No new network, auth, file-access, or schema surface.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 24 plans are complete on disk after this SUMMARY. Ready for `/gsd-verify-work 24`, then independent AI review by a **separate** agent. This implementer does not approve the phase.

PERF-NOTES and PERF-WASM remain Phase 25. Do not copy playground numbers into `reference/performance/manifest.toml`. Do not `git add` `*.json.gz`, `*.trace`, or `dhat-heap.json`. Do not write README or crates.io “Rust is N×” claims.

---
*Phase: 24-shared-hot-path-waves-through-3*
*Completed: 2026-09-21*

## Self-Check: PASSED

- FOUND: `BENCHMARKING.md`
- FOUND: `.planning/phases/24-shared-hot-path-waves-through-3/24-06-SUMMARY.md`
- FOUND: `docs/playground-scene-spot-check.md`
- FOUND: `ceda4cb` Task 1
- FOUND: `5715c2b` Task 2
- FOUND: `git ls-files '*.json.gz'` empty
- FOUND: SUMMARY states independent AI review is not self-approved
- FOUND: no implementer-authored `24-REVIEW.md`
