# Phase 35 Profiles and Attempts

Scope: native `--release` headless survey timing (`playground-scene-spot`) on one machine; local observations, not public speed claims.

## Phase summary

Final commit `2020757911f1757356d2c2ec80e55055aa0eeaa0`. The engine is BEFORE_COMMIT plus A1, A5, A6, A8 and A9. The before and final medians come from the cumulative ABBA in §Final verification (`spot-before` vs `spot-final`, `--runs 5`, 2026-10-10). "Gain beyond noise" is `yes` only when the final median is below the before minimum in both pairs. The phase promised no gain (v1.4 decision). The percentages are the measured pair changes on this shared host and are not public speed claims. The D-03 browser observation was not run, so no target was added.

| Target | Hot path (profile share) | Attempts | Kept | Before median (min-max) | Final median (ABBA pair 1 / pair 2) | Gain beyond noise |
| --- | --- | --- | --- | --- | --- | --- |
| liquid-tumbler | `pressure::damping` 22.9% self (61 particle iterations per step; float order locked by D-07) | A0, A1, A2, A3, A3b, A4, A5, A3r, A3br | A1, A5 | 25.135 (24.735-25.699) / 25.346 (25.133-25.454) | 23.670 / 23.204 | yes (−5.8% / −8.5%) |
| tesla-valve | `proxy::visit_sorted_tag_indices_in_aabb` 13.7% self (CCD fixture queries 6.9%, body-contact candidate rows 6.8%); emission 8.9% and lifetime resequencing 9.0% inclusive | A4, A5, A8, A9 | A5, A8, A9 | 4.056 (3.965-4.407) / 4.028 (3.980-4.061) | 3.258 / 3.440 | yes (−19.7% / −14.6%) |
| stacked-drip | `pressure::damping` 17.1% self, `consider_window` 13.9% self, proxy rebuild and sort 13.1% inclusive | A1, A2, A4, A5 | A1, A5 | 2.764 (2.713-2.808) / 2.719 (2.658-2.766) | 2.489 / 2.546 | yes (−9.9% / −6.4%) |
| washing-machine | `push_fixture_particle_hit` 15.2% self (full-scan CCD for moving drum fixtures at particle iteration 0, 29.1% inclusive) | A6 (A7 not tried) | A6 | 2.161 (2.094-2.280) / 2.182 (2.131-2.190) | 1.474 / 1.490 | yes (−31.8% / −31.7%) |
| particles | `pressure::damping` 22.8% self, `consider_window` 18.6% self, proxy rebuild and sort 8.9% inclusive | A1, A2 | A1 | 1.657 (1.627-1.673) / 1.713 (1.662-2.025) | 1.573 / 1.587 | yes (−5.1% / −7.4%; smallest margin, 1.573 vs min 1.627) |

Attempts (all IDs in §Attempts):

- A0 (research), rejected. A branchless damping loop was slower: its unconditional stores lengthen the dependent store-to-load chain, and D-07 locks the loop's float order.
- A1 (35-03), kept `91b27a6d6`. Reusing the sorted contact-proxy order across particle iterations put all three targets below the base minimum in both pairs: liquid-tumbler −3.4% / −4.7%, stacked-drip −3.0% / −5.4% and particles −1.5% / −3.9%.
- A2 (35-03), reverted. A1 plus a bounded insertion sort was faster on liquid-tumbler and stacked-drip, but particles and washing-machine regressed in both pairs, which fails D-11.
- A3 (35-04), reverted under D-11 (`0a2fe8cb8`, reverted by `7f0b36b18`). Hoisting the chain child edge gained −9.4% / −10.4% on liquid-tumbler, but soup-stirrer, which has no chain fixture, was above its base max in both isolated pairs (+1.5% / +1.5%).
- A3b (35-04), reverted under D-11 (never committed). The no-branch `Shape::Edge` variant gained −7.6% / −7.3% on liquid-tumbler, but fountain, which has no chain, was above its base max in both isolated pairs (+1.7% / +1.8%).
- A4 (35-05), reverted. A per-row binary search in the AABB tag query gained on no target and regressed liquid-tumbler plus four scenes in both pairs.
- A5 (35-05), kept `216de3929`. A bitset walk for body-contact candidate rows gained on liquid-tumbler (−3.0% / −3.0%) and tesla-valve (−0.9% / −2.2%). Stacked-drip gained in targeted pair 1 only.
- A6 (35-06), kept `2256dd8cd`. A verified conservative query pad for moving fixtures at iteration 0 replaces the `0..n` CCD scan: washing-machine −29.0% / −28.9%, and soup-stirrer, water-wheel and theo-jansen are also faster.
- A7 (35-06), not tried. Per-fixture transform validation was the fallback in case A6 failed, and A6 passed D-11.
- A8 (35-07), kept `5695f4b39`. Ungrouped particle creation skips the O(n) group-record rebuild: tesla-valve −7.8% / −12.2%, and fountain and water-wheel are about −20%.
- A9 (35-07), kept `f7041fc75`. The cheaper lifetime eviction index uses a deterministic hasher, an in-place resequence and bulk map builds: tesla-valve −9.1% / −8.1%, and fountain and water-wheel are about −27%.
- A3r (quick 261010-ibo), reverted under D-11 (never committed). A3 ported onto the final engine gained −8.8% / −10.4% on liquid-tumbler with 25/25 fingerprints. Jelly-drop and water-wheel, which have no chain fixture, were above their base max in both isolated pairs (jelly-drop +1.3% / +6.9%, water-wheel +2.3% / +2.2%).
- A3br (quick 261010-ibo), reverted under D-11 (never committed). A3b on the final engine gained −8.5% / −10.3% on liquid-tumbler with 25/25 fingerprints. Wave-tank, which has no chain fixture, was above its base max in both isolated pairs (+1.4% / +1.1%).

A3 and A3b stay reverted, and this record does not count them as kept. The flagged regressions may have been host noise:

- Neither scene takes the chain path that A3 and A3b changed. A3 adds one per-row branch, and A3b adds no new per-row branch.
- The diagnostic pairs, run after both decisions were fixed, did not reproduce the regressions. Soup-stirrer measured 1.169 / 1.173 against a base of 1.166 / 1.168 (max 1.173 / 1.182). Fountain measured 1.150 / 1.131 against a base of 1.140 / 1.133.
- Across the 7 and 12 scenes re-checked for A3 and A3b, the check fired once per attempt.
- Other plans show the same kind of noise on this host. A1's full run flagged fountain and drawing-particles, and neither held in isolation. A8's isolated pair 1 flagged six scenes, and pair 2 flagged none.
- D-11 as written counts a scene above its base max in both pairs as a regression. Both decisions stand. Notes for Phase 36 lists the retry.

Behavior: 25/25 fingerprints equal the phase before; authored scene settings, controls and web sources unchanged.

## Method

- Host: Apple M4 Max (`sysctl -n machdep.cpu.brand_string`), `arm64` (`uname -m`), 16 logical cores, macOS 26.6.2. The host is shared with an editor, browsers and other apps; load averages are recorded next to every timing run.
- Compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- BEFORE_COMMIT: `809582431faecfdfa13b87f0d8376527bea7f0b4` (docs-only commit on top of the 35-01 tooling commit `4390cda66`; engine and scene code equal the survey baseline `96a3ac6a6`).
- Phase before binary: `target/phase35/bin/spot-before`, a copy of `cargo build --release -p liquidfun-wasm --bin playground-scene-spot` built from BEFORE_COMMIT.
- Phase before output: `target/phase35/before-full.jsonl` (`spot-before --runs 3`, defaults 60 warmup and 120 measured steps).
- Profiling binary: `cargo build --profile profiling -p liquidfun-wasm --bin playground-scene-spot` → `target/profiling/playground-scene-spot` (release plus debug symbols). Its fingerprints match the release build.
- Launch stalls: on this host syspolicyd assesses each newly linked or copied executable before it can start; `spot-before` sat at `_dyld_start` for 34 minutes behind a queue of test binaries. Launch every new binary once with `--warmup 1 --steps 1 --runs 1 --scene particles` before any timed run, and never time a first launch.
- `target/` is gitignored and `cargo clean` removes everything under `target/phase35/`. The scripts below are recorded here so they can be recreated; the before binary would have to be rebuilt from BEFORE_COMMIT.

### Comparison script

`target/phase35/keep_rule.py` (stdlib only, never committed):

```python
# keep_rule.py BEFORE.jsonl AFTER.jsonl TARGET[,TARGET...]
import json, sys
def load(path):
    return {s["scene"]: s for s in map(json.loads, filter(str.strip, open(path)))}
before, after = load(sys.argv[1]), load(sys.argv[2])
targets = [t for t in sys.argv[3].split(",") if t]
shared = sorted(set(before) & set(after))
mismatch = [s for s in shared if before[s]["fingerprint"] != after[s]["fingerprint"]]
gained = set()
for t in targets:
    b, a = before[t], after[t]
    gain = a["median_ms_per_step"] < b["min_ms_per_step"]
    if gain:
        gained.add(t)
    print(f"{t}: before median {b['median_ms_per_step']:.3f} min {b['min_ms_per_step']:.3f} max {b['max_ms_per_step']:.3f} | after median {a['median_ms_per_step']:.3f} -> gain={gain}")
# Every scene that did not meet the gain rule, other targets included, is checked for regression.
over = [s for s in shared if s not in gained and after[s]["median_ms_per_step"] > before[s]["max_ms_per_step"]]
print("fingerprint mismatches:", mismatch or "none")
print("medians above before max (any scene without a gain):", over or "none")
```

### A/B procedure (every later plan)

Run nothing else heavy (no cargo build or test) while timing. Record `uptime` load averages next to each timing run.

1. At plan start, build the release bin from HEAD and copy it to `target/phase35/bin/spot-base-<plan>`.
1. After the change, build again and copy to `target/phase35/bin/spot-<attempt-id>`.
1. Targeted A/B, interleaved ABBA, same flags: `base --scene T... --runs 5 > b1.jsonl`, `after ... > a1.jsonl`, `after ... > a2.jsonl`, `base ... > b2.jsonl`. Run `keep_rule.py b1 a1 T` and `keep_rule.py b2 a2 T`. The target gain must hold in both pairs.
1. Full catalog, `--runs 3`: base then after. `keep_rule.py base-full after-full <targets>`. Re-run any listed scene in isolation with ABBA `--runs 5`. Listed scenes include other targets of a multi-target attempt that did not gain. A scene counts as a regression only if its after median exceeds its base max in both pairs.
1. Fingerprints: `keep_rule.py target/phase35/before-full.jsonl after-full.jsonl ""` must print `fingerprint mismatches: none` for all 25 scenes. The fingerprint check is never waived.
1. Keep rule (D-11): keep only when the target gain holds in both ABBA pairs for at least one targeted scene, no scene (including the other targets of a multi-target attempt) has a confirmed regression, and all 25 fingerprints match the phase before. Otherwise revert, save the diff to `target/phase35/attempts/<attempt-id>.patch`, and record the attempt.
1. Record each attempt as one row in `## Attempts`.

### Profile reader

