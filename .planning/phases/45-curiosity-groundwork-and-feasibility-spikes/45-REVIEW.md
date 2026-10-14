---
phase: 45-curiosity-groundwork-and-feasibility-spikes
reviewed: 2026-10-14T00:00:00Z
depth: standard
files_reviewed: 29
files_reviewed_list:
  - tools/xtask/src/playground/fingerprint_gate.rs
  - tools/xtask/src/playground/fingerprint_gate/args.rs
  - tools/xtask/src/playground/fingerprint_gate/tests.rs
  - tools/xtask/src/playground/spot.rs
  - tools/xtask/src/playground.rs
  - tools/xtask/src/main.rs
  - justfile
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/puzzle/tests.rs
  - web/src/player/runtime.ts
  - web/src/components/PlaygroundShell.tsx
  - web/tests/scenes.test.ts
  - web/tests/demo-media-model.test.ts
  - web/e2e/shell.spec.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scene-view-bounds.ts
  - web/src/catalog/scene-records-classic.ts
  - web/src/catalog/scene-records-testbed.ts
  - web/src/catalog/scene-records-machines.ts
  - web/src/catalog/previews.tsx
  - web/src/catalog/preview-kit.tsx
  - web/src/catalog/previews-classic.tsx
  - web/src/catalog/previews-testbed.tsx
  - web/src/catalog/previews-machines.tsx
  - docs/adding-a-scene.md
  - docs/benchmarks/scene-fingerprints.json
  - AGENTS.md
findings:
  critical: 0
  warning: 1
  info: 5
  total: 6
status: issues_found
---

# Phase 45: Code Review Report

**Reviewed:** 2026-10-14
**Depth:** standard
**Files Reviewed:** 29
**Status:** issues_found

## Summary

The review covered the new `scene-fingerprint-gate` xtask, its flag parser and tests, and the `spot::capture` refactor. It also covered the `ALL_SCENE_IDS` derivation and its consumers in `liquidfun-wasm`, the web tests that now take their counts from the catalog, and the catalog record and preview split. The catalog split was spot-checked against `git diff 04650ff5a HEAD`. The family modules import only leaf modules (`scene-record-shared`, `scene-view-bounds`, control modules) plus `import type` from `./scenes`, so the split adds no runtime import cycle. The new `never` guard in `ScenePreview` makes a missing preview case a compile error, which matches what `docs/adding-a-scene.md` says.

The gate's join and compare logic is correct:
- Duplicate ids are rejected on both sides.
- Missing and changed ids fail the gate, and added ids pass.
- Report order is deterministic.
- `--stamp` goes through `stamp::parse_stamp_name`, which blocks path traversal.
- No production `unwrap()` was added, and errors propagate as typed `PlaygroundError`s.

Running `cargo xtask playground scene-fingerprint-gate --stamp 2026-10-14T16-57-32Z` reported PASS, with 25/25 unchanged.

The main gap is that the gate does not check the capture parameters that determine the fingerprints.

## Warnings

### WR-01: The gate ignores the warmup and measured step counts that determine the fingerprints

**File:** `tools/xtask/src/playground/fingerprint_gate.rs:27`, `:52-57`
**Issue:** The end-state fingerprint is taken "after warmup plus measured steps of the first run" (`crates/liquidfun-wasm/src/scene_spot.rs:230`). It therefore depends on `warmup_steps` and `measured_steps`. The baseline records both values in `provenance` (`60` and `120`), and every `scene-spot.json` records them at the top level. The gate never compares them. `PROVENANCE_FIELDS` covers only `os`, `arch` and `compiler`, and those produce warnings only.

Suppose someone captures with `cargo xtask playground scene-spot --steps 10` and passes the result through `--spot` or `--stamp`. The gate then reports every protected scene as "changed" and blames the code, when the real cause is a mismatched capture. The same false failure would hit every fresh-mode run if the `scene-spot` default step counts ever changed. The gate does fail closed, but its diagnosis is wrong, and that weakens a gate whose job is to attribute fingerprint changes to code changes.

