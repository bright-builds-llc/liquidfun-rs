---
phase: 37-puzzle-core-in-the-bridge
reviewed: 2026-10-12T01:03:07Z
depth: standard
files_reviewed: 39
files_reviewed_list:
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/frame/puzzle.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle/angles.rs
  - crates/liquidfun-wasm/src/puzzle/api_tests.rs
  - crates/liquidfun-wasm/src/puzzle/builder.rs
  - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
  - crates/liquidfun-wasm/src/puzzle/first_pour.rs
  - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-empty.txt
  - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt
  - crates/liquidfun-wasm/src/puzzle/goal.rs
  - crates/liquidfun-wasm/src/puzzle/goal/tests.rs
  - crates/liquidfun-wasm/src/puzzle/hooks.rs
  - crates/liquidfun-wasm/src/puzzle/info.rs
  - crates/liquidfun-wasm/src/puzzle/replay.rs
  - crates/liquidfun-wasm/src/puzzle/replay/tests.rs
  - crates/liquidfun-wasm/src/puzzle/session_tests.rs
  - crates/liquidfun-wasm/src/puzzle/spec.rs
  - crates/liquidfun-wasm/src/puzzle/tests.rs
  - crates/liquidfun-wasm/src/puzzle/tools.rs
  - crates/liquidfun-wasm/src/puzzle/tools/tests.rs
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot/frame_hash.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun-wasm/src/session/escape.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - justfile
  - scripts/puzzle-spike.sh
  - web/scripts/puzzle-replay.ts
  - web/scripts/puzzle-replay/fixture.ts
  - web/scripts/puzzle-replay/hash.ts
  - web/scripts/puzzle-replay/spike.ts
  - web/scripts/puzzle-replay/wasm.ts
  - web/src/catalog/puzzles.ts
  - web/src/physics/frame.ts
  - web/tests/frame-puzzle.test.ts
  - web/tests/puzzle-replay.test.ts
findings:
  critical: 0
  warning: 1
  info: 6
  total: 7
status: issues_found
---

# Phase 37: Code Review Report

**Reviewed:** 2026-10-12T01:03:07Z
**Depth:** standard
**Files Reviewed:** 39
**Status:** issues_found
**Reviewer:** Claude (gsd-code-reviewer), an independent AI review. This is not a human approval.
**Diff:** `git diff 958d6fad7..HEAD` (excluding `.planning/`)

## Summary

I reviewed the Phase 37 puzzle core: `puzzle/` (spec, angles, tools, builder, goal, hooks, info, replay), the `SessionSource` refactor in `session.rs` and `session/escape.rs`, the frame puzzle scalars in Rust and TS, the reference-solution replay harness in Rust and Bun, and the determinism spike tooling. I checked them against the locked decisions in 37-CONTEXT.md.

Overall the code is careful and matches the phase decisions. No critical issues were found.

**Areas checked:**

- **Fail-closed placement parsing (Rust):** The order is count cap, then lane-length check with `checked_mul`, then per-pose destructuring. After that come the kind allowlist, the angle (`u8::try_from` then `< 24`), the position (`i16::try_from`), the build area and the budget. Every failure is a typed `PlacementError` with a static message. The reserved 4th stride value is rejected.
- **Level id parsing:** It uses its own allowlist, and the cross-parsing tests cover both directions plus the TS mirror. No integer overflow or truncating `as` cast was found in the puzzle path (`checked_add` on the hook step counter and `saturating_add` on counts).
- **Determinism:**
  - The puzzle path reads no wall clock and iterates no `HashMap`/`HashSet`; counted groups are a `Vec`.
  - Static tool rotation uses the literal `TAU_24` table with body angle 0, so no `sin`/`cos` is called.
  - The grid-to-meter conversion is a correctly rounded division of an exact integer.
  - Placements are sorted canonically with a total `Ord`, and equal elements are identical, so `sort_unstable` is deterministic.
- **Goal and latch logic:**
  - `observe` latches once and checks the win before the loss.
  - A hold that completes exactly at `step_limit` wins.
  - A loss latches at `step_limit`.
  - Strict containment excludes edge particles, NaN particles and ungrouped particles.
  - The unit tests cover splash, dip reset, slosh after a win, and a hold after a loss.