Profiles are recorded with `samply record --save-only --unstable-presymbolicate -r 4000 -o target/phase35/profiles/<scene>.json.gz -- target/profiling/playground-scene-spot --scene <scene> --runs 1` (writes `<scene>.json.gz` plus `<scene>.json.syms.json`), then read with `python3 target/phase35/top_functions.py target/phase35/profiles/<scene>.json.gz 25 'SessionCore>::advance'`. Only stacks under `SessionCore::advance` count, so scene construction is excluded; warmup steps are included (same per-step mix). The profiled run's fingerprint must equal the before run's.

`target/phase35/top_functions.py` (stdlib only, never committed):

```python
# top_functions.py: self + inclusive % from samply's processed profile + .syms.json sidecar
import bisect, gzip, json, re, sys
from collections import Counter
path, top, needle = sys.argv[1], int(sys.argv[2]), sys.argv[3] if len(sys.argv) > 3 else None
p = json.load(gzip.open(path)); syms = json.load(open(path.replace('.json.gz', '.json.syms.json')))
strings = syms['string_table']; tables = {}
for lib in syms['data']:
    e = sorted((x['rva'], x['size'], strings[x['symbol']]) for x in lib['symbol_table'])
    tables[lib['debug_name']] = ([r[0] for r in e], e)
t = p['threads'][0]; ft, st, fn, rt = t['frameTable'], t['stackTable'], t['funcTable'], t['resourceTable']
def name(f):
    a, res = ft['address'][f], fn['resource'][ft['func'][f]]
    if res is None or res < 0: return '?'
    lib = p['libs'][rt['lib'][res]]['debugName']
    if lib not in tables: return lib
    s, e = tables[lib]; i = bisect.bisect_right(s, a) - 1
    return re.sub(r'::h[0-9a-f]{16}$', '', e[i][2]) if i >= 0 else lib
selfc, incl, total = Counter(), Counter(), 0
for s in t['samples']['stack']:
    names = []
    while s is not None: names.append(name(st['frame'][s])); s = st['prefix'][s]
    if not names or (needle and not any(needle in n for n in names)): continue
    total += 1; selfc[names[0]] += 1
    for n in set(names): incl[n] += 1
for label, c in (('self', selfc), ('inclusive', incl)):
    print('==', label)
    for n, k in c.most_common(top): print(f'{100*k/total:6.2f}%  {n[:160]}')
```

Inclusive shares of named frames that fall outside the top 25 and immediate-caller splits came from a scratch extension of the same reader (`target/phase35/shares.py`: same symbol lookup, counts stacks under `SessionCore::advance` that contain a frame matching each name, and counts the caller frame of selected self-time functions). "Full-scan CCD" counts stacks where `push_fixture_particle_hit` is called directly by `World::filtered_collision_hits` (the `for particle in 0..candidate.positions.len()` fallback at `crates/liquidfun/src/world/particle_coupling.rs:158`) instead of through `query_particles_for_fixture`.

## Fresh before run

`target/phase35/bin/spot-before --runs 3` (60 warmup, 120 steps) on BEFORE_COMMIT, 2026-10-08T02:23Z. Load averages: `9.40 8.62 7.91` before, `9.57 8.80 8.00` after. All 25 scenes finished with `timed_out` false.

| Rank | Scene | Median ms/step | Min ms/step | Max ms/step | End particles | Fingerprint |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 1 | liquid-tumbler | 25.429 | 25.413 | 25.787 | 3800 | `a6c5bd97664706d5` |
| 2 | tesla-valve | 4.061 | 4.018 | 4.070 | 2850 | `23f7e49044ac5cb7` |
| 3 | stacked-drip | 2.798 | 2.785 | 2.800 | 2000 | `f2cf7c52ea222dc4` |
| 4 | washing-machine | 2.164 | 2.159 | 2.204 | 3168 | `d4d0d0663303a1af` |
| 5 | water-wheel | 1.661 | 1.528 | 2.083 | 3200 | `f80c9a620e51fe2e` |
| 6 | particles | 1.645 | 1.636 | 1.758 | 4569 | `6d3e96f312d14408` |
| 7 | liquid-bubbler | 1.535 | 1.531 | 1.573 | 3000 | `a6d759013dcce046` |
| 8 | soup-stirrer | 1.241 | 1.231 | 1.247 | 2792 | `38661d5bf1e5f405` |
| 9 | soup | 1.214 | 1.201 | 1.229 | 2965 | `3e413d5a6ee64a59` |
| 10 | fountain | 1.166 | 1.160 | 1.191 | 3200 | `34f6bded79a5df08` |
| 11 | wave-tank | 0.981 | 0.965 | 1.009 | 2500 | `653ca1fc8eb83325` |
| 12 | wave-machine | 0.936 | 0.934 | 0.964 | 2256 | `ac22aea195c85cf0` |
| 13 | hydraulic-fountain | 0.849 | 0.838 | 0.860 | 3200 | `c9c1aa23e854c29e` |
| 14 | liquid-timer | 0.795 | 0.776 | 0.812 | 2247 | `705be2a1a0d4e4d4` |
| 15 | impulse | 0.786 | 0.783 | 0.789 | 2279 | `8b02cdf816e25ab3` |
| 16 | elastic-particles | 0.572 | 0.560 | 0.581 | 1327 | `d8303cb7def524eb` |
| 17 | dam-break | 0.559 | 0.557 | 0.849 | 1920 | `4fa65d209b7e782f` |
| 18 | float-or-sink | 0.536 | 0.518 | 0.546 | 1800 | `8037c9b1dbe97fbd` |
| 19 | jelly-drop | 0.411 | 0.310 | 0.737 | 793 | `e38828993cdf3f6c` |
| 20 | color-mixer | 0.382 | 0.327 | 0.398 | 1154 | `3231c14625c0e213` |
| 21 | surface-tension | 0.316 | 0.314 | 0.317 | 937 | `0131dc4886aff960` |
| 22 | rigid-particles | 0.273 | 0.263 | 0.276 | 1327 | `9dc7520b2771f7fc` |
| 23 | theo-jansen | 0.210 | 0.209 | 0.218 | 141 | `2fae3a044142fab3` |
| 24 | sparky | 0.159 | 0.154 | 0.159 | 550 | `731c2b32c0a76400` |
| 25 | drawing-particles | 0.012 | 0.012 | 0.012 | 24 | `10e5b0afea0512d6` |

Stability: this run is noisy for several scenes under the shared-host load. Min/max spread is above 10% for water-wheel (34%), dam-break (52%), jelly-drop (104%) and color-mixer (19%). The five targets stayed between 0.6% and 7.4% (particles 7.4%, the rest at or below 2%).

Comparison with the committed survey (`docs/benchmarks/scene-survey.md`, commit `96a3ac6a6`): most medians are 2–6% higher than the survey on this busier host (tesla-valve is 1.4% lower; the noisy small scenes sparky, color-mixer, water-wheel and jelly-drop are 12–83% higher), so compare only saved binaries run back to back. The top four keep their order. Water-wheel (rank 7 in the survey) ranks 5 here, just above particles, but only because one of its three runs hit 2.083 ms; its own spread is 34%. A second confirmation invocation of the same binary right after (load `9.43` to `9.89`, saved as `target/phase35/before-full-2.jsonl`, all 25 fingerprints equal) gave water-wheel 1.346 (1.327–1.401) at rank 7 and the survey's top five in survey order: liquid-tumbler 25.513, tesla-valve 4.139, stacked-drip 2.794, washing-machine 2.221, particles 1.668. The target set stays the D-01 five. `before-full.jsonl` remains the phase reference for fingerprints.

## Browser observation (D-03)

Not run: the 35-02 executor had no browser tool (no in-app browser pane or browser automation in its toolset), and `just web-build` would link fresh native build-script executables into the same syspolicyd launch queue that held `spot-before` at `_dyld_start` for 34 minutes this session. The native top five remain the target set.

## Target profiles

All five were recorded on 2026-10-08 at about 02:25Z from `target/profiling/playground-scene-spot` built at BEFORE_COMMIT `809582431`, sequentially with nothing else of ours running (load averages `7.63 8.42 7.94` before the first, `7.00 8.23 7.88` after the last). Each profiled run printed the same fingerprint as `before-full.jsonl`. The ms/step printed under samply (27.42, 4.36, 2.93, 2.39, 1.78) includes sampling overhead and is not timing evidence. Shares are percentages of samples under `SessionCore::advance`. The four smaller scenes ran for about half a second each, so their shares carry roughly ±1 percentage point of sampling noise.

These fresh profiles agree with the research profiles (HEAD `1e5908c45`, same engine code). No target's top function changed. Small shifts: stacked-drip `consider_window` self is 13.9% here versus 16.8% in research, and washing-machine's full-scan CCD share is 29.1% versus 30.0%.

### liquid-tumbler

- Profile: `target/phase35/profiles/liquid-tumbler.json.gz` (commit `809582431`), 20,195 samples under `SessionCore::advance`.
- Top 10 self:
  1. `pressure::damping` 22.92%
  1. `contact_scan::consider_window` 9.69%
  1. `pressure::pressure` 7.71%
  1. `ParticleStorage::refresh_solver_weights` 5.68%
  1. stable `quicksort::<ContactProxy>` (proxy sort) 5.52%
  1. `contact_scan::fill_stored_contacts` 5.18%
  1. `ChainShape::child_edge` 4.89%: 2.94% via `Shape::distance_to_point` (body contacts), 1.92% via `Shape::ray_cast` (CCD)
  1. `_platform_memmove` 3.82%: `BoundaryCandidate::new_with_buffers` 1.28%, `boundary::support::copy_into` 0.79%, proxy sort 0.69%
  1. `small_sort_general_with_scratch::<ContactProxy>` (proxy sort) 3.09%
  1. `push_fixture_particle_hit` 2.79%
- Inclusive: `World::update_particle_contacts` 27.00% (`fill_stored_contacts` 26.71%); `contact_scan::rebuild_proxies` 11.84% (proxy sort `driftsort_main::<ContactProxy>` 10.97%); `World::update_body_contacts` 18.01% (`body_contact::generate` 16.79%, `collect_candidate_rows` with its `sort_unstable::<usize>` 6.18%); `run_collision` 14.40% (`filtered_collision_hits` 12.00%, `query_particles_for_fixture` 11.48%, full-scan CCD 0.00%); `pressure::damping` 23.30%; `pressure::pressure` 8.38%; `refresh_solver_weights` 5.71%; `begin_boundary` 2.38%; `solve_lifetimes` 0.03%; `backup_step_limit_state` 0.30%. No scene hook frames (`on_advance`, `on_after_step`) appear.

**Hot path:** `pressure::damping` 22.9% self (particle-contact damping loop, run in each of the 61 particle iterations per step)

The damping loop is a dependent scatter whose float order D-07 locks; its only attempt (A0, branchless) was rejected. The reachable redundant work is the per-iteration proxy rebuild and sort (11.8% inclusive), the per-particle chain edge rebuild (4.9%) and the candidate-row sort (6.2% inclusive with the AABB visit).

Planned attempts: 35-03 proxy order reuse; 35-04 chain edge hoist; 35-05 AABB query and candidate rows.

### tesla-valve

