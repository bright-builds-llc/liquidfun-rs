---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Native Performance Closing
status: executing
stopped_at: Completed 24-05-PLAN.md
last_updated: "2026-09-21T21:19:36.191Z"
last_activity: 2026-09-21
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 21
  completed_plans: 20
  percent: 95
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-21)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 24 — Shared hot-path waves through 3×

## Current Position

Phase: 24 (Shared hot-path waves through 3×) — EXECUTING
Plan: 6 of 6
Status: Ready to execute
Last activity: 2026-09-21

Progress: [█████████░] 86%

v1.2 Native Performance Closing. Gate is unprofiled `just playground-dam-break-bench` (Rust `--release` vs `oracle-release` Dam Break Medium ≤ 3×). Evidence dir `target/dam-break-perf/<utc-stamp>/`. v1.1 phase directories remain on disk. No package publication or release tag.

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |
| Phase 22 P01 | 31 min | 2 tasks | 6 files |
| Phase 22 P02 | 9 min | 2 tasks | 4 files |
| Phase 22 P03 | 8 min | 2 tasks | 3 files |
| Phase 22 P04 | 9 min | 2 tasks | 6 files |
| Phase 22 P05 | 13 min | 2 tasks | 11 files |
| Phase 22 P06 | 8 min | 2 tasks | 10 files |

v1.1 plan durations remain in `.planning/milestones/v1.1-STATE.md`.
| Phase 23 P01 | 8 min | 2 tasks | 7 files |
| Phase 23 P04 | 5 min | 2 tasks | 3 files |
| Phase 23 P02 | 14 min | 2 tasks | 6 files |
| Phase 23 P03 | 4min | 2 tasks | 1 files |
| Phase 23 P05 | 10 min | 2 tasks | 5 files |
| Phase 23-baseline-pair-and-named-audit P06 | 6 min | 2 tasks | 2 files |
| Phase 23 P07 | 8 min | 3 tasks | 1 files |
| Phase 23 P08 | 9 min | 2 tasks | 2 files |
| Phase 23 P09 | 6 min | 2 tasks | 1 files |
| Phase 24-shared-hot-path-waves-through-3 P01 | 15 min | 2 tasks | 5 files |
| Phase 24-shared-hot-path-waves-through-3 P02 | 5 min | 2 tasks | 2 files |
| Phase 24-shared-hot-path-waves-through-3 P03 | 11 min | 2 tasks | 3 files |
| Phase 24 P04 | 265 | 2 tasks | 24 files |
| Phase 24-shared-hot-path-waves-through-3 P05 | 36min | 2 tasks | 13 files |

## Accumulated Context

### Decisions

- [v1.2]: Dam Break Medium native pair is the numeric gate — Rust wall time ≤ 3× pinned C++ on the same host, scalar `--release` vs `oracle-release`.
- [v1.2]: Hunt shared particle/rigid hot paths; other scenes are spot-checked. Do not revive the Phase 12 sealed 32-case public matrix.
- [v1.2]: Hold the scalar deterministic compatibility baseline. SIMD/parallel stay explicit opt-in.
- [v1.2]: Native is the C++ comparison. After the native gate, record a lightweight WASM/playground sanity check; WASM is not compared to C++.
- [v1.2]: Scripted local pair + CPU profiles write dated gitignored evidence under `target/dam-break-perf/<utc-stamp>/`; committed notes name hot functions without becoming a sealed public claim.
- [v1.2]: Unprofiled `just playground-dam-break-bench` is the 3× authority; samply / `[profile.profiling]` / `step_profiled` / dhat timings are never that number.
- [v1.2]: Phase numbering continues after 21 (Phases 22–25). Do not reset to Phase 1.

v1.1 playground decisions remain in `.planning/milestones/v1.1-STATE.md` and PROJECT.md Key Decisions.