**Fix:** Treat the step counts as hard preconditions, separate from the soft host and toolchain warnings. Check them before calling `compare_fingerprints`:
```rust
/// Capture parameters that determine the end-state fingerprint; a mismatch makes the comparison meaningless.
const DETERMINING_FIELDS: [&str; 2] = ["warmup_steps", "measured_steps"];

fn ensure_same_capture_parameters(
    baseline: &serde_json::Value,
    observed: &serde_json::Value,
) -> Result<(), PlaygroundError> {
    for field in DETERMINING_FIELDS {
        let maybe_baseline = baseline.pointer(&format!("/provenance/{field}")).and_then(serde_json::Value::as_u64);
        let maybe_observed = observed.get(field).and_then(serde_json::Value::as_u64);
        if maybe_baseline.is_none() || maybe_baseline != maybe_observed {
            return Err(gate_error(format!(
                "observed {field} {maybe_observed:?} differs from baseline {maybe_baseline:?}; recapture with the baseline step counts"
            )));
        }
    }
    Ok(())
}
```
Add a unit test that gives the observed capture a different `measured_steps` and asserts the specific error.

## Info

### IN-01: The `just` recipe does not quote the forwarded arguments

**File:** `justfile:184-185`
**Issue:** `cargo xtask playground scene-fingerprint-gate {{args}}` splits `--spot` and `--baseline` paths that contain spaces.
**Fix:** Add `set positional-arguments` and forward with `"$@"`, or document that paths must not contain spaces. Other recipes in the file use the same pattern, so this is optional.

### IN-02: The `seen` coverage check in the gravity slider test can no longer catch a variant missing from `ALL_SCENE_IDS`

**File:** `crates/liquidfun-wasm/src/scene/gravity_slider.rs:119-147`
**Issue:** The `seen` array now has length `ALL_SCENE_IDS.len()` and is filled by iterating `ALL_SCENE_IDS`. Suppose a `SceneId` variant gets a `variant_index` arm, which the exhaustive match requires, but is left out of `ALL_SCENE_IDS`. The array stays all `true` and the assertion still passes. The guarantee that every variant is covered now rests only on `all_scene_ids_match_the_web_catalog_order`, which covers catalog ids but not a variant that is not in the catalog. The assertion message "every SceneId variant must be built" overstates what is checked. In practice this is a small gap, because every variant is currently in the catalog.
**Fix:** Size `seen` from the largest `variant_index` value plus 1 (for example `const VARIANT_COUNT: usize = 25;` next to `variant_index`), or reword the message to "every ALL_SCENE_IDS entry".

### IN-03: Four copies of the `SCENE_IDS` bracket parser

**File:** `crates/liquidfun-wasm/src/scene.rs:326-337`
**Issue:** The new test helper is the fourth copy of the `export const SCENE_IDS = [` split. The others are in `crates/liquidfun-wasm/src/puzzle/tests.rs:7`, `crates/liquidfun-wasm/src/scene_spot/tests.rs:13` and `tools/xtask/src/playground/spot/survey_table.rs:7`. If the `scenes.ts` formatting changes (for example single quotes or a trailing comment), every copy must be updated to match.
**Fix:** Move the crate-local copies into one `#[cfg(test)]` helper, for example `crate::scene::test_support::catalog_scene_ids`, and reuse it from `puzzle/tests.rs` and `scene_spot/tests.rs`.

### IN-04: Several gate tests assert only `is_err()`

**File:** `tools/xtask/src/playground/fingerprint_gate/tests.rs:110-231`; `tools/xtask/src/playground/fingerprint_gate/args.rs:129-138`
**Issue:** These tests do not check which error was returned: the duplicate-id tests, the empty-baseline tests, the malformed-fingerprint tests, the wrong-kind tests, the schema-version test and `traversal_stamp_is_rejected`. A future regression could fail one of these inputs for an unrelated reason, such as a broken fixture helper, and the test would still pass.
**Fix:** Compare against the exact `PlaygroundError`, or at least assert on the message substring (for example `"repeats scene id `a`"` or `"must match YYYY-MM-DDTHH-MM-SSZ"`). The parser tests in `args.rs` already do this.

### IN-05: The failure message for a partial capture does not name the likely cause

**File:** `tools/xtask/src/playground/fingerprint_gate.rs:76-83`
**Issue:** A `--spot` or `--stamp` capture made with `--scene` filters (`full_catalog: false`) fails with "N missing". The report shows `full_catalog false` in the observed label, but the error line does not mention it.
**Fix:** When `observed_json["full_catalog"] == false` and `report.missing` is non-empty, add a hint such as "observed capture is not full-catalog; rerun `just playground-scene-spot` without --scene".

---

_Reviewed: 2026-10-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