- Profile: `target/phase35/profiles/tesla-valve.json.gz` (commit `809582431`), 2,458 samples under `SessionCore::advance`. Particles grow from 0 to 2,850 over the run.
- Top 10 self:
  1. `_platform_memmove` 8.42%: `ParticleStorage::prepare_create` 3.30%, `libsystem_malloc` (realloc) 3.25%, `BoundaryCandidate::new_with_buffers` 0.37%
  1. `proxy::visit_sorted_tag_indices_in_aabb` in `query_particles_for_fixture` (CCD) 6.92%
  1. `proxy::visit_sorted_tag_indices_in_aabb` in `body_contact::collect_candidate_rows` 6.75%
  1. `push_fixture_particle_hit` 6.63%
  1. `pressure::damping` 5.57%
  1. `contact_scan::consider_window` 5.49%
  1. `body_contact::generate` 5.29%
  1. `PolygonShape::ray_cast` 5.29%
  1. SipHash `Hasher::write` 3.58% (from `Identity::hash` with the std random-state hasher)
  1. `contact_scan::fill_stored_contacts` 3.58%
  - Next: `storage::validation::membership_ranges` 2.97%, `BTreeMap<(Reverse<i32>, u64), ParticleId>::insert` 2.64% (from `EvictionIndex::insert_at`)
- Inclusive: `World::update_particle_contacts` 13.10%; `contact_scan::rebuild_proxies` 3.95% (proxy sort 3.62%); `World::update_body_contacts` 24.74% (`body_contact::generate` 22.17%, `collect_candidate_rows` 12.04%); `run_collision` 25.31% (`filtered_collision_hits` 24.57%, `query_particles_for_fixture` 23.19%, full-scan CCD 0.00%); `pressure::damping` 6.10%; `pressure::pressure` 2.40%; `refresh_solver_weights` 1.30%; `begin_boundary` 0.65%; `backup_step_limit_state` 1.26%.
- Scene and lifetime frames, separate from the solver: `TeslaValveHooks::on_advance` 8.91% (all of it `World::create_particle_with_def`, of which `prepare_create` 8.79%); `on_after_step` 4.56% (`destroy_particles_in_shape` 4.56%); `ParticleLifetimeState::solve_lifetimes` 9.03% (`EvictionIndex::resequence_to_storage_order` 8.62%).

**Hot path:** `visit_sorted_tag_indices_in_aabb` 13.7% self (two instantiations: CCD fixture queries 6.9% under `run_collision`, body-contact candidate rows 6.8% under `update_body_contacts`)

The profile is spread out. Emission (8.9% inclusive) and lifetime resequencing (9.0% inclusive) are each about as large as the AABB query cost, and `_platform_memmove` (8.4% self) is mostly emission copies and reallocations.

Planned attempts: 35-05 AABB query and candidate rows; 35-07 emission and lifetime index.

### stacked-drip

- Profile: `target/phase35/profiles/stacked-drip.json.gz` (commit `809582431`), 2,009 samples under `SessionCore::advance`.
- Top 10 self:
  1. `pressure::damping` 17.07%
  1. `contact_scan::consider_window` 13.94%
  1. `contact_scan::fill_stored_contacts` 7.81%
  1. `pressure::pressure` 6.07%
  1. `_platform_memmove` 5.08%: `BoundaryCandidate::new_with_buffers` 1.49%, proxy sort 1.05%, `boundary::support::copy_into` 0.85%
  1. stable `quicksort::<ContactProxy>` (proxy sort) 4.93%
  1. `small_sort_general_with_scratch::<ContactProxy>` (proxy sort) 4.13%
  1. `ParticleStorage::refresh_solver_weights` 3.93%
  1. `push_fixture_particle_hit` 3.29%
  1. `proxy::visit_sorted_tag_indices_in_aabb` in `collect_candidate_rows` 2.99% (tied with `body_contact::generate` 2.99%)
- Inclusive: `World::update_particle_contacts` 35.09% (`fill_stored_contacts` 34.89%); `contact_scan::rebuild_proxies` 13.14% (proxy sort 12.15%); `World::update_body_contacts` 16.97% (`body_contact::generate` 14.63%, `collect_candidate_rows` 6.67%); `run_collision` 13.69% (`filtered_collision_hits` 10.75%, `query_particles_for_fixture` 8.71%, full-scan CCD 1.19%); `pressure::damping` 17.72%; `pressure::pressure` 7.37%; `refresh_solver_weights` 3.98%; `begin_boundary` 2.89%; `solve_lifetimes` 0.05%; `backup_step_limit_state` 0.50%. No scene hook frames appear.

**Hot path:** `pressure::damping` 17.1% self (particle-contact damping loop, 12 particle iterations per step), with `consider_window` at 13.9% self and the proxy rebuild plus sort at 13.1% inclusive close behind

Planned attempts: 35-03 proxy order reuse; 35-05 AABB query and candidate rows.

### washing-machine

- Profile: `target/phase35/profiles/washing-machine.json.gz` (commit `809582431`), 1,695 samples under `SessionCore::advance`.
- Top 10 self:
  1. `push_fixture_particle_hit` 15.22%: 14.16% called directly from the full-scan loop in `filtered_collision_hits`, 1.06% through the spatial query
  1. `contact_scan::consider_window` 11.56%
  1. `pressure::damping` 10.91%
  1. `boundary::collision::collision_start_from_previous_transform` 9.50% (all under `push_fixture_particle_hit`)
  1. `contact_scan::fill_stored_contacts` 5.37%
  1. `particle_coupling::particle_travel_aabb` 4.25%
  1. `ParticleStorage::refresh_solver_weights` 3.83%
  1. `pressure::pressure` 3.30%
  1. `_platform_memmove` 3.30%: `Vec<Slot<ParticleSystem>>::clone` (per-step world backup) 1.06%, `BoundaryCandidate::new_with_buffers` 0.47%
  1. `material::solid` 2.48%
- Inclusive: `World::update_particle_contacts` 21.18%; `contact_scan::rebuild_proxies` 4.19% (proxy sort 3.66%); `World::update_body_contacts` 9.03% (`body_contact::generate` 7.61%, `collect_candidate_rows` 2.89%); `run_collision` 34.22% (`filtered_collision_hits` 33.27%, `query_particles_for_fixture` 2.71%, **full-scan CCD 29.14%**); `pressure::damping` 11.33%; `pressure::pressure` 4.25%; `refresh_solver_weights` 3.89%; `begin_boundary` 0.88%; `solve_lifetimes` 0.53%; `backup_step_limit_state` 2.60%. No scene hook frames appear.

**Hot path:** `push_fixture_particle_hit` 15.2% self (full-scan CCD for the moving drum fixtures at particle iteration 0, `particle_coupling.rs:158`; 29.1% inclusive with `collision_start_from_previous_transform` and `particle_travel_aabb`)

Planned attempts: 35-06 moving-fixture CCD filter.

### particles

- Profile: `target/phase35/profiles/particles.json.gz` (commit `809582431`), 1,131 samples under `SessionCore::advance`.
- Top 10 self:
  1. `pressure::damping` 22.81%
  1. `contact_scan::consider_window` 18.57%
  1. `contact_scan::fill_stored_contacts` 7.87%
  1. `pressure::pressure` 7.16%
  1. `ParticleStorage::refresh_solver_weights` 6.72%
  1. `_platform_memmove` 5.04%: `Vec<Slot<ParticleSystem>>::clone` (per-step world backup) 2.48%, `copy_into` 0.53%, `BoundaryCandidate::new_with_buffers` 0.53%, proxy sort 0.53%
  1. stable `quicksort::<ContactProxy>` (proxy sort) 3.63%
  1. `push_fixture_particle_hit` 2.30%
  1. `libsystem_malloc` 2.21%
  1. `small_sort_general_with_scratch::<ContactProxy>` (proxy sort) 2.21%
- Inclusive: `World::update_particle_contacts` 35.54% (`fill_stored_contacts` 35.37%); `contact_scan::rebuild_proxies` 8.93% (proxy sort 8.40%); `World::update_body_contacts` 6.81% (`body_contact::generate` 5.39%, `collect_candidate_rows` 2.03%); `run_collision` 7.60% (`filtered_collision_hits` 6.01%, `query_particles_for_fixture` 2.74%, full-scan CCD 2.83%); `pressure::damping` 23.43%; `pressure::pressure` 8.13%; `refresh_solver_weights` 6.72%; `begin_boundary` 1.41%; `solve_lifetimes` 0.88%; `backup_step_limit_state` 4.86%. No scene hook frames appear.

**Hot path:** `pressure::damping` 22.8% self (particle-contact damping loop), with `consider_window` at 18.6% self and the proxy rebuild plus sort at 8.9% inclusive

Scene construction (`create_particle_group`, O(n²) `prepare_create`) is outside `SessionCore::advance` and outside the timed window; it is not a PERF-08 target.

Planned attempts: 35-03 proxy order reuse.

## Attempts

