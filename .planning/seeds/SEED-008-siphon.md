---
id: SEED-008
status: dormant
planted: 2026-10-14
planted_during: v1.6 Phase 45 (siphon feasibility spike)
trigger_when: A milestone improves particle incompressibility (pressure solve, stiffer water at metre-scale heads) or revisits physics curiosity scenes
scope: Small
---

# SEED-008: Siphon

## Why This Matters

The siphon was one of the six v1.6 curiosity scenes (SEED-004). The Phase 45 feasibility spike found no mechanism that passed, so the owner confirmed a drop on 2026-10-14, and these requirements came back to this seed:

- **SIPHON-01**: Player sees water flow from a higher tank over a crest and down into a lower tank through a primed tube, with the levels converging. Flow stops when the levels are equal. If the spike chose a stand-in mechanism, the caption credits it.
- **SIPHON-02**: Player can raise or lower the outlet tank, and the flow slows, stops or reverses to match.

What the spike measured (1,543 particles, r 0.05 m, primed 0.5 m tube over a 2.2 m crest; pass needs transfer for at least 2 s after the source surface is 0.3 m below the crest, a final level difference under 0.15 m, and no leaks or lost particles):

- All 14 configurations fail: plain water, VISCOUS 0.5/1.0/1.5, TENSILE | VISCOUS, and the stand-ins fake atmosphere, weighted lids and modeled tube pull.
- Root cause: LiquidFun water compresses strongly over metre-scale heads. The 2 m source column settles to about 1.2 m within 0.5 s, so the primed crest empties before a siphon can form.
- The viscous rule-A passes are the primed outlet leg draining, not a siphon (the source does not lose water, and the level difference stays about 1.5 m).
- Tube pull at α 1.0 ends level (difference 0.003 m) but fails rule A and behaves as a gated pump rather than a siphon.
- Native release cost 0.39-0.49 ms/step; no leaks or lost particles anywhere.

Never ship overflow labelled as a siphon, and never say "suction" or "atmospheric pressure" unless that is what is modeled.

## When to Surface

**Trigger:** A milestone improves particle incompressibility (pressure solve, stiffer water at metre-scale heads) or revisits physics curiosity scenes

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Water pressure or incompressibility behavior changes
- New curiosity or playground scenes are planned

Untried idea from the spike: a much shorter head with a pre-settled fill, so compression does not empty the crest.

## Scope Estimate

**Small**: one scene phase once a mechanism passes. Rerun the spike first with the same pre-recorded rules A and B.

## Restore

- Spike rules, results and the owner confirmation: `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-SIPHON-SPIKE.md`
- Commit A (spike added): `dcf7992b3081d2575623930a8386f51adf03ae93`
- Commit B (spike removed): `6366fe335983a23403eb0b67b51a517eac7737f5`
- Restore the harness: `git show dcf7992b3081d2575623930a8386f51adf03ae93 -- crates/liquidfun-wasm/src/siphon_spike.rs` (plus `crates/liquidfun-wasm/src/siphon_spike/geometry.rs`, `crates/liquidfun-wasm/src/siphon_spike/tests.rs`, and the `#[cfg(test)] mod siphon_spike;` line in `crates/liquidfun-wasm/src/lib.rs`)
- Related future idea: SIPHON-03 (Lift-inlet and Prime actions, two-tone tracer water) in `.planning/REQUIREMENTS.md`
