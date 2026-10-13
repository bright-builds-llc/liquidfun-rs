---
phase: 40-timed-valve-and-teaching-levels
verified: "2026-10-13T09:52:13.210Z"
status: passed
score: 5/5 roadmap success criteria verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T09:52:13.210Z"
lifecycle_validated: true
overrides_applied: 0
---

# Phase 40: Timed Valve and Teaching Levels Verification Report

**Phase Goal:** Players can work through 5–8 teaching levels, each introducing one idea, and every level is proven solvable by automated tests.
**Verified:** 2026-10-13T09:52:13Z at HEAD `4fb3f35ff`, which includes the review fix `686dbd808`
**Status:** passed
**Re-verification:** No. This is the initial verification.
**Verifier:** Claude (gsd-verifier), an AI verifier. This is not human approval.

Lifecycle check: `gsd-tools verify lifecycle 40 --require-plans --raw` returned `valid`. All 22 PLAN and SUMMARY files have `lifecycle_mode: yolo` and `phase_lifecycle_id: 40-2026-10-13T03-11-05`, which match 40-CONTEXT.md. No artifact is marked `direct-fallback`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| - | ----- | ------ | -------- |
| 1 | The player can place a timed valve and set its opening time while building. The valve opens at that simulation time with no input during the run, and the same opening time always gives the same outcome. | ✓ VERIFIED | **Rust:** `tools.rs` defines `ToolKind::Valve = 2` and `POSE_STRIDE = 4`. The validator returns `ValveTimeOutOfRange` (status 13) for a valve tick outside `1..=step_limit/30` or a non-zero tick on another kind. In `valve.rs`, `ValveGate::is_due(next_step)` uses `open_step = open_tick × 30`. `PuzzleHooks::on_advance` (hooks.rs:76-97) destroys the gate body with `World::destroy_body` before `world.step`, using only its own step counter. **Tests:** `on_advance_destroys_the_gate_before_step_one_hundred_twenty_runs`, `the_frame_at_step_119_has_the_gate_and_the_frame_at_step_120_does_not`, `identical_valve_placements_give_identical_frames` and `timed_gate_is_reproducible` (equal frame hashes and outcomes). The `timed_gate_open_time_decides_the_outcome` sweep shows ticks 1–2 lose, 3–21 win and 22–24 lose. **TS:** `editor.ts` defaults a new valve to tick 4 and clamps the time step. `reduce-puzzle.ts` has a `setValveTime` event that applies only while building. The `[` and `]` keys (keyboard.ts) also accept AltGr and Option after WR-01. The stepper is always mounted. `valve-format.ts` provides the label and countdown. **Browser:** `e2e/puzzle-valve.spec.ts` builds the timed-gate reference with the tray, canvas and stepper. It reaches `outcome-step` 429 and `peak-held` 582, the same as native. The `valve-running.png` screenshot shows the "2.6 s" countdown on the gate. |
| 2 | The player can play 5–8 levels. Each has a one-line caption that introduces one idea from the listed set. In the jelly pad level, the pad deflects water and its particles never count toward the fill. | ✓ VERIFIED | The 6 levels are `LEVEL_IDS` = `PUZZLE_IDS` = first-pour, thick-syrup, jelly-pad, float-up, timed-gate and grand-finale. They cover plain water, viscous liquid, jelly pad, buoyancy, timed valve and a finale. `level_count_is_within_the_teaching_range` asserts 5–8 levels, the required ids and that the finale comes last. Powder was dropped under D-25: the 40-08 spike found no behavior that is distinct and changes the outcome, and grep shows no `sand-chute` traces. `PUZZLE_CAPTIONS` holds one sentence per level. `puzzle-levels.spec.ts` opens every level at 1280x800 and 390x844 and checks its title and caption. **Jelly:** the pad block has `counted: false` and `LiquidMaterial::Elastic`. `jelly_inside_the_cup_never_counts_at_step_zero` and `..._after_stepping` put more than 100 pad particles in the cup and get `goal_count == 0`. `jelly_pad_needs_the_pad` shows that the reference without the pad loses and that the pad drifts less than 0.3 m. `grand_finale_needs_the_pad` gives the same proof for the finale. |
| 3 | For every level, tests show that a three-star reference solution wins with margin, a run with no tools loses, and small grid nudges of the solution mostly still win. At least one solution replays to the same result in the browser as natively. | ✓ VERIFIED | `levels_tests.rs` loops over every level in `LEVEL_IDS`. It checks that the solution and empty fixtures exist, and that each solution is Won with a Brim step, uses at most `par_tools` placements and wins at least 60 steps before the limit. It checks that each empty fixture has no placements and is Lost. Every fixture is replayed in each lane, and `replay.rs` `check_won_margin`, `check_brim_margin` and `check_lost_margin` enforce the D-03 fill margin. The `<level>_nudges_mostly_win` tests sweep ±1 and ±2 cells, plus ±1 tick for valves, and require at least 70% of valid variants to win, with at least 4 valid variants. They run only in release, for all 6 levels. Idea-necessity controls cover thick-syrup, jelly-pad, float-up, timed-gate and grand-finale. **I re-ran** `cargo test -p liquidfun-wasm --release --lib puzzle::`: 257 passed, 0 failed, 9 ignored. **I re-ran** `just puzzle-replay-wasm`: 14 of 14 fixtures `pass` under the exact contract. **Browser:** the Playwright valve replay matches native (row 1). |
| 4 | The first level shows a placement hint that the player can dismiss. It does not return after dismissal. | ✓ VERIFIED | `caption.ts` `captionModel` shows `FIRST_LEVEL_HINT` on `FIRST_PUZZLE_ID` until it is dismissed. `PuzzleCaption.tsx` is one fixed `data-hud-obstacle` leaf with a "Got it" button (aria "Dismiss hint"). `first-hint-preference.ts` uses the versioned key `liquidfun.puzzle-first-hint.v1`. Every storage access is wrapped in try/catch, and an in-memory fallback covers blocked storage. In `PuzzlePanel.tsx` (lines 130-149), both Dismiss and the first placed tool dismiss the hint. `puzzle-hint.spec.ts` covers dismissal at 3 viewports with no camera refit, a 44 px or larger target and no overlaps. It also checks that the hint stays dismissed after a reload and that placing the first tool dismisses it. `puzzle-first-hint.test.ts` covers a throwing getter, a throwing setItem and an already-dismissed state. |
| 5 | Each level has a portrait frame that fills a phone canvas (about 390×844) and stays within its particle budget. The 25/25 scene fingerprints are unchanged. | ✓ VERIFIED (first-pour caveat accepted, see below) | `puzzle-bounds.ts` has a landscape and a portrait rectangle for each `PuzzleId`. `puzzle-bounds.test.ts` requires every level to fill more than 0.85 of the width and more than 0.33 of the height. It requires every level except first-pour to fill more than 0.5 of the height. It checks that both frames contain the cup, build area, walls, spawn and float travel. **Budget:** `step_zero_particles_fit_the_budget` checks at most 3,000 particles and at most each level's cap. The step-0 counts run from 802 to 2,394. **Fingerprints:** `target/phase40/after-final.txt` matches `base.txt` exactly; I confirmed this with `diff`, and the file has 25 lines. Since plan 11 (`282dd1631..HEAD`), only `PuzzlePanel.tsx`, `keyboard.ts` and a keyboard test changed. No Rust changed, so the fingerprint evidence still holds. `git diff 65a37d35b -- crates/liquidfun` is empty. |