| ID | Plan | Change | Targets | Base median (min-max) | After median (ABBA pair 1 / pair 2) | Fingerprints equal | Regression (any scene) | Decision | Commit | Reason |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A0 | research | branchless particle damping loop (select_unpredictable; two variants incl. signed-zero safe) | liquid-tumbler | 21.2-22.1 | 24.0-25.3 | yes | not measured | rejected | none | slower: unconditional stores lengthen the dependent store-to-load chain; damping is a dependent scatter whose float order D-07 locks |
| A1 | 35-03 | reuse the last sorted contact-proxy order across particle iterations: retag in place, `sort_unstable_by_key((tag, row))`; row-order rebuild when the length differs or a tag fails (`ProxyOrderCache` in `ParticleStorage`) | liquid-tumbler, stacked-drip, particles | base `spot-base-03` (pair 1 / pair 2): liquid-tumbler 24.179 (24.139-24.299) / 24.404 (24.358-24.539); stacked-drip 2.628 (2.616-2.642) / 2.668 (2.643-2.674); particles 1.576 (1.567-1.582) / 1.627 (1.598-1.629) | liquid-tumbler 23.345 / 23.259; stacked-drip 2.550 / 2.525; particles 1.552 / 1.563 | yes (25/25): `target/phase35/A1-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A1-base-full.jsonl` → `A1-full.jsonl` listed fountain (1.165 > 1.161) and drawing-particles (0.0107 > 0.0105); isolated ABBA `--runs 5` (`A1-iso-*`): fountain 1.130 / 1.135 vs base max 1.155 / 1.157, drawing-particles 0.0104 / 0.0104 vs 0.0109 / 0.0109 | kept | `91b27a6d6` | all three targets below the base min in both pairs (liquid-tumbler −3.4% / −4.7%, stacked-drip −3.0% / −5.4%, particles −1.5% / −3.9%); the base itself drifted about 3% between pairs (load 7.8–9.2), and the particles pair 1 margin is small (1.552 vs min 1.567) |
| A2 | 35-03 | A1 plus a bounded insertion sort (at most 8·n moves, then `sort_unstable_by_key` on the partly sorted buffer) for the retained order | liquid-tumbler, stacked-drip, particles | base `spot-A1` (pair 1 / pair 2): liquid-tumbler 22.851 (22.608-23.068) / 23.097 (23.076-23.157); stacked-drip 2.465 (2.453-2.479) / 2.490 (2.474-2.517); particles 1.498 (1.484-1.502) / 1.529 (1.505-1.546) | liquid-tumbler 21.587 / 21.679; stacked-drip 2.341 / 2.347; particles 1.545 / 1.560 | yes (25/25): `target/phase35/A2-full.jsonl` vs `before-full.jsonl` | confirmed: particles above the A1 max in both targeted pairs (1.545 > 1.502, 1.560 > 1.546); washing-machine isolated ABBA (`A2-iso-*`) 2.084 / 2.080 vs A1 max 2.042 / 2.058. Fountain and impulse exceeded the max in pair 1 only; theo-jansen in neither | reverted | none (never committed; diff in `target/phase35/attempts/A2.patch`) | faster than A1 on liquid-tumbler (−5.5% / −6.1%) and stacked-drip (−5.0% / −5.7%) but particles and washing-machine regressed in both pairs, so D-11 fails; likely the budget runs out on their more mixed orders and the insertion moves are paid before the full sort |
| A3 | 35-04 | chain child edge built once per child (body contacts + CCD): `maybe_child_edge` in `body_contact::generate`, `CcdChild { index, maybe_aabb, maybe_edge }` in CCD records | liquid-tumbler | base `spot-base-04` (pair 1 / pair 2): liquid-tumbler 23.590 (23.376-23.961) / 23.815 (23.481-23.897) | liquid-tumbler 21.371 / 21.336 | yes (25/25): `target/phase35/A3-full.jsonl` vs `before-full.jsonl` | confirmed: soup-stirrer (no chain fixture) above the base max in both isolated pairs (`A3-iso-*`): 1.171 > 1.160, 1.181 > 1.176. Full run `A3-base-full.jsonl` → `A3-full.jsonl` listed jelly-drop, liquid-timer, soup, soup-stirrer, surface-tension, tesla-valve and theo-jansen; tesla-valve exceeded the max in pair 2 only (3.926 > 3.912; pair 1 3.926 < 4.060), the other five in neither | reverted | `0a2fe8cb8` (reverted by `7f0b36b18`; diff in `target/phase35/attempts/A3.patch`) | liquid-tumbler gained in both pairs (−9.4% / −10.4%) but D-11 fails on soup-stirrer (+1.5% / +1.5%), a scene that never takes the chain path, so the cost (if real) is the extra per-row branch or a code-layout shift. A later diagnostic ABBA (`A3-diag-*`, not a decision input) did not reproduce it: soup-stirrer 1.169 / 1.173 vs base 1.166 / 1.168, max 1.173 / 1.182 |
| A3b | 35-04 | A3 variant with no new per-row branch: a chain child is queried as a prebuilt `Shape::Edge` at child index 0 through the same `Shape::distance_to_point` / `Shape::ray_cast` call; CCD emits one record per chain child in fixture-then-child order | liquid-tumbler | base `spot-base-04` (pair 1 / pair 2): liquid-tumbler 23.383 (23.262-23.626) / 23.365 (23.112-23.426) | liquid-tumbler 21.608 / 21.651 | yes (25/25): `target/phase35/A3b-full.jsonl` vs `before-full.jsonl` | confirmed: fountain (no chain fixture) above the base max in both isolated pairs (`A3b-iso-*`): 1.168 > 1.164, 1.148 > 1.137. Full run `A3b-base-full.jsonl` → `A3b-full.jsonl` (load 8.5–9.5) listed 12 scenes; hydraulic-fountain, surface-tension and washing-machine exceeded the max in pair 1 only, the other eight in neither | reverted | none (never committed; diff in `target/phase35/attempts/A3b.patch`) | liquid-tumbler gained in both pairs (−7.6% / −7.3%) but D-11 fails on fountain (+1.7% / +1.8%), again a scene without a chain. The diagnostic ABBA (`A3b-diag-*`) did not reproduce it: fountain 1.150 / 1.131 vs base 1.140 / 1.133 |
| A4 | 35-05 | per-y-row binary search in `visit_sorted_tag_indices_in_aabb`: inside the unchanged `first..last` window, two ranged partition-point searches per tag row (`binary_partition_point_in`) replace the x-mask scan; the linear scan stays when rows · 2 · bit length of the window ≥ window. Equivalence tests against a verbatim copy of the scan (grid, 2,000 random tags × 200 boxes, empty, error cases) passed 4/4 | tesla-valve, stacked-drip, liquid-tumbler | base `spot-base-05` (pair 1 / pair 2): liquid-tumbler 24.604 (24.501-24.757) / 24.253 (24.162-24.363); stacked-drip 2.699 (2.572-2.748) / 2.634 (2.620-2.706); tesla-valve 4.103 (4.050-4.150) / 4.154 (4.019-4.504) | liquid-tumbler 24.776 / 24.946; stacked-drip 2.674 / 2.665; tesla-valve 4.146 / 4.188 | yes (25/25): `target/phase35/A4-full.jsonl` vs `before-full.jsonl` | confirmed: liquid-tumbler above the base max in both targeted pairs (24.776 > 24.757, 24.946 > 24.363). The full run `A4-base-full.jsonl` → `A4-full.jsonl` (load 9.9–11.3) listed liquid-tumbler and 15 other scenes; isolated ABBA `--runs 5` of those 15 (`A4-iso-*`): elastic-particles 0.576 / 0.571 vs base max 0.575 / 0.570, liquid-timer 0.801 / 0.802 vs 0.787 / 0.781, rigid-particles 0.271 / 0.273 vs 0.263 / 0.263, surface-tension 0.333 / 0.328 vs 0.329 / 0.318 in both pairs; six more in pair 1 only, liquid-bubbler in pair 2 only, four in neither | reverted | none (never committed; diff in `target/phase35/attempts/A4.patch`) | no target gained in either pair (tesla-valve +1.0% / +0.8%, stacked-drip −0.9% / +1.2%, liquid-tumbler +0.7% / +2.9%), and five scenes were above the base max in both pairs, three of them by 3–6% (liquid-timer +5.1% / +4.2%, rigid-particles +3.4% / +5.3%, surface-tension +5.7% / +3.9%). Likely cause: the per-row searches' scattered probes cost more than the sequential scan they skip for the box sizes these scenes query, so the profile's 13.7% self share in tesla-valve is the scan's cost, not wasted work |
| A5 | 35-05 | ordered bitset walk for body-contact candidate rows: when a fixture query visits 32 or more rows, `collect_candidate_rows` sets one bit per row in a `row_marks` buffer reused across one `generate` call and reads the marked words in ascending order with `trailing_zeros`, clearing each word as it goes, instead of `sort_unstable` + `dedup`; fewer rows keep the sort. Tests against the sort-plus-dedup reference passed on the old and new code | liquid-tumbler, stacked-drip, tesla-valve | base `spot-base-05b` (hard link of `spot-base-05`, engine A1; pair 1 / pair 2): liquid-tumbler 23.424 (23.315-23.538) / 23.698 (23.482-23.848); stacked-drip 2.524 (2.488-2.545) / 2.584 (2.531-2.641); tesla-valve 3.839 (3.809-3.900) / 3.920 (3.876-3.948) | liquid-tumbler 22.724 / 22.978; stacked-drip 2.482 / 2.546; tesla-valve 3.805 / 3.833 | yes (25/25): `target/phase35/A5-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A5-base-full.jsonl` → `A5-full.jsonl` listed fountain (1.138 > 1.134), stacked-drip (2.553 > 2.539) and washing-machine (2.076 > 2.057); isolated ABBA `--runs 5` (`A5-iso-*`): fountain 1.134 / 1.131 vs base max 1.149 / 1.157, stacked-drip 2.485 / 2.482 vs 2.576 / 2.612 (below the base min in both pairs), washing-machine 2.054 / 2.047 vs 2.074 / 2.416. Stacked-drip in targeted pair 2 (no gain) stayed below the base max (2.546 vs 2.641) | kept | `216de3929` | liquid-tumbler (−3.0% / −3.0%) and tesla-valve (−0.9% / −2.2%) below the base min in both targeted pairs; stacked-drip below the min in targeted pair 1 only (−1.7% / −1.5%; pair 2 2.546 vs min 2.531) and in both isolated pairs (−2.3% / −1.8%), but only the targeted pairs count toward the gain. The tesla-valve pair 1 margin is small (3.805 vs min 3.809) |
| A6 | 35-06 | conservative spatial query for moving fixtures at particle iteration 0: `moving_fixture_query_pad` bounds the start remap `S(p) = M p + t` (convex displacement, corner maximum `D0`, `L`, the inf-norm of `M - I`) by the closed form `max(motion, D0) / (1 - L)` with relative, absolute and per-meter rounding margins, a 5% overshoot and one f32 corner check, and `query_particles_for_fixture` uses that pad instead of the `0..n` scan. `None` (full scan) when `L >= 1`, an input is not finite, the check fails, or a candidate velocity is not finite. 7 unit tests compare filtered and full-scan hit lists bit for bit (rotating polygon, rotating and translating circle, chain plus edge, fast rotation, static and iteration 1, and a 0.3 rad swept edge whose hits lie outside the motion-only pad); a motion-only pad fails 3 of them | washing-machine | base `spot-base-06` (pair 1 / pair 2): washing-machine 2.024 (2.000-2.029) / 2.018 (2.006-2.053) | washing-machine 1.436 / 1.435. Shared path, non-targets (`A6-shared-*`, base → after): soup-stirrer 1.130 → 1.013 / 1.146 → 1.014, water-wheel 1.254 → 1.192 / 1.276 → 1.210, theo-jansen 0.195 → 0.147 / 0.198 → 0.144 (all three below the base min in both pairs) | yes (25/25): `target/phase35/A6-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A6-base-full.jsonl` → `A6-full.jsonl` listed hydraulic-fountain (0.797 > 0.797, base max 0.7968) and liquid-timer (0.720 > 0.713); isolated ABBA `--runs 5` (`A6-iso-*`): hydraulic-fountain 0.786 / 0.787 vs base max 0.793 / 0.811, liquid-timer 0.716 / 0.724 vs 0.723 / 0.732 | kept | `2256dd8cd` | washing-machine below the base min in both pairs (−29.0% / −28.9%); the full run also shows liquid-bubbler −18.5%, soup −11.7%, soup-stirrer −9.9% and theo-jansen −24.5%. FIXTURE_CONTACT_FILTER hook calls for moving fixtures at iteration 0 now run in query order instead of row order (hit list unchanged; matches the static path) |
| A8 | 35-07 | ungrouped-append fast path in particle creation: for `maybe_group == None`, `prepare_create` clones `group_records` and re-applies `retain_empty_after_member_removal` to the trailing empty records instead of cloning the group lane and running `rebuild_group_records_for_system` / `membership_ranges` (O(n) per call, twice per emitted particle). Targets `prepare_create` 8.79% inclusive in the 35-02 tesla-valve profile (`membership_ranges` 2.97% self, its group-lane `_platform_memmove` 3.30%). A `debug_assert_eq!` re-checks equality with the full rebuild; the grouped path and every error check are unchanged, and an explicit `i32::MAX` lane check keeps the rebuild's limit. A plain clone is not equal: the rebuild resets the cached statistics timestamp of an empty retained group, which a test reproduces. 4 tests compare storage (`ParticleStorage` `PartialEq`) and errors against a verbatim copy of the old `prepare_create` | tesla-valve (fountain, water-wheel, sparky reported) | base `spot-base-07` (hard link of `spot-A6`; pair 1 / pair 2): tesla-valve 3.895 (3.876-3.928) / 3.960 (3.928-4.015); fountain 1.147 (1.139-1.164) / 1.157 (1.152-1.163); water-wheel 1.229 (1.224-1.247) / 1.235 (1.218-1.254); sparky 0.134 (0.130-0.139) / 0.137 (0.132-0.139) | tesla-valve 3.593 / 3.477; fountain 0.915 / 0.918; water-wheel 1.012 / 0.987; sparky 0.136 / 0.135 | yes (25/25): `target/phase35/A8-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A8-base-full.jsonl` → `A8-full.jsonl` (load 9.5–12.8, rising during the after run) listed color-mixer, dam-break, float-or-sink, impulse, liquid-timer, liquid-tumbler, particles, soup-stirrer, wave-machine and wave-tank; isolated ABBA `--runs 5` of all ten (`A8-iso-*`, load 6.3 → 13.1 during the a1/a2 runs): pair 1 listed dam-break 0.500 > 0.495, float-or-sink 0.482 > 0.478, impulse 0.742 > 0.733, liquid-tumbler 24.475 > 24.382, particles 1.558 > 1.546 and wave-tank 0.899 > 0.895; pair 2 listed none (dam-break 0.509 vs max 0.514, float-or-sink 0.475 vs 0.493, impulse 0.743 vs 0.774, liquid-tumbler 24.189 vs 24.985, particles 1.599 vs 1.625, wave-tank 0.904 vs 1.007). Sparky stayed below the base max in both targeted pairs | kept | `5695f4b39` | tesla-valve below the base min in both pairs (−7.8% / −12.2%); the other emitting scenes also gained in both pairs: fountain −20.2% / −20.7%, water-wheel −17.7% / −20.1%. Sparky (550 particles) did not change measurably |
| A9 | 35-07 | cheaper lifetime eviction index (sub-steps a, b and c): (a) `by_particle` uses an in-tree deterministic `ParticleIdHasher` (multiply-rotate per written integer, SplitMix64 finalizer) instead of SipHash; it is only used for keyed lookup and is never iterated; (b) `resequence_to_storage_order` walks the two `BTreeMap`s once in storage order, assigns `POSITION_GAP * (i + 1)` exactly as `from_ordered_entries` → `insert_new` did, updates each `Placement` in place through `get_mut`, rebuilds both maps with `collect` over already sorted keys, and sets `next_position` to the same `POSITION_GAP * (n + 1)`, instead of building a `Vec` of the order, a fresh `HashMap` and 2n `BTreeMap` inserts; (c) `from_ordered_entries` pre-sizes the map. Targets `solve_lifetimes` 9.03% inclusive in the 35-02 tesla-valve profile (`resequence_to_storage_order` 8.62%, SipHash `Hasher::write` 3.58% self, `BTreeMap::insert` 2.64% self). Tests: 2,000 fixed-seed mixed upserts (finite, zero and negative expirations) and removals with 50 resequences equal a reference that keeps the original rebuild body (whole-index `==`, `storage_order()`, and `oldest(rank)` for every rank at each resequence); `PartialEq` still compares contents | tesla-valve (fountain, water-wheel, sparky reported) | base `spot-base-07b` (hard link of `spot-A8`; pair 1 / pair 2): tesla-valve 3.598 (3.512-3.698) / 3.555 (3.514-3.608); fountain 0.910 (0.892-0.970) / 0.927 (0.920-0.931); water-wheel 0.993 (0.988-1.009) / 1.012 (0.963-1.019); sparky 0.135 (0.131-0.141) / 0.132 (0.130-0.133) | tesla-valve 3.269 / 3.266; fountain 0.665 / 0.672; water-wheel 0.739 / 0.733; sparky 0.135 / 0.138 | yes (25/25): `target/phase35/A9-full.jsonl` vs `before-full.jsonl` | none confirmed. Sparky was above the base max in targeted pair 2 only (0.138 > 0.133; pair 1 0.135 < 0.141). Full run `A9-base-full.jsonl` → `A9-full.jsonl` (load 6.7–9.3; tesla-valve 3.537 → 3.246) listed dam-break, hydraulic-fountain, jelly-drop and particles; isolated ABBA `--runs 5` of those four plus sparky (`A9-iso-*`, load 7.9–9.3): pair 1 listed none; pair 2 listed dam-break 0.499 > 0.496 and jelly-drop 0.238 > 0.233 (pair 1: dam-break 0.493 vs max 0.510, below the base min; jelly-drop 0.234 vs 0.235). Hydraulic-fountain 0.807 / 0.814 vs 0.811 / 0.821, particles 1.557 / 1.579 vs 1.609 / 1.606, sparky 0.138 / 0.135 vs 0.141 / 0.146 | kept | `f7041fc75` | tesla-valve below the base min in both pairs (−9.1% / −8.1%); fountain −26.9% / −27.5% and water-wheel −25.6% / −27.6% also gained in both pairs. The scenes flagged in one pair (sparky, dam-break, jelly-drop) did not exceed their base max in the other |
| A3r | quick 261010-ibo | retry of A3 on the A1+A5+A6+A8+A9 engine: `attempts/A3.patch` ported onto A6's `collect_fixture_hits` loop (the moving-fixture pad match and the `maybe_velocities_finite` cache unchanged; the loop iterates `CcdChild` records and passes them to `push_fixture_particle_hit`). A3's 3 bit-identity tests plus `chain_child_edge_records_cast_like_chain_children` (CCD hits with prebuilt chain child edges equal hits through `Shape::ray_cast(.., child)`, bit for bit); A3's body-contact tests moved to `body_contact/chain_edge_tests.rs` for the file-length limit | liquid-tumbler | base `spot-base-ibo` (hard link of `spot-final`, engine A1+A5+A6+A8+A9; pair 1 / pair 2): liquid-tumbler 21.772 (21.563-21.829) / 22.674 (22.274-22.866) | liquid-tumbler 19.849 / 20.308 | yes (25/25): `target/phase35/A3r-full-a1.jsonl` vs `before-full.jsonl` | confirmed: jelly-drop and water-wheel (no chain fixture in either scene) above the base max in both isolated pairs (`A3r-iso-*`): jelly-drop 0.232 > 0.230, 0.249 > 0.234; water-wheel 0.742 > 0.739, 0.759 > 0.745. Full run `A3r-full-b1.jsonl` → `A3r-full-a1.jsonl` listed 11 scenes (color-mixer, hydraulic-fountain, jelly-drop, liquid-timer, particles, soup, stacked-drip, surface-tension, tesla-valve, water-wheel, wave-tank); liquid-timer, particles, soup, stacked-drip and surface-tension exceeded the max in pair 1 only, the other four in neither | reverted | none (never committed; diff in `target/phase35/attempts/A3r.patch`) | liquid-tumbler gained in both pairs (−8.8% / −10.4%) but D-11 fails on jelly-drop (+1.3% / +6.9% against the base median) and water-wheel (+2.3% / +2.2%). A later diagnostic ABBA (`A3r-diag-*`, not a decision input) did not reproduce either: jelly-drop 0.226 / 0.227 vs base max 0.230 / 0.232, water-wheel 0.719 / 0.714 vs 0.730 / 0.741 |
| A3br | quick 261010-ibo | retry of A3b on the A1+A5+A6+A8+A9 engine: `attempts/A3b.patch` applied cleanly (one `Shape::Edge` record per chain child at child index 0, fixture-then-child order). Mechanical drift only: the per-child edge lookup moved into `chain_child_edge_shape` because A5 had grown `body_contact::generate` and the patch put it at 101 lines (`clippy::too_many_lines`), and A3b's body-contact tests moved to `body_contact/chain_edge_tests.rs` for the file-length limit. A3b's 4 tests pass | liquid-tumbler | base `spot-base-ibo` (pair 1 / pair 2): liquid-tumbler 22.143 (21.956-22.681) / 22.495 (22.369-22.719) | liquid-tumbler 20.263 / 20.170 | yes (25/25): `target/phase35/A3br-full-a1.jsonl` vs `before-full.jsonl` | confirmed: wave-tank (no chain fixture) above the base max in both isolated pairs (`A3br-iso-*`): 0.854 > 0.852, 0.852 > 0.850. Full run `A3br-full-b1.jsonl` → `A3br-full-a1.jsonl` listed 12 scenes (drawing-particles, elastic-particles, impulse, jelly-drop, rigid-particles, soup, soup-stirrer, surface-tension, tesla-valve, theo-jansen, washing-machine, wave-tank); jelly-drop and surface-tension exceeded the max in pair 2 only (surface-tension 0.307 > 0.284), the other nine in neither | reverted | none (never committed; diff in `target/phase35/attempts/A3br.patch`) | liquid-tumbler gained in both pairs (−8.5% / −10.3%) but D-11 fails on wave-tank (+1.4% / +1.1% against the base median). The diagnostic ABBA (`A3br-diag-*`, not a decision input) put wave-tank above the base max in pair 1 (0.852 > 0.847) and not in pair 2 (0.836 vs 0.847) |

