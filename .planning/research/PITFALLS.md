# Pitfalls Research

**Domain:** Adding scripted profiling and C++-matching optimizations to an existing native LiquidFun/Box2D-style Rust port that already has a semantic oracle, a Dam Break pair timer, and a correctness-before-speed policy
**Researched:** 2026-09-20
**Confidence:** HIGH for integration and policy pitfalls verified in this repository; MEDIUM for which single hot path currently dominates the ~300× Dam Break gap until a release-mode CPU profile exists

## Executive Warning

The ~300× Dam Break Medium gap (`docs/playground-dam-break-timing.md`) is a **canary**, not a public performance claim and not evidence that the engine “needs SIMD.” A same-order close (Rust wall time ≤ 3× pinned C++ on that pair) is the v1.2 gate. Phase 12 already defined a sealed 32-case method, but `reference/performance/manifest.toml` is empty (`reviewed_reports = []`), so the project currently publishes **no** accepted performance number. Hobby scope forbids required perf CI, Linux qualification, and crate publish.

Suggested phase order for planners: **measurement/audit first**, **then native hot-path optimization**, **then WASM playground sanity**. Do not invert that order.

## Critical Pitfalls

### Pitfall 1: Treating ~300× as a SIMD or Parallelism Problem

**What goes wrong:**
The first optimization work reaches for `std::simd`, `target-cpu=native`, Rayon, or default multi-thread stepping. The Dam Break pair stays hundreds of times slower, or a modest SIMD win is claimed while the real cost (extra clones, allocations, always-on checks, worse algorithms) remains. Determinism and differential tests break because contact/particle order changed.

**Why it happens:**
A 300× ratio looks like “languages” or “vectorization.” Training data and other physics ports talk about SIMD contact solvers. This repo’s own policy already forbids unproven acceleration as the baseline. A closer analog is the 2026 `box2d-rust` post-mortem: a 2.73× spinner outlier was `B2_VALIDATE` walking islands in **release**, not the collide path; gating validators to debug (matching C) collapsed the gap before SIMD was even interesting.

**How to avoid:**
Audit first. Compare Rust `World::step` to `b2World::Step` for **extra work**: per-step full-world clones, per-kernel `to_vec()` / `ParticleStorage` clones, `check_invariants()` on release paths, `catch_unwind` around the particle graph, scratch allocated every pass instead of reused, O(n) or O(n²) identity scans. Keep the scalar deterministic compatibility baseline. SIMD/parallel remain **explicit opt-in** after the scalar Dam Break pair is in the same order of magnitude and differential hashes still pass.

**Warning signs:**
Plans named “add SIMD particle solver” before a dated CPU profile; `rayon` appearing in `crates/liquidfun/Cargo.toml`; `-C target-cpu=native` or `-ffast-math` on canonical `oracle-release` / Rust `--release` pair builds; “Rust is just slower than C++” in commit messages with no function names.

**Phase to address:**
Measurement/audit (name the extra work). Native optimization may later add **opt-in** SIMD only with profile share evidence and unchanged solver-visible order.

---

### Pitfall 2: Blessing Exploratory Numbers as Phase 12 or a Public “Rust Is X% Slower” Claim

**What goes wrong:**
The first Dam Break sample (Rust ~70.6 s vs C++ ~0.23 s for 600 steps, ~304×) is copied into `reference/performance/manifest.toml`, README, playground copy, or a Phase 12 sealed report. Later faster runs cannot un-claim it. Reviewers treat an empty manifest as if it held a reviewed interval.

**Why it happens:**
The pair recipe is convenient and already documented. Phase 12 machinery (`just phase12-performance-paired`, `policy.json`, 32 sealed cases) looks like the official place to put a number. Hobby urgency wants a headline.

**How to avoid:**
Keep `reviewed_reports = []` unless a **separate**, owner-authorized Phase 12 promotion happens (it is **not** this milestone). Do not copy playground table cells into the manifest. Do not write “Rust is 300× slower” or “Rust is X% slower” as a product claim. A committed audit/notes doc may **cite** the canary with host, SHA, compilers, and “unreviewed local sample” in the same sentence. Public claim rules in `BENCHMARKING.md` still require one immutable manifest-listed report, workload-only wording, and interval math this milestone is not running.

