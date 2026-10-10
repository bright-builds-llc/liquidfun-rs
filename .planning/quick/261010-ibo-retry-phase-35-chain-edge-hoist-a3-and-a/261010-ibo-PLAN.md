---
phase: quick-261010-ibo
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/body_contact/tests.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/scratch.rs
  - crates/liquidfun/src/world/particle_coupling/scratch_tests.rs
  - crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs
  - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
autonomous: true
requirements: [PERF-08, PERF-09]
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261010-ibo
generated_at: "2026-10-10T18:15:14Z"
must_haves:
  truths:
    - "The A3 retry (A3r) and, only if A3r fails, the A3b retry (A3br) were each judged against the HEAD engine binary by the 35-PROFILES.md §Method and D-11 exactly as written: target gain in both targeted ABBA pairs, no scene above its base max in both isolated pairs, and 25/25 fingerprints equal to target/phase35/before-full.jsonl."
    - "Each keep/revert decision was written to target/phase35/<id>.decision before any diagnostic run, and no decision set was re-rolled because of its numbers (a set is voided only by the pre-registered, result-blind host gate)."
    - "No timed leg overlapped a cargo or rustc process, and every leg has uptime lines in target/phase35/<prefix>.uptime."
    - "If an attempt is kept: its engine change is committed only after cargo fmt, clippy -D warnings, the all-targets build, the wasm32 --lib build, bright-builds-check and cargo test --workspace --all-features all pass, and the committed source equals the timed source."
    - "If both attempts fail: git diff HEAD -- crates/ is empty, and target/phase35/attempts/A3r.patch and A3br.patch hold the ported diffs."
    - "35-PROFILES.md gains new A3r (and A3br if tried) rows, the liquid-tumbler target record and Phase summary row are updated, and the original A3 and A3b rows are byte-unchanged."
  artifacts:
    - path: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
      provides: "Attempt rows, run notes and target records for the retry"
      contains: "| A3r |"
    - path: target/phase35/abba-gated.sh
      provides: "ABBA runner with a pre-registered quiet-host gate (local, gitignored)"
    - path: target/phase35/ibo-protocol.txt
      provides: "Pre-registered protocol written before the first timed leg (local, gitignored)"
  key_links:
    - from: crates/liquidfun/src/world/particle_coupling.rs
      to: "EdgeShape::ray_cast"
      via: "push_fixture_particle_hit casts on the prebuilt chain child edge (A3r) or on a per-child Shape::Edge record (A3br)"
      pattern: "maybe_edge|Shape::from\\(edge\\)"
    - from: crates/liquidfun/src/particle/body_contact.rs
      to: "EdgeShape::distance_to_point"
      via: "generate builds the chain child edge once per child"
      pattern: "maybe_child_edge|child_edge"
---

<objective>
Retry the Phase 35 chain-edge hoist on current HEAD (`cfe85b09f`). Port A3 (the larger gain) first and judge it by D-11 and the 35-PROFILES.md §Method as written, with interleaved ABBA timing against the HEAD engine binary. If A3r fails, port and judge A3b once the same way. Keep and commit a change only if it passes.

Purpose: In 35-04, A3 cut liquid-tumbler by 9–10% and A3b by 7–8%, with bit-identical fingerprints. Both were reverted because a scene that does not use chains (soup-stirrer, fountain) was about 1.5% above its base max in both isolated pairs, and later diagnostic pairs did not reproduce that. The host no longer stalls on new binaries (0.26 s launch instead of about 6 min), and the load is currently lower, so a clean retry is now cheap.

Output: either a committed engine change plus updated 35-PROFILES.md records, or the reverted attempts recorded honestly with their patches saved.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md
@.planning/phases/35-speed-up-the-slowest-scenes/35-CONTEXT.md
@.planning/phases/35-speed-up-the-slowest-scenes/35-04-SUMMARY.md

Read 35-PROFILES.md in these sections only: §Phase summary (lines 1-39), §Method through §A/B procedure (lines 41-88), the §Attempts table and run notes (lines 150-173), §Target records (175-181), and §Notes for Phase 36 (231-247).

