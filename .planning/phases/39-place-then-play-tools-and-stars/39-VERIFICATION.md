---
phase: 39-place-then-play-tools-and-stars
verified: 2026-10-13T02:52:00Z
status: passed
score: 5/5 roadmap success criteria verified; 71/71 plan must-have truths verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: 2026-10-13T02:52:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 39: Place-then-Play Tools and Stars Verification Report

**Phase Goal:** Players can solve a level by placing a limited set of fixed-size tools while building, pressing Play and earning up to three stars.
**Verified:** 2026-10-13T02:52:00Z, by an AI verifier (Claude Opus 5.5, acting as gsd-verifier). This is independent AI review under the CLAUDE.md "Independent review" policy, not human approval.
**Status:** passed
**Re-verification:** No. This is the initial verification. It covers HEAD `f71cc9970`, which includes the 5 code-review fixes applied after the Plan 14 gate run.

Method: I re-ran every cheap deterministic gate myself after the review fixes, rather than relying on `target/phase39/checks-14.results`, which predates commits `0b5472f26..f3a895627`. I rebuilt the wasm package and the app before running the browser suites. My logs are in the session scratchpad. Repo evidence is in `target/phase39/`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The tray shows fixed-size tools with remaining counts. Mouse and touch can tap to place, drag, rotate in 15° steps and remove, with targets of at least 44 px. The keyboard can do the same (arrows, R, Delete, Space). | ✓ VERIFIED | `PuzzleToolTray.tsx` shows a data-driven `trayModel` ("Plank, 2 left" → "1 left" → "none left", aria-disabled at 0). `PuzzleSelectionBar.tsx` has rotate ±15° and Remove. CSS sets the editor buttons to 44 px. The e2e "wins by placing, rotating and playing with the mouse", "drags a tool with the grab offset", "plays from the keyboard" (Enter, ArrowLeft×2, Shift+R, Shift+Tab, Space) and "removes the selected tool with Delete" all pass. The touch project "wins with taps on a phone" asserts there are no editor buttons under 44×44. The 22 CSS px hit slop (`hit-test.ts`) gives a 44 px canvas target. **Touch drag was not covered by any e2e, so I checked it myself** with a CDP touch drag in the 390×844 `hasTouch` project. The plank moved `10,64,0` → `20,97,0`: 10 cells right plus the designed 48 CSS px finger lift. The page did not scroll and the scene pointer was not used (`data-pointer-accepted=0`). Tap-rotate CCW gave angle 1, and tap-Remove cleared the tool. |
| 2 | Placements that overlap walls, the cup, water or other tools, or that leave the build area, are shown as invalid and Play stays disabled. The Rust validator rejects the same placements. | ✓ VERIFIED | `validate.rs` has one `placement_statuses` function used by both `inspect` (the wasm `inspectPuzzlePlacements`) and `validate_for_build`. `lib.rs::build_puzzle_core` calls `validate_for_build`, so `newPuzzle` fails closed. SAT with a gap of 2×particle_radius is in `sat.rs`, with no trig. There is a unit test for each reason. `inspector_and_new_puzzle_agree` and `first_pour_solution_fixture_placement_is_valid` pass. The TS/Rust code parity test (`placement-status-parity.test.ts`) passes. The e2e "highlights invalid placements and disables Play" checks the cup overlap status line, `aria-disabled` with `aria-describedby`, and the two-tool overlap. |
| 3 | Play runs the level. Stop/Reset returns to building with placements kept and replays to the same outcome. Clear removes all tools. Placing, removing, re-placing and resetting three times gives the same fingerprint as a single clean placement. | ✓ VERIFIED | `puzzle-run.ts` `playPuzzle`/`stopPuzzle` build fresh sessions with `buildPuzzleSessionSync`. The e2e "Stop keeps placements and replays the same outcome" passes: after Stop it is back at step 0 with the same placements, and the replay fingerprint equals a clean seam run. The Clear e2e passes. D-27 is covered at all 3 levels: the Vitest reducer sequence, Rust `permuted_placements_give_identical_fingerprints`, and the e2e "edit history gives the same fingerprint as a clean placement", where the held-frame FNV hash equals the clean run's. |
| 4 | The player sees the ★ Win, ★ Brim and ★ Thrifty criteria before Play. A winning run awards stars, and the best result per star is kept across runs. | ✓ VERIFIED | `stars.ts` scores Win, Brim (won and latched) and Thrifty (won with placements ≤ parTools). Thresholds come from `levelInfo` and the frame (`brimCount` 810, `parTools` 1 in `first_pour.rs`). `star-store.ts` does a per-level OR merge that never lowers a star. There is one store per playground session (`playground-session.ts:124`). The e2e "keeps the best stars across Edit, Stop and a losing run" checks 3 "none" rows before Play, 3 earned rows, then that best-of stays after Edit, Stop and a losing run. The screenshots show "Filled! 90% of 60%" with all 3 stars. |
| 5 | Fast-forward at 2× and 4× gives the same outcome step and fill as 1×. A synthetic-clock e2e places a tool and wins, including in a touch project. 25/25 scene fingerprints are unchanged. | ✓ VERIFIED | `puzzle-loop.ts` uses `speedStepRequest` and `restoreUnrunSpeedSteps`, and the step cap of 4 is unchanged (grep gate). The e2e "fast-forward keeps outcome, decision and peak" finds identical outcome step, decided step, peak held and held hash at 1×, 2× and 4×. Both the touch and mouse place-and-win e2e pass. I re-ran `just playground-scene-spot` myself (run dir `target/dam-break-perf/2026-10-13T02-49-07Z/`): its 25 lines are identical to `target/phase39/base.txt`. |

