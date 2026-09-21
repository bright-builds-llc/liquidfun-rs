---
status: resolved
trigger: "Investigate issue: dam-break-wall-velocity — web playground dam-break errors after ~1 min; particles embed in walls, extreme velocities, thrash. Find root cause and fix."
created: 2026-09-21T22:46:00Z
updated: 2026-09-21T23:10:00Z
symptoms_prefilled: true
goal: find_and_fix
---

## Current Focus

hypothesis: CONFIRMED — SolveForce cleared pending flag but left force buffer values; next collision/barrier `+=` stacked already-applied forces every iteration
test: dam_break_stays_finite regression + force_consume_zeros unit test
expecting: finite bounded velocities; no wall embedding
next_action: none — user confirmed the web playground dam-break stays stable

## Symptoms

expected: Dam-break water stays outside solid walls and rigid bodies, velocities stay physically bounded, and the playground keeps stepping without a scene error.
actual: After roughly a minute of the dam simulation, the run errors out. Separately (likely related), some particles embed inside walls, acquire extreme velocities, and are thrown around wildly.
errors: Exact message was not pasted. Strong unconfirmed lead is crates/liquidfun-wasm/src/frame.rs rejecting non-finite copied-frame values (FrameError::NonFiniteValue).
reproduction: Run web playground dam-break scene (web/ + crates/liquidfun-wasm scene dam_break). Prefer headless Rust loop stepping same world long enough to catch failure.
started: Reported 2026-09-21 against current workspace. Recent particle cap raise to 10240 and ~10x denser playground scenes may stress an existing contact bug.

## Eliminated

- hypothesis: Copied-frame particle cap (10240) itself causes the scene error
  evidence: Headless World::step repro fails with wall embedding / extreme velocity at step 135 with 1920 particles, well under the frame cap; failure is physics state not frame packing
  timestamp: 2026-09-21T23:05:00Z

- hypothesis: Polygon ray-cast / SolveCollision algorithm differs from upstream half-space clip
  evidence: Rust polygon::ray_cast matches b2PolygonShape::RayCast structure; after force-buffer fix, dam-break runs 3600 steps with max_speed≈0.028 and no embedding — collision path is sufficient once velocities stay bounded
  timestamp: 2026-09-21T23:50:00Z

## Evidence

- timestamp: 2026-09-21T23:05:00Z
  checked: knowledge-base.md keyword scan for particle/velocity/wall/collision/dam-break/NonFinite
  found: no matching entries
  implication: treat as novel investigation

- timestamp: 2026-09-21T23:05:00Z
  checked: headless dam_break with SessionCore step config (1/60, 8, 3, particle_iterations=2)
  found: FAIL at step 135 particle 251 — position x=-8.065 (through left wall), v≈(-14.45,-1.50), speed≈14.5 vs critical≈7.6
  implication: tunneling + super-critical velocity are real and early

- timestamp: 2026-09-21T23:20:00Z
  checked: C++ PrepareForceBuffer / SolveForce vs Rust preparation::force / clear_pending_system_force
  found: C++ memset-zeroes force buffer when m_hasForce goes false→true before next accumulation; Rust force() cleared only the pending bool and left force lane values; collision_candidate / barrier_candidate do forces[i] += …
  implication: stacked reaction forces restore/amplify pre-collision velocity every iteration after LimitVelocity, driving supercritical speeds and wall tunneling

- timestamp: 2026-09-21T23:50:00Z
  checked: after zeroing forces in clear_pending_system_force
  found: 600-step regression passes (max_speed≈1.32); LIQUIDFUN_DAM_BREAK_STEPS=3600 passes (max_speed settles to ≈0.028); force unit tests pass
  implication: root cause fixed; original ~1 min symptom cleared in headless repro

- timestamp: 2026-09-21T23:10:00Z
  checked: user ran the local web playground dam-break after the force-buffer fix
  found: user confirmed the scene no longer errors and the wall-thrash symptom is gone
  implication: human-verify checkpoint passed; session can be archived

## Resolution

root_cause: After SolveForce, Rust cleared `pending_system_force` but did not zero the particle force buffer. Upstream LiquidFun zeroes via PrepareForceBuffer on the next false→true transition. Collision/barrier then did `forces[i] += reaction`, stacking already-applied forces every particle iteration. SolveForce re-applied the growing buffer, velocities exceeded LimitVelocity’s effective bound across iterations, and particles tunneled thin basin walls (then pressure ejected them outward).
fix: Zero the force buffer when consuming the pending marker in `ParticleStorage::clear_pending_system_force` (called from `preparation::force`). Added unit coverage for non-stacking consume and a dam-break headless regression.
verification: Before — `cargo test -p liquidfun-wasm --lib dam_break_stays_finite` failed at step 135 with embedded particle / speed≈14.5. After — same test (600 steps) passes max_speed≈1.32; `LIQUIDFUN_DAM_BREAK_STEPS=3600` passes through a full simulated minute. User confirmed the local web playground dam-break on 2026-09-21.
files_changed:
  - crates/liquidfun/src/particle/storage/runtime.rs
  - crates/liquidfun/src/particle/solver/preparation.rs
  - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
  - .planning/debug/dam-break-wall-velocity.md
