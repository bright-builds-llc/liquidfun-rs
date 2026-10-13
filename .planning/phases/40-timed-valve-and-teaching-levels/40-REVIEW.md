---
phase: 40-timed-valve-and-teaching-levels
reviewed: 2026-10-13T09:44:19Z
depth: standard
files_reviewed: 94
files_reviewed_list:
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle/api_tests.rs
  - crates/liquidfun-wasm/src/puzzle/builder.rs
  - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
  - crates/liquidfun-wasm/src/puzzle/first_pour.rs
  - crates/liquidfun-wasm/src/puzzle/float_up.rs
  - crates/liquidfun-wasm/src/puzzle/grand_finale.rs
  - crates/liquidfun-wasm/src/puzzle/hooks.rs
  - crates/liquidfun-wasm/src/puzzle/inspect.rs
  - crates/liquidfun-wasm/src/puzzle/jelly_pad.rs
  - crates/liquidfun-wasm/src/puzzle/levels_tests.rs
  - crates/liquidfun-wasm/src/puzzle/levels_tests/fixture_table.rs
  - crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
  - crates/liquidfun-wasm/src/puzzle/materials_tests.rs
  - crates/liquidfun-wasm/src/puzzle/probe.rs
  - crates/liquidfun-wasm/src/puzzle/replay.rs
  - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
  - crates/liquidfun-wasm/src/puzzle/replay/fixture/tests.rs
  - crates/liquidfun-wasm/src/puzzle/replay/tests.rs
  - crates/liquidfun-wasm/src/puzzle/sat.rs
  - crates/liquidfun-wasm/src/puzzle/session_tests.rs
  - crates/liquidfun-wasm/src/puzzle/spec.rs
  - crates/liquidfun-wasm/src/puzzle/thick_syrup.rs
  - crates/liquidfun-wasm/src/puzzle/timed_gate.rs
  - crates/liquidfun-wasm/src/puzzle/tools.rs
  - crates/liquidfun-wasm/src/puzzle/tools/tests.rs
  - crates/liquidfun-wasm/src/puzzle/validate.rs
  - crates/liquidfun-wasm/src/puzzle/validate/tests.rs
  - crates/liquidfun-wasm/src/puzzle/valve.rs
  - crates/liquidfun-wasm/src/puzzle/valve/tests.rs
  - crates/liquidfun-wasm/src/session.rs
  - web/e2e/puzzle-editor-helpers.ts
  - web/e2e/puzzle-editor.spec.ts
  - web/e2e/puzzle-helpers.ts
  - web/e2e/puzzle-hint.spec.ts
  - web/e2e/puzzle-levels.spec.ts
  - web/e2e/puzzle-valve.spec.ts
  - web/e2e/puzzle.spec.ts
  - web/package.json
  - web/scripts/puzzle-replay/fixture.ts
  - web/src/catalog/puzzle-bounds.ts
  - web/src/catalog/puzzle-levels.ts
  - web/src/catalog/puzzles.ts
  - web/src/components/PuzzleCaption.tsx
  - web/src/components/PuzzleHud.tsx
  - web/src/components/PuzzleIcons.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/PuzzleSelectionBar.tsx
  - web/src/components/PuzzleToolTray.tsx
  - web/src/player/first-hint-preference.ts
  - web/src/player/puzzle-editing.ts
  - web/src/player/puzzle-lifecycle.ts
  - web/src/player/scene-surface.ts
  - web/src/puzzle/caption.ts
  - web/src/puzzle/controller.ts
  - web/src/puzzle/debug-attributes.ts
  - web/src/puzzle/editor-announce.ts
  - web/src/puzzle/editor.ts
  - web/src/puzzle/hud-format.ts
  - web/src/puzzle/keyboard.ts
  - web/src/puzzle/placement-status.ts
  - web/src/puzzle/placements.ts
  - web/src/puzzle/reduce-puzzle.ts
  - web/src/puzzle/valve-format.ts
  - web/src/render/puzzle-tools.ts
  - web/src/styles/puzzle-editor.css
  - web/src/styles/puzzle.css
  - web/tests/placement-status-parity.test.ts
  - web/tests/playable-target.test.ts
  - web/tests/puzzle-bounds.test.ts
  - web/tests/puzzle-caption.test.ts
  - web/tests/puzzle-chrome.test.ts
  - web/tests/puzzle-debug-attributes.test.ts
  - web/tests/puzzle-editing.test.ts
  - web/tests/puzzle-editor-announce.test.ts
  - web/tests/puzzle-editor-format.test.ts
  - web/tests/puzzle-editor-input.test.ts
  - web/tests/puzzle-editor.test.ts
  - web/tests/puzzle-first-hint.test.ts
  - web/tests/puzzle-format-decision.test.ts
  - web/tests/puzzle-inspector.test.ts
  - web/tests/puzzle-keyboard.test.ts
  - web/tests/puzzle-level-info.test.ts
  - web/tests/puzzle-lifecycle.test.ts
  - web/tests/puzzle-placement-status.test.ts
  - web/tests/puzzle-placements.test.ts
  - web/tests/puzzle-replay.test.ts
  - web/tests/puzzle-run.test.ts
  - web/tests/puzzle-shell-wiring.test.ts
  - web/tests/puzzle-tools-render.test.ts
  - web/tests/puzzle-valve-format.test.ts
  - web/tests/reduce-puzzle-editor.test.ts
  - web/tests/reduce-puzzle.test.ts
