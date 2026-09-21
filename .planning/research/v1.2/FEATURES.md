# Feature Research

**Domain:** Native LiquidFun performance closing (audit, scripted profiling, Dam Break Medium ≤ 3× C++)
**Milestone:** v1.2 Native Performance Closing
**Researched:** 2026-09-20
**Confidence:** HIGH for table stakes, anti-features, and existing bench/oracle dependencies; MEDIUM for which shared hot paths actually dominate the ~300× Dam Break gap until a profiled audit lands

## Scope

This milestone is a **developer-facing performance-closing loop**, not a new physics product, playground catalog, or public benchmark claim.

“v1.2 launch” means: native Dam Break Medium is in the same order of magnitude as pinned C++ on one same-host scalar pair, the hunt is evidenced, and remaining delta is described honestly. It does **not** mean Phase 12 sealed public numbers, Rust ≈ C++ everywhere, or WASM ≈ C++.

Frame every new capability as something a developer or playground visitor can do. Existing v1.0/v1.1 work is a dependency, not a new feature.

### Already built (do not re-scope as new)

| Capability | Where it lives | How v1.2 uses it |
| --- | --- | --- |
| Native Rust engine (math, collision, rigid, joints, particles) with safe handles | `crates/liquidfun` | Shared particle/rigid hot paths are the optimization surface |
| C++ oracle + semantic differential harness | `cargo xtask upstream`, `liquidfun-differential` | Correctness gate after each admitted change; C++ is the pair partner, not the runtime |
| Exploratory Dam Break pair timer | `just playground-dam-break-bench`, `docs/playground-dam-break-timing.md` | Numeric canary (extend with dated reports + profiles; do not treat the first sample as a claim) |
| Six-scene WASM playground | GitHub Pages + `just web-player-smoke` | Post-gate sanity only; no new scenes |
| Phase 12 sealed benchmark *method* | `BENCHMARKING.md`, empty `reference/performance/manifest.toml` | Reuse admission *ideas* (profiled timings are not authority; scalar `--release` vs `oracle-release`). Do not fill the reviewed-report manifest |

First recorded Dam Break Medium sample (unreviewed, one host): native Rust ~70.6 s vs pinned C++ ~0.23 s for 600 timed steps (~300×). That table is diagnosis, not a public claim.

## Feature Landscape

### Table Stakes (Users Expect These)

Features developers assume exist when closing a huge C++ vs Rust physics gap. Missing these = the milestone feels incomplete or untrustworthy.