- [Phase 22]: Keep dam-break-bench as the only playground subcommand; pair stays stdout-only until 22-02. — D-05: do not add profile or timer commands in 22-01.
- [Phase 22]: Mint stamps with exclusive create_dir under target/dam-break-perf and bump one Unix second on AlreadyExists, capped at 8 attempts. — D-01 fail-closed mint; never remove_dir_all or merge into a pre-existing stamp.
- [Phase 22]: Honor LIQUIDFUN_XTASK_GIT as a Command program path for git rev-parse HEAD. — Matches upstream tool_program pattern; unset env still runs git.
- [Phase 22]: Do not mark PERF-PAIR complete in this plan; pair.json and pair.md persistence is 22-02. — 22-01 delivered the split and stamp core only.
- [Phase 22]: Persist pair.json and pair.md only after exclusive stamp mint and a finite C++ wall_ms; zero C++ wall is a bench error with no files written. — D-01/D-02: never Inf, never overwrite an existing stamp.
- [Phase 22]: Keep just playground-dam-break-bench as the one-line cargo xtask alias; do not hide cmake or samply in just. — D-05: unprofiled 3x-authority recipe name stays locked.
- [Phase 22]: Honor LIQUIDFUN_XTASK_CARGO else CARGO else cargo, and LIQUIDFUN_XTASK_STAMP_UNIX as test-only unix seconds. — Command path injection plus integer-only stamp clock for fake-tool CLI collision coverage.
- [Phase 22]: pair.json kind is unprofiled_pair with timing_authority unprofiled_wall_clock; omit samply/profile keys. — D-03/PERF-PAIR: unprofiled wall-clock remains the 3x number.
- [Phase 22]: Declare workspace [profile.profiling] with inherits = release, debug = true, strip = false; do not create [profile.release] or set CARGO_PROFILE_RELEASE_DEBUG. — D-08/D-10: named profile keeps the unprofiled --release gate binary unchanged.
- [Phase 22]: samply_record_argv is flags only: record --save-only --unstable-presymbolicate -o gz -- profiling binary --warmup N --steps M; never cargo. — Program path stays on Command::new so 22-04 can spawn without shell joining; Pitfall 3 forbids samply record cargo.
- [Phase 22]: Missing or non-0.13.1 samply is PlaygroundError kind samply with cargo install --locked, brew install samply, and samply setup; no skip wording. — D-09 fail-closed: no Ok skip path and no placeholder rust.json.gz.
- [Phase 22]: Do not mark PERF-PROFILE complete in this plan; dam-break-profile spawn and just alias are 22-04. — This plan delivered the Cargo profile and pure argv/version core only; PERF-PROFILE still needs the CLI wrap.
- [Phase 22]: just playground-dam-break-profile is a one-line cargo xtask alias with no cmake or samply flags. — D-06/D-07: just remains a one-line printer; xtask owns samply and cargo orchestration.
- [Phase 22]: Profile cargo uses --profile profiling and never cargo run --release; samply argv never starts with cargo. — D-08/D-10/Pitfall 3: named profiling profile keeps the unprofiled --release gate binary unchanged.
- [Phase 22]: Missing samply (LIQUIDFUN_XTASK_SAMPLY at a nonexistent path) fails closed with 0.13.1 install text and writes no rust.json.gz. — D-09/T-22-04-06: success tests inject fake samply; fail-closed tests use a missing production lookup path.
- [Phase 22]: profile-identity.json is kind samply_cpu with not_timing_authority true; the stamp has no pair.json. — D-03: profiled wall times must never become the 3x number or enter pair.json.
- [Phase 22]: dam-break-timers is a separate native binary; dam-break-bench still calls only ordinary advance/World::step. — D-10/D-11: step_profiled must not contaminate the unprofiled 3x gate process.
- [Phase 22]: just playground-dam-break-timers is a one-line cargo xtask alias with no cmake or samply flags. — D-06/D-07: just remains a one-line printer; xtask owns cargo and stamp persist.
- [Phase 22]: A fake timers run writes timers.json into a new stamp with not_timing_authority true and no pair.json. — D-04/D-10: timer walls are diagnostic only and must never become the 3x number.
- [Phase 22]: MAX_ADVANCE_STEPS remains 4; measured steps loop advance_profiled(1). — Pitfall 6: do not pass 600 as a single advance count.
- [Phase 22]: BENCHMARKING.md names gitignored target/dam-break-perf stamps and denies profiled/timer sibling stamps as the 3x number; it does not paste wall-ms. — D-14: one-sentence stamp note is allowed; public Dam Break numbers and the empty manifest stay unmodified.
- [Phase 22]: liquidfun production deps stay bitflags-only; cargo tree --edges normal is liquidfun -> bitflags only. — D-12 isolation: package verify plus cargo tree; do not add profiler, samply, dhat, CMake, or serde to liquidfun.
- [Phase 22]: Independent AI review remains a later policy step; this plan records automated evidence and does not self-approve the phase. — Owner policy 2026-09-16: the implementing agent must not approve its own work. T-22-06-05 accept.
- [Phase 23]: Integration-test crate roots need path attributes into playground_cli children because Cargo treats tests/foo.rs as a crate root. — mod bundle in tests/playground_cli.rs looks for tests/bundle.rs, not tests/playground_cli/bundle.rs. Path attributes match upstream_cli and keep foo.rs plus foo/ without mod.rs or main.rs.
- [Phase 23]: Do not mark PERF-AUDIT or PERF-HEAP complete in 23-01; this split only makes room for later exclusive tests. — Named audit notes and optional dhat remain 23-07 and 23-08. Checking the requirements off after a test-file split would be false.
- [Phase 23]: No Bright Builds TSV exception for playground_cli; every split test file is well under 628 physical lines. — Dispatcher 14, support 304, pair 96, profile 186, timers 48, bundle 1, heap 1. Later plans can add cases in their stub files.
- [Phase 23]: Keep dhat 0.3.3 crate-local on liquidfun-wasm; never add it to workspace.dependencies or liquidfun. — D-11/D-15: published liquidfun stays bitflags-only; optional dhat-heap is private to the dam-break-bench binary.
- [Phase 23]: Honor LIQUIDFUN_DHAT_HEAP_FILE as an OsString path; unset env may use Profiler::new_heap(). — Pitfall 7: production xtask in 23-05 must set the env to a stamp path so dhat-heap.json does not land in the repo root.
- [Phase 23]: Do not mark PERF-HEAP complete in 23-04; xtask heap spawn and run-or-skip remain 23-05 and 23-08. — This plan only makes the optional feature compile. Checking PERF-HEAP off before the dump command and audit skip path exist would be false.
- [Phase 23]: pair.rs cargo argv stays --release without --features so the gate binary stays uninstrumented. — Pitfall 3/D-15: enabling dhat-heap on --release would replace target/release/dam-break-bench. Heap spawn uses --profile profiling in 23-05.
- [Phase 23]: Copy pair.json, pair.md, rust.json.gz, and *syms* sidecars with fs::copy into a new exclusive stamp; never move or overwrite sources. — D-04: Phase 22 writes pair and profile into sibling stamps. Phase 23 must copy, never merge or overwrite.
- [Phase 23]: Split bundle helpers into bundle/ops.rs so every playground file stays ≤628 physical lines. — Combined GREEN file hit 763 lines; foo.rs plus foo/ keeps the Bright Builds cap without a TSV exception.
- [Phase 23]: Do not mark PERF-AUDIT complete in this plan; the named-function audit remains 23-07. — This plan only ships the copy-only bundle command. Checking PERF-AUDIT off before named notes exist would be false.
- [Phase 23]: Run playground unit tests as cargo test -p xtask playground::bundle because xtask has no lib target. — Same as Phase 22: cargo test --lib fails with no library targets found in package xtask.
- [Phase 23]: Keep THIRD_STAMP local in playground_cli/bundle.rs so 23-01 support.rs stays untouched. — Plan allowed a local constant; editing support.rs would dirty 23-01 ownership for a test-only stamp name.
- [Phase 23]: Do not patch playground/bundle.rs; 23-02 already copies rust.json.syms.json and fails closed. — Task 1 and Task 2 CLI tests passed on first run. Patching production would have been scope creep.
- [Phase 23]: Do not mark PERF-AUDIT complete in this plan; fake cmake/samply still cannot name dominating functions. — D-01: fake fixtures cover copy plumbing only. Named-function notes remain 23-07.
- [Phase 23]: Scan rust.json.syms.json first (Path::with_extension on rust.json.gz), then other *syms* files, then decompressed gzip JSON strings. — D-12 / Pitfall 4: samply --unstable-presymbolicate writes a sidecar; gzip-only scans miss names.
- [Phase 23]: Hex-only 0x addresses and non-JSON gzip are SkipNoSymbols, not no-allocator. — D-13: inconclusive parse must not be recorded as no allocator/Vec time.
- [Phase 23]: Heap cargo is --profile profiling --features dhat-heap and never --release so target/release/dam-break-bench stays the gate binary. — D-11 / Pitfall 3: enabling dhat-heap on --release would replace the unprofiled gate artifact.
- [Phase 23]: SkipNoAllocator and SkipNoSymbols return Err before mint_exclusive_stamp or cargo spawn. — D-13: do not run dhat just in case; write no dump on skip.
- [Phase 23]: Do not mark PERF-HEAP complete in this plan; live dump or skip notes remain 23-08. — This plan ships the gate and command. Checking PERF-HEAP off before the live run-or-skip notes exist would be false.
- [Phase 23]: Run playground unit tests as cargo test -p xtask playground because xtask has no lib target. — Same as Phase 22/23-02: cargo test --lib fails with no library targets found in package xtask.
- [Phase 23]: Fake cargo handles run + dam-break-bench + dhat-heap before --profile profiling so LIQUIDFUN_DHAT_HEAP_FILE is written instead of only installing the profiling binary. — Heap CLI uses cargo run --profile profiling --features dhat-heap. Matching --profile first skipped the dump writer and broke needle-match tests.
- [Phase 23]: Do not mark PERF-HEAP complete in 23-06; live dump or skip notes remain 23-08. — This plan is fake-tool CLI coverage. Checking PERF-HEAP off before the live run-or-skip notes exist would be false.
- [Phase 23]: Canonical live Dam Break stamps cite MEASURED_HEAD 6d98531 with pair 2026-09-21T04-32-19Z, profile 2026-09-21T04-32-56Z, bundle 2026-09-21T04-34-32Z, durable_copy same-tree. — Task 1 docs commit moved HEAD; pair was re-run so same-HEAD bundle succeeds. pair.json walls are the only 3x number. Live sidecar is rust.json.syms.json.
- [Phase 23]: Do not mark PERF-AUDIT complete in 23-07; named-function notes remain 23-08. — This plan is live capture only. Checking PERF-AUDIT off before docs/native-performance-audit.md would be false.
- [Phase 23]: Rank dominating frames from live rust.json.syms.json plus gecko sample weights; do not paste the hunt list as causes. — Sidecar-first ranking is the PERF-AUDIT source; hunt-list symbols stay Not found until they have a live share.
- [Phase 23]: particle_rows is about 93.6 percent self (ParticleId.position scans); classify as algorithm/shape versus FindContacts indexA/indexB. — C++ FindContacts keeps Proxy.index; Rust re-scans particle_ids per contact candidate. SIMD is not the first move.
- [Phase 23]: Copy rust_over_cpp_ratio 327.53395024734476 only from bundle pair.json at MEASURED_HEAD 6d98531ac799987c209d3fd1e572e482fcab5da6. — Unprofiled pair.json remains timing authority; samply, profiling, step_profiled, and dhat clocks are not_timing_authority.
- [Phase 23]: Run private dhat after allocator/Vec needles; cite heap stamp 2026-09-21T04-47-09Z; never git add dhat-heap.json. — Needles matched on the live sidecar so dhat ran once at 60+600; dump stays gitignored under target/dam-break-perf.
- [Phase 23]: Independent AI review remains 23-09; this plan does not self-approve. — Owner policy 2026-09-16: the implementing agent must not approve its own work.
- [Phase 23]: D-16 independent AI review is not performed by this implementing agent; /gsd-verify-work 23 and a separate AI reviewer remain required. — Owner policy 2026-09-16: the implementing agent must not approve its own work. This plan records automated isolation evidence only.
- [Phase 23]: BENCHMARKING.md points at docs/native-performance-audit.md as unreviewed SHA-bound notes; raw json.gz, dhat-heap.json, and trace stay gitignored and out of the empty manifest. — D-09/D-15: keep committed docs as unreviewed samples. Do not paste wall-ms or copy playground numbers into reference/performance/manifest.toml.
- [Phase 23]: xtask playground units run as cargo test -p xtask playground because the package has no lib target. — cargo test -p xtask --lib playground fails with no library targets found; same as Phase 22 and 23-02/23-05.
- [Phase 24-shared-hot-path-waves-through-3]: Carry dense ParticleIndex on neighborhood Proxy and a private pair_rows lane aligned with public ParticleNeighborPair ParticleId pairs. — D-01/D-03: restore FindContacts Proxy.index without changing the public ParticleId API.
- [Phase 24-shared-hot-path-waves-through-3]: Previous public ParticleContact IDs go through maybe_live_row to resolve_live; do not store last-step rows on public contacts or add a HashMap cache. — D-01/D-02: compaction remaps rows; the generational identity map is not a hot-path HashMap cache.
- [Phase 24-shared-hot-path-waves-through-3]: liquidfun stays scalar and safe: no rayon, std::simd, unsafe indexing, or workspace unsafe_code change. — PERF-BASELINE / D-15: wave 1 is index-preserving shape, not SIMD or unsafe.
- [Phase 24-shared-hot-path-waves-through-3]: Admit wave 1 from exclusive unprofiled pair 2026-09-21T15-50-46Z with rust_over_cpp_ratio 21.00845179052245, strictly below Phase 23 327.53395024734476. — D-04/D-05: locked 1920/60+600 just playground-dam-break-bench into a new exclusive stamp. pair.json is the only 3x number.
- [Phase 24-shared-hot-path-waves-through-3]: Do not claim PERF-GATE: 21.01 remains greater than 3; leftover D-07/D-09 waves stay required and check_invariants was not gated in wave 1. — D-06/D-08: wave 1 is necessary and insufficient. samply duration is not_timing_authority.
- [Phase 24-shared-hot-path-waves-through-3]: Sibling samply stamp 2026-09-21T15-52-27Z is leftover-ranking input only (kind samply_cpu, not_timing_authority true, no pair.json). — D-05/D-14: retarget 24-03 from the sibling profile stamp; never copy samply ms into the Dam Break ratio.
- [Phase 24-shared-hot-path-waves-through-3]: Admit leftover check_invariants / slice_contains from 24-02 sidecar 2026-09-21T15-52-27Z (~32.85% self, first D-07 named share). — Wave 1 pair 21.008 remained greater than 3. Live leftover ranking named slice_contains under check_invariants first in D-07 order.
- [Phase 24-shared-hot-path-waves-through-3]: Gate candidate.check_invariants only on replace_solver_candidate behind debug_assertions; creation.rs and mutation.rs stay fail-closed. — D-08 / T-24-03-01: C++ NDEBUG analog on the solver commit path only. User-facing checked mutation remains typed and fail-closed.
- [Phase 24-shared-hot-path-waves-through-3]: Admit from exclusive unprofiled pair 2026-09-21T16-07-02Z with rust_over_cpp_ratio 15.08956518243927, strictly below Wave 1 21.00845179052245. Do not claim PERF-GATE. — D-04/D-05/D-14: new exclusive 1920/60+600 pair.json is the only 3x number. 15.09 remains greater than 3.
- [Phase 24-shared-hot-path-waves-through-3]: Sibling samply stamp 2026-09-21T16-07-39Z is leftover-ranking input only for 24-04. — D-05: samply retargets the next leftover; profiled duration is not_timing_authority.
- [Phase 24]: PERF-GATE uses newest same-HEAD unprofiled pair.json rust_over_cpp_ratio, not samply duration.
- [Phase 24]: Stop leftover kernels at <= 3 (D-06); do not gold-plate remaining ~2% frames.
- [Phase 24]: Failed leftovers keep their stamps and revert the kernel; never call a non-improving sample a win.
- [Phase 24]: PERF-CANARY2 is same-cluster; Fountain and Water Wheel extra wall is emission 1 to 3200 particle-contact work, not a rigid-solve hang. No second native profile stamp. — D-12 / PERF-CANARY2 default: all five scenes finished without timeout or a different failure mode.
- [Phase 24]: Spot JSON is exclusive kind native_scene_spot with not_timing_authority true; no rust_over_cpp_ratio and no pair.json. Gate stamps 2026-09-21T20-36-30Z and 2026-09-21T20-38-50Z were not overwritten. — D-11/D-15: spot walls must never become the Dam Break 3x number.
- [Phase 24]: live_particle_count snapshots the live particle system; construction particle_count is stale after Fountain emit. — Fountain and Water Wheel grew 1 to 3200 in stamp 2026-09-21T21-10-50Z, proving on_advance hooks ran.
- [Phase 24]: just playground-scene-spot is a one-line cargo xtask printer; warmup 60 / measured 120 / advance(1); dam-break-bench is not the spot timer. — D-05/D-06: just stays a one-line printer; spot defaults are not the Dam Break 600-step pair.

