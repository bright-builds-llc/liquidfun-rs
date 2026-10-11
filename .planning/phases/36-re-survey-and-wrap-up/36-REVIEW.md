---
phase: 36-re-survey-and-wrap-up
reviewed: 2026-10-11T03:38:15Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - tools/xtask/src/playground/spot.rs
  - tools/xtask/tests/playground_cli/spot.rs
  - docs/benchmarks/scene-survey.md
  - docs/playground-scene-spot-check.md
  - README.md
  - BENCHMARKING.md
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 36: Code Review Report

**Reviewed:** 2026-10-11T03:38:15Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Scope: `git diff 9d8718bc1 HEAD` for the six files listed above. I checked the before/after provenance against the raw evidence in `target/phase35/v14-*`.

**WR-01 fix (35-REVIEW): correct.**
- `disclaimer_for` returns `FULL_DISCLAIMER` for an empty filter and `FILTERED_DISCLAIMER` otherwise. The same value goes to both the JSON and the stdout summary, so they cannot disagree.
- `scene_filter` and `full_catalog` are both derived from `spot_args.scenes`.
- The module doc no longer says "all-catalog".
- Full-catalog stdout is unchanged: same disclaimer text, same lines. The JSON only gains the two new fields, `scene_filter: []` and `full_catalog: true`. Nothing in the repository reads `scene-spot.json` with a strict schema, so this addition is safe.

**Tests: good.**
- The CLI tests pin both branches end to end. They check the JSON fields, the exact full disclaimer, that the filtered disclaimer appears on stdout, and that the full-catalog wording is absent from a filtered stdout.
- The unit tests pin `FULL_DISCLAIMER` against a literal copy, which guards against wording drift.
- `cargo test -p xtask --test playground_cli scene_spot` passed 4/4, and `cargo test -p xtask --bin xtask disclaimer_for` passed 2/2.
- `just markdown-check` exits 0.

**Provenance: matches the evidence.** I recomputed both tables from `v14-{b1,a1,a2,b2}.jsonl` and `v14-rank-b1.jsonl` with an independent script. Every published value matches:
- every median, change %, min-max and class;
- the 24/1/0 counts and the rank order;
- 25/25 fingerprints against `before-full.jsonl` (SHA-256 `be36d215…`) in all six legs.

The rest of the provenance also checks out:
- **Binaries:** the SHA-256 values (`5bfc7b61…`, `a7d837b1…`) match the files on disk. `spot-v14-final` is byte-identical to `target/release/playground-scene-spot` and to `bin/spot-A2r`.
- **Runs:** each leg's `runs`, `warmup_steps` and `measured_steps` match the published flags.
- **Gate logs:** the UTC spans, 1-minute load ranges (5.31–6.42 and 5.63–6.37) and `GATE-WAITED` counts match `v14.uptime` and `v14-rank.uptime`. The voided rank attempt and its reason match `v14-rank.void-1791689300/`.
- **Pre-registration:** the protocol was written at 03:21:56Z, before the first leg started at 03:22:04Z.
- **Scene settings:** the "Authored scene settings did not change" claim holds. `git diff 809582431 edcafebc0 -- web/src/catalog crates/liquidfun-wasm/src` is empty.
- **Kept changes:** the list of seven kept changes matches `35-PROFILES.md`.

**Local, honest claims: mostly yes.**
- The README and BENCHMARKING additions add only a link with the "local observations, not public speed claims" framing. They cite no numbers and make no FPS or universal claims.
- The survey page keeps the framing, says no FPS promise is made, and warns against comparing numbers across sections.
- The one problem is the rewritten PERF-CANARY2 paragraph in `docs/playground-scene-spot-check.md` (WR-01 below). It widens a five-scene 2026-09-21 observation into a present-tense claim about every playground scene, and Phase 35's own profiles contradict that claim.

## Warnings

### WR-01: PERF-CANARY2 now claims all playground scenes share the particle-contact hot path, which Phase 35 evidence contradicts

**File:** `docs/playground-scene-spot-check.md:26-30`

**Issue:**