| Feature | Why Expected | Complexity | Notes |
| --- | --- | --- | --- |
| **Developer can audit shared particle/rigid stepping against the C++ oracle** and get a committed notes doc that names hot functions, extra per-particle work, per-step allocations, `--release` checks, and algorithm/shape differences, plus the current Dam Break Medium wall-time ratio | A ~300× gap is almost never “Rust bounds checks.” Mature ports land near 1.2–2× after removing extra work. Without named functions, later “optimizations” are guesses | HIGH | Compare Rust `World::step` / particle neighborhood, contacts, solvers, and rigid coupling to pinned LiquidFun. Hunt quadratic validators, `Vec` rebuilds, clones, HashMap-ordered work, and identity maps that C++ does not pay every step. First suspects already visible in-tree: `ParticleNeighborhood::from_view` allocates+sorts proxies every call; some solver kernels `Vec::new()` per pass. Do not wait for SIMD. **REQ seed:** `PERF-AUDIT` |
| **Developer can re-run the Dam Break Medium pair on demand** (same host, scalar Rust `--release` vs C++ `oracle-release`, locked 1920-particle playground recipe, 60 warm-up + 600 timed `World::step` / `b2World::Step`) and see wall ms, ms/step, and Rust/C++ ratio | Same-workload, same-host, same-opt-level pairing is the industry floor (box2d-rust uses the C benchmark app vs a line-for-line Rust port, serial vs serial, interleaved). Unpaired or debug-vs-release numbers are meaningless | LOW–MEDIUM | Extend existing `just playground-dam-break-bench` / xtask. Keep construction, insertion, warm-up, capture, and rendering outside the timer. Persist dated reports; do not only print Markdown to stdout. **REQ seed:** `PERF-PAIR` |
| **Developer can capture CPU profiles of the timed Dam Break `--release` binary through a repeatable `just` / xtask script** and land them in a gitignored evidence directory with host, git HEAD, compiler, and command identity | Flamegraphs/call trees are how Rapier, Avian, and box2d-rust actually find wins. “We think contacts are slow” is not an audit | MEDIUM | On this macOS hobby host, script `samply` (Firefox Profiler) or `cargo flamegraph` / Instruments Time Profiler. Build with debug info on a release-class profile (`inherits = "release"`, `debug = true`); profiled timings are **never** wall-clock authority (`BENCHMARKING.md`). Gitignore the dumps (`target/…` is already ignored). **REQ seed:** `PERF-PROFILE` |
| **Developer can admit an optimization only from evidence** (named hot function or typed allocation/cache/scaling bottleneck + unprofiled Dam Break pair improvement + existing correctness gates still green) | Ports that skip admission ship SIMD/unsafe/parallel “fixes” that hide the real extra work, break determinism, or regress other scenes | MEDIUM | Lightweight analog of Phase 12 `optimization-check`, **not** the 32-case sealed matrix. Minimum bar: candidate is scalar `release`; Dam Break Medium unprofiled wall ratio improves; relevant profile share or typed bottleneck; differential/unit/determinism/safety regressions used by this engine still pass; no playground-scene cheat (do not lower particle count). **REQ seed:** `PERF-ADMIT` |
| **Developer can land profile-guided fixes on *shared* particle/rigid hot paths** so Dam Break and other particle/rigid scenes benefit from the same change | A Dam Break-only special case is a demo hack, not an engine close | HIGH | Neighborhood/proxy rebuild, contact generation, particle–body coupling, pressure/damping/integrate, and rigid contact solve are the shared surface. Scene controllers and WASM frame copies are out of scope until native stepping is honest. **REQ seed:** `PERF-SHARED` |
| **Developer can demonstrate native Dam Break Medium wall time ≤ 3× pinned C++** on the same host under scalar `--release` vs `oracle-release` | Owner-locked numeric gate. “Same order of magnitude” means 300× → ≤3×, not 1.00× parity and not a README trophy | HIGH | Re-run the pair after admitted fixes. Record host, git HEAD, compilers, warmup/steps, both wall times, and ratio. Leave `reference/performance/manifest.toml` empty. **REQ seed:** `PERF-GATE` |
| **Developer can spot-check the other five playground scenes** (Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel) after shared-path fixes | Users will feel a Dam Break-only win as a lie if Fountain still crawls. A second sealed 32-case matrix is the wrong response | LOW–MEDIUM | Profile or time native `--release` headless stepping; look for improvement or non-catastrophic regression. No C++ pair required per scene; no Phase 12 case hashes. **REQ seed:** `PERF-SPOT` |
| **Developer can keep the scalar deterministic compatibility baseline** while closing the gap; SIMD and parallelism stay explicit opt-in (off by default) | Rapier documents that SIMD lane width is its own determinism domain and cannot mix with enhanced-determinism. Avian’s 3× came from default-on parallel graph coloring — the opposite of this project’s lock | LOW (policy) / HIGH (if violated) | Workspace already `unsafe_code = "forbid"`. Do not lift that to chase the canary. Safe layout/allocation/algorithm fixes first. **REQ seed:** `PERF-BASELINE` |
| **Developer can document the remaining Dam Break delta honestly** after the gate (ratio, suspected leftover causes, what was not attempted) | Incomplete closes that are marketed as “fast as C++” destroy trust. box2d-rust still publishes 1.25× with named leftovers | LOW | Committed notes, not a sealed report. Do not write “Rust is X% slower” into README as a universal claim. **REQ seed:** `PERF-NOTES` |
| **Visitor can still run the six playground scenes after the native gate**, with a lightweight WASM/playground sanity check recorded honestly (not versus C++) | Native stepping wins should not break Pages; WASM is a different runtime (no C++ oracle, extra copy lanes, browser budget) | LOW–MEDIUM | After `PERF-GATE`, rebuild WASM and run `just web-player-smoke` plus an optional Dam Break step-time / realtime-factor note in the gitignored evidence dir. Compare WASM to *previous WASM* or to native Rust, never to `oracle-release`. **REQ seed:** `PERF-WASM` |

### Differentiators (Competitive Advantage)

