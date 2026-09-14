---
phase: 14-repair-windows-particle-group-invariants
status: investigation-in-progress
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
---

# Phase 14 Windows particle-group diagnosis

The historical and current Windows failures are preserved below. The first invalid transition and raw storage category are **not yet demonstrated**; Plan 02 remains blocked on that evidence. Local or supported-platform results are D2 only.

## Guidance and scope

AGENTS.md standing authorization, its Bright Builds sidecar, standards-overrides.md, and the architecture, code-shape, testing, verification, and Rust standards govern this investigation. Preserve the existing owned candidate and production behavior during probes. Publication uses normal gated commits and ordinary pushes; it does not authorize releases, evidence waivers, or history rewriting.

## Attempt index

| Attempt | Source boundary | Retained directory | Status |
| --- | --- | --- | --- |
| 20260914T015720Z-historical | Current baseline `5aadb6c105b98ae09443f74e44c57f8ce7eae19d`; original source separately identified below | `target/phase14-diagnostics/5aadb6c105b98ae09443f74e44c57f8ce7eae19d/attempt-20260914T015720Z-historical/` | Historical read-only collection complete; local execution pending host recovery |
| 20260914-task1 | `8b4eb5656dc69306225c900e6c44cb56b108f749` | `target/phase14-diagnostics/8b4eb5656dc69306225c900e6c44cb56b108f749/attempt-20260914-task1/` | Exact inventory and original public regression pass locally |

No existing Phase 14 diagnostic attempt directories or diagnosis file existed when this attempt was created. Subsequent attempts must append separate records and directories.

## Preserved failure identities

Both API records identify repository `bright-builds-llc/liquidfun-rs`, workflow `.github/workflows/ci.yml`, event `push`, run attempt 1, completed status and failure conclusion. Both Windows jobs use label `windows-2025` and install `1.97.0-x86_64-pc-windows-msvc`, `rustc 1.97.0 (2d8144b78 2026-07-07)`.

| Generation | Run / Windows job | Full source SHA | Runner | Property target result |
| --- | --- | --- | --- | --- |
| Audited original | [30070539790 / 89410277623](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/30070539790/job/89410277623) | `4b6b15ba562cedcad3f37e379d01be9e9cb2e949` | GitHub Actions 1000001443 | Two tests fail |
| Current post-13.1 | [34777296141 / 103777632638](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34777296141/job/103777632638) | `5aadb6c105b98ae09443f74e44c57f8ce7eae19d` | GitHub Actions 1000004118 | 0 passed; 3 failed; 0 ignored; 0 filtered |

Both failures report the exact panic:

```text
internal error: entered unreachable code: checked creation cannot invalidate authoritative storage
```

Original location: `crates\liquidfun\src\world\particle_object.rs:1171:13`. Current location: `crates\liquidfun\src\world\particle_object.rs:344:13`. Neither retained job log contains a stack backtrace; the current log explicitly requests `RUST_BACKTRACE=1`. No backtrace is reconstructed from source.

Original failing tests are `persisted_minimized_regression_covers_the_complete_operation_vocabulary` and `versioned_public_sequences_replay_exactly`. Current failing tests are those two plus `persisted_audited_windows_seed`.

### Log and metadata SHA-256 digests

All paths below are relative to the retained historical attempt directory above.

| File | SHA-256 |
| --- | --- |
| `30070539790-run.json` | `06218e063e2b43b071683e466cd0a56d3207bd74f23509ffcfa026e76ec515c5` |
| `30070539790-job.json` | `31e824e8e68c0b689a92632ca06897c57e270af5d1b9e5f5a84cb42e4d56587d` |
| `30070539790-windows.log` | `09c0e58ace17a896eab82b7fc4aa6dc50efe6d15a82570365d842314f40035a8` |
| `34777296141-run.json` | `da53bbbd01e6917ea43e7e50b91c39dc6fafa67f38df191b451c156a60d98001` |
| `34777296141-job.json` | `8cbad024f6e6a4667dac1c2c6af3e1824f441574a4a55bf34531cde171b3b249` |
| `34777296141-windows.log` | `7a8ee1dce1a0b137baf68d50f5d035611000c316005701f2d898c86cf15b4d5f` |
| `34777296141-linux-job.json` | `2ece72304208680fb1f2c2a3a75eeb07a33f8f628fb511f171ab14173f0e5b61` |
| `34777296141-linux-log` | `4f02b396c8559f9371225eef3ff8f29366f1d662e57b119aa8c47b25c8183e65` |
| `34777296141-macos-job.json` | `b62522a4542aec646db22e9b8cb115236a05488cb9fb440c01882936b1d58608` |
| `34777296141-macos-log` | `b1b0200bf43907707286441eb0accf8c0c69ec38948940a85387d947b8229600` |

