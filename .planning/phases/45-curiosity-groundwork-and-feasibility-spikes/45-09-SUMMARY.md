---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "09"
subsystem: docs-scene-checklist
tags: [docs, scene-catalog, checklist, prep-01, prep-02, d-05, d-13]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:55:33.599Z"
requires:
  - "45-02: just scene-fingerprint-gate (--spot / --stamp) and the report's 'added (accepted)' line"
  - "45-03: ALL_SCENE_IDS and all_scene_ids_match_the_web_catalog_order"
  - "45-04: number-free PAGE_SUMMARY and catalog-derived web/e2e counts"
  - "45-05: scene-records-<family>.ts / previews-<family>.tsx split and the ScenePreview never guard"
provides:
  - "docs/adding-a-scene.md: per-scene checklist (R1-R12, W1-W6, S1-S3, T1-T6, E1-E5), gates and the D-05 baseline growth procedure"
  - "AGENTS.md Repo-Local Guidance pointer to docs/adding-a-scene.md"
affects: [45-10, 46, 47, 48, 49, 50, 51]
tech-stack:
  added: []
  patterns:
    - "Each checklist row names exactly one enforcement class: compile, test, compile + test, convention or optional"
key-files:
  created:
    - docs/adding-a-scene.md
  modified:
    - AGENTS.md
key-decisions:
  - "45-09: docs/adding-a-scene.md is the per-scene checklist for Phases 46-51; placeholders <id> (kebab), <module> (snake_case file name), <Variant> and <family>; not-yet-existing modules are never written as full repository paths"
  - "45-09: per-scene ms/step comes from cargo xtask playground scene-spot --scene <id> (the just recipe takes no arguments); the baseline grows only in its own commit, then the gate is rerun with --spot on that capture"
requirements-completed: [PREP-01]
duration: 3min
completed: 2026-10-14
---

# Phase 45 Plan 09: Per-scene Checklist Summary

**`docs/adding-a-scene.md` lists all 26 sites a new scene touches in the post-Phase-45 tree, each tagged with what enforces it. It names every gate with its exact command and documents the own-commit baseline growth procedure. AGENTS.md links to it from Repo-Local Guidance.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-14T16:54:01Z
- **Completed:** 2026-10-14T16:55:33Z
- **Tasks:** 2
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- **Rust bridge table:** 10 rows (R1, R2, R3/R5, R4, R6, R7, R8, R9, R10, R12). Each row uses the current names: `ALL_SCENE_IDS`, `all_scene_ids_match_the_web_catalog_order`, `survey_scenes_match_catalog_scene_ids`, `only_idle_scenes_are_scripted`, `EscapeLimits::for_scene` and `variant_index`.
- **Web table:** 5 rows (W1, W2, W3/W4, W5, W6). Records go in `scene-records-<family>.ts`, with the new family array appended to the `SCENES` spread. Previews go in `previews-<family>.tsx`, and the `never` guard makes a missing case a compile error.
- **Scripts, tests and e2e table:** 11 rows (S1-S3, T1-T4, T6, E1, E2, E5).
- **Already derived (no edit) list:** `PAGE_SUMMARY`, the shell.spec counts, the scenes and demo-media test counts, the gravity slider sizes, the `SURVEY_SCENES` length, the puzzle count, hud-safe-frame iteration, and the historical scene-survey.md.
- **Gates section:** `just scene-fingerprint-gate` (fresh, `--stamp`, `--spot`; 25 protected ids at Phase 45, new ids accepted as additions), the growth rule, `just readme-svg` with `readme-svg.test.ts`, the portrait-bounds test, `hud-safe-frame.spec.ts` (built with `just web-build`, per 45-04), `scale-legend.test.ts`, the credits path rule in `links.ts`, the budget of about 3,000 particles with recorded `median_ms_per_step`, `bun scripts/bright-builds-check.ts all`, and web typecheck plus unit tests.
- **Growth section:** a three-step ordered list using repeated `1.` markers, plus a manual `join` comparison labelled "fallback, not the gate", with a line-count check.
- **AGENTS.md:** one bullet at line 44, after the standing-authorization bullet and before `### Hobby-project scope`. The managed block (lines 1-35) is unchanged.