Not required for a generic “make it faster” PR. Valuable here because the project already has an oracle, a locked Dam Break recipe, and a sealed method it is *choosing not* to revive as a public claim.

| Feature | Value Proposition | Complexity | Notes |
| --- | --- | --- | --- |
| **Canary-first same-order close** instead of filling the Phase 12 32-case public matrix | Developers get a useful engine on the actual playground recipe without pretending 32 sealed workloads are reviewed. Empty `reviewed_reports = []` stays truthful | MEDIUM | This *is* the milestone shape. Phase 12 remains optional strict tooling |
| **Named-function audit committed in-repo** | Most ports only ship “~faster.” A dated notes doc that names functions and the Dam Break delta is the artifact reviewers and future phases can trust | MEDIUM | Profiles themselves stay gitignored; the *names and suspected causes* are committed |
| **Paired C++ extra target on the exact playground recipe** | Unique vs Rapier/Avian (no LiquidFun C++ oracle) and stronger than ad hoc Criterion micros | LOW | Already exists; v1.2 makes it the gate and persists reports |
| **Lightweight admission without a public claim** | Captures Phase 12’s good rule (profiles ≠ authority; correctness hashes stay accepted) without 150-sample calibration or manifest promotion | MEDIUM | A short checklist in notes + scripts is enough; do not require `cargo xtask performance optimization-check` over the 32-case record |
| **Engine-wide hunt, scene-local gate** | Fixes land in `liquidfun`; Dam Break is the numeric bar; other scenes are spot-checks | HIGH | Prevents “optimize the bench” theater |
| **Post-gate WASM honesty** | Playground visitors are the only current public users; recording whether Dam Break is less stuttery without claiming WASM≈C++ matches hobby scope | LOW | Optional extra: native-wasm `dam-break-bench` vs previous native, still not vs C++ |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
| --- | --- | --- | --- |
| **Revive Phase 12 sealed 32-case public performance claims** / copy Dam Break numbers into `reference/performance/manifest.toml` | “Real” benchmarks look more scientific | Manifest is empty by design; exploratory pair is not the sealed matrix (different workloads, sample policy, identity hashes). Filling it with playground timings would be a false claim | Keep Phase 12 method on the shelf; persist unreviewed Dam Break reports under gitignore; commit notes only |
| **Required performance CI** or a dedicated Linux x64 / `PERFORMANCE_CONTROLLED_HOST_IDENTITY` completion gate | CI would “keep us honest” | Conflicts with `PROJECT-SCOPE.md`: local checks + one macOS Cargo job; expensive suites are optional manual. Pair timing is minutes-to-hours and host-specific | On-demand `just` / xtask on the developer machine; optional manual re-run |
| **Default parallelism or SIMD** in the compatibility baseline | Avian 0.4 and Box3D show large SIMD/thread wins | Changes contact/constraint/particle order and determinism; Rapier’s `simd8` cannot mix with enhanced-determinism; this repo forbids `unsafe` and holds scalar baseline | Explicit opt-in features later, after the scalar canary is closed and a separate decision exists |
| **WASM ≈ C++** (or browser Dam Break vs `oracle-release`) | Visitors care about the playground | Different ISA, allocator, and frame-copy shell; not a fair pair. Would revive a comparison the owner locked out | Native is the C++ pair; WASM sanity vs previous WASM / native Rust only |
| **“Rust is N× slower/faster” README or crates.io blurb** | Marketing a close | `BENCHMARKING.md` forbids universal summaries; one host, one scene, unreviewed | Bounded notes: workload, host, compilers, ratio, “not a public claim” |
| **Scene editor, new playground scenes, or particle-count cosmetics** | More demos / “looks realtime” | Out of milestone; lowering Medium from 1920 particles fakes the gate | Keep the locked recipe; optimize shared stepping |
| **Crate publication / git release tag / npm package** | “Ship the speedup” | Publication remains separately authorized; v1.1 already established archive ≠ release | Native/WASM sanity only |
| **Criterion catalog micros as the numeric gate** | Already have `liquidfun-benchmarks` | Wrong granularity vs the playground canary; easy to optimize a micro that Dam Break never hits | Optional supporting evidence only; Dam Break pair remains the gate |
| **`codegen-units = 1`, fat LTO, PGO, or `-march=native` / `-ffast-math` as the close** | Easy compiler knobs | Can move IEEE/ordering vs the oracle; native-tuned flags are explicitly non-canonical in stack policy; they also do not explain 300× | Keep ordinary `--release` vs `oracle-release`; if a profile later proves codegen-units, document as optional local, not the gate definition |
| **`unsafe` indexing / `get_unchecked` / C++ FFI in production `liquidfun`** | Bounds checks are a popular suspect | Typical bounds-check wins are 1–15%, not 300× (Shnatsel; box2d-rust leftover is ~1.25× and SIMD-shaped). Workspace `unsafe_code = "forbid"` | Safe slice splits, reuse buffers, remove extra work first |
| **Dam Break-only LOD, skipped solver passes, or reduced iterations** | Hits 3× quickly | Breaks LiquidFun behavior and differential evidence | Shared-path algorithmic/allocation fixes with tests |
| **Treating profiled runs as timing authority** | One script is simpler | Instrumentation distorts wall time; Phase 12 already forbids this | Unprofiled pair for the ratio; profiles for *why* |
| **Overwriting failed/old evidence** or committing `.trace` / samply binaries | Clean git tree | Loses diagnosis; large binaries do not belong in git | Dated gitignored directories; committed notes cite them by date/path |
| **Comparing debug Rust to release C++** (or mixing opt-levels) | Accidental `cargo run` | Rapier documents ~100× without `--release`. That is not this gap if the existing pair already used `--release` vs `oracle-release` | Scripts must pass `--release` / `oracle-release` and record compilers |
| **Substituting Rapier/Avian/modern Box2D** for LiquidFun particles | Those engines are faster in marketing charts | Wrong behavior oracle; project forbids treating unrelated Box2D as LiquidFun | Optimize this engine against the pinned C++ oracle |
| **Broad `no_std`, mobile, or complete-engine WASM certification** | Portability story | Unrelated to the native canary | Bounded playground WASM already exists; leave it as post-gate sanity |