**Score:** 5/5 roadmap truths verified. All 71 plan must-have truths (Plans 01–14) are also satisfied; see the per-plan summary below.

### Plan must-haves (merged, condensed)

| Plan | Truths | Status | Key evidence |
| --- | --- | --- | --- |
| 01 Rust validator and inspector | 6 | ✓ | 13 status codes; SAT; `inspectPuzzlePlacements`; newPuzzle parity; brim 810, par 1, 2 planks, empty no_place; base.txt has 25 lines |
| 02 TS editor core | 6 | ✓ | `placement-status.ts`, `editor.ts` (canonical order), `hit-test.ts` (slop 22), `editor-format.ts`; Vitest |
| 03 Stars, keyboard, speed | 5 | ✓ | `stars.ts`, `star-store.ts` (allowlist, null-prototype snapshot), `star-format.ts`, `keyboard.ts`, `speed.ts` |
| 04 Frame parsing | 3 | ✓ | `frame.ts`/`frame-puzzle.ts` fail-closed D-11 rules; Vitest |
| 05 Peak and Brim in Rust | 5 | ✓ | `peak.rs` tests (splash, monotonic, freeze, decision); `peak.observe` in `hooks.rs`; `status_is_consistent`; permuted fingerprint; margin (peak 1331 at step 600 against brim 810 + 45) |
| 06 Wasm and level info | 5 | ✓ | Generated package exports the inspector and getters; `decodeInspection` fails closed; parity test |
| 07 Phase machine | 8 | ✓ | `reduce-puzzle.ts` `canPlay`, D-10 decision, committed vs live inspection; Vitest |
| 08 Replay fixtures | 4 | ✓ | Fixture diff since phase base only adds lines (`peak_held`, `brim_step`). `just puzzle-replay-wasm`: won @201 final 689 peak 811 brim 262; lost @720 final 0. Release replay 27/27 |
| 09 Shell wiring | 5 | ✓ | `puzzle-run.ts`, `puzzle-editing.ts` → `inspectPlacementLanes`; star recording at the decision and at Stop |
| 10 Fast-forward | 2 | ✓ | `puzzle-loop.ts` speed request and debt restore; `MAX_STEPS_PER_FRAME = 4` |
| 11 Overlay and pointer | 4 | ✓ | `drawPuzzleTools` and `attachPuzzleEditorInput` wired in `scene-surface.ts`; debug attributes in `PlaygroundStage.tsx` |
| 12 HUD components | 6 | ✓ | Tray, selection bar and transport carry `data-hud-obstacle`; `PuzzlePanel` keydown → `puzzleKeyCommand`; canvas `tabindex=0 role=application` |
| 13 E2E | 7 | ✓ | 29/29 puzzle e2e pass (chromium and touch) |
| 14 D-30 gates | 5 | ✓ | I re-ran them all post-fix (see the spot-checks) |

### Required Artifacts

