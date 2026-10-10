---
phase: 35-speed-up-the-slowest-scenes
reviewed: 2026-10-10T01:44:02Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot/fingerprint.rs
  - crates/liquidfun-wasm/src/scene_spot/tests.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/body_contact/tests.rs
  - crates/liquidfun/src/particle/contact_scan.rs
  - crates/liquidfun/src/particle/lifetime/eviction.rs
  - crates/liquidfun/src/particle/storage.rs
  - crates/liquidfun/src/particle/storage/creation.rs
  - crates/liquidfun/src/particle/storage/creation_fast_path_tests.rs
  - crates/liquidfun/src/particle/storage/runtime.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/moving_fixture_query.rs
  - crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs
  - docs/benchmarks/scene-survey.md
  - tools/xtask/src/playground.rs
  - tools/xtask/src/playground/spot.rs
  - tools/xtask/src/playground/spot/args.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/playground_cli/spot.rs
findings:
  critical: 0
  warning: 1
  info: 5
  total: 6
status: issues_found
---

# Phase 35: Code Review Report

**Reviewed:** 2026-10-10T01:44:02Z
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

I reviewed the full diff `059173843..HEAD` for the 22 files in scope. The phase kept five engine changes: A1 (proxy order reuse), A5 (bitset row ordering), A6 (moving-fixture CCD pad), A8 (ungrouped-create fast path) and A9 (eviction index hasher and in-place resequence). It also added a survey fingerprint and a `--scene` filter to the tooling.

The engine changes match the old paths under the storage invariants the code already maintains. I checked each one against the focus areas:

- **A1, proxy order cache.** Today, a retained buffer comes only from `rebuild_proxies`: a full sort, a re-sort, or a partial row-order prefix left by an error. So when its length matches, it is always a permutation of `0..len`. `(tag, row)` is a total order with unique keys, so `sort_unstable` is deterministic and gives the same buffer as the fresh rebuild. Diameter errors return before the buffer is touched, and tag errors fall back to the row-order rebuild, so errors are unchanged. After `clear_contact_scan`, `contact_proxies()` is still empty, so `maybe_current_contact_proxies` and public views behave as before. I found no invalidation bug. IN-01 covers the one invariant that nothing enforces.
- **A5, bitset row order.** It equals `sort_unstable` + `dedup` for rows below `proxies.len()`. The production caller guarantees that bound through `maybe_current_contact_proxies`. The mark buffer is zero again on every return path.
- **A6, moving-fixture pad.** The soundness argument holds:
  - The start map is affine. For polygons, edges and chains the linear part is `R_cur R_prevᵀ`. For circles it is `|q_cur|²|q_prev|² I`. I confirmed both against `collision_start_from_previous_transform`.
  - The inf-norm Lipschitz bound and the convex-corner maximum are correct.
  - f32 rounding of the start and of `p + dt·v` is covered: by `ROUNDING_PER_METER · scale` for the AABB part, and by the `+ROUNDING_PER_METER` added to `L` for the `δ` part.
  - Every failure falls back to the full scan: non-finite input, `L >= 1`, a failed corner check, AABB construction, or the tag range.
  - Non-finite velocities keep the full scan, so the first failing particle and its error stay the same.
  - Proxies and candidate positions agree, because positions change only in `Integrate`, after `Collision`.
  - Each query visits a row at most once, and `sort_by_key` is stable, so the hit list is identical. Only the order of hook calls changes (IN-03).
- **A8 and A9.** Both match the rebuild when the storage and index invariants hold:
  - A8: non-empty records keep their range, so `set_range` does nothing, and empty records get the same normalization. The `i32::MAX` lane check gives the same boundary.
  - A9: positions `GAP·(i+1)` follow storage order, unplaced keys are skipped, and `next_position` comes out the same.
  - Determinism: `ParticleIdHasher` is deterministic, and nothing iterates `by_particle`.
  - IN-04 notes that both fast paths drop the incidental validation the rebuild used to do.
- **`unsafe`, panics and `unwrap`.** The changed production code adds no `unsafe` and no `unwrap`. The only indexing that could panic is in `order_rows_with_marks`, and only if a caller breaks an undocumented precondition (IN-02).

The one Warning is in the evidence tooling. A stamp filtered with `--scene` still calls itself a survey of every catalog scene and does not record the filter.

## Warnings

### WR-01: Filtered scene-spot stamps claim to cover every catalog scene and do not record the filter

**File:** `tools/xtask/src/playground/spot.rs:1, 25, 218, 281`

**Issue:** With `--scene`, the stamp's `scene-spot.json` holds only the requested scenes. It still embeds `DISCLAIMER`, which reads "Unreviewed local survey of every playground catalog scene", and prints the same text in the summary. The report has no field that records the filter or says the run was partial. The module doc on line 1 also still says "all-catalog". Phase 36 builds a whole-catalog before/after table from these stamps. A filtered stamp could be taken for a full survey, and the label is false. That conflicts with the project's transparency rule. A careful reader can count the entries in `scenes`, but the artifact itself makes the wrong claim.

**Fix:** Record the filter and choose the wording from it:

```rust
const FULL_DISCLAIMER: &str = "Unreviewed local survey of every playground catalog scene. Not a C++ pair, not Phase 12, and not the Dam Break 3× number.";
const FILTERED_DISCLAIMER: &str = "Unreviewed local survey of the requested playground catalog scenes only (see scene_filter). Not a C++ pair, not Phase 12, and not the Dam Break 3× number.";

fn disclaimer(spot_args: &SpotArgs) -> &'static str {
    if spot_args.scenes.is_empty() { FULL_DISCLAIMER } else { FILTERED_DISCLAIMER }
}

// in assemble_report:
"disclaimer": disclaimer(spot_args),
"scene_filter": spot_args.scenes,          // [] means full catalog
"full_catalog": spot_args.scenes.is_empty(),
```