## Expected Behavior (audit, profiling, admission, close)

### Audit

**Developer can** produce a committed notes document that a second person can follow without re-deriving the hunt.

Expected contents:

1. Locked recipe identity (1920 particles, radius/spacing, dt, solver iterations) matching `docs/playground-dam-break-timing.md`.
1. Current unprofiled pair: Rust wall, C++ wall, ratio, host, git HEAD, `rustc`, AppleClang/oracle identity.
1. Named Rust functions (and C++ counterparts when the extra work is a shape mismatch) that dominate `--release` samples.
1. Classified suspected causes: extra per-particle work, per-step allocation, checks that survive `--release`, algorithm/shape differences — not “Rust is slow.”
1. What was *not* found (so the next phase does not re-litigate SIMD as the first move).

Industry analog: box2d-rust’s largest win was **release-mode `B2_VALIDATE` / `b2ValidateIsland` walking islands quadratically**, not the contact SIMD they added later. Audit must specifically ask “does `--release` still run debug-shaped invariant walks?” Some `check_invariants()` calls in this tree are `debug_assert`; others are live `?` on mutation/permutation/depth paths — those are audit items, not presumed guilt.

### Scripted profiling

**Developer can** run one discoverable recipe that:

1. Rebuilds native `--release` and `oracle-release` playground Dam Break extras (existing xtask already does this for the pair).
1. Captures a CPU profile of the **Rust timed loop** (and optionally C++ for contrast) with symbols.
1. Writes a dated directory under a gitignored root (recommend `target/native-perf-closing/<date-or-git>/` so `/target/` already ignores it).
1. Leaves stdout/stderr logs, the unprofiled pair table, and a pointer the committed notes can cite.

Profiles may use a `profiling` Cargo profile (`inherits = "release"`, `debug = true`). That build is for diagnosis only. Gate numbers always come from unprofiled `--release`.

### Optimization admission

**Developer can** land a hot-path change only when all of these hold:

1. Evidence names a function or typed bottleneck (allocation, cache, scaling) with non-trivial profile share — Phase 12 used 10% as the floor; reuse that *idea* for this canary, not the 32-case JSON record.
1. Unprofiled Dam Break Medium pair improves (ratio down) on the same host and recipe.
1. Existing native tests and relevant differential/determinism checks the change can affect still pass. A physics mismatch is a failed candidate, never a faster sample.
1. Scalar deterministic baseline unchanged: no default rayon, no silent SIMD, no `--fast-math`, no skipped LiquidFun passes.
1. Other playground scenes are at least spot-checked before calling the hunt done (not before every tiny commit).