<locked_rules>
- D-05: never change authored scene settings. No nondeterministic parallelism, no SIMD that changes results, no new `unsafe`.
- D-07: a kept change leaves all 25 scene fingerprints identical to `target/phase35/before-full.jsonl`. A change that alters float results is rejected and not explained away.
- D-11: same machine and flags (60 warmup, 120 steps, `--runs 5` for the ABBA sets and `--runs 3` for the full catalog, as §Method does). Keep only if the target's after median is below the base minimum (in both targeted pairs), no scene's after median is above its base maximum (confirmed per §Method: above the base max in both isolated pairs), and all fingerprints match. Otherwise revert, save the patch and record the attempt.
- Do NOT loosen D-11. The decision uses exactly the §Method pair counts. This plan's only noise lever is the result-blind quiet-host gate below. It decides whether a set is valid from load and process data alone, never from timing results. Extra pairs taken after a decision are labelled diagnostics and cannot change it (as in 35-04).
- Never time while any cargo or rustc process runs (any repository). Record `uptime` at the start and end of every leg.
- Never run mdformat on `.planning/**`. Never `git add -A` or `git add .`: stage explicit paths only. Never touch the untracked empty file `=` in the repo root or `.planning/config.json`.
- Work on the main tree, not a worktree, because `target/phase35/` holds the artifacts.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
</locked_rules>

<facts_checked_at_planning>
- `git diff f7041fc75 HEAD -- crates/` is empty, so the HEAD engine is the 35-07 final engine.
- The SHA-256 of `target/phase35/bin/spot-final` and of `target/release/playground-scene-spot` both start with `bd2b2eb59057ecbc`, matching the recorded `bd2b2eb5…`.
- `git apply --check target/phase35/attempts/A3.patch` fails. `git apply --3way --check` applies 4 of its 5 files cleanly; `particle_coupling.rs` conflicts because A6 rewrote the CCD child loop.
- `git apply --3way --check target/phase35/attempts/A3b.patch` applies all 4 files cleanly.
- A3 changes `CcdFixtureRecord.children` from `Vec<(ChildIndex, Option<Aabb>)>` to `Vec<CcdChild>`. `moving_fixture_query_tests.rs` (added by A6, not in A3.patch) builds tuple children in `record()` (around line 133) and reads `.children[i].1` at lines 244, 274 and 451, so it must be updated for A3r.
- `push_fixture_particle_hit` does not store the child index in `FilteredCollisionHit`, so A3b's child index 0 on per-child edge records does not leak into hits.
- `body_contact/tests.rs` is 556 lines, and A3 adds about 98 lines of tests to it. If `bun scripts/bright-builds-check.ts all` flags the file length, move the A3 tests into a sibling module `body_contact/chain_edge_tests.rs` (declared next to `mod tests;`) instead of raising a limit.
- Current host: load about 2.7 (1 min) / 4.5 (5 min); no cargo or rustc process running.
</facts_checked_at_planning>

<interfaces>
Current HEAD CCD loop (crates/liquidfun/src/world/particle_coupling.rs, around lines 267-330):

```rust
for fixture in fixtures {
    let static_fixture = fixture.previous_transform == fixture.current_transform;
    for (child, maybe_aabb) in &fixture.children {
        let maybe_pad = match *maybe_aabb {
            Some(_) if !proxies_match => None,
            Some(_) if static_fixture || particle_iteration != 0 => Some(motion),
            Some(aabb) if *maybe_velocities_finite.get_or_insert_with(|| ..) => {
                moving_fixture_query_pad(aabb, motion, fixture.previous_transform,
                    fixture.current_transform, fixture.body_local_center, fixture.is_circle)
            }
            _ => None,
        };
        if let (Some(pad), Some(aabb)) = (maybe_pad, *maybe_aabb) {
            let queried = query_particles_for_fixture(proxies, diameter, aabb, pad, |row| {
                push_fixture_particle_hit(candidate, fixture, *child, maybe_aabb.as_ref(), row, ..)
            })?;
            if queried { continue; }
        }
        for particle in 0..candidate.positions.len() {
            push_fixture_particle_hit(candidate, fixture, *child, maybe_aabb.as_ref(), particle, ..)?;
        }
    }
}

fn push_fixture_particle_hit<H: CollisionDecisionHook>(
    candidate: &BoundaryCandidate, fixture: &CcdFixtureRecord, child: ChildIndex,
    maybe_aabb: Option<&Aabb>, particle: usize, time_step: f32, particle_iteration: u32,
    hook_run: &mut ContactHookRun<'_, H>, hits: &mut Vec<FilteredCollisionHit>,
) -> Result<(), StepError>
```

