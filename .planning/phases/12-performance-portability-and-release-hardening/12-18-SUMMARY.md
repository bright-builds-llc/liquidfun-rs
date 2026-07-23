---
phase: 12-performance-portability-and-release-hardening
plan: "18"
subsystem: dependency-policy
tags: [cargo-deny, eframe, egui, tiny-skia, renderer-isolation]
requires:
  - phase: 12-03
    provides: compiled eframe/egui desktop shell with passive semantic authority
provides:
  - Macroquad-free workspace and lockfile
  - zero-waiver advisory policy with no ttf-parser dependency edge
  - exact replacement-renderer capability identity and capture hashes
  - extracted-package proof that liquidfun remains renderer-free
affects: [phase-12-release-audit, dependency-policy, packaging, desktop-testbed]
tech-stack:
  added: []
  patterns: [explicit eframe feature closure, zero-waiver dependency policy, hash-bound capability evidence]
key-files:
  created:
    - .planning/phases/12-performance-portability-and-release-hardening/12-18-SUMMARY.md
  modified:
    - crates/liquidfun-testbed/Cargo.toml
    - Cargo.lock
    - deny.toml
    - crates/liquidfun-testbed/CAPABILITY.md
    - tools/xtask/tests/package_cli.rs
key-decisions:
  - "Disable only eframe's inherited winit/default feature while retaining accessibility, default fonts, Wayland, web screen reader, wgpu, and X11 so the optional Adwaita CSD cannot reintroduce ttf-parser."
  - "Allow only the reviewed permissive licenses required by the exact replacement graph; keep the advisory ignore list empty."
patterns-established:
  - "Replacement desktop dependencies use an explicit feature closure so optional transitive security state cannot enter through a broad default."
  - "Capability records bind exact adapter versions and deterministic artifact hashes while explicitly excluding pixels, timing, parity, and performance from compatibility authority."