Failing admission means: keep the experiment, do not merge, do not update the committed “current delta” as if it passed.

### Same-order performance close

**Developer can** show:

> On this host, scalar Rust `--release` Dam Break Medium timed wall time ≤ 3 × pinned C++ `oracle-release` for the locked 60+600-step pair.

That is **same order of magnitude**, not parity. 3× still leaves C++ faster; it is the honest first bar from ~300×. Success does **not** authorize:

- a Phase 12 reviewed report
- README engine-wide claims
- WASM vs C++
- skipping remaining-delta notes

**Visitor can** open Dam Break on Pages after the gate and complete play/pause/reset without a new scene catalog. If WASM stepping is still far from realtime, say so; do not imply the native 3× gate transferred to the browser.

## Feature Dependencies

```
Existing engine + oracle + Dam Break pair
    └──requires──> PERF-AUDIT (named hot functions + current delta)
                       └──requires──> PERF-PROFILE (scripted CPU profiles, gitignored)
                       └──requires──> PERF-PAIR (dated unprofiled pair reports)
                                          └──requires──> PERF-ADMIT + PERF-SHARED
                                                             └──requires──> PERF-GATE (≤ 3×)
                                                             └──enhances──> PERF-SPOT (other five scenes)
                                                                                └──requires──> PERF-GATE
                                                                                     └──requires──> PERF-WASM
                                                                                     └──requires──> PERF-NOTES
PERF-BASELINE ──constrains──> PERF-SHARED / PERF-ADMIT
Phase 12 sealed matrix ──conflicts──> PERF-GATE as a public claim
Default SIMD/parallel ──conflicts──> PERF-BASELINE
WASM vs C++ ──conflicts──> PERF-WASM
```

### Dependency Notes

- **Audit requires the existing pair and both source trees:** without `just playground-dam-break-bench` and the pinned oracle, “hot” is anecdotal.
- **Profiles require a release-class binary with symbols:** otherwise the committed notes cannot name functions.
- **Shared-path fixes require admission:** landing layout changes before a profile invites Dam Break-only folklore.
- **The 3× gate requires unprofiled pair reports, not flamegraphs:** instrumentation is not timing authority.
- **Spot-checks enhance the gate; they do not replace it:** one numeric bar (Dam Break Medium).
- **WASM sanity requires the native gate first:** otherwise browser noise is used to “debug” native 300×.
- **PERF-BASELINE conflicts with default SIMD/parallel:** those remain explicit later opt-in, not this milestone’s close.
- **Phase 12 manifest promotion conflicts with this milestone’s honesty rules:** method may be cited; reviewed_reports stay empty.

## MVP Definition

### Launch With (v1.2)

Minimum to call Native Performance Closing done.

- [ ] **PERF-AUDIT** — Committed notes name hot functions, suspected extra work, and the Dam Break Medium delta
- [ ] **PERF-PAIR** + **PERF-PROFILE** — Repeatable local scripts write dated pair tables and CPU profiles under gitignore
- [ ] **PERF-ADMIT** + **PERF-SHARED** — Profile-guided shared particle/rigid fixes; no scene cheats
- [ ] **PERF-GATE** — Same-host scalar pair, Rust wall ≤ 3× C++
- [ ] **PERF-SPOT** — Other five playground scenes profiled or timed as spot-checks
- [ ] **PERF-WASM** — Post-gate playground/WASM sanity, not vs C++
- [ ] **PERF-BASELINE** + **PERF-NOTES** — Scalar determinism retained; remaining delta documented; no Phase 12 public claim

### Add After Validation (later in v1.2 or a follow-on)

- [ ] Optional second native canary (e.g. a rigid-heavy catalog row) **if** Dam Break ≤ 3× and profiles show a *different* dominant cluster — still not a sealed 32-case matrix
- [ ] Optional in-engine diagnostic parent timers (`DiagnosticProfileParent`: `particle_prepare` / `particle_solve` / `rigid_solve`) wired into the Dam Break script for cheaper iteration — still not public authority
- [ ] Safe buffer reuse / stack-like scratch if profiles prove per-step `Vec` growth — only after the first extra-work cuts