**Warning signs:**
Diffs that fill `reviewed_reports`; README performance badges; copying `70594.221` into `protocol/benchmarks/` or `reference/performance/`; using the 32-case matrix hashes as if Dam Break Medium were one of those sealed cases (it is not).

**Phase to address:**
Measurement/audit (script + notes labeling). Re-check documentation during native optimization and WASM sanity so later deltas are not marketed as certified.

---

### Pitfall 3: Timing the Wrong Construction

**What goes wrong:**
New scripts time scene construction, particle insertion, warm-up, `capture_frame`, SVG/WASM copies, rAF catch-up, or `hooks.on_advance` work that C++ does not include. Or they time a different recipe (192 vs 1920 particles, different radius/spacing, `oracle-debug`, `cargo test` debug binary). The ≤ 3× gate becomes meaningless.

**Why it happens:**
The honest pair already exists and is easy to “improve” by shrinking the timed region or the scene. The Rust timer lives in the `liquidfun-wasm` crate (`dam-break-bench`) wrapping `SessionCore::advance`, while C++ calls `b2World::Step` only. Playground JS also calls `advance` then `captureFrame` under a 4-step catch-up cap.

**How to avoid:**
Reuse the locked Medium/Normal recipe: 1920 particles, radius `0.06324555`, spacing `0.101193`, gravity `(0, -10)`, `dt = 1/60`, velocity 8 / position 3 / particle 2, 60 untimed warm-up + 600 timed steps. Construction, insertion, warm-up, capture, and rendering stay **outside** the timer (already true in `crates/liquidfun-wasm/src/dam_break_bench.rs` and `tools/reference/src/playground_dam_break_bench.cpp`). If the Rust wrapper’s `on_advance` or `SessionCore` bookkeeping is suspected, **split** a `World::step`-only timer rather than silently dropping C++ work. Record git HEAD, OS/arch, CPU, logical cores, `rustc --release` vs `oracle-release` (no `-ffast-math`, no `-march=native`).

**Warning signs:**
Timed `SessionCore::create`; `--warmup 0` used as the reported gate; particle count ≠ 1920; C++ binary from `oracle-debug`; measuring `just web-player-smoke` or Pages FPS against C++; moving `rustc --version` **inside** the timed loop.

**Phase to address:**
Measurement/audit (script contract). Re-verify the same construction after each optimization batch.

---

### Pitfall 4: Using Profiled Timings as the ≤ 3× Authority

**What goes wrong:**
The gate is declared passing because a `samply` / Instruments / `cargo flamegraph` / `World::step_profiled` run looks “only 2.8×.” Unprofiled `--release` wall-clock is still 10×–300×. Or the inverse: a profiled run looks worse, so SIMD is added to please the profiler.

**Why it happens:**
Phase 12 already warns that diagnostic profiles may guide work but **profiled timings are never public timing authority**. `reference/performance/policy.json` sets `timing_authority: unprofiled_wall_clock`. Sampling, debuginfo, and `Instant::now` around every phase add overhead. `World::step` uses `DiagnosticStepProfiler::disabled()` (no `Instant::now`); `step_profiled` enables it. Sampling profilers typically add low-single-digit to ~7% overhead, but debuginfo and a `profiling` Cargo profile are **not** the gate binary.

**How to avoid:**
Scripts must produce **two** artifacts: (1) unprofiled pair table for the numeric gate, (2) CPU profiles (and optionally allocation traces) stored under a **gitignored** dated evidence directory. Never compare a profiled Rust binary to unprofiled C++, or profiled C++ to unprofiled Rust. Do not call `step_profiled` inside `dam-break-bench`. After a hot-path change, re-run **unprofiled** `just playground-dam-break-bench` (or the successor recipe) as the authority.

**Warning signs:**
One command that both records a flamegraph and prints the gate ratio; `CARGO_PROFILE_RELEASE_DEBUG=true` times quoted as the 3× result; `DiagnosticStepProfiler::enabled` on the bench path; Instruments Time Profiler milliseconds pasted into `docs/playground-dam-break-timing.md` as the C++ pair.