Records were acquired with `gh api repos/bright-builds-llc/liquidfun-rs/actions/runs/RUN`, the corresponding `/actions/jobs/JOB`, and `gh run view RUN -R bright-builds-llc/liquidfun-rs --job JOB --log`.

The same current source and attempt have successful Linux default-features job `103777632709` (`ubuntu-24.04`, GitHub Actions 1000004120) and macOS default-features job `103777632737` (`macos-15`, GitHub Actions 1000004121). Their metadata and logs are retained separately above; those platform outcomes do not change the failed Windows result.

## Exact inputs and existing scaffold limits

- Original seed: `4149329052036581951` / `0x3995_60c9_ead9_4a3f`.
- Original controls: `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59]`.
- Original proptest persistence hash: `ca6e3bac494f125289898c2abc649305ee64e5d673f60339b47915ff6342257b`.
- Current additional failure seed: `190752942043209832`.
- Current additional controls: `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 225]`.
- Current persistence hash: `0a8fad7ffc31f0efb1d13432f6db169734cd4853b049b79db710f1cb1eabdfb9`.
- Separate vocabulary seed remains `0x7d7b_4a19_10c2_3023`, with controls `[0, 0, 1, 2, 3, 0, 0, 1, 4, 0, 0, 0, 0]`.

The original constants and all 14 expanded operations already reside in `crates/liquidfun/tests/particle_group_properties.rs` and remain unchanged. The named regression checks generator equality, initializes `Model`, then executes operations 1–13 outside its catcher. Only operation 14 is caught; Windows expects the existing panic there. Its current failure therefore does not prove the final operation was reached. The historical logs contain no per-operation labels, so the earliest reached operation is unknown pending tracing.

The lib replay must reproduce `Model::new`: world creation, zero gravity, primary and foreign particle-system creation, and the foreign control group at `(100.0, 100.0)` before the first original operation. Integration-test `cfg(test)` does not enable lib test hooks, so the public regression and private lib replay must run as separate Cargo targets.

## Current call path and pending proof

`World::plan_particle_group` in `world/particle_object/group.rs` validates owner and append target, samples the recipe, preflights diagnostic IDs, clones the source system, appends each particle/lifetime, refreshes contacts, and invokes `ParticleStorage::plan_group`. The storage planner validates entry state, prepares generated topology in a cloned candidate, optionally joins the temporary group into its target, computes solid depth, and validates again. `commit_particle_group` publishes the group shell, complete system and diagnostic allocation only afterward.

The shared mapper in `world/particle_object.rs` maps multiple raw storage categories to the same panic; the panic text does not select one category. Required next evidence is a live-source invariant result, every candidate-stage result through the first failure, its raw category/predicate and relevant lane/identity/index/float bits, with matching Windows source and inputs. No mapper substitution or production repair has been made.

## Local execution status

At task start, the orchestrator was investigating a previously stalled macOS test binary before Rust entry. This executor has not run Cargo or claimed a local passing test. Task 1's exact test inventory and single-test result remain pending the recovered gate route.

### Task 1 execution after host recovery

The preceding startup status is historical. At source `8b4eb5656dc69306225c900e6c44cb56b108f749`, a local Linux ARM64 container using immutable image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, explicit `RUSTUP_TOOLCHAIN=1.97.0`, read-only pinned rustup volume, offline registry and isolated Cargo target volume completed both required commands:

```text
cargo test -p liquidfun --test particle_group_properties -- --list
3 tests, 0 benchmarks

cargo test -p liquidfun --test particle_group_properties persisted_audited_windows_seed -- --exact --nocapture
1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
```

`list.log` enumerates exactly the three current names listed above; `exact.log` proves exactly one selected test. Logs reside in the Task 1 directory. This is local D2 evidence only and does not classify either preserved Windows failure as passing. The root's host recovery and separate CI Markdown-plugin repair are recorded in `14-CI-ENVIRONMENT.md`; the latter is preserved by later diagnostic workflow edits.

## Replay mapping prepared from current source

These are source-derived inputs, not observed execution outcomes. The copied private replay must preserve each branch, bounded selection, and API operation in the integration Model. All recipe transforms, strength, angular/linear velocity, color and omitted lifetime use the same `ParticleGroupRecipe::new` defaults; the trace records sampled position/velocity bits before append. Coordinates use `f32` arithmetic in the same order as the Model.

