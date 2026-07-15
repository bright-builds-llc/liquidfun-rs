---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "08"
subsystem: particle-contacts
tags: [rust, particles, spatial-proxies, stable-identity, contact-filtering]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "05"
    provides: borrow-scoped stable-ID particle inspection and coherent position edits
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "06"
    provides: authoritative particle lifecycle and permutation-safe stable identity
provides:
  - finite checked 12-bit-by-12-bit particle proxy tags and deterministic source neighborhoods
  - stable-ID particle contacts with pinned fields and borrowed flag-gated filter decisions
  - ordered listener begin/end effects that retain repeated occurrence multiplicity
affects: [09-09, 09-13, phase-10]
tech-stack:
  added: []
  patterns: [owned semantic spatial snapshot, borrowed one-call decision adapter, source-timed ordered effect]
key-files:
  created:
    - crates/liquidfun/src/particle/proxy.rs
    - crates/liquidfun/src/particle/contact.rs
    - crates/liquidfun/tests/particle_contacts.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/lifetime.rs
    - crates/liquidfun/src/particle/lifetime/tests.rs
    - crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs
key-decisions:
  - "Retain dense input order for equal proxy tags as a deterministic representative of the pinned comparator's equivalent-tag class without exposing tags or rows."
  - "Expose contact filtering as a borrowed one-call decision closure and return owned contacts plus ordered listener effects instead of registering callbacks."
  - "Evaluate previous listener eligibility from current particle flags, then emit new begins in generated order before remaining old ends in sorted row-pair order."
patterns-established:
  - "Spatial broad-phase results cross public boundaries only as stable ParticleId pairs; packed tags and rows remain private."
  - "Contact generation validates the complete current/previous identity set before invoking a borrowed decision adapter."
requirements-completed: [PART-07, PART-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T07:20:00Z
duration: 20 min
completed: 2026-07-15
---

# Phase 9 Plan 8: Particle Spatial Proxies and Contacts Summary

**Finite checked proxy neighborhoods now generate stable-ID particle contacts with pinned fields, exact flag gates, and source-ordered listener effects.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-15T07:00:09Z
- **Completed:** 2026-07-15T07:20:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Implemented the pinned packed-tag layout, deterministic proxy sorting, neighborhood enumeration, and source-expanded AABB candidate bounds without exposing row identity.
- Generated source-ordered contact flags, weight, and normal from stable particle pairs while invoking the borrowed filter only for flagged contacts.
- Diffed listener-visible old and new contact occurrences with begins first, remaining ends second, unordered pair matching, and exact repeated-occurrence preservation.
- Found and fixed a tracked-lifetime ordering defect exposed by the mandatory full-suite gate.

## Task Commits

1. **Task 1: Port proxy tags and ordered neighborhood generation** - `f8aa6f6` (feat)
2. **Task 2: Generate, filter, and diff particle contacts** - `5794b6a` (feat)
3. **Deviation: Repair tracked lifetime insertion ordering** - `64a8b0b` (fix)
4. **Task 1 hardening: Reject unusable derived diameter scales** - `954a0a5` (fix)
5. **Deviation: Align the independent lifetime model** - `b41c8d7` (fix)

## Files Created/Modified

- `crates/liquidfun/src/particle/proxy.rs` - Checked tags, source neighborhoods, semantic pair records, and expanded-bounds enumeration.
- `crates/liquidfun/src/particle/contact.rs` - Contact generation, borrowed filtering, listener diffing, and ordered effects.
- `crates/liquidfun/tests/particle_contacts.rs` - External reachability, edge/cell/property, field, filter, order, and multiplicity regressions.
- `crates/liquidfun/src/particle.rs` - Curated particle child-module exports.
- `crates/liquidfun/src/lib.rs` - Curated crate-root particle proxy/contact exports.
- `crates/liquidfun/src/particle/lifetime.rs` - Tracked insertions now always dirty expiration order.
- `crates/liquidfun/src/particle/lifetime/tests.rs` - Readable minimized finite-then-infinite insertion regression.
- `crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs` - Independent tracked-insertion ordering model.

## Decisions Made

- Equal packed proxy tags retain their dense input order. The pinned comparator treats equal tags as equivalent, so this chooses a reproducible representative while preserving stable semantic output.
- Contact decisions remain synchronous and borrow-scoped. No persistent callback, reusable handle, or Phase 10 pair/triad generation path was added.
- Listener snapshots use current endpoint flags, matching the pinned pre-contact snapshot point, and invalidate repeated old occurrences one at a time.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tracked infinite insertion left a previously sorted finite expiration queue clean**

- **Found during:** Task 2 full-suite verification
- **Issue:** After a finite lifetime queue had been sorted, appending an infinite particle at elapsed tick zero stored expiration zero, which compared unchanged against the default lane value and failed to dirty the ordering. The next lifetime solve stopped at the appended infinite row and left an expired finite row live.
- **Fix:** Mark every tracked particle insertion dirty after its expiration is accepted, and retain the minimized finite-sort/infinite-insert/finite-expire sequence as a readable regression.
- **Files modified:** `crates/liquidfun/src/particle/lifetime.rs`, `crates/liquidfun/src/particle/lifetime/tests.rs`
- **Verification:** Focused regression and the 128-case independent lifecycle state machine pass; the opaque generated proptest seed was removed.
- **Committed in:** `64a8b0b`

**2. [Rule 1 - Bug] Independent lifecycle model retained the obsolete zero-expiration dirty predicate**

- **Found during:** Plan metadata pre-commit verification
- **Issue:** After production correctly dirtied every tracked insertion, the property model still dirtied only nonzero expirations. Appending a zero-expiration infinite row after a prior sort made the model reselect a pending older row while production correctly selected the source-ordered live row.
- **Fix:** Dirty the independent model after every tracked insertion, matching the insertion invariant proved by the readable lifetime regression.
- **Files modified:** `crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs`
- **Verification:** The minimized property sequence, 128-case lifecycle state machine, and full suite pass without an opaque seed artifact.
- **Committed in:** `b41c8d7`

***

**Total deviations:** 2 auto-fixed (2 bugs).
**Impact on plan:** The production fix and its independent model now agree on the lifecycle invariant required by contact-step integration without expanding the particle-contact surface.

## Issues Encountered

- Full-suite runs exposed the minimized production lifetime sequence and the model's stale insertion rule. Root-cause repairs resolved both; no generated opaque seed remains.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Stable particle contacts and ordered effects are ready for Phase 9 fixture/body contact, hook, journal, and rigid-coupling integration.
- Phase 10 pair/triad generation and particle solver passes remain explicitly excluded.

## Self-Check: PASSED

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
