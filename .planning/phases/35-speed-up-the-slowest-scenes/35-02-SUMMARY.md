---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-08T02:28:27.712Z
phase: 35-speed-up-the-slowest-scenes
plan: "02"
subsystem: testing
tags: [profiling, samply, benchmark, scene-survey, fingerprint]

requires:
  - phase: 35-01
    provides: "--scene filter and per-scene end-state fingerprint in playground-scene-spot"
provides:
  - "Phase before binary target/phase35/bin/spot-before (built at 809582431) and its 25-scene before run target/phase35/before-full.jsonl"
  - "35-PROFILES.md: method, ABBA A/B procedure, keep_rule.py and top_functions.py text, fresh before table, five target profiles with named hot paths, Attempts table with A0, Target records"
affects: [35-03, 35-04, 35-05, 35-06, 35-07, 35-08]

tech-stack:
  added: []
  patterns:
    - "Every keep/revert decision compares saved binaries under target/phase35/bin with interleaved ABBA runs and keep_rule.py"

key-files:
  created:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
  modified: []

key-decisions:
  - "Target set stays the D-01 five; water-wheel's rank 5 in the fresh run was one noisy run (34% spread) and a confirmation run put it back at rank 7"
  - "Hot paths: liquid-tumbler, stacked-drip and particles pressure::damping (22.9%, 17.1%, 22.8% self); tesla-valve visit_sorted_tag_indices_in_aabb 13.7% self; washing-machine push_fixture_particle_hit 15.2% self in the full-scan CCD (29.1% inclusive)"
  - "D-03 browser observation not run (no browser tool; a web build would hit the syspolicyd launch queue)"

patterns-established:
  - "Launch every freshly built or copied binary once with a 1-step run before timing it (syspolicyd stalls)"

requirements-completed: []

duration: 40min
completed: 2026-10-08
---

# Phase 35 Plan 02: Before Run and Target Profiles Summary

**A saved phase "before" binary with a fresh 25-scene before run and fingerprints, the ABBA keep-rule method, and samply profiles that name a hot path for each of the five target scenes**

## Performance

- **Duration:** about 40 minutes. 34 of them were a macOS launch stall: `spot-before` sat at `_dyld_start` while syspolicyd worked through a queue of earlier test binaries, about 40 s each.
- **Started:** 2026-10-08T01:49:05Z
- **Completed:** 2026-10-08T02:28:27Z
- **Tasks:** 2
- **Files modified:** 1 (`35-PROFILES.md`), plus local gitignored artifacts under `target/phase35/`

## Accomplishments

- `target/phase35/bin/spot-before` was built from `809582431`, a docs-only commit whose engine code equals the 35-01 tooling commit. `target/phase35/before-full.jsonl` holds 25 lines, each with a 16-hex fingerprint. No scene timed out.
- The fresh before run gives liquid-tumbler 25.429, tesla-valve 4.061, stacked-drip 2.798, washing-machine 2.164 and particles 1.645 ms/step. Load averages were about 9.4 to 9.6. Water-wheel landed at rank 5 because of one 2.083 ms run, a 34% spread. A confirmation run saved as `before-full-2.jsonl` matched all 25 fingerprints and put the survey's top five back in survey order. The record says this outright, and the target set is unchanged.
- Spreads above 10% are recorded honestly: water-wheel 34%, dam-break 52%, jelly-drop 104% and color-mixer 19%. The five targets stayed at or below 7.4%.
- All five targets were profiled with samply at 4 kHz. Each profiled run reproduced its before fingerprint. Hot paths:
  - liquid-tumbler: `pressure::damping` 22.9% self
  - tesla-valve: `visit_sorted_tag_indices_in_aabb` 13.7% self, plus emission at 8.9% and lifetime resequencing at 9.0% inclusive
  - stacked-drip: `pressure::damping` 17.1% self
  - washing-machine: `push_fixture_particle_hit` 15.2% self, with the full-scan CCD at 29.1% inclusive
  - particles: `pressure::damping` 22.8% self
- The fresh profiles agree with the research profiles. No top function changed.
- `35-PROFILES.md` records:
  - the method (host, rustc, BEFORE_COMMIT, binary paths, launch-stall warning)
  - the full `keep_rule.py` and `top_functions.py` texts
  - the 7-step ABBA procedure and the D-11 keep rule
  - the Attempts table with A0, the rejected branchless damping attempt
  - Target records for all five targets, each marked `profile recorded`

## Task Commits

1. **Task 1: Save the before binary, record the fresh before run, and write the method and keep rule**: `32b5f3916` (docs)
1. **Task 2: Profile the five targets with samply, name each hot path, and record the browser observation**: `57c1ee3b4` (docs)

## Files Created/Modified

- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: the phase record that later fix plans append to
- Local only (gitignored): `target/phase35/bin/spot-before`, `target/phase35/before-full.jsonl`, `before-full-2.jsonl`, `before-full.uptime`, `keep_rule.py`, `top_functions.py`, `shares.py`, and `profiles/*.json.gz` with their `.syms.json` sidecars

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Process deviations

**1. [Process] Task 1 committed separately**
- The plan only specifies a commit at the end of Task 2. To keep commits atomic per task, Task 1 landed as `32b5f3916`. Task 2's commit still contains only `35-PROFILES.md`, as its acceptance requires.

**2. [Rule 2 - Evidence] Added a confirmation before run and a scratch shares helper**
- **Issue:** The fresh before run put water-wheel at rank 5 with a 34% spread. Some inclusive shares the plan asks for, such as `solve_lifetimes` and the full-scan CCD, fall outside the reader's top-25 list.
- **Fix:** A second `spot-before --runs 3` (`before-full-2.jsonl`) settled the ranking. A scratch `shares.py` with the same symbol lookup computed the named inclusive shares and caller splits; the record describes how it works. Neither is committed, and neither is a harness.

**3. Profiling binary built while the before binary was stalled**
- The profiling build ran while `spot-before` sat stalled at `_dyld_start`, before any timing run started. This queued its launch assessment early. No timing overlapped a build.

---

**Total deviations:** 3, all minor process and evidence additions. No source change.

## Issues Encountered

- syspolicyd launch stall of 34 minutes for the copied `spot-before`. `/usr/bin/log show` showed Gatekeeper scanning one queued test binary about every 40 s, each with a failed notarization network check. Warmup launches cleared both binaries before any timing ran. The method section now requires a 1-step warmup launch for every new binary.
- The host was busy (load about 7 to 10). The before run is about 2 to 6% slower than the committed survey for most scenes, so later plans must compare saved binaries back to back, which the procedure already requires.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

- 35-03 through 35-07 start from §Target profiles in `35-PROFILES.md`, compare against `target/phase35/bin/spot-before`, and apply the A/B procedure and `keep_rule.py`.
- `target/` is not committed. If `cargo clean` runs, rebuild `spot-before` from `809582431` and recreate the scripts from their text in `35-PROFILES.md`.
- PERF-08 and PERF-09 stay Pending. They complete only after the fix plans and final verification.

## Self-Check: PASSED

- FOUND: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
- FOUND: target/phase35/bin/spot-before
- FOUND: target/phase35/before-full.jsonl (25 lines)
- FOUND: target/phase35/profiles/{liquid-tumbler,tesla-valve,stacked-drip,washing-machine,particles}.json.gz
- FOUND: commit 32b5f3916
- FOUND: commit 57c1ee3b4