- **Scene behavior after the `SessionSource` refactor:**
  - `EscapeLimits::for_scene` gives exactly the old `(12, 48)` or wave-tank `(80, 80)` limits.
  - `particle_iterations(id)` and `build_scene(id, &presets)` are unchanged on the scene arm.
  - The new `is_puzzle()` guards only short-circuit puzzle sources.
  - Scene hooks inherit `maybe_puzzle_status() -> None`, so scene frames carry no puzzle scalars.
  - `git diff --stat 958d6fad7..HEAD -- crates/liquidfun` is empty.

The one Warning is at the JS-to-wasm boundary: the JS glue coerces numbers before Rust's fail-closed checks see them. The Info items are parity gaps between Rust and TS helpers and some small robustness notes.

## Warnings

### WR-01: JS typed-array coercion bypasses the fail-closed placement checks

**File:** `crates/liquidfun-wasm/src/lib.rs:62-71` (with generated glue `web/src/generated/liquidfun-wasm/liquidfun_wasm.js:300-311, 660-672`, and `web/scripts/puzzle-replay/fixture.ts:254-270`)

**Issue:** `newPuzzle(levelId, kinds: &[u8], poses: &[i32])` validates the slices it receives. But the wasm-bindgen glue copies JS input with `Uint8Array.prototype.set` / `Uint32Array.prototype.set` (`passArray8ToWasm0`, `passArray32ToWasm0`), and constructing typed arrays coerces numbers the same way. Two examples:

- `Uint8Array.of(257)` becomes `1`, which is `Plank`.
- `Int32Array.from([8.9, 64, 23.7])` becomes `[8, 64, 23]`.

`NaN` and `Infinity` become `0`, and out-of-range values wrap modulo 2^8 or 2^32. So invalid JS placement values (a non-integer drag position, an unknown kind code, an angle like 24.5) silently turn into valid canonical placements instead of failing closed. Rust cannot detect this because the coercion happens before the copy.

No production caller exists yet: the Phase 38 editor will be the first. The current fixture path is safe because `parseI32` validates first. Still, the "fail-closed" contract of the boundary (D-08/D-09) is only as strong as the TS encoder, and none exists yet.

**Fix:** Add one shared TS encoder that every caller uses, and validate before building typed arrays. For example, in `web/src/physics/` (and reuse it in `fixtureLanes`):

```ts
export function encodePlacementLanes(
  placements: readonly { kind: number; x: number; y: number; angle: number }[],
): { kinds: Uint8Array; poses: Int32Array } {
  for (const { kind, x, y, angle } of placements) {
    const ok =
      Number.isInteger(kind) && kind >= 0 && kind <= 0xff &&
      Number.isInteger(x) && x >= -32768 && x <= 32767 &&
      Number.isInteger(y) && y >= -32768 && y <= 32767 &&
      Number.isInteger(angle) && angle >= 0 && angle < 24;
    if (!ok) throw new Error("Invalid puzzle placement");
  }
  return {
    kinds: Uint8Array.from(placements, (p) => p.kind),
    poses: Int32Array.from(placements.flatMap((p) => [p.x, p.y, p.angle])),
  };
}
```

Also document on `ProofSession::new_puzzle` that callers must pass already-validated integer typed arrays.

## Info

### IN-01: Rust and TS fixture parsers report different errors for some malformed expectation blocks

**File:** `crates/liquidfun-wasm/src/puzzle/replay.rs:106-125` vs `web/scripts/puzzle-replay/fixture.ts:204-224`

**Issue:** Both module docs say the two parsers produce "the same `fixture line N: ...` errors". They do not, because they check in a different order:

- Rust parses the `outcome_step` value as soon as it reads that row, before it looks for `final_count`.
- TS first checks all three field rows and the trailing-line check, and only then parses the numbers.

For example, `level first-pour\noutcome won\noutcome_step x\n` gives:

- Rust: `fixture line 3: \`x\` is not a non-negative integer`
- TS: `fixture line 4: missing \`final_count\` line`

Both still reject the fixture, so accept and reject behavior matches. Only the diagnostics differ.