The old text was a dated observation about five scenes: "Dam Break and these five spot-checks share the same dominant particle-contact cluster… No second native profile stamp was captured." Phase 36 rewrote it in the present tense as "Dam Break and the playground scenes share the same dominant particle-contact cluster." It also says Fountain and Water Wheel emission "is still particle-contact work."

Phase 35 shows otherwise:
- `35-PROFILES.md` lines 28-29: A8, which skips the group-record rebuild on ungrouped creation, and A9, a cheaper lifetime eviction index, each cut about 20% and 27% from fountain and water-wheel. That is particle-creation and lifetime bookkeeping, not contact work.
- `35-PROFILES.md` line 13: the profiled hot path for tesla-valve is the AABB tag query plus emission and lifetime resequencing.
- `35-PROFILES.md` line 14: for washing-machine it is fixture CCD.
- Scenes such as theo-jansen (141 particles) and drawing-particles (24 particles) were never profiled as particle-contact bound.

The sentence "No second native profile stamp was captured" has also lost its date context. It now reads as if no playground scene was ever profiled, but Phase 35 profiled the slowest scenes.

The project's transparency rule requires every claim to match verified evidence, and this paragraph does not.

**Fix:** Restore the claim's scope and date, and point to the newer evidence. For example:

```markdown
## PERF-CANARY2

On 2026-09-21, Dam Break and the five scenes sampled then shared the same
dominant particle-contact cluster, and no second native profile stamp was
captured at that time. Phase 35 later profiled the slowest catalog scenes and
found other hot paths as well (fixture CCD, emission, group-record rebuild and
lifetime eviction); see
[35-PROFILES.md](../.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md).
```

You could also move the paragraph under `## History` as a historical note.

## Info

### IN-01: The history note dates the five-scene sample to the wrong milestone range

**File:** `docs/playground-scene-spot-check.md:8-9, 17-18`

**Issue:**
- `cfdbaabd4` is `feat(24-05)`, recorded during v1.2 and after the v1.2 shared-path fixes, so "Those numbers predate the v1.2 to v1.4 engine changes" overstates the gap.
- Line 9's "after the shared-path gate" is v1.2 wording that now describes the current recipe, which makes it misleading.

**Fix:**
- Change line 17-18 to "predate the later v1.3 and v1.4 engine changes".
- Drop "after the shared-path gate" from line 9, or move it into the History paragraph.

### IN-02: The "same binary" wording for the current ranking is looser than what ran

**File:** `docs/benchmarks/scene-survey.md:46, 122-124`

**Issue:**
- Line 122-124 says `target/phase35/bin/spot-v14-final --runs 3` is "the same binary and flags that `just playground-scene-spot` runs". That is not quite what ran. The recipe compiles the current tree through `cargo xtask` with `--quiet` and mints an xtask stamp. This run invoked a saved binary directly and minted no stamp.
- Line 46 lists the build command. Per `v14-protocol.txt`, that `cargo build` was a no-op: `target/release` was already fresh ("Finished" in 0.05 s), and the binary is byte-identical to `bin/spot-A2r`. Both lines are true, but the extra detail would make the provenance exact.

**Fix:**
- Reword line 122-124 to "the same build of the same source and flags that `just playground-scene-spot` uses (run directly, so no xtask stamp was minted)".
- Optionally add to line 46: "cargo reported the artifact up to date; it is byte-identical to the A2r keep binary".

### IN-03: The filtered summary tells the reader to "see scene_filter", but stdout never prints it

**File:** `tools/xtask/src/playground/spot.rs:26, 288-307`

**Issue:** `FILTERED_DISCLAIMER` is printed on stdout and says "(see scene_filter)". `render_summary` does not print the filter, so a reader of the terminal output has to open `scene-spot.json` to see which scenes were requested. This is not wrong, since the artifact line names the file, but it is an easy improvement.

**Fix:** Add a summary line, for example:

```rust
"- scene filter: `{filter}`\n",
// …
filter = if spot_args.scenes.is_empty() { "full catalog".to_owned() } else { spot_args.scenes.join(", ") },
```

To pass the filter in, you can read it from `report["scene_filter"]` instead of adding another parameter. Add a matching assertion in `scene_spot_filters_requested_scenes`.

---

_Reviewed: 2026-10-11T03:38:15Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