**Score:** 5/5 truths verified

#### SC5 judgement: first-pour portrait fill (about 34% of phone height)

I accept this caveat for SC5:

- **Width is full.** first-pour's portrait frame fills the full phone width, and the shared Phase 38 D-21 rule (more than 0.85 of the width and more than 0.33 of the height) passes.
- **Height cannot grow.** The level is 12 m wide, and the repository framing rule says to keep every wall inside the frame. It also allows a wide scene to "still have some vertical margin". With the walls kept, the height fill cannot get larger.
- **Changing it would break earlier decisions.** D-09 and D-07 lock first-pour's geometry and its fixture outcomes. Re-authoring the level to fill more height would change its fixtures and stars.
- **The owner already approved it.** The repository owner approved this exact frame at the Phase 38 human checkpoint (38-VERIFICATION `human_approved_at` 2026-10-12T15:20:04Z).
- **The screen looks right.** In `level-first-pour-390x844.png`, the HUD (caption, hint, star list and tray) takes the rest of the screen, and the subject is easy to recognize.

So this is a recorded limitation, not a gap. If the owner wants a taller first-level frame, that needs a follow-up that re-authors first-pour and re-records its fixtures.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/liquidfun-wasm/src/puzzle/valve.rs` | Valve scheduling and gate record | ✓ VERIFIED | 87 lines. Used by hooks.rs and builder. |
| `crates/liquidfun-wasm/src/puzzle/hooks.rs` | Pre-step valve opening | ✓ VERIFIED | `on_advance` destroys due gates. `collect_segments` drops opened gates. |
| `crates/liquidfun-wasm/src/puzzle/{thick_syrup,jelly_pad,float_up,timed_gate,grand_finale}.rs` | One const `LevelSpec` per level | ✓ VERIFIED | Registered in `LevelId`, `LEVEL_IDS` and `level_spec`. |
| `crates/liquidfun-wasm/src/puzzle/fixtures/*.txt` | Solution and empty fixtures per level | ✓ VERIFIED | 14 files: 6 pairs plus the timed-gate early and late fixtures. |
| `crates/liquidfun-wasm/src/puzzle/levels_tests.rs` and `levels_tests/level_proofs.rs` | Table-driven proofs, nudge sweeps and window sweeps | ✓ VERIFIED | Both are in the lib test binary (Phase 37 D-20). |
| `crates/liquidfun-wasm/src/puzzle/materials_tests.rs` | Jelly never counts | ✓ VERIFIED | Covers step 0 and after stepping. |
| `web/src/catalog/puzzles.ts` and `puzzle-levels.ts` | `PUZZLE_IDS`, titles, captions and hint | ✓ VERIFIED | Mirror the 6 Rust ids. |
| `web/src/catalog/puzzle-bounds.ts` | Frames for every level | ✓ VERIFIED | Tested by puzzle-bounds.test.ts. |
| `web/src/player/first-hint-preference.ts`, `puzzle/caption.ts` and `components/PuzzleCaption.tsx` | Hint persistence and caption leaf | ✓ VERIFIED | Wired in PuzzlePanel. |
| `web/src/puzzle/valve-format.ts`, `editor.ts`, `reduce-puzzle.ts`, `keyboard.ts` and `components/PuzzleSelectionBar.tsx` | Valve time UI | ✓ VERIFIED | Exercised by the valve e2e. |
| `web/e2e/puzzle-valve.spec.ts`, `puzzle-hint.spec.ts` and `puzzle-levels.spec.ts` | Browser replay, hint and every-level route checks | ✓ VERIFIED | 57/57 puzzle e2e tests pass on HEAD (re-run). |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `SessionCore::advance` | `PuzzleHooks::on_advance` | Pre-step hook | ✓ WIRED | Opens the valve before `world.step`. Scenes keep their own `on_advance` unchanged, and the fingerprints are identical. |
| TS editor `openTick` | Rust pose lane | `placements.ts`, stride 4 | ✓ WIRED | The browser replay reaches the native outcome step. |
| `LEVEL_IDS` (Rust) | `PUZZLE_IDS` (TS) | include_str allowlist test | ✓ WIRED | 6 ids in the same order. |
| `PuzzlePanel` | `firstHintMemory` / `captionModel` | Signal and effect | ✓ WIRED | Dismiss and the first tool placement both persist. |
| Fixtures | wasm `newPuzzle` | `just puzzle-replay-wasm` | ✓ WIRED | 14/14 exact on re-run. |
| Result card Next | following level | `maybeNextPuzzleId` | ✓ WIRED | e2e checks the Next link per level and "No next level yet" on the finale. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Valve label and countdown | `openTick`, frame `stepIndex` | Editor state plus the Rust frame step | Yes. The screenshot shows a live "2.6 s" countdown. | ✓ FLOWING |
| Fill meter and outcome per level | `goalCount`, `outcome` | Rust `PuzzleHooks::on_after_step` | Yes. The e2e outcome and peak-held attributes equal the fixtures. | ✓ FLOWING |
| Caption leaf | `captionModel(levelId, hintDismissed)` | TS metadata plus storage | Yes. The e2e checks the text per level. | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| All levels' proofs, nudge sweeps and window sweeps | `cargo test -p liquidfun-wasm --release --lib puzzle::` | 257 passed, 0 failed, 9 ignored (9.1 s) | ✓ PASS |
| Wasm replay of every fixture | `just puzzle-replay-wasm` | 14/14 `REPLAY … pass` | ✓ PASS |
| Web type check and unit tests | `bunx tsc --noEmit`; `bun run test:unit` | Clean; 97 files, 1,378 tests passed | ✓ PASS |
| Puzzle e2e on HEAD, after the WR-01 fix | `bun run build:app && CI=true bun run test:puzzle` | 57 passed | ✓ PASS |
| Scene fingerprints | `diff target/phase40/base.txt target/phase40/after-final.txt` | Identical, 25 lines. Not re-run: no Rust changed after plan 11. | ✓ PASS (recorded) |
| Engine untouched | `git diff --stat 65a37d35b -- crates/liquidfun` | Empty | ✓ PASS |

The plan-11 gate log `target/phase40/checks-11.results` records 27/27 gates at `exit=0`, including fmt, clippy, the build, the workspace tests and hud-safe-frame. The worktree is clean after my checks, apart from the orchestrator's `.planning/config.json`.

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
| ----------- | ------------ | ----------- | ------ | -------- |
| TOOL-05 | 40-01, 02, 04, 07, 11 | Place a timed valve and set its opening time while building | ✓ SATISFIED | SC1 |
| LEVEL-01 | 40-03, 06, 07, 08, 09, 10, 11 | 5–8 teaching levels with one-line captions, drawn from the listed ideas | ✓ SATISFIED | SC2. The 6 levels cover 6 of the 7 listed ideas, and powder was dropped under D-25. |
| LEVEL-02 | 40-03, 06, 07, 09, 10, 11 | Every level proven solvable: three-star win with margin, empty run loses, nudges mostly win | ✓ SATISFIED | SC3 |
| LEVEL-03 | 40-05, 11 | One-time, dismissible first-level hint | ✓ SATISFIED | SC4 |
| LEVEL-04 | 40-06, 07, 09, 10, 11 | Portrait frame per level that fills a phone, within the particle budget | ✓ SATISFIED (first-pour height caveat) | SC5 |

No requirements are orphaned. REQUIREMENTS.md maps exactly TOOL-05 and LEVEL-01 to LEVEL-04 to Phase 40, and every ID appears in a plan's `requirements`. The traceability table still shows LEVEL-01 to LEVEL-04 as "Pending". The orchestrator should mark them Complete when it closes the phase.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `crates/liquidfun-wasm/src/puzzle/hooks.rs` | 85-96 | `take_body()` runs before `destroy_body`, and the error detail is dropped (review IN-01) | ℹ️ Info | Diagnostics only. The JS session is disposed on any advance error. |
| `web/src/puzzle/valve-format.ts` | 13-15 | The TS max tick is not saturated at 255 like Rust (IN-02) | ℹ️ Info | No current level is affected: the step limits are 720 and 900. |
| `crates/liquidfun-wasm/src/puzzle/spec.rs` | 15 | A stale enum-wide `dead_code` allowance hides the unused `Powder` variant (IN-03) | ℹ️ Info | Hygiene |
| `web/src/components/PuzzleSelectionBar.tsx` | 72-78 | The valve time is `aria-hidden`, so screen readers cannot read it without changing it (IN-05) | ℹ️ Info | Accessibility follow-up |
| `web/src/puzzle/placements.ts` | 95-98 | A seam valve with no `openTick` reads as tick 0 and fails closed (IN-06) | ℹ️ Info | Test seam only |
| Tool tray at 390x844 | — | The grand-finale chips truncate to "Pl… ×2" and "Va… ×1" (`deferred-items.md`) | ℹ️ Info | Icons, counts and accessible names stay. This is cosmetic. |

There are no TODO, FIXME or placeholder stubs in the phase files, and no blockers. The one review warning (WR-01) was fixed in `686dbd808`, and the re-run e2e covers that fix.

### Human Verification Required

None are required for this verification. I reviewed the captured screenshots: `level-first-pour-390x844.png`, `level-grand-finale-390x844.png` and `valve-running.png`. The valve label and countdown, the caption and hint leaf, and the portrait layouts render as intended. The first-pour frame matches the frame the owner already approved in Phase 38.

### Gaps Summary

There are no gaps. The phase ships a timed valve tool, wired end-to-end in Rust and TS, and six teaching levels within the 5–8 range. Powder was dropped with recorded spike evidence under D-25, and buoyancy shipped. Each level has fixture-backed proofs in every lane: margin checks on solution and empty runs, nudge sweeps in release, and idea-necessity controls. The fixtures replay exactly through wasm, and the browser replays the timed-gate reference to the native outcome. The first-level hint persists through blocked storage. Every level has a portrait frame and fits the particle budget, and the 25 scene fingerprints are unchanged. The remaining items are advisory review notes and a cosmetic tray truncation.

---

_Verified: 2026-10-13T09:52:13Z_
_Verifier: Claude (gsd-verifier)_