### Pending Todos

Execute 24-04 leftover waves from samply stamp `2026-09-21T16-07-39Z` until unprofiled pair.json ≤ 3. Do not pre-select SIMD, PGO, or `unsafe` indexing.

### Blockers/Concerns

- samply setup / signing on this Mac is host-specific; Phase 22 must fail closed with install text.
- Unprofiled Dam Break Medium pair is still above 3× (`docs/playground-dam-break-timing.md`); that gap is the canary, not a public claim.
- First leftover admitted at unprofiled 15.08956518243927x (2026-09-21T16-07-02Z); leftover D-07/D-09 ranking uses sibling samply 2026-09-21T16-07-39Z. Do not pre-select SIMD, PGO, or unsafe indexing.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260919-eut | Split oversized WASM scene tests to satisfy Bright Builds CI | 2026-09-19 | 888b5e8 | [260919-eut-split-oversized-wasm-scene-tests-to-sati](./quick/260919-eut-split-oversized-wasm-scene-tests-to-sati/) |
| 260919-jco | Final responsive shell review fixes | 2026-09-19 | 680b191 | [260919-jco-final-whole-change-responsive-playground](./quick/260919-jco-final-whole-change-responsive-playground/) |
| 260919-wbn | Raise webapp particle frame cap 20x to 10240 and increase scene particle counts about 10x, packing finer so world volumes stay similar | 2026-09-20 | 3a047bf | [260919-wbn-raise-webapp-particle-frame-cap-20x-to-1](./quick/260919-wbn-raise-webapp-particle-frame-cap-20x-to-1/) |

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements.
- Historical decisions: `.planning/milestones/v1.1-STATE.md`, `.planning/milestones/v1.0-STATE.md`, `.planning/MILESTONES.md`.

## Session Continuity

Last session: 2026-09-21T21:18:31.107Z
Stopped at: Completed 24-05-PLAN.md
Resume file: None