**Phase to address:**
Measurement/audit (script split). Native optimization must remeasure unprofiled after every candidate.

---

### Pitfall 5: Mixing Debug and Release (Rust or C++)

**What goes wrong:**
Developers profile `cargo run -p liquidfun-wasm --bin dam-break-bench` without `--release`, or compare Rust `--release` to `oracle-debug`, or leave always-on invariant walks that C++ compiles out of `NDEBUG`. The box2d-rust validator-in-release story is the warning: **this repo already has release-path checks** that C++ does not run every step.

**Why it happens:**
`debug_assert_eq!(self.check_invariants(), Ok(()))` is correctly debug-only in some storage methods, but `replace_solver_candidate` clones all of `ParticleStorage` and calls `check_invariants()` as a **release** `?` error. `check_identity_map` scans `dense_to_id[..dense].contains(&id)` (quadratic in particle count) whenever invariants run. `World::step` always clones bodies, fixtures, joints, particle systems, broad phase, and contact manager for limit rollback (`backup_step_limit_state`) **before** the step, including successful Dam Break steps. `run_particle_solver` clones body/system/group arenas **again**, then clones them a second time as candidates.

**How to avoid:**
Gate scripts to `--release` + `oracle-release` and fail closed if `cfg!(debug_assertions)` is true in the timed binary. Audit `check_invariants`, `ParticleStorage::clone`, arena `clone`, and `to_vec()` on the **release** Dam Break step. Match C++: expensive structure validators belong in debug or in tests, not in the scalar release hot path, **without** dropping fail-closed API errors that are part of the public contract. Do not “fix” debug slowness and call the milestone done.

**Warning signs:**
`opt-level = 0` times; `oracle-debug` C++ ms/step near Rust; flamegraphs dominated by `check_invariants` / `clone` / `Vec::to_vec` that disappear from the write-up because “we’ll SIMD the pressure kernel”; CI job running the pair under `cargo test`.

**Phase to address:**
Measurement/audit (prove debug vs release and name clone/invariant cost). Native optimization (remove extra release work; keep debug asserts).

---

### Pitfall 6: Comparing WASM Playground Frames to C++ (or Using WASM as the Native Gate)

**What goes wrong:**
Pages FPS, `acceptedStepCount` catch-up stutter, or `wasm-bindgen` `advance`+`captureFrame` is compared to `playground-dam-break-bench` C++. Native hits ≤ 3× while the playground still stutters (or the reverse). Someone times `wasm32-unknown-unknown` with `Instant::now` and treats traps or `performance.now` as the Dam Break pair.

**Why it happens:**
The crate that owns the native bench is named `liquidfun-wasm`. The playground’s realtime factor in the exploratory doc is a **reading aid** for stutter, not a C++ comparison. v1.1 already documented that the native profiler uses `Instant::now()` and must not be imported into the WASM bridge. JS caps catch-up at 4 steps/frame and copies typed arrays every frame.

**How to avoid:**
Native vs C++ is **only** the same-host scalar `--release` / `oracle-release` Dam Break pair. WASM/playground is a **post-gate sanity check**: record whether the hosted scene feels improved, with host/browser notes, and **never** a ratio versus C++. Do not add `Instant` profiling inside the `cdylib`. Do not lift the 4-step cap to “make WASM look faster.”

**Warning signs:**
Tables with a `wasm` row next to `pinned_cpp`; using Chromium `just web-player-smoke` duration as evidence of 3×; `cargo build --target wasm32-unknown-unknown --bin dam-break-bench`; README “playground is N× C++.”

**Phase to address:**
WASM playground sanity check. Measurement scripts must refuse a WASM target for the C++ pair.

---

### Pitfall 7: Speeding Up by Breaking the Semantic Oracle

**What goes wrong:**
A hot-path rewrite changes contact generation order, particle compaction, island solve order, or skip conditions. Dam Break **looks** fine (water still falls). Differential scenarios, determinism hashes, or Phase 9/10 particle probes fail—or worse, only fail on another scene (Fountain, Jelly Drop, Color Mixer).