findings:
  critical: 0
  warning: 1
  info: 6
  total: 7
status: issues_found
---

# Phase 40: Code Review Report

**Reviewed:** 2026-10-13T09:44:19Z
**Depth:** standard
**Files Reviewed:** 94
**Status:** issues_found

## Summary

Scope: the diff `65a37d35b..HEAD`. Production code got the most attention: the
Rust puzzle modules (`tools.rs`, `valve.rs`, `hooks.rs`, `builder.rs`,
`spec.rs`, `validate.rs`, `inspect.rs`, `replay/fixture.rs`), `session.rs`, the
fixture parser in `web/scripts`, and the web puzzle code (reducer, editor,
placements seam, valve format, keyboard, HUD models, caption and hint storage,
tool overlay rendering, CSS). The level-geometry constants and the tests were
skimmed.

Overall the work is good. The core contracts hold:

- **Valve lifecycle.** `on_advance` destroys a due gate before eviction and
  `world.step` in both `advance` and `advance_profiled`. It is keyed only to the
  hooks' own step counter. The TS overlay's `stepIndex >= openStep` "open"
  switch lines up with the Rust segment removal: the frame at `openStep - 1`
  still has the gate, and the frame at `openStep` does not.
- **Lane decoding fails closed.** Stride 4 is checked by `check_lanes`. A valve
  needs a tick in `1..=step_limit/30` and a plank needs 0. Anything else maps to
  `ValveTimeOutOfRange` (status 13), and the outline is still reported.
  `Placement: Ord` (kind, x, y, angle, open_tick) matches TS
  `comparePlacements`.
- **Fixture grammar parity.** Rust and TS check the kind before the arity, and
  both parse the open-tick token before x, y and angle. Both also give the same
  message for a bare `placement` line. `FIXTURE_FILES` and the Rust `FIXTURES`
  table list the same 14 files.
- **Storage.** `first-hint-preference.ts` reads and writes storage only inside
  `try`. Only the exact `"dismissed"` token counts, so tampered storage just
  shows the hint again.
- **Reducer.** `setValveTime` is ignored outside `building`. The shell's
  `stepValveTime` consumes the key only when a valve is selected.
- **HUD.** The caption leaf stays mounted with a fixed CSS box per layout, and
  only Dismiss mounts and unmounts inside it. The new leaves carry
  `data-hud-obstacle`, and a new e2e helper (`overlappingLeaves`) checks that
  leaves do not overlap each other.
- **Project rules.** No `unsafe`, no production `unwrap`, and no `HashMap` in
  solver-visible order. New items have docs.

The one warning is a keyboard-layout gap in the documented `[` / `]` valve
shortcut. The info items are latent divergences, a diagnostics hole on the
valve failure path, a stale lint allowance, and two small accessibility and
authoring gaps.

## Warnings

### WR-01: `[` / `]` valve shortcut does not work on layouts that type brackets with a modifier

**File:** `web/src/puzzle/keyboard.ts:44-46, 64-67` (help text at `web/src/components/PuzzlePanel.tsx:64`)
**Issue:** `puzzleKeyCommand` returns `undefined` whenever `ctrlKey`, `metaKey`
or `altKey` is set, and only then looks at `input.key`. Many layouts type
brackets with a modifier:

- macOS German: Option+5 / Option+6, which sets `altKey`.
- Windows German/French: AltGr+8 / AltGr+9. AltGr is reported as `ctrlKey && altKey`.

On those layouts `event.key` is `"["` or `"]"`, but the command is dropped.
Keyboard-only users there cannot use the shortcut that `KEYBOARD_HELP` promises.
They can still Tab to the stepper buttons. Arrow keys, R and Delete are not
affected, because those layouts type them without a modifier.
**Fix:** `event.key` already gives the produced character, so let the bracket
keys through when the modifier is part of producing it. Only Ctrl/Meta without
AltGraph would block them:

```ts
const isBracket = input.key === "[" || input.key === "]";
const altGraph = input.altGraph === true; // from event.getModifierState("AltGraph")
if (isBracket ? (input.metaKey || (input.ctrlKey && !altGraph)) : (input.ctrlKey || input.metaKey || input.altKey)) {
  return undefined;
}
```

Add a unit test in `web/tests/puzzle-keyboard.test.ts` for
`{ key: "[", altKey: true }` and for the AltGraph case.

## Info

### IN-01: Valve open failure clears the gate state before destruction succeeds and drops the error detail

**File:** `crates/liquidfun-wasm/src/puzzle/hooks.rs:84-96`, `crates/liquidfun-wasm/src/session.rs:243-244, 285-286`
**Issue:**

