# Phase 35 Profiles and Attempts

Scope: native `--release` headless survey timing (`playground-scene-spot`) on one machine; local observations, not public speed claims.

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

## Target records

- liquid-tumbler: profile recorded (hot path `pressure::damping` 22.9%); attempts: none yet; status: open
- tesla-valve: profile recorded (hot path `visit_sorted_tag_indices_in_aabb` 13.7%); attempts: none yet; status: open
- stacked-drip: profile recorded (hot path `pressure::damping` 17.1%); attempts: none yet; status: open
- washing-machine: profile recorded (hot path `push_fixture_particle_hit` 15.2%); attempts: none yet; status: open
- particles: profile recorded (hot path `pressure::damping` 22.8%); attempts: none yet; status: open