**Why it happens:**
Correctness-before-speed is the standing policy, but a 300× fire invites “temporary” algorithmic shortcuts: spatial hash with `HashMap` iteration, sorting contacts differently, skipping empty material flags incorrectly, dropping transactional rollback and then mutating in place with different failure semantics.

**How to avoid:**
Every optimization candidate that touches `crates/liquidfun` must re-run the relevant differential/determinism suite (particle + rigid probes that Dam Break actually exercises, plus a spot-check of other catalog scenes). Treat a physics mismatch as a **correctness failure**, never a timing sample (Phase 12 already encodes this). Keep fail-closed construction errors; do not swap them for `unwrap` in the solver. Prefer matching upstream pass order (`PassId` graph) with cheaper storage, not a new pass order.

**Warning signs:**
“Tests still pass” meaning only `dam_break_bench` unit tests; ignored `liquidfun-differential` failures; comments that HashMap order “shouldn’t matter for water”; skipped `particle_iterations` loops.

**Phase to address:**
Native optimization (gate each candidate). Measurement/audit should record a baseline differential green SHA before changing kernels.

---

### Pitfall 8: HashMap Iteration or Default Rayon in Solver-Visible Order

**What goes wrong:**
A “faster neighborhood” uses `HashMap`/`HashSet` as the iteration order for contacts, proxies, or islands. Default Rayon `par_iter` on particle contacts. Results differ by run, by stdlib, or by thread count. The oracle cannot match. The ≤ 3× number is not reproducible.

**Why it happens:**
Rust collections are the path of least resistance. Stack research and Phase 8 already banned `HashMap`/`HashSet` in solver-visible order. Rayon is not in the production crate today; adding it as a default feature would violate “single-threaded baseline; explicit experimental mode only after parity.”

**How to avoid:**
Neighborhoods, contacts, pairs, triads, islands, and proxy lists stay order-preserving (`Vec`, indices, explicit sorts defined by compatibility policy). `AssociationMap`’s `HashMap` is for user data, not solver iteration—do not start iterating it during `World::step`. If parallelism is ever prototyped, it is a **named opt-in** with the scalar path still the Dam Break gate. Reject `-C target-cpu=native` on the pair.

**Warning signs:**
`use rayon::` under `crates/liquidfun`; `HashMap` in `particle/solver` or `contact_manager` hot loops; tests that pass only with `RUST_TEST_THREADS=1` because of races; Dam Break ratio that jitters tens of percent between runs on a quiet machine.

**Phase to address:**
Native optimization (forbid in plans). Measurement/audit can `rg` the hot path for `HashMap`/`par_iter` before trusting a profile.

---

### Pitfall 9: Reviving Phase 12’s Sealed 32-Case Matrix, Linux Host, or Perf CI as the Hobby Gate

**What goes wrong:**
v1.2 is blocked on `scripts/phase12-performance.sh paired`, five calibrated runs × 32 cases, `PERFORMANCE_CONTROLLED_HOST_IDENTITY`, or `.github/workflows/performance.yml`. Or the empty manifest is treated as a failed release audit. Hobby completion never happens.

**Why it happens:**
Phase 12 method is complete, discoverable (`just phase12-performance-*`), and looks more “serious” than a one-scene pair. Older plans required a dedicated Linux x64 runner. `PROJECT-SCOPE.md` (2026-09-16) removed that as a completion blocker.

**How to avoid:**
Dam Break Medium native pair **is** the numeric gate. Phase 12 remains available as an **optional** strict profile. Do not schedule automatic expensive performance workflows. Do not require Linux. Local macOS + existing macOS Cargo CI is enough. Do not implement `cargo xtask performance optimization-check` admission as if it updated public claims (it does not; even Phase 12 says passing that command does not fill the manifest).

**Warning signs:**
Milestone acceptance text citing 32/32 sealed cases; new required GitHub Actions performance job; secrets for controlled-host identity in ordinary PRs; “cannot optimize until Phase 12 calibrate is green.”

**Phase to address:**
Measurement/audit (define the local pair + gitignored evidence, explicitly out of scope the sealed matrix). Reaffirm in later phases.