- `take_body()` clears `maybe_body` before `world.destroy_body(body)` runs. If
  destruction fails, the gate counts as open (`closed_segments()` is empty and
  `is_due` is false) while the body may still be in the world.
- The `HandleError` is thrown away (`|_error|`). `SessionCore::advance` does
  not set `last_failure_detail` for `on_advance` errors, unlike the
  eviction, step and `on_after_step` paths. So `ProofSession::advance` raises a
  bare "step failed" message.

The JS session is disposed after any `advance` error (`failWith`), so this is
diagnostics, not a live inconsistency.
**Fix:** Destroy first and clear on success. For example, add a
`ValveGate::maybe_body()` peek, call `destroy_body`, then `take_body()`. Also
record the engine error, either by returning a richer error from `on_advance`
or by setting `last_failure_detail` in `advance` when `on_advance` fails, as
`on_after_step` already does.

### IN-02: TS and Rust open-tick ceilings can diverge

**File:** `web/src/puzzle/valve-format.ts:13-15`, `web/src/puzzle/editor.ts:228-230`, `crates/liquidfun-wasm/src/puzzle/spec.rs:148-150`
**Issue:**

- Rust `max_open_tick` saturates to `u8::MAX` (255). TS `maxOpenTick` does
  not. Above a 7650-step limit, the stepper would offer ticks that Rust rejects
  with status 13.
- For a step limit under 30, TS `stepSelectedValveTime` computes
  `Math.min(0, Math.max(1, ...)) = 0` and writes an invalid tick 0.

No current level triggers either case: the step limits are 720 and 900.
**Fix:** Mirror the saturation (`Math.min(255, Math.floor(stepLimit / 30))`).
In `stepSelectedValveTime`, return the editor unchanged when `maxTick < 1`.
Alternatively, add a Rust test that every registered level has
`1 <= max_open_tick` and `step_limit < 7650`.

### IN-03: `LiquidMaterial` blanket `dead_code` allowance is stale and hides the unused `Powder` variant

**File:** `crates/liquidfun-wasm/src/puzzle/spec.rs:15`
**Issue:** `#[cfg_attr(not(test), allow(dead_code))] // The teaching levels (Phase 40) author the other materials.`
The Phase 40 levels now use `Viscous` and `Elastic`. Only `Powder` (and
`maybe_powder_strength`) is unused in production, but the allowance covers the
whole enum and the comment is out of date.
**Fix:** Drop the enum-level allowance. Put a narrow
`#[cfg_attr(not(test), allow(dead_code, reason = "..."))]` on `Powder` only,
with a current reason.

### IN-04: The float start area belonging to a `no_place` zone is a convention, not a check

**File:** `crates/liquidfun-wasm/src/puzzle/spec.rs:121-123`, `crates/liquidfun-wasm/src/puzzle/builder.rs:80-100`
**Issue:** The `floats` doc says a float's start area "should be an authored
`no_place` zone". `validated_trackers` checks only that the centre is finite and
that the radius and density are positive. A future level could leave a float
where a player-placed static tool overlaps its starting circle, and the dynamic
body would then be pushed out on the first step. Float Up does cover its well
correctly today (`NO_PLACE` x 0.4..1.4, y 2.0..3.25).
**Fix:** Add a level test that, for every registered spec, each float's
bounding square in grid cells is contained in the union of `spec.no_place`. Or
treat floats as implicit no-place zones in `placement_statuses`, the way
liquid blocks already are.

### IN-05: Screen-reader users cannot read the selected valve's current time without changing it

**File:** `web/src/components/PuzzleSelectionBar.tsx:72-78`, `web/src/puzzle/editor-announce.ts:113-128`
**Issue:**

- The readout `<span class="puzzle-valve-time">` is `aria-hidden="true"`.
- Selection and placement announcements ("Valve 1 of 1 selected.") do not
  include the open time.
- "Valve opens at X s" is announced only after a step changes the time.

So an assistive-technology user has to change the time to learn it, and at a
clamp, where the step is a no-op, nothing is announced.
**Fix:** Either include the time in the stepper group's label, e.g.
`aria-label={`Valve opening time ${valve().text}`}`, or append "opens at X s"
to the select and place announcements for valve tools.

### IN-06: A seam valve with no `openTick` defaults to the invalid tick 0

**File:** `web/src/puzzle/placements.ts:95-98`
**Issue:** `readSeamOpenTick` maps a missing `openTick` to 0 so that Phase 39
plank seams stay valid. A valve entry without `openTick` also becomes tick 0
and is then shown as status 13 ("Valve time out of range"). This fails closed,
but `DEFAULT_VALVE_OPEN_TICK` (4) is the more useful default.
**Fix:** Default by kind: `entry["openTick"] === undefined ? defaultOpenTick(kind) : readSeamNumber(entry, "openTick")`.
Validate `kind` first, so the order does not change. If the strict behaviour is
intended, document it.

---

_Reviewed: 2026-10-13T09:44:19Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