35-03 run notes: binaries `target/phase35/bin/spot-base-03` (plan start `51041deb0`, engine equal to BEFORE_COMMIT), `spot-A1` (the A1 source, committed as `91b27a6d6`) and `spot-A2` (A1 plus `attempts/A2.patch`). A1 was timed 2026-10-08 05:53–05:57Z (load 7.8–9.2) and A2 11:26–11:30Z (load 6.2–8.2); `uptime` lines are in `target/phase35/A1.uptime`, `A1-iso.uptime`, `A2.uptime` and `A2-iso.uptime`. No cargo build or test ran during either window; queued test binaries sat at `_dyld_start` behind syspolicyd (no CPU). The A2 binary itself waited about 3 h 14 min at `_dyld_start` before its warmup launch, so A2 was timed about 5.5 h after A1. That is why the A1 numbers in the A2 row (`spot-A1` as base) sit about 1–3% below the A1 row's after numbers; each row compares only binaries run back to back.

35-04 run notes: binaries `target/phase35/bin/spot-base-04` (plan start `0cc0c0986`, byte-identical to `spot-A1`), `spot-A3` (the A3 source, committed as `0a2fe8cb8`; working-tree diff SHA-256 `2d0965a6…` checked at commit time) and `spot-A3b` (`7f0b36b18` plus `attempts/A3b.patch`). A3 was timed 2026-10-08 15:05–15:09Z (load 6.4–10.5) and A3b 21:25–21:28Z (load 7.2–9.5); `uptime` lines are in `target/phase35/A3.uptime`, `A3-iso.uptime`, `A3b.uptime`, `A3b-iso.uptime` and the `*-diag.uptime` files. No cargo build or test ran during either window. The diagnostic pairs `A3-diag-*` and `A3b-diag-*` (fountain and soup-stirrer, `--runs 5`, 21:28Z) were run after both decisions were fixed and do not change them. They do show that on this host a single isolated ABBA set can put a scene that does not take the changed path about 1.5% above the base max in both pairs. Across 7 and 12 re-checked scenes, the regression check fired once per attempt. The two chain-edge variants are recorded as reverted under D-11 as written.