| Artifact | Status | Details |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/puzzle/{sat,validate,inspect,peak}.rs`, `frame/puzzle.rs` | ✓ VERIFIED | Substantive (55–304 lines). Wired into `lib.rs`, `hooks.rs` and `frame.rs`. No `unwrap()` in production code |
| `crates/liquidfun-wasm/src/puzzle/fixtures/*.txt` | ✓ VERIFIED | Only `peak_held`/`brim_step` lines added. Outcome lines are byte-unchanged |
| `web/src/puzzle/{placement-status,editor,hit-test,editor-format,stars,star-store,star-format,keyboard,speed}.ts` | ✓ VERIFIED | All imported by production modules |
| `web/src/physics/puzzle-inspector.ts`, `puzzle-loader.ts` | ✓ VERIFIED | Used by `puzzle-editing.ts` and `puzzle-run.ts` |
| `web/src/player/{puzzle-run,puzzle-editing,puzzle-editor-input,puzzle-loop}.ts` | ✓ VERIFIED | Wired through `puzzle-lifecycle.ts` and `scene-surface.ts` |
| `web/src/render/puzzle-tools.ts` | ✓ VERIFIED | Drawn in particles → tools → glass order (`scene-surface.ts`) |
| `web/src/components/Puzzle{ToolTray,SelectionBar,Transport,StarList}.tsx`, `styles/puzzle-editor.css` | ✓ VERIFIED | Mounted from `PuzzlePanel.tsx` / `PuzzleResultCard.tsx` |
| `web/e2e/puzzle-editor.spec.ts`, `puzzle-touch.spec.ts`, `puzzle-editor-helpers.ts`; touch project in `playwright.config.ts` | ✓ VERIFIED | 29/29 pass |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| `lib.rs build_puzzle_core` | `validate.rs validate_for_build` | Same `placement_statuses` pass as the inspector | ✓ WIRED |
| `inspect.rs` | `validate.rs inspect_level` | wasm wrapper | ✓ WIRED |
| `hooks.rs on_after_step` | `PeakHeldTracker::observe` | `self.peak.observe(...)` | ✓ WIRED |
| `frame.rs set_puzzle_status` | `frame/puzzle.rs status_is_consistent` | fail-closed check | ✓ WIRED |
| `puzzle-editing.ts` | `puzzle-loader.ts inspectPlacementLanes` | commit and live inspections | ✓ WIRED |
| `puzzle-run.ts playPuzzle/stopPuzzle` | `buildPuzzleSessionSync` | fresh run or preview session | ✓ WIRED |
| `puzzle-loop.ts` | `speed.ts` | `speedStepRequest`, `restoreUnrunSpeedSteps` | ✓ WIRED |
| `scene-surface.ts` | `drawPuzzleTools`, `attachPuzzleEditorInput`, `applyPuzzleEdit` | overlay and editor pointer | ✓ WIRED |
| `PuzzlePanel.tsx onKeyDown` | `keyboard.ts puzzleKeyCommand` → controller | panel keydown | ✓ WIRED |
| `star-store.ts` | `catalog/puzzles.ts PUZZLE_IDS` | allowlist | ✓ WIRED |
| `web/scripts/puzzle-replay/fixture.ts` | Rust `replay/fixture.rs` | shared grammar; Bun replay passes | ✓ WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
| --- | --- | --- | --- | --- |
| Tool overlay and tint | outlines and statuses | Rust `inspectPuzzlePlacements` on every committed edit, live during drags | Yes (e2e status-line text comes from the Rust code) | ✓ FLOWING |
| Star rows and result card | `peakHeldCount`, `brimStep`, `decidedStep` | wasm frame getters → `frame-puzzle.ts` → reducer → `runStarsForState` | Yes (card text matches the fixture-derived expectation) | ✓ FLOWING |
| Best-of stars | `starStore` | `recordDecision` / `recordStop` | Yes (e2e across Edit, Stop and a loss) | ✓ FLOWING |
| Tray counts | `levelInfo` `toolKinds`/`toolCounts` | wasm `PuzzleLevelInfo` | Yes ("2 left" → "1 left" → "none left") | ✓ FLOWING |

### Behavioral Spot-Checks (all run by me at HEAD f71cc9970, 2026-10-13 02:45–02:51Z)

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Engine untouched | `git diff --stat 424735672 -- crates/liquidfun` | empty | ✓ PASS |
| Protected tests unedited | `git diff --quiet 424735672 -- web/tests/{hash,portrait-bounds,readme-svg}.test.ts web/e2e/hud-safe-frame.spec.ts` | exit 0 | ✓ PASS |
| Rust puzzle tests | `cargo test -p liquidfun-wasm --lib puzzle::` | 168 passed, 0 failed, 2 ignored (Phase 37 tuning probes, covered by the fixture replays) | ✓ PASS |
| Release replay | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` | 27 passed | ✓ PASS |
| Wasm replay | `just puzzle-replay-wasm` | solution won @201 final 689 peak 811 brim 262; empty lost @720 final 0 | ✓ PASS |
| fmt / clippy (wasm crate) | `cargo fmt --all --check`; `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit 0 / exit 0 | ✓ PASS |
| Typecheck | `cd web && bunx tsc --noEmit` | exit 0 | ✓ PASS |
| Vitest | `cd web && bun run test:unit` | 94 files, 1288 tests passed | ✓ PASS |
| Bright Builds | `bun scripts/bright-builds-check.ts all` | findings=0 (1476 files scanned) | ✓ PASS |
| Puzzle e2e (after wasm and app rebuild) | `just web-wasm && cd web && bun run build:app && CI=true bun run test:puzzle` | 29 passed | ✓ PASS |
| Player smoke | `just web-player-smoke` | 45 passed | ✓ PASS |
| Intermittent scene test | `playwright test e2e/player.spec.ts:207 --repeat-each=10` | 10/10 passed | ✓ PASS |
| Scene fingerprints | `just playground-scene-spot`, then diff with `target/phase39/base.txt` | 25 lines, identical | ✓ PASS |
| Touch drag (ad-hoc, scratchpad only) | CDP `Input.dispatchTouchEvent` drag in a 390×844 `hasTouch` project | `plank:10,64,0` → `plank:20,97,0`, no scroll; tap rotate and remove work | ✓ PASS |
| Grep gates | wall clock, innerHTML, storage, test seam, levels kept out of scenes, step cap, no TS geometry | all hold | ✓ PASS |
| Lifecycle | `gsd-tools verify lifecycle 39 --require-plans --raw` | `valid` | ✓ PASS |

Not re-run by me: full-workspace `cargo build/test --all-features`, whose wasm crate is not a default member, and the HUD safe-frame e2e. The Plan 14 gate and the post-fix `hud.log` (64/64) cover these. The review fixes touched no engine or scene-rendering code, and my 25/25 fingerprint run plus my player smoke run cover the scene side.

### Requirements Coverage

| Requirement | Source plans | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| TOOL-01 | 02, 06, 12, 13, 14 | Per-level tray with fixed-size tools and remaining counts | ✓ SATISFIED | SC1; tray e2e |
| TOOL-02 | 02, 07, 09, 11, 12, 13, 14 | Place, move, rotate 15° and remove by mouse or touch; tap-to-place; ≥44 px targets | ✓ SATISFIED | SC1; mouse and touch e2e; my touch-drag check |
| TOOL-03 | 01, 02, 06, 09, 11, 13, 14 | Invalid placements highlighted; Play disabled | ✓ SATISFIED | SC2 |
| TOOL-04 | 05, 07, 08, 09, 11, 12, 13, 14 | Play, Stop/Reset keeping placements, Clear | ✓ SATISFIED | SC3 |
| TOOL-06 | 03, 09, 12, 13, 14 | Keyboard place, nudge, rotate, remove, play | ✓ SATISFIED | "plays from the keyboard" and "removes … with Delete" e2e; keyboard Vitest |
| TOOL-07 | 03, 07, 10, 12, 13, 14 | 2×/4× without changing the outcome | ✓ SATISFIED | SC5 fast-forward e2e |
| STAR-01 | 03, 04, 05, 06, 07, 12, 13, 14 | Three criteria visible before Play | ✓ SATISFIED | SC4; star rows in building |
| STAR-02 | 03, 04, 05, 07, 08, 09, 12, 13, 14 | Stars on a win; best result per star kept | ✓ SATISFIED | SC4; best-of e2e |

Orphaned requirements: none. REQUIREMENTS.md maps exactly these 8 IDs to Phase 39. TOOL-05 is mapped to Phase 40 and is correctly out of scope.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| (phase-39 source files) | — | No TODO/FIXME/placeholder, no `innerHTML`, no wall-clock in puzzle logic, no `unwrap()` in Rust production code | — | none |
| `web/src/puzzle/hud-format.ts` vs `reduce-puzzle.ts` | — | Review W-IN-02: during an active drag, Play looks enabled while `canPlay` is false. Pressing it does nothing. | ℹ️ Info | Play is never accepted with an invalid or stale inspection. Only the look differs mid-drag. |
| `web/src/player/puzzle-editor-input.ts` | — | Review W-IN-06: an in-flight press survives a phase change (an edge case using Space twice mid-press) | ℹ️ Info | Can place a tool after a Play→Stop round trip during one press. It does not affect a run. |
| `web/src/components/PuzzlePanel.tsx` | — | Review W-IN-05: an inspector failure shows the build-failure copy | ℹ️ Info | Copy mismatch only |
| `web/playwright.config.ts:26`, `web/e2e/player-helpers.ts:160-169` | — | Pre-existing `tsc` errors outside the typecheck include | ℹ️ Info | Pre-existing (logged in 39-13/39-14) |

All 5 review warnings (R-WR-01, W-WR-01..04) are fixed, with regression tests, in commits `0b5472f26`, `a833050a4`, `3805f1fe2`, `eff2a486d`, `fa7b71fed` and `f3a895627`. I confirmed the tests are present and pass in Vitest and the Rust lib suite (`a_disallowed_kind_without_an_outline_reports_the_decoding_reason`, the failure-view block in `puzzle-run.test.ts`, the pointer-button and key-repeat tests). The 12 info findings stay open and are non-blocking.

### Intermittent player-smoke failure: assessment

In the first post-fix `just web-player-smoke` run (`target/phase39/post-fix/player-smoke.log`), 1 of 45 tests failed: `player.spec.ts:207`, where `resetNearZero` read `data-step-index` as null. Evidence since then:

- the fixer reran the suite twice (45/45 each) and ran the test alone 6/6
- I ran the full smoke suite once (45/45) and the test alone 10/10 on a freshly rebuilt app
- across all runs that is 1 failure in 21 executions of that test

The only review-fix change on the scene path is `releaseFailedPuzzle(session)` inside `failScene`. It cancels a pending puzzle inspection and clears `maybeOwned`. It runs only after a scene has already failed, so it cannot cause a scene to stall in loading. Note also that NOTES.md's appeal to "history in Phases 18 and 20" is weaker than stated. Those earlier fixes involved the same helper but a different symptom (the step comparison, not a null attribute). The trace from the failing run was overwritten by later runs, so the root cause is unproven. My judgment: this is a low-rate flake in the native-scene smoke test, not a Phase 39 regression, and it does not block the phase goal. If it recurs, keep the trace and error-context before rerunning.

### Human Verification Required

None required for the goal. Every success criterion was verified with automated or agent-run checks.

Optional, non-blocking: looking at the 48 px touch lift and the tool tints on a physical iPhone in Safari/WebKit. All automated touch coverage uses Chromium emulation. This is a matter of feel and polish. Per PROJECT-SCOPE.md, non-Chromium and real-device checks are best-effort.

### Gaps Summary

There are no gaps. The goal is achieved in the codebase:

- Rust is the single placement authority. The inspector and `newPuzzle` share one status pass.
- The TS editor, tray, selection bar, transport, overlay and keyboard map are wired end to end.
- Play, Stop, Clear and the D-10 decision timing work. Stars and the session best-of store record correctly.
- Fast-forward is outcome-invariant.
- All D-30 gates hold at HEAD after the review fixes:
  - 25/25 fingerprints
  - engine untouched
  - fixtures replay unchanged
  - 29/29 puzzle e2e and 45/45 player smoke
  - Vitest, tsc, Bright Builds, fmt and clippy all pass
- One coverage hole was found and closed by agent check: no e2e drags a placed tool by touch. Consider adding that touch-drag case to `puzzle-touch.spec.ts` as a regression guard.

---

_Verified: 2026-10-13T02:52:00Z_
_Verifier: Claude Opus 5.5 (gsd-verifier, AI independent review)_
