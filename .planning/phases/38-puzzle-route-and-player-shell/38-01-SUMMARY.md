---
phase: 38-puzzle-route-and-player-shell
plan: "01"
subsystem: web-routing
tags: [typescript, routing, puzzle, fingerprint-baseline, PUZ-02, PUZ-06]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T02:37:57.533Z"
requires:
  - web/src/routing/hash.ts maybeParseSceneRoute / normalizeSceneRoute (unchanged)
  - web/src/catalog/puzzles.ts PUZZLE_IDS (unchanged)
  - web/src/player/runtime.ts maybeReadySceneId, titleForRoute, DEFAULT_TITLE (unchanged)
provides:
  - web/src/routing/playable-route.ts PlayableRoute, NormalizedPlayableRoute, maybeParsePlayableRoute, normalizePlayableRoute
  - web/src/player/playable-target.ts PlayableTarget, maybePlayableTarget, samePlayableTarget, titleForPlayableRoute, playableRouteIdentity
  - web/src/catalog/puzzle-levels.ts PUZZLE_TITLES, FIRST_PUZZLE_ID, puzzleTitleForId, puzzleHash, maybeNextPuzzleId
  - target/phase38/base.txt (25 scene fingerprints for phase base 589cb0fb9)
affects:
  - 38-07 (widens runtime.ts parameter types and dispatches on PlayableTarget)
  - 38-06 (renders unknown-puzzle maybeRaw as text)
  - Phase 38 final 25/25 fingerprint gate (compares against target/phase38/base.txt)
tech-stack:
  added: []
  patterns:
    - wrapper parser that claims one new first segment and delegates everything else to the unchanged scene parser
key-files:
  created:
    - web/src/routing/playable-route.ts
    - web/src/player/playable-target.ts
    - web/src/catalog/puzzle-levels.ts
    - web/tests/playable-route.test.ts
    - web/tests/playable-target.test.ts
  modified: []
key-decisions:
  - "Puzzle routing is a wrapper (playable-route.ts) over the byte-identical hash.ts; only parts[0] === 'puzzle' is claimed, so #/puzzles and full URLs stay scene unknown routes"
  - "D-23 baseline taken from a fresh scene-spot run (stamp 2026-10-12T02-36-59Z) on HEAD whose Rust code equals 589cb0fb9; identical to Phase 37 after-final.txt"
requirements: [PUZ-02, PUZ-06]
duration: 3min
completed: 2026-10-12
---

# Phase 38 Plan 01: Playable route parser, puzzle targets and level titles Summary

**`#/puzzle/<id>` now parses through a wrapper over the unchanged scene parser into `puzzle` or `unknown-puzzle` routes. A `PlayableTarget` union carries titles and identities, and a TS title table supplies "First Pour" before wasm loads. The 25-scene fingerprint baseline for phase base `589cb0fb9` was recorded before any web change.**

## Performance

- **Duration:** about 3 min (02:35Z to 02:38Z). The scene-spot run took most of it and ran alongside the TS work.
- **Tasks:** 2 of 2
- **Files:** 5 created, 0 modified

## Accomplishments

### Task 1: D-23 fingerprint baseline (not committed, `target/` is gitignored)

- `target/phase38/phase-base.txt` = `589cb0fb953d8676b2209caf7ee627e5aab2d6e3`.
- `git diff --quiet 589cb0fb9 HEAD -- crates tools Cargo.toml Cargo.lock` was empty, so the baseline ran on the main tree and no worktree was needed.
- `just playground-scene-spot` wrote stamp **`target/dam-break-perf/2026-10-12T02-36-59Z`** (exit 0). Its log and stamp path are in `target/phase38/baseline.log`.
- `target/phase38/base.txt` has 25 sorted lines, each `<scene-id> <16 hex>`.
- **Phase 37 cross-check: identical.** `diff target/phase37/after-final.txt target/phase38/base.txt` showed no difference.

### Task 2: wrapper parser, targets and titles (commit `0db728c70`)

- `maybeParsePlayableRoute` splits the hash the same way `hash.ts` does. Only a first segment of `puzzle` is claimed: two parts with an allowlisted id give `{kind:"puzzle", id}`, and anything else gives `{kind:"unknown-puzzle", maybeRaw: parts.slice(1).join("/")}`. Every other hash returns `maybeParseSceneRoute(hash)` unchanged.
- `normalizePlayableRoute` gives puzzle kinds no replacement hash and delegates to `normalizeSceneRoute` for everything else.
- `PlayableTarget` contains `scene` (ready ids only, via `maybeReadySceneId`) and `puzzle`. `samePlayableTarget` needs both sides defined with the same kind and id. `titleForPlayableRoute` gives `"First Pour · puzzle · liquidfun-rs playground"`, `DEFAULT_TITLE` for unknown-puzzle, and `titleForRoute` for scene routes. `playableRouteIdentity` uses an exhaustive switch over all five kinds.
- `puzzle-levels.ts` contains `PUZZLE_TITLES`, `FIRST_PUZZLE_ID` (throws if the registry is empty), `puzzleTitleForId`, `puzzleHash`, and `maybeNextPuzzleId`, which is undefined for `first-pour`.

## Verification

- `bunx vitest run tests/playable-route.test.ts tests/playable-target.test.ts tests/hash.test.ts`: 3 files, 35 tests passed. The 27 new tests are 11 route and 16 target/title tests, above the minimum of 18.
- `bun run typecheck` (`tsc --noEmit`): clean.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet 589cb0fb9 -- web/src/routing/hash.ts web/src/catalog/puzzles.ts web/tests/hash.test.ts`: exit 0. `grep -c PUZZLE_IDS web/src/routing/hash.ts` prints 0.
- `git show --stat HEAD` lists exactly the five plan files.
- Threat mitigations: T-38-01 is covered because `<img ...>` stays verbatim in `maybeRaw`. T-38-02 is covered because `https://evil.example/#/puzzle/first-pour` gives the same result as the scene parser. T-38-03 is covered by the byte-identity check above.

## Deviations from Plan

None. The plan was executed as written. The web package has no formatter or lint script, so typecheck and the bright-builds checks were the applicable pre-commit checks. The plan required one commit for the five files, so the TDD task had no separate RED commit.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: web/src/routing/playable-route.ts, web/src/player/playable-target.ts, web/src/catalog/puzzle-levels.ts, web/tests/playable-route.test.ts, web/tests/playable-target.test.ts
- FOUND: commit 0db728c70
- FOUND: target/phase38/base.txt (25 lines), target/phase38/phase-base.txt, target/phase38/baseline.log