35-05 run notes: binaries `target/phase35/bin/spot-base-05` (plan start `aed221fc0`; a hard link of `spot-base-04`, valid because `git diff aed221fc0 -- crates/` was empty, so the engine equals A1) and `spot-A4` (built from the A4 working tree, source digest in `target/phase35/A4.source-sha`). A4 was timed 2026-10-09 02:17–02:20Z (load 8.0–11.3) and its isolated pairs 02:24Z (load 8.5–12.1); `uptime` lines are in `target/phase35/A4.uptime` and `A4-iso.uptime`. No cargo build or test of ours ran during timing. Another repository's `cargo test` ran on the shared host just before the targeted run and again before the isolated run; timing waited until it exited each time. A4 failed D-11 decisively, so it was never committed. A commit-then-revert would have needed a full `cargo test` cycle (about 70 freshly linked test executables under syspolicyd stalls) for code that leaves the tree immediately, as with A3b. A5 used `spot-base-05b` (a hard link of `spot-base-05`, because A4 left the engine unchanged) and `spot-A5` (built from the A5 working tree, source digest in `target/phase35/A5.source-sha`, checked again at commit time). A5 was timed 2026-10-09 03:06–03:09Z (load 5.3–8.0) and its isolated pair at 03:09Z (load 7.0–7.5); `uptime` lines are in `target/phase35/A5.uptime` and `A5-iso.uptime`. The full checks ran after timing, as in 35-04 (03:10–09:16Z; `cargo test -p liquidfun --all-features` alone took 4 h 43 min under launch stalls), and the working-tree digest still matched when `216de3929` was committed.

35-06 run notes: binaries `target/phase35/bin/spot-base-06` (plan start `285259f4d`; a hard link of `spot-A5`, valid because `git diff 216de3929 285259f4d -- crates/` was empty) and `spot-A6` (built from the A6 working tree, source digest in `target/phase35/A6.source-sha`, checked again at commit time). A6 was timed 2026-10-09 09:52–09:54Z (load 5.1–5.8); `uptime` lines are in `target/phase35/A6.uptime`, `A6-shared.uptime`, `A6-full.uptime` and `A6-iso.uptime`. No cargo build or test ran during timing. Before timing, a mutation check replaced the pad with `motion` alone: the swept-edge, washing-machine and fast-rotation tests failed, so the tests detect a pad that ignores the start remap. A6 passed D-11, so A7 (per-fixture transform validation) was not tried. The full checks ran after timing (09:59–16:58Z; `cargo test -p liquidfun --all-features` alone took 5 h 51 min under launch stalls), and the working-tree digest still matched when `2256dd8cd` was committed.

35-07 run notes: binaries `target/phase35/bin/spot-base-07` (plan start `cba903103`; a hard link of `spot-A6`, valid because `git diff 2256dd8cd cba903103 -- crates/` was empty), `spot-A8` (built from the A8 working tree, source digest in `target/phase35/A8.source-sha`), `spot-base-07b` (a hard link of `spot-A8`) and `spot-A9` (built from the A8 plus A9 working tree, digest in `A9.source-sha`); A8 was committed as `5695f4b39` and A9 as `f7041fc75` after the combined checks, with the working-tree digest unchanged since the full checks started (`A8A9-tree.source-sha`). A8 and A9 touch independent files, so both were built and timed before one combined full check cycle, and each was judged on its own pair of binaries. A8 was timed 2026-10-09 18:01–18:15Z and A9 18:26–18:29Z (load 6.3–13.1; the A8 isolated pair 1 after runs saw the load rise from 7.4 to 13.1); `uptime` lines are in `target/phase35/A8.uptime`, `A8-full.uptime`, `A8-iso.uptime`, `A9.uptime`, `A9-full.uptime` and `A9-iso.uptime`. Timing waited each time until another repository's `cargo` processes on the shared host had exited, and no cargo build or test of ours ran during timing. The first full check run failed clippy on two A9 doc-comment backtick lints and two test `assert!` equality lints (`A8A9-checks-1.log`); after fixing those, a release rebuild was byte-identical to `spot-A9` (SHA-256 `bd2b2eb5…`), so the timed binary matches the committed source (`A9-final.source-sha`). Both tests modules were run before timing: the A9 equivalence tests passed against the original resequence body, then against the new one (`A8-unit.log`, `A9-unit.log`).

Quick 261010-ibo run notes: HEAD `cfe85b09f` (crates equal to `f7041fc75`). Binaries `target/phase35/bin/spot-base-ibo` (a hard link of `spot-final`, SHA-256 `bd2b2eb5…`, checked before the link), `spot-A3r` (`f028bf58…`, built from the A3r working tree, source digest `8c747d84…` in `A3r.source-sha`) and `spot-A3br` (`7707a16e…`, source digest `0d0428bb…` in `A3br.source-sha`). A release rebuild of the A3r tree was byte-identical to `spot-A3r`. Each new binary was launched once with `--warmup 1 --steps 1 --runs 1 --scene particles` before timing; launches took under a second, with no `_dyld_start` stall. The protocol was pre-registered in `target/phase35/ibo-protocol.txt` at 18:50:38Z, before the first timed leg; an identity-only addendum recorded the A3br binary after the A3r decision. Timing used `target/phase35/abba-gated.sh`, which is `abba.sh` plus a result-blind gate: before each leg it waits while cargo or rustc runs (any repository) or the 1-minute load is above 6.0, and a set is void if cargo or rustc runs or the 1-minute load is above 10.0 at the end of any leg. A voided set may be re-run at most twice, and the third run counts regardless. Counted sets, all on 2026-10-10: A3r targeted 18:52–18:54Z (1-minute load 4.7–5.7), A3r full 19:58–19:59Z (5.8–6.0), A3r isolated 20:04–20:06Z (5.1–6.3), A3br targeted 20:53–20:55Z (5.4–6.1), A3br full 20:55–20:56Z (4.5–5.1), A3br isolated 20:56–20:57Z (4.1–4.7). `uptime` lines are in `A3r.uptime`, `A3r-full.uptime`, `A3r-iso.uptime`, `A3br.uptime`, `A3br-full.uptime` and `A3br-iso.uptime`. Other repositories' agents on the shared host ran `cargo check` and `cargo test` throughout, so five sets were voided, all because cargo or rustc was running at the end of a leg (none for load): A3r full twice (`A3r-full.void-*`), A3r isolated once (`A3r-iso.void-*`) and A3br targeted twice (`A3br.void-*`). Their timings were moved aside unread and were not decision inputs. Before the A3r full set's second re-run, a wait for 5 minutes free of cargo and rustc timed out after 50 minutes; the re-run then went ahead behind the per-leg gate, which waited 585 s, and was valid. The A3br targeted third run counted regardless and logged no void condition. There were no GATE-TIMEOUTs. The gate means no cargo or rustc process was running at the start or end of any counted leg. No cargo build or test of this repository ran during timing; the A3r and A3br clippy runs (19–21 min each, contending with editor workspace checks) finished before their timing began. Each decision file (`A3r.decision`, `A3br.decision`) was written before any diagnostic run. The diagnostic pairs `A3r-diag-*` (jelly-drop and water-wheel) and `A3br-diag-*` (wave-tank), run at 21:16–21:19Z (load 5.8–6.6, the A3r set after a 1,110 s gate wait), do not change either decision. Both attempts were reverted, and `git diff HEAD -- crates/` is empty.

## Target records

- liquid-tumbler: profile recorded (hot path `pressure::damping` 22.9%); attempts: A0 rejected (research, branchless damping), A1 kept, A2 reverted, A3 reverted (chain edge hoist, −9.4% / −10.4% but a soup-stirrer regression under D-11), A3b reverted (no-branch variant, −7.6% / −7.3% but a fountain regression under D-11); neither regression reproduced in a later diagnostic pair; A4 reverted (per-row AABB query, +0.7% / +2.9%, no gain and regressions under D-11), A5 kept (candidate-row bitset walk, −3.0% / −3.0%); current median 22.7–23.0 ms/step (`spot-A5` in the 35-05 pairs, base 23.4–23.7); final 23.670 / 23.204 ms/step vs before 25.135 / 25.346 in the cumulative ABBA (§Final verification); retry (quick 261010-ibo, base `spot-base-ibo` = final engine): A3r reverted (−8.8% / −10.4%, but jelly-drop and water-wheel above the base max in both isolated pairs under D-11) and A3br reverted (−8.5% / −10.3%, but wave-tank above the base max in both isolated pairs), 25/25 fingerprints for both; in later diagnostic pairs, A3r's two scenes did not exceed the base max, and wave-tank did in pair 1 only; the final engine is unchanged; status: closed
- tesla-valve: profile recorded (hot path `visit_sorted_tag_indices_in_aabb` 13.7%); attempts: A4 reverted (per-row AABB query, +1.0% / +0.8%, no gain), A5 kept (candidate-row bitset walk, −0.9% / −2.2%), A8 kept (ungrouped particle creation without the group-record rebuild, −7.8% / −12.2%), A9 kept (cheaper lifetime eviction index, −9.1% / −8.1%); current median 3.27 ms/step (`spot-A9` in the 35-07 pairs, base `spot-A8` 3.56–3.60, `spot-A6` 3.90–3.96); final 3.258 / 3.440 ms/step vs before 4.056 / 4.028 in the cumulative ABBA; status: closed
- stacked-drip: profile recorded (hot path `pressure::damping` 17.1%); attempts: A1 kept, A2 reverted, A4 reverted (per-row AABB query, −0.9% / +1.2%, no gain), A5 kept (candidate-row bitset walk; gain in targeted pair 1 only, −1.7% / −1.5%); current median 2.48–2.55 ms/step (`spot-A5` in the 35-05 pairs, base 2.52–2.58); final 2.489 / 2.546 ms/step vs before 2.764 / 2.719 in the cumulative ABBA; status: closed
- washing-machine: profile recorded (hot path `push_fixture_particle_hit` 15.2%); attempts: A6 kept (conservative spatial query for moving fixtures at iteration 0, −29.0% / −28.9%); A7 not tried; current median 1.435–1.436 ms/step (`spot-A6` in the 35-06 pairs, base 2.018–2.024); final 1.474 / 1.490 ms/step vs before 2.161 / 2.182 in the cumulative ABBA; status: closed
- particles: profile recorded (hot path `pressure::damping` 22.8%); attempts: A1 kept, A2 reverted (particles regressed); current median 1.55–1.56 ms/step (A1 ABBA after, base 1.58–1.63); final 1.573 / 1.587 ms/step vs before 1.657 / 1.713 in the cumulative ABBA; status: closed

## Final verification