| Step | Expanded kind / first / second | Model operation and exact inputs |
| --- | --- | --- |
| Init | World / systems / foreign group | `World::new`, gravity ZERO, primary then foreign system; explicit `(100,100)` (`42c80000`, `42c80000`), New/WATER/empty flags |
| 1 | CreateExplicit / 71787583439247902 / 4278873410 | Base `(120,0)` (`42f00000`, `00000000`); points base, base+(0.4,0), base+(3,0); New/WATER/CAN_BE_EMPTY |
| 2 | Append / 47981589295040740 / 2859925585 | Select live group by first modulo live count; last member position+(0.3,0), otherwise ZERO; AppendTo/WATER/empty flags |
| 3 | CreateFilled / 56436091559000168 / 3363853189 | Circle center `(32,0)` (`42000000`, `00000000`), radius 0.45; filled source, stride 0.4, New |
| 4 | CreateStroke / 26405087307191014 / 1573865849 | Edge from `(24,0)` (`41c00000`, `00000000`) to base+(0.8,0); stroke source, stride 0.3, New |
| 5 | CreateReactive / 33181658083951976 / 1977780943 | Base `(32,0)`; points base, base+(0.4,0), base+(0.2,0.35); New/REACTIVE\|SPRING\|ELASTIC/SOLID |
| 6 | Join / 6396812461881086 / 381279734 | Select both live groups by modulo; if equal choose first distinct live replacement; retain first identity |
| 7 | Split / 10357184829122123 / 617336322 | Select live group by modulo; preserve 8-group bound using member_count-1 before public split |
| 8 | SetFlags / 5749946986639520 / 342723547 | Select by modulo; second modulo 4 is 3, so SOLID\|CAN_BE_EMPTY |
| 9 | CreateLifetime / 25775792647897003 / 1536356964 | Position `(44,0)` (`42300000`, `00000000`); New/DESTRUCTION_LISTENER/CAN_BE_EMPTY; lifetime 0.001 |
| 10 | Step / 53393286103193178 / 3182487851 | `StepConfiguration::new(1.0/60.0,8,3)`, 2 particle iterations, `NoDecisionHook`, default limits |
| 11 | DestroyMembers / 5176720587242457 / 308556591 | Select live group by modulo; destroy group particles with listener request true |
| 12 | Compact / 1779414347435098 / 106061360 | `compact_pending_particles(primary_system)` |
| 13 | InvalidJoin / 13697499052459992 / 816434565 | Selected live primary group joined with initialized foreign control group; exact WrongParticleSystem expected |
| 14 | Append / 31141724268561272 / 1856191412 | Same append selection/position rules as step 2; no shorter setup or earlier-call catcher |

The bound is 24 operations, 8 live groups, 32 particles. Explicit, filled, stroke, reactive and lifetime creation respectively reserve their existing logical bound checks for 3, 8, 5, 3 and 1 extra particles; append checks 1.

Exact binary32 constants used by the recipes are `0 = 00000000`, `1 = 3f800000`, `0.4 = 3ecccccd`, `3 = 40400000`, `0.3 = 3e99999a`, `0.45 = 3ee66666`, `0.8 = 3f4ccccd`, `0.2 = 3e4ccccd`, `0.35 = 3eb33333`, `0.001 = 3a83126f`, and `1/60 = 3c888889`. Defaults are identity transform (position ZERO, sine 0, cosine 1), zero velocities, zero color, strength 1, no explicit stride and infinite lifetime 0. Sampled position/velocity bits and dynamic append selection are recorded at runtime rather than guessed from platform output.

The temporary replay's source is copied from the integration Model; only snapshot normalization and retained lifecycle-category bookkeeping are replaced with diagnostic observations. Inspected `snapshot.rs` queries borrow `&Model` and the relevant world/storage view paths calculate/read owned values without updating caches. All initialization, mutating public calls, group ordering, bound checks, selector arithmetic, recipe fields, and error branches remain. The unchanged integration target independently retains its original invariant and rollback assertions.

## Falsifiable source hypothesis awaiting Windows observation

`particle/storage/solver_state.rs::zeroed_lane` currently calls `try_reserve_exact(declared_capacity)` before resizing to the actual particle count. The default world particle-system capacity is `i32::MAX`. A Windows allocation rejection during SOLID depth preparation or later scratch cloning could therefore map to `InvalidLaneBundle` even for this bounded replay. This is a hypothesis only. A valid source, successful generated topology, failed depth stage and observed allocation error/capacity are needed to establish it. A different raw stage or predicate falsifies this localization. No reserve, capacity, flag, tolerance or mapper change is authorized by this hypothesis alone.