A3's target shape (from attempts/A3.patch):

```rust
struct CcdChild { index: ChildIndex, maybe_aabb: Option<Aabb>, maybe_edge: Option<EdgeShape> }
// ccd_fixture_records: maybe_edge = match &shape { Shape::Chain(chain) => chain.child_edge(child).ok(), _ => None }
// push_fixture_particle_hit(candidate, fixture, ccd_child: &CcdChild, particle, ..):
//   travel filter uses ccd_child.maybe_aabb;
//   hit = match &ccd_child.maybe_edge {
//       Some(edge) => edge.ray_cast(input, fixture.current_transform),
//       None => fixture.shape.ray_cast(input, fixture.current_transform, ccd_child.index),
//   }.map_err(|_error| StepError::ParticleLifecycleInvariant)?
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Verify the base binary, port A3 onto HEAD as A3r, and build spot-A3r</name>
  <files>crates/liquidfun/src/particle/body_contact.rs, crates/liquidfun/src/particle/body_contact/tests.rs, crates/liquidfun/src/world/particle_coupling.rs, crates/liquidfun/src/world/particle_coupling/scratch.rs, crates/liquidfun/src/world/particle_coupling/scratch_tests.rs, crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs</files>
  <behavior>
    - The A3 tests carried over from the patch pass: `chain_fixture_contacts_match_per_particle_distance` and `polygon_fixture_contacts_unchanged_by_hoist` (both compare `to_bits()` with the per-particle path under a rotated transform), and `ccd_children_carry_chain_edges_only`.
    - A new CCD bit-identity test, `chain_child_edge_records_cast_like_chain_children`, goes in moving_fixture_query_tests.rs. It uses the open chain from `chain_and_edge_children_filtered_hits_equal_full_scan` and a moving transform at particle iteration 0. It builds two `CcdFixtureRecord`s for the same chain, one with `maybe_edge` set for every child and one with `maybe_edge: None` for every child, which forces `Shape::ray_cast(.., child)`. It asserts that `full_scan` hit bits are equal for both and that the list is not empty.
    - All existing tests in moving_fixture_query_tests.rs and scratch_tests.rs still pass. The `record()` helper fills `maybe_edge` the same way `ccd_fixture_records` does, so the chain test exercises the prebuilt-edge path.
  </behavior>
  <action>
1. **Base check (no source edits yet).**
   - `git status --short` must show only `?? =`.
   - `git diff f7041fc75 HEAD -- crates/` must be empty.
   - `shasum -a 256 target/phase35/bin/spot-final` must start with `bd2b2eb5`.
   - Hard-link the base binary: `ln target/phase35/bin/spot-final target/phase35/bin/spot-base-ibo`. Copy nothing.
   - If any check fails, stop and report. Do not time against an unverified base.

1. **Port A3 (per D-05 and D-07; the change must be a pure hoist with the same float operations).**
   - Run `git apply --3way target/phase35/attempts/A3.patch`.
   - Resolve the one conflict in `particle_coupling.rs` against A6's loop. Change the loop to `for ccd_child in &fixture.children`, match on `ccd_child.maybe_aabb` in the `maybe_pad` match and in the `if let (Some(pad), Some(aabb))`, and pass `ccd_child` (replacing `*child, maybe_aabb.as_ref()`) to both `push_fixture_particle_hit` calls.
   - Keep every other A6 line (the pad match, the `maybe_velocities_finite` cache and its comments) unchanged.
   - Keep A3's `CcdChild` struct and doc comment, the `maybe_edge` build in `ccd_fixture_records`, the `match &ccd_child.maybe_edge` cast, and the `EdgeShape` import.
   - Keep `chain.child_edge(child).ok()` with the fallback to `Shape::ray_cast`: the fallback reports the original error at the original point. Keep the existing comment that explains why.
   - Then `git reset -q` so the changes are unstaged. `--3way` may stage clean files.

1. **Update `moving_fixture_query_tests.rs`.**
   - `record()` builds `CcdChild { index, maybe_aabb, maybe_edge }`, with `maybe_edge` from `Shape::Chain(chain) => chain.child_edge(child).ok()`.
   - Replace `.children[i].1` with `.children[i].maybe_aabb` at the three call sites.
   - Add the new test from `<behavior>` in Arrange/Act/Assert form.
   - Check that `scratch.rs` and `scratch_tests.rs` from the patch still compile against A6/A8/A9 code. Fix only mechanical drift.

1. **Run the focused tests.**
   - `cargo test -p liquidfun --all-features --lib -- body_contact particle_coupling` must pass, including the new test.
   - Then run `cargo fmt --all`, `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings` and `bun scripts/bright-builds-check.ts all`. Resolve any file-length finding as described in `<facts_checked_at_planning>`.

1. **Build and record the timed source.**
   - `cargo build --release -p liquidfun-wasm --bin playground-scene-spot`, then `cp target/release/playground-scene-spot target/phase35/bin/spot-A3r`.
   - Write `git diff HEAD -- crates/ | shasum -a 256 > target/phase35/A3r.source-sha` and save the diff itself as `target/phase35/attempts/A3r.patch`.

1. **Warm-launch both binaries.** Wait until no cargo or rustc process is running, then launch `spot-A3r` and `spot-base-ibo` once each with `--warmup 1 --steps 1 --runs 1 --scene particles`. Both must exit 0, and the `spot-A3r` run must print a fingerprint line.

1. **Do not commit.** Commits happen only in Task 3, after timing.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs && cargo test -p liquidfun --all-features --lib -- body_contact particle_coupling && cargo clippy -p liquidfun --all-targets --all-features -- -D warnings && test -x target/phase35/bin/spot-A3r && test -s target/phase35/A3r.source-sha && git diff --quiet f7041fc75 HEAD -- crates/</automated>
  </verify>
  <done>A3 is ported onto HEAD with A6's moving-fixture pad intact. Focused tests pass, including the 3 A3 tests and the new CCD bit-identity test. `spot-A3r`, `A3r.source-sha`, `attempts/A3r.patch` and the hard link `spot-base-ibo` exist, and both binaries have been warm-launched. Nothing is committed.</done>
</task>

<task type="auto">
  <name>Task 2: Pre-register the protocol, time A3r with gated ABBA, decide by D-11, and fall back to A3b once if A3r fails</name>
  <files>target/phase35/abba-gated.sh, target/phase35/ibo-protocol.txt (local, gitignored); crates/ files from Task 1 only if A3r fails and A3b is ported</files>
  <action>
1. **Create `target/phase35/abba-gated.sh` and `chmod +x` it.**
   - Base it on the existing `target/phase35/abba.sh`: same arguments `PREFIX BASE AFTER RUNS SCENE...`, the same per-leg uptime logging to `PREFIX.uptime`, and the same two `keep_rule.py` calls at the end.
   - Add a `LEGS` environment variable, default `"b1 a1 a2 b2"`. Legs `b*` run BASE and legs `a*` run AFTER.
   - **Quiet gate, before each leg.** Wait while `pgrep -x cargo || pgrep -x rustc` matches, or while the 1-minute load from `sysctl -n vm.loadavg | awk '{print $2}'` is above 6.0. Poll every 15 s. After 1800 s, log `GATE-TIMEOUT <time> <uptime>` and proceed.
   - **Void rule, checked at the end of each leg.** If cargo or rustc is running, or the 1-minute load is above 10.0, mark the set void.
   - **Voided set handling.** Move the set's jsonl files into `PREFIX.void-<epoch>/`, append `VOID <reason>` to the uptime log, do not run `keep_rule.py`, and exit 3.
   - **Full-catalog mode.** When `LEGS="b1 a1"`, run only `keep_rule.py PREFIX-b1.jsonl PREFIX-a1.jsonl TARGETS`.
   - Because the script may wait, run it with `run_in_background` when the gate could block, then read its output.

1. **Pre-register the protocol.** Before the first timed leg, write `target/phase35/ibo-protocol.txt` containing:
   - the UTC time and `git rev-parse HEAD`;
   - the base and after binary SHA-256 values;
   - the gate thresholds above;
   - the decision rule, quoting D-11 and §Method steps 3–5;
   - the run order below;
   - this statement: "at most 2 re-runs of a voided set; the third run counts regardless and is recorded with its load; no valid set is re-run".

1. **Run order for an attempt ID (A3r first).** All runs are in `target/phase35/` with base `bin/spot-base-ibo`.
   1. Targeted: `./abba-gated.sh A3r bin/spot-base-ibo bin/spot-A3r 5 liquid-tumbler`. The gain holds only if both keep_rule lines print `gain=True`.
   1. Full catalog: `LEGS="b1 a1" ./abba-gated.sh A3r-full bin/spot-base-ibo bin/spot-A3r 3`. The scene list is empty, so all 25 scenes run. Check that each file has 25 lines.
   1. Fingerprints: `python3 keep_rule.py before-full.jsonl A3r-full-a1.jsonl ""` must print `fingerprint mismatches: none`. Also confirm `none` in the targeted pair output.
   1. Isolated re-check: take every scene the full-catalog keep_rule listed under "medians above before max". This includes liquid-tumbler if it did not gain there. Run `./abba-gated.sh A3r-iso bin/spot-base-ibo bin/spot-A3r 5 <those scenes>`. A scene is a confirmed regression only if both pair lines list it. If the full run listed nothing, skip this step and record "none listed".

1. **Decide and write the decision first.**
   - Write `target/phase35/A3r.decision` before running anything else. It holds the targeted pair numbers (base median, min and max; after median), the scenes the full run listed, the isolated pair results, the fingerprint result, the decision (`kept` or `reverted`) with the D-11 clause that decided it, and the time.
   - Keep A3r only if all three D-11 conditions hold.
   - Optional diagnostic pairs may run after the decision file exists. Name them `A3r-diag-*`; they never change the decision.

1. **If A3r is reverted, try A3b once.**
   - Confirm that `git apply --reverse --check target/phase35/attempts/A3r.patch` succeeds, then restore with `git checkout -- crates/`.
   - Run `git apply --3way target/phase35/attempts/A3b.patch`, then `git reset -q`.
   - Run `cargo test -p liquidfun --all-features --lib -- body_contact particle_coupling`. A3b's 4 tests and all existing moving_fixture_query tests must pass. Fix only mechanical drift; A3b's design (one `Shape::Edge` record per chain child at child index 0, in fixture-then-child order) is unchanged.
   - Run `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings`.
   - Build the release binary and save it as `bin/spot-A3br`. Write `A3br.source-sha` and save the diff as `attempts/A3br.patch`.
   - Warm-launch `spot-A3br`, then repeat the run order and decision with ID `A3br`.

1. **If either attempt is kept, run the cumulative pair now, before Task 3's cargo runs.**
   - Command: `./abba-gated.sh <ID>-cum bin/spot-before bin/spot-<ID> 5 liquid-tumbler`.
   - Warm-launch `spot-before` first if it has not run this session.
   - This pair feeds the Phase summary row. It is evidence only and does not affect the decision.

1. **If both attempts are reverted,** confirm with `git apply --reverse --check` that `attempts/A3br.patch` matches the tree, then run `git checkout -- crates/`. Afterwards `git diff HEAD -- crates/` must be empty. Both patches remain in `target/phase35/attempts/`.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && test -s ibo-protocol.txt && test -s A3r.decision && test -s A3r.uptime && test -s A3r-full-a1.jsonl && [ "$(wc -l < A3r-full-a1.jsonl)" -eq 25 ] && python3 keep_rule.py before-full.jsonl A3r-full-a1.jsonl "" | grep -q "fingerprint mismatches"</automated>
  </verify>
  <done>
   - `ibo-protocol.txt` predates the first timed leg.
   - A3r was judged with a targeted ABBA, a full-catalog run, a fingerprint check and isolated ABBA pairs, and its decision file was written before any diagnostic. A3br was judged the same way only if A3r was reverted.
   - Every leg has uptime lines. Voided sets, if any, are preserved and logged.
   - If an attempt was kept, its cumulative pair exists.
   - If none was kept, `crates/` equals HEAD and both patches are saved.
  </done>
</task>

<task type="auto">
  <name>Task 3: Run the full checks and commit if kept, then record the retry in 35-PROFILES.md</name>
  <files>crates/ files from the kept attempt (if any), .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md</files>
  <action>
1. **If an attempt (ID) was kept, check and commit the code. Never start a timed run while these checks run.**
   - Run the following in order, logging to `target/phase35/ibo-checks.log` with every exit code:
     1. `cargo fmt --all`
     1. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
     1. `cargo build --workspace --all-targets --all-features`
     1. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` (the build without `--lib` fails for a pre-existing reason; see deferred-items.md)
     1. `bun scripts/bright-builds-check.ts all`
     1. `cargo test --workspace --all-features`
   - **If a check fails:** fix the cause. A failure outside the changed modules counts as pre-existing only if it reproduces on a temporary `git worktree add` of HEAD under the scratchpad; record it in `deferred-items.md`. Otherwise do not commit.
   - **Source digest:** `git diff HEAD -- crates/ | shasum -a 256` must equal `<ID>.source-sha`.
   - **If fmt or a fix changed the source:**
     - Rebuild the release bin and compare its SHA-256 with `spot-<ID>`.
     - If the binary differs, re-run the targeted ABBA and the full-catalog fingerprint check with the new binary as `<ID>-post`, using the same protocol. Keep the change only if it still passes. Then re-run the checks.
     - Record any of this.
   - **Commit:** stage only the changed `crates/` paths. Use the message `perf(35): hoist chain child edge in body contacts and CCD (<ID>, quick 261010-ibo)`. The body gives the liquid-tumbler pair changes, 25/25 fingerprints and "D-11 as written". End the message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