### Future Consideration (not this milestone)

- [ ] SIMD / parallel opt-in features (Rapier/Avian/Box3D-shaped) after scalar close + explicit determinism policy
- [ ] Phase 12 sealed matrix production, calibration, and reviewed-report promotion
- [ ] WASM vs native performance engineering (beyond sanity)
- [ ] Crate publication, new playground scenes, scene editor
- [ ] Relaxing `unsafe_code = "forbid"` for intrinsics (box2d-rust’s leftover 1.25× → 1.0× step)

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
| --- | --- | --- | --- |
| PERF-AUDIT committed named-function notes | HIGH | MEDIUM | P1 |
| PERF-PAIR dated unprofiled Dam Break reports | HIGH | LOW | P1 |
| PERF-PROFILE scripted CPU profiles (gitignored) | HIGH | MEDIUM | P1 |
| PERF-ADMIT evidence-gated landings | HIGH | MEDIUM | P1 |
| PERF-SHARED shared hot-path fixes | HIGH | HIGH | P1 |
| PERF-GATE Dam Break Medium ≤ 3× C++ | HIGH | HIGH | P1 |
| PERF-BASELINE scalar deterministic default | HIGH | LOW | P1 |
| PERF-SPOT other five scenes | MEDIUM | LOW | P1 |
| PERF-NOTES remaining delta | HIGH | LOW | P1 |
| PERF-WASM post-gate playground sanity | MEDIUM | LOW | P1 |
| Diagnostic parent timers in the pair script | MEDIUM | MEDIUM | P2 |
| Criterion micros as supporting evidence | LOW | LOW | P2 |
| Second native canary scene | MEDIUM | MEDIUM | P2 |
| SIMD/parallel opt-in | MEDIUM | HIGH | P3 |
| Phase 12 sealed public reports | LOW (hobby now) | VERY HIGH | P3 |
| New playground scenes / editor / publish | LOW | HIGH | P3 — anti-feature for v1.2 |

**Priority key:**

- **P1:** Must have for v1.2 launch
- **P2:** Should have if the canary is closed and cheap
- **P3:** Explicitly later / out of milestone

## Competitor Feature Analysis

How “close the C++ vs Rust physics gap” actually ships in neighboring engines. Use as capability expectations, not as engines to swap in.

| Capability | box2d-rust 1.3 (C Box2D v3 port) | Rapier 2D/3D | Avian 0.4 | liquidfun-rs v1.2 approach |
| --- | --- | --- | --- | --- |
| Pair vs C/C++ oracle | Yes: C `benchmark` app vs Rust example, **serial vs serial** (`-w=1`), same scenes/dt/substeps, warm-up excluded, interleaved to defeat thermal bias. Geo-mean ~**1.25×** (range 1.13–1.47×) as of 2026-07-19 | No LiquidFun/Box2D C++ oracle; compares to itself / PhysX anecdotes | No C++ LiquidFun oracle; compares to Rapier and its own previous version | **Keep playground Dam Break pair** as the one numeric gate; do not add a 10-scene public matrix this milestone |
| First huge win | **Release validators**, not SIMD: `B2_VALIDATE` / `b2ValidateIsland` in release made island-churn **quadratic**; gating to debug took spinner **2.73× → 1.21×**. First measurement 1.9× → 1.45× after that | Official docs: Rapier can be **~100× slower** without `--release` | Profile-driven parallel solver | Audit extra `--release` work and allocations first; the ~300× canary is extra-work-shaped, not 1.25× codegen |
| Profiling | Re-measure interleaved after every change; WASM perf noted separately and **not** mixed into the C ratio | `profiling` crate, Tracy/Puffin in testbed; `cargo flamegraph` used in issue hunts | Flamegraphs in the 0.4 write-up (narrow phase, graph-color solver) | Scripted samply/flamegraph/Instruments into gitignored evidence; committed function names |
| SIMD / threads | Safe `[f32; 4]` contact solver later (1.45× → 1.25×); remaining gap attributed to C SSE2 vs rustc; **serial by design** (no C task system) | SIMD/parallel are **features**; `simd8` conflicts with enhanced-determinism; parallelism can *slow* small scenes | Default-ish parallel graph coloring: solver **>3×**, total **~2×** vs prior Avian | **Anti-feature as default.** Scalar close first; opt-in only later |
| Admission / honesty | README publishes methodology, pin, host, and leftover causes; WASM “may run below realtime” disclosed | Common-mistakes page instead of a sealed matrix | Blog + PR with profiles | Committed notes + empty Phase 12 manifest; no “Rust is faster” |
| WASM | Live demos; WASM profiled **per scene**, not vs C | First-class WASM packages | Bevy-centric | Post-gate sanity only; never WASM vs C++ |
| Bounds-check theater | Not listed as the 1.9× cause | — | — | Do not start with `unsafe` indexing; typical wins 1–15% |

