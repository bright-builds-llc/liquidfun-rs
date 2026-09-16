---
status: verifying
trigger: "Phase11 run35137245396: particle-forces-and-statistics@1 physics_mismatch:checkpoint-0001 in debug and sanitizer"
created: 2026-09-16T19:01:57Z
updated: 2026-09-16T19:21:31Z
---

## Current Focus

hypothesis: C++ catalog capture omits parity-bearing particle primitive records.
test: Add actual upstream DrawParticles geometry to the canonical checkpoint, retaining stable semantic ownership and default style.
expecting: Existing strict comparator and focused red regression pass without projection or tolerance changes.
next_action: Independent reviewer inspects final mapping retirement; root reruns final repository gates and canonical acceptance. All local final lifecycle checks passed.

## Symptoms

expected: Native Rust matches pinned upstream 7f20402173fd143a3988c921bc384459c6a858f2 within existing tolerances.
actual: Both debug and sanitizer canonical comparisons fail particle-forces-and-statistics@1 checkpoint-0001.
errors: physics_mismatch:checkpoint-0001
reproduction: cargo xtask catalog compare --scenario particle-forces-and-statistics --timestep 0.016666668 --velocity-iterations 8 --position-iterations 3 --particle-iterations 1 --oracle-preset oracle-debug --session-profile one-shot --output json --commands auto
started: Observed on candidate dc29747e2d66d84e923088efa0b0d16785cc7df7, canonical run 35137245396 attempt 1.

## Plan

- [x] Preserve local failing comparison and isolate field-level mismatch.
- [x] Compare native implementation and pinned upstream; prove root cause with focused regression.
- [x] Apply minimal correction; run focused regression and local comparison.
- [x] Report evidence to root for independent review and canonical Linux acceptance.

## Eliminated

- hypothesis: Canonical Linux-only arithmetic divergence.
  evidence: Exact CLI reproduces on macOS ARM64 with the same resolved SHA.
- hypothesis: Particle force solver causes checkpoint-0001 divergence.
  evidence: All counts and simulation time match; native emits three circles and oracle emits none. Scenario has no simulation step.

## Evidence

- checked: Independent review and pinned b2ParticleSystem::SolveZombie.
  found: Destroyed handles are freed and reused. Catalog semantic mappings survive, so new particle capture can bind the oldest stale ID. Reading a stale handle after compaction is unsafe.
  implication: Retire mappings at the actual lifetime boundary; preserve pending identity until a positive step invokes compaction, including paused systems.
- checked: Production catalog lifecycle regression: two particles, mark first, zero-dt compact, positive step, create replacement.
  found: RED particle-lifetime-before.log reports replacement inherited destroyed identity. Pre-step retirement passes particle-lifetime-after.log. Extended tests cover both configured/group positive step paths, zero-dt compact/group step, and paused/unpaused systems.
  implication: Remove only zombie mappings before positive Step while handles are still live; never read freed handles. Current catalog exposes no finite lifetime, age-based capacity eviction, or substepping controls; future additions must revisit this boundary.

- checked: Parent-supplied retained provider logs and run identity.
  found: Debug and sanitizer fail the same scenario checkpoint; workflow uploaded zero artifacts.
  implication: Local semantic capture is required; local success cannot substitute for canonical Linux acceptance.

- checked: Exact native CLI reproduction and new catalog_round_trip integration.
  found: Same resolved SHA c5d6c8ddfd3630748c12b54f0471509044886aa04ece284ee56e36966639eadc; RustOnly particle circles while all counts/time match. Retained particle-diagnostic-before.json and particle-focused-before.log under target/phase15-candidate/oracle/.
  implication: Repair actual C++ checkpoint data, not comparator policy or native force solver.
- checked: Phase11 context D13/D14/D16/D26 and runner/catalog.rs comparison API.
  found: Primitive presence/order/style and geometry are canonical comparison fields. Physics-only projection is a separate Phase13 caller-specific path.
  implication: CLI projection would suppress required data rather than repair capture.
- checked: Pinned b2World::DrawParticleSystem and b2ParticleSystem range force/impulse APIs.
  found: DrawParticles receives actual position/radius and optional existing color buffer without allocation. Range APIs distribute one total vector; catalog adapter incorrectly called singleton APIs with the whole vector for every particle.
  implication: Capture through upstream draw callback; correct range dispatch independently after stepped red proof.
- checked: New C++ stepped catalog tests against independently stepped pinned b2World.
  found: Force fails in particle-range-before-02.log; impulse fails in particle-impulse-before.log. Both pass after range dispatch correction in particle-range-after.log.
  implication: Both adapter defects have causal red/green evidence without self-generated expected constants.
- checked: Local debug comparison and focused native suites.
  found: All three producer scenarios pass (rope 3, forces 4, AABB 3 checkpoints); catalog_native 6/6, catalog_round_trip 11/11, particle_forces_statistics 11/11; protocol suite passes.
  implication: macOS ARM64 diagnostic verification passes; canonical Linux acceptance remains pending.
- checked: Fresh final local oracle-debug, oracle-release, and oracle-asan-ubsan builds with updated adapter manifest.
  found: All three protocol suites pass, including independent stepped range tests and actual geometry/default/explicit-color capture tests. All nine preset/scenario comparisons pass, with 3/4/3 checkpoints for rope/forces/AABB. Logs and reports: target/phase15-candidate/oracle/particle-final2-*.
  implication: Final source works at the local Apple Clang 21/macOS ARM64 boundary; Linux Clang22 canonical acceptance is still required. Other non-particle debug primitive capture omissions were pre-existing and remain outside this bounded fix.
- checked: Final lifecycle revision, all three C++ presets with ASAN/UBSAN halt options, nine CLI comparisons, catalog_native and catalog_round_trip Rust suites.
  found: Three protocol suites pass including all four lifecycle variants; nine comparisons pass; 17 catalog Rust tests pass. Evidence: target/phase15-candidate/oracle/particle-lifetime-final-*.
  implication: Review finding repaired with pre-free retirement and causal lifecycle regression; independent rereview and canonical Linux remain parent-owned.

## Resolution

root_cause: tools/reference/src/catalog_checkpoint.cpp hardcodes an empty debug_primitives array although native canonical capture contains particle circles and both producers report their count as three. Phase11 D13/D14 requires primitive presence/order and geometry in canonical comparisons.
fix: Serialize actual upstream particle circles with stable owner IDs and shared presentation conventions; retain system/handle ownership; retire zombie mappings before each positive world step while handles are still live; dispatch contiguous particle force/impulse ranges once through pinned upstream range APIs. Keep helper separate and bind new source/test headers in adapter provenance manifest.
verification: Local red/green evidence preserved for capture, force/impulse distribution, and handle reuse. Final lifetime revision passes 17 catalog Rust tests, debug/release/sanitizer C++ protocol suites, all four lifecycle variants, and nine producer-scenario comparisons. The unchanged engine force/statistics suite previously passed 11 tests. cargo fmt and git diff --check pass. No tolerance changes, comparator projection, or native engine behavior changes. Root owns full commit gates and canonical Linux acceptance.
files_changed: [tools/reference/src/catalog_checkpoint.cpp, tools/reference/src/catalog_checkpoint.hpp, tools/reference/src/catalog_particle_draw.hpp, tools/reference/src/catalog_run_session.cpp, tools/reference/adapter-inputs.txt, tools/reference/CMakeLists.txt, tools/reference/tests/protocol_tests.cpp, tools/reference/tests/protocol_tests/catalog_particles.hpp, crates/liquidfun-differential/tests/catalog_round_trip.rs]