1. **Record in 35-PROFILES.md (always). Do not edit the existing A3 or A3b rows or the 35-04 run notes.**
   1. Append an `A3r` row after the A9 row, plus an `A3br` row if A3b was tried. Use the table's 11 columns:
      - Plan: `quick 261010-ibo`.
      - Base: `spot-base-ibo` (hard link of `spot-final`, engine A1+A5+A6+A8+A9), median (min-max) for pair 1 / pair 2.
      - After medians for both pairs.
      - Fingerprints: `yes (25/25)` or the mismatches.
      - Regression: the full-run list and the isolated pair results.
      - Decision.
      - Commit: the hash, or `none (never committed; diff in target/phase35/attempts/<ID>.patch)`.
      - Reason: give the percentages. Keep the wording neutral and do not call a regression noise.
   1. Add a paragraph `Quick 261010-ibo run notes:` after the 35-07 run notes. It covers the binaries and SHA prefixes, the timing windows (UTC) and load ranges from the `.uptime` files, the gate and any voided sets or GATE-TIMEOUTs, `ibo-protocol.txt`, the fact that no cargo or rustc ran during timing, and any labelled diagnostics.
   1. **liquid-tumbler entry in §Target records:** append the retry outcome, and if kept, the cumulative `spot-before` vs `spot-<ID>` medians.
   1. **liquid-tumbler row in §Phase summary:**
      - Attempts column: append `A3r` (and `A3br` if tried).
      - Kept column and Final median column, only if kept: append `<ID>` to Kept, and append `; after <ID> (quick 261010-ibo): X / Y` from the cumulative pair to Final median.
      - Gain-beyond-noise column: update it the same way, using the before minimum of that pair.
   1. **§Phase summary attempts list:** add one bullet for each new ID.
   1. **§Notes for Phase 36:**
      - Append the outcome to leftover idea 1.
      - If kept, add a bullet saying that the re-survey binary is now the kept commit's release bin (SHA prefix), not `spot-final`.
   - Never run mdformat on this file.
   - Stage only `35-PROFILES.md` (and `deferred-items.md` if touched). Commit with `docs(35): record chain-edge hoist retry (quick 261010-ibo)` and the same Co-Authored-By line.