---

### Pitfall 10: Leaving Always-On Transactional Full-World Clones While “Optimizing” Kernels

**What goes wrong:**
Pressure/damping kernels get micro-optimized, but every `World::step` still clones the entire rigid + particle world for rollback, and every particle solve clones arenas twice more, plus per-pass velocity `to_vec()` and `replace_solver_candidate` (clone all lanes + `check_invariants`). C++ LiquidFun mutates in place with member scratch. The ratio stays two orders of magnitude off. Profiles that only sample the pressure kernel mislead.

**Why it happens:**
Clone-on-write was a correctness shortcut for limit-exceeded rollback and particle coupling errors (`execution.rs`, `particle_coupling.rs`). It is the right **recovery** model, the wrong **happy-path** implementation. Material kernels copy lanes because `&mut storage` cannot coexist with contact slices—an aliasing design issue, not a physics issue.

**How to avoid:**
Audit must rank **bytes copied per step** and **invariant calls per step** alongside CPU samples. Optimization should: take the limit-rollback snapshot **lazily** (only on the failing path, or copy-on-write only dirty subsystems); reuse scratch buffers; mutate velocity/position lanes in place or via split borrows; keep `check_invariants` on debug or on transactional **commit of mutations**, not on every successful Dam Break integrate. C++-matching means matching **work**, not matching C++ class layout.

**Warning signs:**
Flamegraphs with `clone` / `memcpy` / `check_invariants` / `backup_step_limit_state` near the top that the plan ignores; “we inlined `pressure`” with no change in wall ms; allocations scaling with 1920 × 2 particle iterations × number of material passes.

**Phase to address:**
Measurement/audit (quantify clone/alloc). Native optimization (remove extra happy-path work). WASM sanity will inherit native step cost; do not “fix” WASM copies first.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Clone entire `World` / `ParticleStorage` before every step for rollback | Easy exact restore on `StepError::LimitExceeded` | Copies dominate Dam Break; 300×-class self-inflicted cost | Never on the successful-step path; snapshot only if a limit failure must restore |
| `to_vec()` then `replace_solver_velocities` (with O(n) change scan) to dodge aliasing | Safe kernels without `unsafe` or refactor | Extra alloc + walk per material/pressure pass × particle iterations | Temporary during audit; not after native optimization for Dam Break hot kernels |
| Always-on `check_invariants()` in release on candidate commit | Catches lane bugs early | Quadratic identity scan on 1920 particles, every commit | Debug/tests; not every Dam Break step |
| `catch_unwind` around find-pairs and particle solve | Poison the world instead of UB across hooks | Unwind tables + extra frames every step | Keep for hook panic isolation; do not add more layers while chasing 3× |
| Using `SessionCore::advance` as the only timed API | One recipe shared with the playground | Wrapper/hook/bookkeeping can be blamed on “physics” | Allowed if documented and C++ still times only `Step`; split if profiles show wrapper cost |
| Filling Phase 12 manifest from one Dam Break laptop run | Looks like official evidence | False public claim; empty-manifest contract broken | Never |
| Default Rayon / SIMD to “catch C++” | Fast to type | Order + FP changes; policy violation | Explicit opt-in **after** scalar ≤ 3× and oracle still green |
| Required Linux perf CI | Matches historical Phase 12 | Blocks hobby milestone | Optional strict profile only |
| Timing WASM rAF as the gate | Visible to users | Wrong comparison; Instant/WASM traps | Post-gate qualitative check only |

## Integration Gotchas