**Fix:** Use the same order in both. For example, in TS, parse each value right after its `expectField` call, as Rust does. Optionally add a shared corpus of malformed fixtures whose exact error strings both test suites assert.

### IN-02: Rust frame validation accepts a `Lost` step that is not the step limit; TS rejects it

**File:** `crates/liquidfun-wasm/src/frame.rs:122-140` vs `web/src/physics/frame.ts:179-181`

**Issue:** `set_puzzle_status` checks only `step <= step_limit && step <= observed_step` for any latched outcome. `parseOutcomeStep` in TS also requires `outcome === "lost"` ⇒ `outcomeStep === stepLimit`. A tracker regression could therefore produce a frame that Rust accepts and the browser then rejects as "Invalid Rust/WASM frame". The error would surface far from its cause.

**Fix:** Add the same rule in Rust:

```rust
let lost_is_at_limit = !matches!(status.outcome, Outcome::Lost { step } if step != status.step_limit);
```

and include it in `steps_are_consistent`, with a test next to `set_puzzle_status_rejects_inconsistent_status`.

### IN-03: A mid-batch step failure leaves the hook counter ahead of `step_index` for good

**File:** `crates/liquidfun-wasm/src/session.rs:205-248`, `crates/liquidfun-wasm/src/puzzle/hooks.rs:63-76`

**Issue:** `advance(n)` commits `step_index` only after all `n` substeps succeed. `PuzzleHooks::on_after_step` advances `observed_step` on every successful substep. If substep `k > 1` fails, `observed_step` is `k - 1` ahead of `step_index`. From then on every `capture_frame` fails with `FrameCaptureFailed`, because of the `observed_step == step_index` check. This fails closed, which is correct. But the session cannot recover, and the error the caller sees later is a frame-capture error, not the original step failure. Scenes have the same uncommitted world-advance behavior; for puzzles it also skews the latched outcome step.

**Fix:** Either document this on `advance` (puzzle sessions must be rebuilt after any `advance` error, which already matches D-10), or commit `step_index` per substep so that the counter and the index stay equal even after a partial batch.

### IN-04: `count_in_goal` silently truncates misaligned lanes

**File:** `crates/liquidfun-wasm/src/puzzle/goal.rs:52-68`

**Issue:** The `zip` stops at the shorter of `positions` and `groups`. The engine view guarantees aligned lanes today, but if that invariant breaks, the count drops silently and the outcome changes, with no error. `capture_frame` already treats a lane mismatch as a hard error, so the puzzle path is less strict than the frame path.

**Fix:** Check `positions.len() == groups.len()` in `PuzzleHooks::on_after_step` (and in the builder's initial count) and return `SessionError::StepFailed` (or `SceneConstruction`) on a mismatch. Keep `count_in_goal` pure.

### IN-05: The spike comparison ignores scenes that only appear in the right-hand run, and the expected line count is hard-coded

**File:** `web/scripts/puzzle-replay/spike.ts:74-94`, `scripts/puzzle-spike.sh:9`

**Issue:** `compareSpikes` iterates only over `left`. A scene present only in `right` is never reported, so the comparison is not symmetric. Separately, `expected_lines=900` duplicates `3 scenes × SPIKE_STEPS = 300` from both `spike.ts` and `scene_spot/frame_hash.rs`, so changing the scene set or the step count needs three edits.

**Fix:** In `compareSpikes`, throw when `right` has a key that `left` lacks (or compare `left.size === right.size`). In the shell script, derive the expected count, or at least add a comment that names the two constants it mirrors.

### IN-06: The placement-count error message hard-codes the limit

**File:** `crates/liquidfun-wasm/src/puzzle/tools.rs:292`

**Issue:** `"Rust/WASM puzzle placement count exceeds 16"` repeats `MAX_PLACEMENTS`. The message must stay a `&'static str`, so if the constant changes, the message silently goes stale.

**Fix:** Drop the number (`"... placement count exceeds the limit"`), or add a unit test asserting that the message contains `MAX_PLACEMENTS.to_string()`.

---

_Reviewed: 2026-10-12T01:03:07Z_
_Reviewer: Claude (gsd-code-reviewer), independent AI review_
_Depth: standard_