- Final commit: `2020757911f1757356d2c2ec80e55055aa0eeaa0` (35-07 close-out). The last engine commit is `f7041fc75` (A9); `git diff f7041fc75 HEAD -- crates/` is empty, so the final engine is A1 + A5 + A6 + A8 + A9.
- Final binary: `target/phase35/bin/spot-final`, a hard link of `spot-A9` (SHA-256 `bd2b2eb5…`). `target/release/playground-scene-spot` on this tree has the same SHA-256, and 35-07 showed that a release rebuild of the committed source is byte-identical, so no new binary was linked. Both `spot-final` and `spot-before` were launched once with `--warmup 1 --steps 1 --runs 1 --scene particles` before timing (both started at once).
- Timing ran 2026-10-10 00:24:56–00:27:27Z with no cargo build or test of this repository running (`target/phase35/final.sh`; `uptime` lines in `target/phase35/final.uptime`). Load averages rose from 5.64 to 8.87. Another repository's `cargo test` was in progress the whole time; it was mostly waiting at launch. A second one in a third repository started at about 00:25Z. ABBA interleaving is the mitigation.

### Fingerprints

`spot-final --runs 3 > target/phase35/final-full.jsonl` (25 lines, all `timed_out` false), then `python3 target/phase35/keep_rule.py target/phase35/before-full.jsonl target/phase35/final-full.jsonl liquid-tumbler,tesla-valve,stacked-drip,washing-machine,particles`:

- `fingerprint mismatches: none`: **25/25 equal** to the phase before run.
- `medians above before max (any scene without a gain): none`. The five targets all printed `gain=True` against `before-full.jsonl`. Every one of the 25 final medians is below its `before-full.jsonl` median, but that file was recorded on 2026-10-08 under a different load. The cumulative evidence is the back-to-back pairs below, and the whole-catalog table belongs to Phase 36.
- The four ABBA files (`final-b1`, `final-a1`, `final-a2`, `final-b2`) also matched their before fingerprints. No bisect or revert was needed.

### Cumulative ABBA (spot-before vs spot-final)

`--scene liquid-tumbler --scene tesla-valve --scene stacked-drip --scene washing-machine --scene particles --runs 5`, in the order before, final, final, before. `keep_rule.py final-b1 final-a1` and `keep_rule.py final-b2 final-a2` printed `gain=True` for all five targets in both pairs, `fingerprint mismatches: none` and no medians above the before max.

| Target | Pair 1: before median (min-max) → final median | Pair 2: before median (min-max) → final median | Change (pair 1 / pair 2) |
| --- | --- | --- | --- |
| liquid-tumbler | 25.135 (24.735-25.699) → 23.670 | 25.346 (25.133-25.454) → 23.204 | −5.8% / −8.5% |
| tesla-valve | 4.056 (3.965-4.407) → 3.258 | 4.028 (3.980-4.061) → 3.440 | −19.7% / −14.6% |
| stacked-drip | 2.764 (2.713-2.808) → 2.489 | 2.719 (2.658-2.766) → 2.546 | −9.9% / −6.4% |
| washing-machine | 2.161 (2.094-2.280) → 1.474 | 2.182 (2.131-2.190) → 1.490 | −31.8% / −31.7% |
| particles | 1.657 (1.627-1.673) → 1.573 | 1.713 (1.662-2.025) → 1.587 | −5.1% / −7.4% |

Particles has the smallest margin: 1.573 against a before minimum of 1.627 in pair 1. Its pair 2 before run had one slow run at 2.025.

### Checks on the final HEAD

These ran sequentially after the timing, 00:28–01:35Z (`target/phase35/checks-08.sh`, log `checks-08.log`, exit codes in `checks-08.results`):