Common mistakes when connecting new profiling/optimization work to **this** repo’s existing timers, oracle, and playground.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `just playground-dam-break-bench` | Treat the printed table as Phase 12 / public claim | Keep it unreviewed diagnosis; successor scripts write gitignored reports + a labeled notes doc |
| `just phase12-performance-paired` | Use it as the v1.2 definition of done | Leave optional; Dam Break pair is the hobby gate |
| `World::step` vs `World::step_profiled` | Time `step_profiled` for the 3× ratio | Gate uses unprofiled `step`; profiles are separate |
| Rust `SessionCore` vs C++ `b2World` | Time capture/rendering on one side | Both sides: construction + warm-up out; only step in |
| `liquidfun-wasm` crate name | Build `wasm32-unknown-unknown` for the pair | Native `--release --bin dam-break-bench` vs `oracle-release` extra target |
| `cargo xtask playground dam-break-bench` order | All Rust samples, then all C++ (thermal/order bias) | For a 300× gap, order is secondary; when closing toward 3×, interleave or alternate first-engine like Phase 12 `interleaved_rust_cpp` |
| Semantic oracle | Skip differential after a “perf-only” edit | Mismatch is not a timing sample; re-run particle/rigid probes |
| Playground JS clock | Compare `acceptedStepCount` stutter to C++ ms/step | Native pair first; WASM sanity records playground behavior without a C++ ratio |
| Instruments / samply / flamegraph | Quote sampled time as wall authority | Save traces beside an unprofiled pair from the same SHA/host |
| `reference/performance/manifest.toml` | Paste exploratory ms | Leave `reviewed_reports = []` |
| macOS debuginfo profiles | Compare `profile.profiling` (release+debug) to C++ `-O2` without debug | Record profile flavor; gate binary stays ordinary `--release` |
| Evidence directory | Commit `.trace` / `samply.json` / flamegraph SVG | Gitignore dated evidence (`target/` already covers `/target/`; do not put traces under `docs/` or `reference/performance/`) |

## Performance Traps

Patterns that work at small scale but fail as usage grows. Thresholds are **this** project’s Dam Break Medium (1920 particles, 2 particle iterations) unless noted.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full-world `clone` every `step` | `backup_step_limit_state` / arena clone at top of profiles; ratio ≫ 3× | Lazy rollback; copy only on failure | Already broken at 1920 particles (~300× canary) |
| Per-kernel velocity/position `to_vec` | Alloc churn; `replace_solver_velocities` change-scan | In-place lanes or reused scratch | Each extra copy × 2 iterations; worse at 8k particles |
| Release `check_invariants` with O(n²) identity `contains` | Time grows faster than particle count | Debug-only or linear maps | Noticeable well below 1920 if called every pass |
| `HashMap` neighborhoods | Run-to-run Dam Break jitter; oracle mismatch | Ordered vectors / explicit sort | First nondeterministic seed or stdlib change |
| Default `par_iter` | Different results vs C++; CI flakes | Scalar default | Immediately vs oracle; also thermal noise |
| Profiled vs unprofiled mixed | 3× “pass” that fails on `just playground-dam-break-bench` | Split scripts; unprofiled authority | As soon as the gap is small enough that overhead matters (~3× target) |
| Debug Rust vs release C++ | Fake 1000×+ | Fail closed on debug timed binary | Every local `cargo run` without `--release` |
| WASM capture every frame | Playground stutter after native 3× | Post-gate check; copied frames already bounded | Browser path; not the C++ pair |
| Criterion / Phase 12 32-case as Dam Break substitute | Wrong workload (128/1024/8192 sealed hashes ≠ playground recipe) | Keep Dam Break pair as canary | Always, if used as the 3× gate |
| Sequential 70 s Rust then 0.2 s C++ as the only method near 3× | Order/thermal bias of a few percent | Interleave when the ratio is O(1) | When claiming ≤ 3×, not when diagnosing 300× |
| Optimizing only Dam Break water | Fountain/Jelly/Color Mixer regress | Spot-check other scenes after shared hot-path edits | After the first shared solver change |

## Security Mistakes

Domain-specific issues for this performance milestone (not generic web OWASP).

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing Instruments/samply traces with absolute home paths | Leaks username/machine layout | Gitignored evidence dir; redact paths in the committed notes doc |
| Wiring hobby PRs to `.github/workflows/performance.yml` + `PERFORMANCE_CONTROLLED_HOST_IDENTITY` | Secret required; accidental public hardware fingerprint | Do not enable as required CI; keep workflow_dispatch optional |
| Copying host CPU strings into README as a “certified” result | Over-claim; stale identity | Notes doc: unreviewed, SHA-bound; no manifest promotion |
| Logging full scenario JSON with user playground gestures into committed reports | Unnecessary PII-ish pointer paths | Pair timer has no pointer path; keep it that way |