## Task Commits

1. **Task 1: Write docs/adding-a-scene.md:** `437a130da` (docs)
1. **Task 2: Point AGENTS.md Repo-Local Guidance at the checklist:** `60007dc8d` (docs)

## Verification

- `mdformat --version` reports mdformat 1.0.0 with mdformat-gfm 1.0.0. `mdformat docs/adding-a-scene.md` was run, then `just markdown-check` passed.
- The path-existence loop printed nothing, so every backticked repository path exists.
- Row counts: R 10 (needed 9 or more), W 5 (needed 5 or more), S/T/E 11 (needed 10 or more). `grep -cE "^[2-9]\. "` returned 0.
- Every required string is present: `just scene-fingerprint-gate`, `just readme-svg`, `portrait-bounds`, `hud-safe-frame.spec.ts`, `scale-legend.test.ts`, `links.ts`, `3,000`, `ms/step`, `bun scripts/bright-builds-check.ts all`, "own commit" (in the growth section) and "fallback" beside `join`.
- The documented fallback commands were run against the real baseline and capture `target/dam-break-perf/2026-10-14T16-00-20Z/scene-spot.json`. Both exited 0. The `jq` fingerprint lookup returned `4fa65d209b7e782f` for `dam-break`, which matches the baseline.
- Task 2 verify chain: the pointer is at line 44, which is after line 37 and before `### Hobby-project scope`. `git diff HEAD~1 -- AGENTS.md` shows exactly 1 added line, in hunk `@@ -43,0 +44 @@`.
- `bun scripts/bright-builds-check.ts all`: findings=0. AGENTS.md is 539 lines.

## Decisions Made

- `<module>` is a separate placeholder because scene files are snake_case (`tesla_valve.rs`) while catalog ids are kebab-case (`tesla-valve`). The credits row and the gate both use `scene/<module>.rs`.
- Every "Forced by" cell holds exactly one of the five allowed values. W1 is `test`, because the Rust catalog-order test fails until R4 and R7 match `SCENE_IDS`. W3/W4 is `optional`, because only scenes with live controls need it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The ms/step command uses `cargo xtask playground scene-spot --scene <id>`, not `just playground-scene-spot --scene <id>`**
- **Found during:** Task 1
- **Issue:** The `playground-scene-spot` recipe in the justfile takes no arguments, so the command named in the plan facts would fail.
- **Fix:** The doc names the xtask form, which accepts `--scene <id>` (it is in the playground USAGE string). For the full-catalog capture used in baseline growth, it keeps `just playground-scene-spot`.
- **Files modified:** `docs/adding-a-scene.md`
- **Commit:** `437a130da`

The plan's output section asks for `generated_by: gsd-executor`. I used `gsd-execute-phase`, as the orchestrator asked, matching Plans 02, 04 and 05.

## Requirement Status

- PREP-01: this plan documents the growth rule next to the gate, as D-05 requires. It was already marked complete by 45-02.
- PREP-02 is not ticked, per the orchestrator's instruction. The checklist (success criterion 2) is now written, and Plan 10 or phase completion ticks the requirement.

## Known Stubs

None.

## Threat Flags

None. T-45-19 is mitigated: paths are checked by script, every row names what enforces it, and the growth rule repeats the baseline's `growth_rule`. T-45-20 is mitigated: one line was added outside the managed block, confirmed by the diff.

## Next Phase Readiness

- Phases 46-51 follow `docs/adding-a-scene.md`. Phase 46 creates `scene-records-curiosity.ts` and `previews-curiosity.tsx`.
- Plan 10 can include the checklist's existence and AGENTS.md pointer in its final-tree gate run.

## Self-Check: PASSED

- FOUND: docs/adding-a-scene.md
- FOUND: AGENTS.md pointer at line 44
- FOUND: commit 437a130da
- FOUND: commit 60007dc8d