- `cargo fmt --all --check`: pass
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pass
- `cargo build --workspace --all-targets --all-features`: pass
- `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown`: **fail (pre-existing, deferred in 35-01)**. Exit 101 with E0432 in the native-only bins `dam-break-bench`, `dam-break-timers` and `playground-scene-spot`, which import items gated `#[cfg(not(target_arch = "wasm32"))]`. These are the same errors recorded in `deferred-items.md` before any engine change.
- `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass. Plans 35-03 through 35-07 used this wasm32 check.
- `bun scripts/bright-builds-check.ts all`: pass
- `cargo test --all-features` (default member `liquidfun`): pass, 75 `test result: ok`, 1,079 passed, 0 failed, 52 min 30 s
- `cargo test -p liquidfun-wasm --all-features`: pass, 316 passed, 0 failed
- `just web-build` (wasm package and site): pass. The generated files were unchanged, and the tree stayed clean.
- `cd web && bun run test:unit`: pass, 56 files, 472 tests
- `just web-smoke` (optional): **failed, unrelated and pre-existing**. 61 of 62 passed. `e2e/rust-wasm-proof.spec.ts:170` expects the status text `Loading Rust/WASM session…`, which no file under `web/src/` contains; it also matches 4 `role=status` outputs. `web/`, `scripts/` and `justfile` are unchanged since BEFORE_COMMIT (log `target/phase35/web-smoke-08.log`).

### Authored behavior (D-05)

`git diff 809582431faecfdfa13b87f0d8376527bea7f0b4 HEAD --stat -- crates/liquidfun-wasm/src/scene/ web/src/` prints nothing. The full `git diff 809582431 HEAD --stat` lists only planning files under `.planning/` and engine internals plus tests under `crates/liquidfun/src/particle/` and `crates/liquidfun/src/world/` (21 files, 3,029 insertions, 95 deletions). That covers `body_contact.rs` and its tests, `contact_scan.rs`, `lifetime/eviction.rs`, `storage.rs`, `storage/creation.rs` with `creation_fast_path_tests.rs`, `storage/runtime.rs`, `particle_coupling.rs`, `moving_fixture_query.rs` and `moving_fixture_query_tests.rs`. No scene module, control, preset, web source or asset changed.

## Notes for Phase 36

- Re-survey at final commit `2020757911f1757356d2c2ec80e55055aa0eeaa0` or a later commit with the same `crates/`. The last engine change is `f7041fc75`. Its release `playground-scene-spot` has SHA-256 `bd2b2eb5…`, saved as `target/phase35/bin/spot-final`; `spot-before` remains the phase baseline.
- `docs/benchmarks/scene-survey.md` still holds the pre-phase table (stamp 2026-10-07T06-20-54Z on `96a3ac6a6`). Phase 35 did not edit it or the README. The whole-catalog before/after table and user-facing notes are Phase 36 work.
- `target/phase35/final-full.jsonl` gives a first look at the whole catalog. Every scene's median is below its `before-full.jsonl` median: water-wheel −54%, fountain −42%, jelly-drop −42% and theo-jansen −29%. The two files were recorded on different days under different load, so they are not before/after evidence. Run back-to-back pairs of saved binaries.
- Leftover ideas from reverted and untried work:
  1. The chain child edge hoist (A3 and A3b; `target/phase35/attempts/A3.patch` and `A3b.patch`) cut liquid-tumbler by 7–10%. It was reverted only because one non-chain scene was above its base max in both isolated pairs, and that did not reproduce. A retry should use more isolated pairs, or an agreed noise allowance, before the keep decision. Quick 261010-ibo retried both on the final engine with a pre-registered quiet-host gate and D-11 unchanged. A3r gained −8.8% / −10.4% and A3br −8.5% / −10.3%, both with 25/25 fingerprints, and both were reverted. In each, scenes without a chain fixture were above their base max in both isolated pairs: jelly-drop and water-wheel for A3r, wave-tank for A3br. Diagnostic pairs afterwards did not reproduce A3r's two scenes, and wave-tank was above the base max in pair 1 only. The patches are `attempts/A3r.patch` and `A3br.patch`. Under D-11 as written, this hoist has now failed four times, each time on a different non-chain scene. Keeping it would need a user decision on the confirmation rule, not another re-run.
  1. The bounded insertion sort for the retained proxy order (A2, `attempts/A2.patch`) helped liquid-tumbler and stacked-drip but hurt particles and washing-machine. A cheaper presortedness test that picks the strategy might keep the gain without the regressions.
  1. Per-fixture transform validation (A7) was never tried, because A6 passed.
  1. The particle-contact damping loop (`pressure::damping`, the top self share in three targets) is a dependent scatter whose float order D-07 locks. A0 showed that branchless stores are slower.
  1. Other costs were seen in the profiles but never attempted. The per-step world backup clone (`Vec<Slot<ParticleSystem>>::clone`) is 2.5% in particles and 1.1% in washing-machine. Emission reallocation copies (`_platform_memmove`) are about 3% in tesla-valve.
  1. Scene construction in particles (`create_particle_group`, O(n²) `prepare_create` on the grouped path) is outside `SessionCore::advance` and outside the timed window. It affects load time, not ms/step.
- Unrelated pre-existing items are in `deferred-items.md`:
  1. The wasm32 build of `liquidfun-wasm` without `--lib` fails because of the native-only bins.
  1. A release-only `unreachable expression` warning appears at `boundary/support.rs:98`.
  1. One `web-smoke` spec expects the stale text `Loading Rust/WASM session…`.
  1. Some xtask test targets stall under syspolicyd.

## D-11 calibration (quick 261010-mrp)

This section is a proposal for the user. It changes no decision. D-11 in 35-CONTEXT.md, every Attempts row, the target records and the Phase summary are unchanged.

### Protocol

- A/A design: B is `target/phase35/bin/spot-base-ibo` (inode 545134829) and A is `target/phase35/bin/spot-AA` (inode 550580308). A is a `cp` of B, not a link. Both have SHA-256 `bd2b2eb5…`, the final Phase 35 build. A was launched once untimed before any timed leg. It started in 0.36 s.
- Pre-registration: `target/phase35/mrp-protocol.txt` has `written_utc` 2026-10-10T21:29:57Z. The first leg in `AA1.uptime` started at 21:30:33Z. The frozen analysis script `target/phase35/aa_calib.py` has SHA-256 `0393cf95…`. It was not changed after timing, so there is no addendum. Before it was hashed, one crash on empty input was fixed, during a parse-only run with no A/A data.
- Sequence: the §Method sequence that judged A3r in quick 261010-ibo, unchanged, through `abba-gated.sh` with the ibo gate and void rules. Each of 3 rounds ran a targeted ABBA on liquid-tumbler (`--runs 5`), the full catalog b1 then a1 (`--runs 3`), and an isolated ABBA (`--runs 5`) on every scene the full run listed.
- Definitions: a is the A leg and b is the B leg of the same pair. `ratio% = (a_median / b_median - 1) * 100`. `excess% = (a_median / b_max - 1) * 100`. D-11 as written confirms a regression when excess% > 0 in both isolated pairs.
- F formula, quoted from the protocol: "a confirmed regression requires the scene's after median to exceed its base median by more than F% in BOTH isolated pairs; F = max(2, floor(M) + 1) where M is the largest A/A isolated-pair ratio% across all counted rounds". The screening step stays unchanged. The full catalog still lists scenes above base max, and only listed scenes get isolated pairs.
- An A/A test measures timing noise only. It cannot show code-layout effects of a real change.

### Run notes

HEAD `079de026d`. No build of this repository ran during the plan, and the executor ran no cargo or rustc command. Counted sets, all on 2026-10-10: round 1 21:49:38–21:53:30Z (1-minute load 3.06–6.72), round 2 21:54:04–21:56:54Z (3.62–6.16), round 3 21:57:09–22:01:11Z (3.56–5.78). Two sets were voided, both round 1's targeted set, because another repository's `cargo test -p open-bitcoin-node` was running at a leg end (`AA1.void-1791667895`, `AA1.void-1791667958`). Their keep output was moved into those directories unread. Before the third run, which counts regardless (`NO_VOID=1`), a result-blind wait for 300 s free of cargo and rustc took 1,005 s. It is logged as `PRE-WAIT` in `AA1.uptime`, following the ibo precedent. The third run logged no `VOID-IGNORED`, so no void condition occurred. There were 3 GATE-WAITED entries of 15 s each and no GATE-TIMEOUT. The gate and the void check mean no cargo or rustc ran at the start or end of any counted leg. The `uptime` lines are in `AA1.uptime`, `AA1-full.uptime`, `AA1-iso.uptime`, `AA2.uptime`, `AA2-full.uptime`, `AA2-iso.uptime`, `AA3.uptime`, `AA3-full.uptime` and `AA3-iso.uptime`. Per-round records are in `AA1.round`, `AA2.round` and `AA3.round`, and the analysis output is in `mrp-results.txt`.

### Per-round results

| Round | Full run listed above base max (a) | Isolated-pair confirmed (b) | D-11 would revert a no-op | liquid-tumbler targeted ratio% p1 / p2 (false gain?) | Fingerprints |
| --- | --- | --- | --- | --- | --- |
| 1 | color-mixer, float-or-sink, impulse, liquid-timer, liquid-tumbler | float-or-sink | yes | +0.619 / −0.441 (no; gain in pair 2 only) | 25/25 equal (full b1 and a1 vs `before-full.jsonl`) |
| 2 | color-mixer, drawing-particles, liquid-timer, rigid-particles, sparky, stacked-drip, surface-tension, theo-jansen | none | no | +0.909 / +0.609 (no) | 25/25 equal |
| 3 | color-mixer, impulse, jelly-drop, liquid-bubbler, liquid-tumbler, tesla-valve, theo-jansen | none | no | +0.514 / +0.280 (no) | 25/25 equal |

In every round, the script's listing matched keep_rule's output for both the full run and the isolated pairs.

Round 1 isolated pairs (`AA1-iso-*`):

| Scene | ratio% p1 | ratio% p2 | excess% p1 | excess% p2 | confirmed |
| --- | --- | --- | --- | --- | --- |
| float-or-sink | +1.854 | +2.580 | +0.148 | +0.863 | yes |
| color-mixer | −0.880 | +1.897 | −1.610 | −0.338 | no |
| liquid-timer | −2.905 | −1.768 | −4.713 | −21.247 | no |
| impulse | −3.586 | −1.846 | −4.652 | −4.182 | no |
| liquid-tumbler | −0.147 | −0.553 | −0.400 | −2.082 | no |

Round 2 isolated pairs (`AA2-iso-*`):

| Scene | ratio% p1 | ratio% p2 | excess% p1 | excess% p2 | confirmed |
| --- | --- | --- | --- | --- | --- |
| color-mixer | +1.212 | +3.096 | −0.094 | +1.632 | no |
| liquid-timer | −1.322 | −1.386 | −3.648 | −1.775 | no |
| surface-tension | −4.179 | −4.977 | −5.198 | −5.709 | no |
| rigid-particles | −3.348 | −0.409 | −4.744 | −1.946 | no |
| theo-jansen | −6.945 | +1.828 | −8.917 | +0.054 | no |
| drawing-particles | −4.292 | −3.491 | −6.004 | −4.302 | no |
| sparky | −8.073 | +0.553 | −10.224 | +0.037 | no |
| stacked-drip | +0.235 | +1.797 | −1.732 | −0.045 | no |

Round 3 isolated pairs (`AA3-iso-*`):

| Scene | ratio% p1 | ratio% p2 | excess% p1 | excess% p2 | confirmed |
| --- | --- | --- | --- | --- | --- |
| color-mixer | +2.606 | −2.000 | +1.276 | −2.331 | no |
| jelly-drop | +2.250 | −4.754 | +2.117 | −5.356 | no |
| impulse | +2.013 | −0.603 | −0.451 | −1.298 | no |
| theo-jansen | −1.126 | +2.341 | −1.364 | +0.093 | no |
| liquid-tumbler | −0.241 | −0.280 | −0.435 | −0.478 | no |
| liquid-bubbler | −2.598 | +2.323 | −2.774 | +0.270 | no |
| tesla-valve | +0.657 | −0.893 | −0.113 | −0.924 | no |

### False-alarm count (d)

D-11's regression clause as written would have reverted 1 of 3 no-op rounds (round 1, float-or-sink). The false-gain count is 0 of 3: liquid-tumbler never gained in both targeted pairs.

float-or-sink's A/A ratios were +1.854% and +2.580%. Those are larger in both pairs than the confirmed ratios behind three reverts: soup-stirrer (A3, +1.515% / +1.511%), fountain (A3b, +1.781% / +1.726%) and wave-tank (A3br, +1.389% / +1.105%). In each of the five recorded confirmations, at least one of the two pairs was at or below +2.255%, inside the A/A isolated range.

### Noise (e)

Pair-type distributions over all three rounds (nearest-rank quantiles):

| Pair type | n | ratio% min / p50 / p90 / max | excess% min / p50 / p90 / max | excess% > 0 |
| --- | --- | --- | --- | --- |
| targeted (liquid-tumbler) | 6 | −0.441 / +0.514 / +0.909 / +0.909 | −2.876 / −0.493 / −0.009 / −0.009 | 0 |
| full | 75 | −6.172 / +0.001 / +2.241 / +3.075 | −7.759 / −1.300 / +0.818 / +2.778 | 20 |
| isolated | 40 | −8.073 / −0.603 / +2.323 / +3.096 | −21.247 / −1.610 / +0.270 / +2.117 | 9 |

In the full runs, 20 of 75 A/A scene medians (27%) were above the base max. In the isolated runs, 9 of 40 pairs were. With no change at all, any one isolated pair exceeded its base max about 1 time in 4.

Per scene, over full and isolated pairs:

| Scene | n pairs | max ratio% | max excess% | median base spread% |
| --- | --- | --- | --- | --- |
| wave-machine | 3 | −0.752 | −1.218 | 0.808 |
| dam-break | 3 | +0.359 | −3.120 | 3.388 |
| fountain | 3 | −2.461 | −3.763 | 2.126 |
| float-or-sink | 5 | +2.580 | +0.863 | 2.171 |
| color-mixer | 9 | +3.096 | +2.778 | 1.518 |
| jelly-drop | 5 | +2.731 | +2.117 | 2.812 |
| water-wheel | 3 | −0.722 | −2.028 | 2.536 |
| particles | 3 | +0.992 | −0.050 | 1.418 |
| liquid-timer | 7 | +3.075 | +1.659 | 2.798 |
| surface-tension | 5 | +1.577 | +0.714 | 1.836 |
| elastic-particles | 3 | −1.691 | −4.183 | 2.151 |
| rigid-particles | 5 | +2.076 | +0.174 | 2.683 |
| soup | 3 | −0.706 | −2.849 | 2.897 |
| soup-stirrer | 3 | +1.224 | −0.305 | 2.455 |
| impulse | 7 | +2.986 | +0.972 | 3.055 |
| theo-jansen | 7 | +2.816 | +0.943 | 2.370 |
| liquid-tumbler | 7 | +1.986 | +1.695 | 0.946 |
| drawing-particles | 5 | +2.241 | +1.207 | 2.734 |
| sparky | 5 | +2.826 | +0.536 | 1.869 |
| hydraulic-fountain | 3 | −0.100 | −0.889 | 3.027 |
| wave-tank | 3 | −0.634 | −1.104 | 1.444 |
| liquid-bubbler | 5 | +2.323 | +0.270 | 1.028 |
| stacked-drip | 5 | +1.863 | +0.609 | 2.165 |
| washing-machine | 3 | −1.650 | −1.717 | 3.180 |
| tesla-valve | 5 | +0.657 | +0.195 | 0.857 |

Base spread is `(b_max - b_min) / b_median * 100` for the B leg of each pair. The median spread is 0.8–3.4%, so one scene's base max is often only 1–2% above its base median.

### Proposed floor

- M = 3.096%, from color-mixer in round 2, isolated pair 2.
- F = max(2, floor(3.096) + 1) = **4%**.
- Leave-one-round-out check (g): holding out round 1 gives F_1 = 4% (from round 2), holding out round 2 gives F_2 = 3% (from round 3's color-mixer +2.606%), and holding out round 3 gives F_3 = 4%. No held-out round had a scene above its F_k in both pairs. The floor moves by at most 1 point when a round is dropped.
- Limits: the sample is 3 rounds and 40 isolated pairs at 1-minute loads of 3.06–6.72. That is close to ibo's counted sets (4.1–6.3) and below 35-04's (6.2–10.5), where A3 and A3b were judged. Under F, a real regression smaller than 4% in both pairs would not be confirmed.
- F is a proposal for the user to accept, change or reject. D-11 is unchanged until the user decides.

### What-if (informational)

Under F = 4%, a scene is floor-confirmed only if its ratio% is above 4% in both isolated pairs. Each attempt's fingerprints were recomputed against `before-full.jsonl` from `A3-full.jsonl`, `A3b-full.jsonl`, `A3r-full-a1.jsonl` and `A3br-full-a1.jsonl`, and matched the recorded 25/25. The D-11-as-written recomputation reproduced every recorded confirmation.

| Attempt | Base | Target gain both pairs | Fingerprints | D-11-as-written confirmed (recorded decision) | max isolated ratio% (scene, pair) | Confirmed under F | Would-be decision under F |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A3 | `spot-base-04` | yes (−9.404% / −10.409%) | 25/25 | soup-stirrer (+1.515% / +1.511%) (reverted) | +2.231 (soup, pair 2) | none | keep |
| A3b | `spot-base-04` | yes (−7.591% / −7.333%) | 25/25 | fountain (+1.781% / +1.726%) (reverted) | +2.500 (water-wheel, pair 1) | none | keep |
| A3r | `spot-base-ibo` | yes (−8.832% / −10.436%) | 25/25 | jelly-drop (+1.356% / +7.074%), water-wheel (+2.248% / +2.255%) (reverted) | +7.074 (jelly-drop, pair 2) | none | keep |
| A3br | `spot-base-ibo` | yes (−8.491% / −10.336%) | 25/25 | wave-tank (+1.389% / +1.105%) (reverted) | +9.118 (surface-tension, pair 2) | none | keep |

Some single isolated pairs in the attempts were above M: A3r jelly-drop +7.074% (pair 2), liquid-timer +6.271% and particles +4.353% (pair 1), and A3br surface-tension +9.118% (pair 2). None repeated in the other pair. The A/A rounds cannot say whether such single-pair spikes are noise or a real cost.

These what-ifs change no decision. A3, A3b, A3r and A3br remain reverted; any re-judgement of A3r needs a fresh run after the user agrees a floor.