## UX Pitfalls

Honesty pitfalls for visitors and future contributors (not a product UI redesign).

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| README / Pages copy “Rust is 300× slower than C++” | Sounds like a product benchmark; ages instantly | Point to unreviewed canary doc; update after the gate with the same caveats |
| Declaring the playground “realtime” because native ≤ 3× C++ | WASM still catch-up-stutters | WASM sanity records playground honestly |
| Sealed Phase 12 language in hobby release notes | Contributors chase Linux runners | “Local Dam Break pair; manifest still empty” |
| Silent scene-size change (192 vs 1920) in the table | False “win” | Lock Medium/Normal literals on both sides (already tested in `dam_break_bench.rs`) |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Pair scripts:** Recipe exists — verify they still exclude construction/warm-up/capture, use `--release` + `oracle-release`, lock 1920 particles, and write **unprofiled** wall times
- [ ] **CPU profiles:** Flamegraph committed or shown in chat — verify a matching **unprofiled** pair from the same git SHA and host; profiles are gitignored, not the gate
- [ ] **Audit notes:** Hot functions named — verify they are not SIMD speculation; extra clones/allocs/checks/algorithm differences are listed with evidence
- [ ] **3× gate:** A profiled or debug run looks close — verify ordinary `just playground-dam-break-bench` (or successor) unprofiled ratio
- [ ] **Manifest:** Someone “recorded performance” — verify `reference/performance/manifest.toml` still has `reviewed_reports = []` and no copied 70594.221
- [ ] **Oracle:** Physics “still looks like Dam Break” — verify differential/determinism on the changed SHA
- [ ] **Order/parallel:** Faster water — verify no `HashMap` solver iteration and no default Rayon
- [ ] **WASM:** Playground “checked” — verify it is **not** vs C++; native gate already passed; Instant profiler not in the cdylib
- [ ] **Phase 12 revival:** 32-case script green — verify it is **not** required for hobby v1.2
- [ ] **Other scenes:** Dam Break improved — spot-check Fountain / Jelly Drop / Color Mixer / Water Wheel / Float or Sink, not a second sealed matrix
- [ ] **Debug vs release:** Invariants “fixed” — verify `debug_assert` vs release `check_invariants` / clone paths separately
- [ ] **Crate name:** `liquidfun-wasm` bench ran — verify native target, not `wasm32-unknown-unknown`

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| SIMD/Rayon/HashMap landed first | HIGH | Revert to scalar ordered baseline; restore differential green; restart from audit |
| Exploratory numbers copied into manifest or README | MEDIUM | Revert manifest to empty `reviewed_reports`; rewrite docs as unreviewed canary; do not “amend” a fake reviewed report |
| Gate measured debug, WASM, or profiled binary | LOW | Re-run the locked unprofiled pair; discard the bad table from any claiming doc |
| Differential broken by a perf patch | HIGH | Stop timing; restore failing seed/request; do not keep the speedup; separate attempt directory if an evidence run failed |
| Always-on clones left in place | MEDIUM | Profile `clone`/`check_invariants`; move snapshot to failure path; remeasure unprofiled Dam Break |
| Phase 12/Linux CI revived as blocker | LOW | Re-read `PROJECT-SCOPE.md`; drop required jobs; keep Dam Break pair |
| WASM compared to C++ | LOW | Delete the ratio; record playground-only notes after native gate |
| Thermal/order bias near 3× | LOW | Interleave engines; repeat a few paired samples; do not change physics |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls. Phase numbering continues after 21; names below are the v1.2 sequence the planner should keep.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. SIMD/parallel first | Measurement/audit (ban as first work); Native optimization (opt-in only later) | No SIMD/Rayon in default `liquidfun` features; audit names extra work |
| 2. Blessing numbers / “Rust is X% slower” | Measurement/audit | `manifest.toml` unchanged empty reports; notes labeled unreviewed |
| 3. Wrong timing construction | Measurement/audit | Same literals as C++ bench; warm-up/capture outside timer |
| 4. Profiled vs unprofiled | Measurement/audit | Two artifacts; gate command does not enable `step_profiled` or samply |
| 5. Debug vs release | Measurement/audit + Native optimization | Scripts fail closed without `--release`/`oracle-release`; invariant cost named |
| 6. WASM vs C++ | WASM sanity (after native gate); scripts refuse wasm pair | No `pinned_cpp` row beside WASM; Instant not in cdylib profiler |
| 7. Breaking differential tests | Native optimization | Differential/determinism green on the optimized SHA |
| 8. HashMap / default Rayon | Native optimization | Source scan of solver-visible order; Dam Break ratio stable |
| 9. Phase 12 matrix / Linux perf CI as gate | Measurement/audit (scope) | Acceptance text cites Dam Break pair + hobby CI only |
| 10. Happy-path full-world clones | Measurement/audit (quantify); Native optimization (remove) | Profile/alloc evidence; unprofiled ratio moves toward ≤ 3× |
| Spot-check other scenes | Native optimization | Shared hot-path PR lists other scene timings, not a sealed 32-case claim |
| Playground stutter honesty | WASM sanity | Notes: native pair result + qualitative/WASM-only timing, no C++ ratio |