Then have `render_summary` read the disclaimer from the report, and update the module doc. Add a CLI test assertion in `scene_spot_filters_requested_scenes` that `report["full_catalog"] == false`.

## Info

### IN-01: The proxy order reuse relies on a permutation invariant that nothing enforces

**File:** `crates/liquidfun/src/particle/contact_scan.rs:159-178` (also `crates/liquidfun/src/particle/storage/runtime.rs:319-341`)

**Issue:** `retag_retained_order` treats any retained buffer whose length matches as a permutation of `0..len`. It checks only that each row is in bounds (`positions.get(proxy.row)`). This holds today, because every buffer comes from `rebuild_proxies`. However, `install_contact_proxies` is `pub(crate)` and accepts any `Vec`; the body-contact tests already install hand-made buffers. A future caller that installs a filtered buffer or one with duplicate rows would break the invariant silently. Some particles would get no contacts and others would get duplicate contacts, with no error raised. Before A1, such a buffer was discarded and rebuilt.

**Fix:** Document the precondition on `install_contact_proxies`, which should accept only buffers produced by `fill_stored_contacts`. Add a debug-only check in the reuse branch:

```rust
debug_assert!({
    let mut seen = vec![false; proxies.len()];
    proxies.iter().all(|p| !std::mem::replace(&mut seen[p.row], true))
}, "retained proxy buffer must be a permutation of rows");
```

### IN-02: `order_rows_with_marks` has an undocumented precondition and can panic if it is broken

**File:** `crates/liquidfun/src/particle/body_contact.rs:60-82`

**Issue:** The function indexes `row_marks[word]` with `word = row / 64`, but sizes the buffer from `row_count = proxies.len()`. A row at or above `proxies.len()` panics. `first_word = usize::MAX` is a sentinel, so the slice `row_marks[first_word..=last_word]` also panics on an empty `rows`. Today that slice is only reached when `rows.len() >= 32`. Both preconditions hold today, because `maybe_current_contact_proxies` guarantees `row < positions.len() == proxies.len()`. Nothing documents or asserts them, though, and `collect_candidate_rows` is called directly from tests with arbitrary proxies.

**Fix:** State "every row is below `row_count`; `rows` is non-empty" in the doc comment and add `debug_assert!(!rows.is_empty() && rows.iter().all(|&r| r < row_count))`. Alternatively, size `row_marks` from the largest row seen rather than from `proxies.len()`.

### IN-03: The iteration-0 moving-fixture query changes the order of `should_collide_fixture_particle` calls

**File:** `crates/liquidfun/src/world/particle_coupling.rs:268-276, 300-314`; `crates/liquidfun/src/world/step/hook.rs:35-41`

**Issue:** For moving fixtures at particle iteration 0, `FIXTURE_CONTACT_FILTER` hook calls now come in proxy (tag) order instead of row order. The comment in the code says so. The set of calls is the same and the stable sort keeps the hit list identical. A stateful or order-dependent user hook, however, can now make different decisions, and so produce a different trajectory. D-07's bit-identity gate cannot see this, because no catalog scene uses such a hook. The public hook docs do not say the call order is unspecified. This matches the static path and upstream's tag-order enumeration, so it is acceptable, but users are not told.

**Fix:** Add a sentence to the `should_collide_fixture_particle` doc: "Call order across particles is unspecified; decisions must not depend on earlier calls in the same step." Alternatively, add a test that pins only the resulting hit set for a hook that counts its calls.

### IN-04: The A8 and A9 fast paths no longer perform the rebuild's incidental validation

**File:** `crates/liquidfun/src/particle/storage/creation.rs:395-423`; `crates/liquidfun/src/particle/lifetime/eviction.rs:163-197`

**Issue:** The old ungrouped `prepare_create` ran `rebuild_group_records_for_system`. That function calls `membership_ranges` and `validate_groups`, so corrupted group records failed with `InvalidGroupRange`. In release builds the fast path now copies such records forward without complaint; only the `debug_assert_eq!` catches the difference, and only in debug builds. In the same way, the old resequence rebuilt `by_particle` from the BTreeMap keys and dropped orphan entries. The in-place version keeps them, so `len()` would then disagree with `storage_order().len()`. Neither difference can occur while the invariants hold, and `check_invariants` already runs as a debug assertion. The change does mean that errors are no longer strictly identical when state is corrupt.

**Fix:** Optional. Add `debug_assert_eq!(self.by_particle.len(), self.finite.len() + self.infinite.len())` at the end of `resequence_to_storage_order`. Note in both doc comments that equivalence assumes the storage and index invariants.

### IN-05: The survey doc says the fingerprint proves more than it does

**File:** `docs/benchmarks/scene-survey.md:23` (fingerprint scope in `crates/liquidfun-wasm/src/scene_spot/fingerprint.rs:51`)

**Issue:** "Equal fingerprints mean a bit-identical trajectory" claims too much:

- The fingerprint is a 64-bit FNV-1a hash of one end state.
- It hashes particle positions, velocities and colors, and body pose and velocity.
- It does not hash particle flags, lifetimes and eviction order, group records, particle identities, or contacts. A8 and A9 changed group records and eviction order, which the fingerprint sees only indirectly.

The project's transparency rule asks that documented claims match the evidence.

**Fix:** Reword: "Equal fingerprints mean the hashed end state (particle positions, velocities and colors, body poses and velocities) is bit-identical; differences outside those fields are not covered."

---

_Reviewed: 2026-10-10T01:44:02Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