1. **Write the quick SUMMARY** at `.planning/quick/261010-ibo-retry-phase-35-chain-edge-hoist-a3-and-a/261010-ibo-SUMMARY.md`. Its frontmatter must include `generated_by: gsd-executor`, `lifecycle_mode: direct-fallback`, `phase_lifecycle_id: quick-261010-ibo` and `generated_at` set to the real current UTC time.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs && grep -q "^| A3r | quick 261010-ibo" .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md && git diff cfe85b09f HEAD -- .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md | grep -E "^-\| A3b? \|" | wc -l | grep -qx 0 && git status --short | grep -v '^?? =$' | grep -v '261010-ibo' | wc -l | grep -qx 0</automated>
  </verify>
  <done>
   - If kept: one perf commit whose source digest matches the timed binary (or a recorded re-time), with every listed check passing per `ibo-checks.log`.
   - If not kept: `crates/` equals HEAD.
   - In both cases, 35-PROFILES.md has the new row(s), run notes, the target record, the summary row and the Notes update, with the A3 and A3b rows unchanged, and it is committed in a docs commit.
   - SUMMARY.md is written with the lifecycle frontmatter.
   - The working tree is clean apart from `=` and the quick-task files.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| none external | Local engine refactor and local timing; no network, input parsing, credentials or new dependencies. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-ibo-01 | Tampering (evidence integrity) | keep decision in target/phase35 | mitigate | `ibo-protocol.txt` is written before the first leg. `<ID>.decision` is written before any diagnostic. Voiding is result-blind (load and process data only). Committed source must match `<ID>.source-sha`. |