**Implication:** A 300× LiquidFun gap that already used `--release` vs `oracle-release` should be treated as **wrong extra work / extra allocation / extra algorithm**, the same class as box2d-rust’s release validators — not as a reason to turn on Avian-style parallelism or to publish a sealed 32-case claim.

## Sources

### This repository (HIGH)

- `.planning/PROJECT.md` — v1.2 goal, Dam Break ≤ 3× gate, shared-path hunt, WASM-after-native, scalar baseline
- `PROJECT-SCOPE.md` — hobby local checks; no dedicated perf host; optional expensive suites
- `BENCHMARKING.md` — Phase 12 method; empty reviewed-report manifest; profiles ≠ timing authority; admission ideas
- `docs/playground-dam-break-timing.md` — locked Medium recipe; first ~300× sample; `just playground-dam-break-bench`
- `reference/performance/manifest.toml` — `reviewed_reports = []`
- `reference/performance/policy.json` — unprofiled wall clock; `release_scalar`; 10% profile floor for Phase 12 admission
- `Justfile` — `playground-dam-break-bench`, `phase12-performance-*`, `web-player-smoke`
- `tools/xtask/src/playground.rs` — pair driver builds `oracle-release` extra + `liquidfun-wasm` `dam-break-bench`
- `crates/liquidfun-wasm/src/dam_break_bench.rs` — 1920 particles, 60+600 steps
- Workspace `Cargo.toml` — `unsafe_code = "forbid"`
- `crates/liquidfun/src/particle/proxy.rs` — per-call neighborhood allocate/sort (audit target, not a proven 300× cause)

### Neighboring engines and profiling practice (HIGH / MEDIUM)

- [box2d-rust 1.3.0 README — paired C vs Rust methodology and validator-in-release win](https://docs.rs/crate/box2d-rust/latest) — HIGH, updated 2026-07-19
- [box2d-rust performance roadmap / misattribution notes](https://docs.rs/crate/box2d-rust/latest/source/todo.md) — HIGH
- [Rapier common mistakes — ~100× without `--release`; codegen-units](https://rapier.rs/docs/user_guides/rust/common_mistakes/) — HIGH
- [Rapier getting started — SIMD vs enhanced-determinism; parallelism can slow small scenes](https://rapier.rs/docs/user_guides/javascript/getting_started) — MEDIUM (JS guide; same feature tradeoff)
- [Avian Physics 0.4 — profile-guided parallel solver, ~3× solver / ~2× total](https://joonaa.dev/blog/09/avian-0-4) — HIGH as a *what not to default on* analog
- [Erin Catto, SIMD for Collision (Box3D, 2026-07) — SIMD helps some hulls, not all scenes](https://box2d.org/posts/2026/07/simd-for-collision/) — HIGH
- [samply — macOS/Linux/Windows sampling profiler, release + debug info](https://github.com/mstange/samply) — HIGH
- [cargo-flamegraph 0.6.13 (2026-06-03) — macOS via xctrace](https://github.com/flamegraph-rs/flamegraph) — HIGH
- [Shnatsel, bounds checks typically 1–3%, max ~15%; alloc often dominates](https://shnatsel.medium.com/how-to-avoid-bounds-checks-in-rust-without-unsafe-f65e618b4c1e) — MEDIUM

### Lower confidence (do not drive requirements)

- Third-party “Rust 1.85 vs C++23 game physics” blog roundups — LOW (methodology not LiquidFun-shaped; possible SEO). Independent engines at matched algorithms are usually within tens of percent, which **supports** treating 300× as extra work, but the article is not a source for gates.

---
*Feature research for: liquidfun-rs v1.2 Native Performance Closing*
*Researched: 2026-09-20*
