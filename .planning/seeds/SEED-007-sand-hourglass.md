---
id: SEED-007
status: dormant
planted: 2026-10-14
planted_during: v1.6 Phase 45 (hourglass feasibility spike)
trigger_when: A milestone adds granular-material behavior to the engine (friction-like particle rule or public particle flag editing) or revisits physics curiosity scenes
scope: Small
---

# SEED-007: Sand hourglass

## Why This Matters

The sand hourglass was one of the six v1.6 curiosity scenes (SEED-004). The Phase 45 feasibility spike found no particle rule that piles, so the owner confirmed a drop on 2026-10-14, and these requirements came back to this seed:

- **SAND-01**: Player sees sand pour through a narrow neck and build a peaked pile, with a measured slope of at least 20°, in the lower bulb.
- **SAND-02**: Player can flip the hourglass. The glass rotates visibly, or flips instantly as a fallback, and the sand runs again.

What the spike measured (564 particles, r 0.05 m, 0.5 m neck; pass needs an edge-relative median slope of at least 20° over 25-30 s, ±5°, no jam, and at least 15° above same-geometry water):

- Best measured slope: `c1-stick-slip-v0.2-k0.5` (POWDER plus stick-slip velocity damping, v_slip 0.2, k 0.5), median 3.85° (3.83-3.87°), water margin 3.09°. Every configuration's median was between 0.24° and 3.85°. Short heaps of up to about 28° formed during the pour, but they slumped within a second or two.
- Freezing settled grains to `WALL` keeps them perfectly still without any engine change, but the 30-step rest test fires only after the deposit has already flattened.
- 2 × 2 rigid clumps flow through the neck without jamming and settle at 2.7°.
- Flip cost: batched destroy-and-rebuild about 2.1 ms (1.96-2.28 ms), per-particle position and velocity edits about 0.13 ms, and 36.5 ms when rebuilding 152 rigid clumps. Particle order and count were preserved in every flip.

The flip works. What is missing is a rule that holds a slope.

## When to Surface

**Trigger:** A milestone adds granular-material behavior to the engine (friction-like particle rule or public particle flag editing) or revisits physics curiosity scenes

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- The engine gains a friction-like or granular particle behavior
- A public `World::set_particle_flags` (or similar flag editing) is added
- New curiosity or playground scenes are planned

Untried ideas from the spike: rules that act during the pour, such as freezing after a shorter rest test, before the heap slumps.

## Scope Estimate

**Small**: one scene phase once a rule that piles exists. Rerun the spike first, and keep the pre-recorded edge-relative pass rule with its water-control guard.

## Restore

- Spike rules, results and the owner confirmation: `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-HOURGLASS-SPIKE.md`
- Commit A (spike added): `09ab08395ee8ab8ad7adc8a2460d1c1f22f34d72`
- Commit B (spike removed): `effc9c98f60062a98eebb38ed65587ede7b34f36`
- Restore the harness: `git show 09ab08395ee8ab8ad7adc8a2460d1c1f22f34d72 -- crates/liquidfun-wasm/src/hourglass_spike.rs` (plus `crates/liquidfun-wasm/src/hourglass_spike/flip.rs`, `geometry.rs`, `hooks.rs`, `metrics.rs`, and the `#[cfg(test)] mod hourglass_spike;` line in `crates/liquidfun-wasm/src/lib.rs`)
- Related future idea: SAND-03 (neck-width jamming control and a sand-versus-water flow comparison) in `.planning/REQUIREMENTS.md`