| T-ibo-02 | Tampering (behavior) | body_contact::generate, CCD hits | mitigate | Bit-identity unit tests (`to_bits()`), plus the 25/25 fingerprint check against `before-full.jsonl` per D-07. |
| T-ibo-03 | Denial of service | CCD error path | accept | `child_edge(child).ok()` falls back to `Shape::ray_cast`, which reports the original error at the original point. The child index comes from `child_count`, so the build cannot fail. |
</threat_model>

<verification>
- `python3 target/phase35/keep_rule.py target/phase35/before-full.jsonl target/phase35/<ID>-full-a1.jsonl ""` prints `fingerprint mismatches: none` for any kept ID.
- `target/phase35/*.uptime` for this task show no leg that started while cargo or rustc ran.
- `git log -3 --format=%B` ends each new commit with the Co-Authored-By line.
- `git diff cfe85b09f HEAD -- .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md` shows only additions and the listed in-place edits to the liquid-tumbler summary row, target record and Notes; no A3 or A3b row line is removed.
</verification>

<success_criteria>
- A3r is judged by D-11 as written. A3br is judged the same way only if A3r failed.
- A kept change is committed only after the full local checks pass and its source matches the timed source. A failed attempt leaves no code on HEAD and keeps its patch.
- 35-PROFILES.md records the retry honestly: new rows, run notes, liquid-tumbler target record and Phase summary row, with the originals untouched.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-ibo-retry-phase-35-chain-edge-hoist-a3-and-a/261010-ibo-SUMMARY.md`, with the lifecycle frontmatter fields listed in Task 3.
</output>