## Sources

- Repository policy: [BENCHMARKING.md](../../BENCHMARKING.md) (exploratory pair must not enter `reference/performance/manifest.toml`; unprofiled wall-clock authority; empty reviewed-report manifest)
- Canary sample: [docs/playground-dam-break-timing.md](../../docs/playground-dam-break-timing.md) (2026 local macOS aarch64 pair; unreviewed)
- Hobby scope: [PROJECT-SCOPE.md](../../PROJECT-SCOPE.md) (no required Linux host or perf CI)
- Milestone decisions: [`.planning/PROJECT.md`](../PROJECT.md) (v1.2 Dam Break ≤ 3×; scalar baseline; WASM post-gate)
- Phase 12 authorities: `reference/performance/manifest.toml` (`reviewed_reports = []`), `reference/performance/policy.json` (`timing_authority: unprofiled_wall_clock`, `allowed_optimization_mode: release_scalar`)
- Timing construction: `crates/liquidfun-wasm/src/dam_break_bench.rs`, `tools/reference/src/playground_dam_break_bench.cpp`, `tools/xtask/src/playground.rs` (`cargo run --release -p liquidfun-wasm --bin dam-break-bench`)
- Extra happy-path work (audit targets): `crates/liquidfun/src/world/step/execution.rs` (`backup_step_limit_state` every step), `crates/liquidfun/src/world/particle_coupling.rs` (arena clones), `crates/liquidfun/src/particle/storage/runtime.rs` (`replace_solver_candidate` clone + `check_invariants`; `replace_solver_velocities` O(n) scan), `crates/liquidfun/src/particle/storage/lifecycle.rs` (`check_identity_map` quadratic `contains`), `crates/liquidfun/src/particle/solver/material.rs` and `pressure.rs` (`to_vec()` per kernel), `crates/liquidfun/src/world/observation/profile.rs` (`step` disabled vs `step_profiled`)
- Solver-order policy: repository stack notes forbidding `HashMap`/`HashSet` iteration and default parallel stepping; no `rayon` in production `Cargo.toml` as of this research
- Analogous port post-mortem: [box2d-rust 1.3.0 performance notes](https://docs.rs/crate/box2d-rust/latest) (2026-07-19): release-mode C validators, not SIMD, caused a 2.73× outlier; SIMD came after (HIGH confidence for the *pattern*, not for this repo’s 300× cause)
- Profiler overhead: [samply README](https://github.com/mstange/samply) (profile release + debuginfo, not as timing authority); [flamegraph-rs/flamegraph](https://github.com/flamegraph-rs/flamegraph); [xctrace(1)](https://keith.github.io/xcode-man-pages/xctrace.1.html)
- v1.1 WASM Instant pitfall: `.planning/research/v1.1/PITFALLS.md`

---
*Pitfalls research for: native LiquidFun performance closing (audit, scripted profiling, Dam Break ≤ 3× C++)*
*Researched: 2026-09-20*