requirements-completed: [API-12, DOCS-07, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T23:37:38Z
duration: 24m
completed: 2026-07-23
---

# Phase 12 Plan 18: Renderer Dependency Policy Summary

**Macroquad, both ttf-parser branches, and both temporary advisory waivers are gone while the exact eframe/egui/tiny-skia adapter retains deterministic capability evidence and published-package isolation.**

## Performance

- **Duration:** 24m
- **Started:** 2026-07-23T23:13:37Z
- **Completed:** 2026-07-23T23:37:38Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Removed Macroquad and its obsolete dependency closure from the private testbed and refreshed `Cargo.lock` through Cargo.
- Replaced the temporary advisory allowlist with `ignore = []`, eliminated the remaining optional `ttf-parser 0.25.1` edge, and admitted only reviewed permissive replacement licenses.
- Regenerated the complete passive capability matrix for `eframe-egui-0.35.0+tiny-skia-0.12.0` and bound all three PNGs plus the machine report to exact SHA-256 hashes.
- Proved the sole publishable `liquidfun` package remains renderer-free through resolved metadata, archive inspection, and an external 171-entry build and test.

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove Macroquad and both temporary advisory waivers** - `0361335` (fix)

## Files Created/Modified

- `crates/liquidfun-testbed/Cargo.toml` - Removes Macroquad and defines the exact security-bounded eframe feature closure.
- `Cargo.lock` - Removes Macroquad, both ttf-parser versions, the optional Adwaita CSD, and their unreachable transitive packages.
- `deny.toml` - Clears advisory ignores and allows the reviewed permissive licenses used by the replacement graph.
- `crates/liquidfun-testbed/CAPABILITY.md` - Records exact replacement versions, measurements, artifact hashes, passive authority, and explicit parity/performance exclusions.
- `tools/xtask/tests/package_cli.rs` - Enforces the post-replacement zero-waiver, no-Macroquad package-policy contract.

## Decisions Made

- Kept eframe 0.35.0 and all required desktop capabilities, but replaced the broad `winit/default` feature with explicit platform and renderer features because its optional Wayland Adwaita decoration path was the only remaining route to unmaintained ttf-parser.
- Added no advisory exception. The exact replacement graph instead passes cargo-deny with an empty ignore list and a reviewed permissive license allowlist.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Removed the replacement stack's optional ttf-parser edge**

- **Found during:** Task 1 cargo-deny verification
- **Issue:** Eframe's inherited `winit/default` feature enabled `wayland-csd-adwaita -> ab_glyph -> owned_ttf_parser -> ttf-parser 0.25.1`, so deleting Macroquad alone still failed RUSTSEC-2026-0192 and contradicted the zero-waiver goal.
- **Fix:** Disabled eframe default features and explicitly retained `accesskit`, `default_fonts`, `wayland`, `web_screen_reader`, `wgpu`, and `x11`, omitting only the optional Adwaita client-side decoration edge.
- **Files modified:** `crates/liquidfun-testbed/Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo tree --workspace --target all` and `Cargo.lock` contain no Macroquad, ttf-parser, owned_ttf_parser, or sctk-adwaita; the capability matrix and full test suite pass.
- **Committed in:** `0361335`

**2. [Rule 2 - Missing Critical] Completed the replacement graph's license policy**

- **Found during:** Task 1 cargo-deny verification
- **Issue:** The predecessor renderer plans added the exact replacement stack without admitting its permissive BSD-2-Clause, BSD-3-Clause, BSL-1.0, CC0-1.0, ISC, OFL-1.1, and Ubuntu-font-1.0 licenses, so cargo-deny could not pass.
- **Fix:** Added only those reviewed permissive identifiers to the existing license allowlist.
- **Files modified:** `deny.toml`
- **Verification:** `cargo deny --locked check` reports advisories, bans, licenses, and sources all green.
- **Committed in:** `0361335`

**3. [Rule 3 - Blocking] Replaced the retired waiver regression**

- **Found during:** Task 1 focused policy verification
- **Issue:** The Phase 11 package regression still required both retired advisory ignores and a Macroquad testbed dependency, directly opposing this plan's acceptance criteria.
- **Fix:** With parent approval, changed only that test to require an empty ignore list, exact replacement dependencies, private publication state, and no Macroquad.
- **Files modified:** `tools/xtask/tests/package_cli.rs`
- **Verification:** The focused regression and the exact ordered full test gate pass.
- **Committed in:** `0361335`

**Total deviations:** 3 auto-fixed (2 missing critical security-policy repairs, 1 blocking regression repair)

**Impact on plan:** All deviations were required to achieve the plan's zero-waiver dependency boundary. The approved renderer versions and semantic authority did not change.

## Issues Encountered

- The repository-wide `just markdown-check` baseline remains red in seven files outside this plan's ownership: `UPSTREAM.md`, `ARCHITECTURE.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `TESTING.md`, `UPSTREAM-CORPUS.md`, and `docs/decisions/0001-oracle-selection.md`. The failure list was captured before editing and reproduced afterward with plain mdformat 1.0.0 under Python 3.13. The plan-owned `crates/liquidfun-testbed/CAPABILITY.md` passes the exact formatter independently and is absent from the final global failure list.
- Cargo-deny reports policy-allowed duplicate-version warnings but no advisory, license, ban, or source failure.

## Known Stubs

None.

## Verification

- Dependency absence passed across all target graphs: no Macroquad, ttf-parser, owned_ttf_parser, or sctk-adwaita remains.
- `cargo deny --locked check` passed with an empty advisory ignore list.
- `cargo xtask package verify` passed with 171 extracted entries built and tested outside the repository.
- The replacement capability matrix passed 20/20 with zero logical steps and captures; its report and three PNG hashes exactly match `CAPABILITY.md`.
- Focused capability tests passed 4/4 and the zero-waiver package-policy regression passed.
- Exact ordered Rust gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `uvx --python 3.13 --from mdformat==1.0.0 mdformat --check crates/liquidfun-testbed/CAPABILITY.md` passed.
- `git diff --cached --check` passed before the task commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The inherited renderer dependency-security obligation is closed with no waiver and no unmaintained font-parser edge.
- Release and packaging plans can rely on the exact replacement identity, deterministic diagnostic hashes, passive authority boundary, and renderer-free consumer package.
- The seven unrelated Markdown baseline files remain for their current owners; this plan did not expand into them.

## Self-Check: PASSED

- Confirmed all five implementation files and this summary exist.
- Confirmed task commit `0361335` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
