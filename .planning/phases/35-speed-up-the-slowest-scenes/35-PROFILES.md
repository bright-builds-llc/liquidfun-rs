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
| A1 | 35-03 | reuse the last sorted contact-proxy order across particle iterations: retag in place, `sort_unstable_by_key((tag, row))`; row-order rebuild when the length differs or a tag fails (`ProxyOrderCache` in `ParticleStorage`) | liquid-tumbler, stacked-drip, particles | base `spot-base-03` (pair 1 / pair 2): liquid-tumbler 24.179 (24.139-24.299) / 24.404 (24.358-24.539); stacked-drip 2.628 (2.616-2.642) / 2.668 (2.643-2.674); particles 1.576 (1.567-1.582) / 1.627 (1.598-1.629) | liquid-tumbler 23.345 / 23.259; stacked-drip 2.550 / 2.525; particles 1.552 / 1.563 | yes (25/25): `target/phase35/A1-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A1-base-full.jsonl` → `A1-full.jsonl` listed fountain (1.165 > 1.161) and drawing-particles (0.0107 > 0.0105); isolated ABBA `--runs 5` (`A1-iso-*`): fountain 1.130 / 1.135 vs base max 1.155 / 1.157, drawing-particles 0.0104 / 0.0104 vs 0.0109 / 0.0109 | kept | `91b27a6d6` | all three targets below the base min in both pairs (liquid-tumbler −3.4% / −4.7%, stacked-drip −3.0% / −5.4%, particles −1.5% / −3.9%); the base itself drifted about 3% between pairs (load 7.8–9.2), and the particles pair 1 margin is small (1.552 vs min 1.567) |
| A2 | 35-03 | A1 plus a bounded insertion sort (at most 8·n moves, then `sort_unstable_by_key` on the partly sorted buffer) for the retained order | liquid-tumbler, stacked-drip, particles | base `spot-A1` (pair 1 / pair 2): liquid-tumbler 22.851 (22.608-23.068) / 23.097 (23.076-23.157); stacked-drip 2.465 (2.453-2.479) / 2.490 (2.474-2.517); particles 1.498 (1.484-1.502) / 1.529 (1.505-1.546) | liquid-tumbler 21.587 / 21.679; stacked-drip 2.341 / 2.347; particles 1.545 / 1.560 | yes (25/25): `target/phase35/A2-full.jsonl` vs `before-full.jsonl` | confirmed: particles above the A1 max in both targeted pairs (1.545 > 1.502, 1.560 > 1.546); washing-machine isolated ABBA (`A2-iso-*`) 2.084 / 2.080 vs A1 max 2.042 / 2.058. Fountain and impulse exceeded the max in pair 1 only; theo-jansen in neither | reverted | none (never committed; diff in `target/phase35/attempts/A2.patch`) | faster than A1 on liquid-tumbler (−5.5% / −6.1%) and stacked-drip (−5.0% / −5.7%) but particles and washing-machine regressed in both pairs, so D-11 fails; likely the budget runs out on their more mixed orders and the insertion moves are paid before the full sort |
| A3 | 35-04 | chain child edge built once per child (body contacts + CCD): `maybe_child_edge` in `body_contact::generate`, `CcdChild { index, maybe_aabb, maybe_edge }` in CCD records | liquid-tumbler | base `spot-base-04` (pair 1 / pair 2): liquid-tumbler 23.590 (23.376-23.961) / 23.815 (23.481-23.897) | liquid-tumbler 21.371 / 21.336 | yes (25/25): `target/phase35/A3-full.jsonl` vs `before-full.jsonl` | confirmed: soup-stirrer (no chain fixture) above the base max in both isolated pairs (`A3-iso-*`): 1.171 > 1.160, 1.181 > 1.176. Full run `A3-base-full.jsonl` → `A3-full.jsonl` listed jelly-drop, liquid-timer, soup, soup-stirrer, surface-tension, tesla-valve and theo-jansen; tesla-valve exceeded the max in pair 2 only (3.926 > 3.912; pair 1 3.926 < 4.060), the other five in neither | reverted | `0a2fe8cb8` (reverted by `7f0b36b18`; diff in `target/phase35/attempts/A3.patch`) | liquid-tumbler gained in both pairs (−9.4% / −10.4%) but D-11 fails on soup-stirrer (+1.5% / +1.5%), a scene that never takes the chain path, so the cost (if real) is the extra per-row branch or a code-layout shift. A later diagnostic ABBA (`A3-diag-*`, not a decision input) did not reproduce it: soup-stirrer 1.169 / 1.173 vs base 1.166 / 1.168, max 1.173 / 1.182 |
| A3b | 35-04 | A3 variant with no new per-row branch: a chain child is queried as a prebuilt `Shape::Edge` at child index 0 through the same `Shape::distance_to_point` / `Shape::ray_cast` call; CCD emits one record per chain child in fixture-then-child order | liquid-tumbler | base `spot-base-04` (pair 1 / pair 2): liquid-tumbler 23.383 (23.262-23.626) / 23.365 (23.112-23.426) | liquid-tumbler 21.608 / 21.651 | yes (25/25): `target/phase35/A3b-full.jsonl` vs `before-full.jsonl` | confirmed: fountain (no chain fixture) above the base max in both isolated pairs (`A3b-iso-*`): 1.168 > 1.164, 1.148 > 1.137. Full run `A3b-base-full.jsonl` → `A3b-full.jsonl` (load 8.5–9.5) listed 12 scenes; hydraulic-fountain, surface-tension and washing-machine exceeded the max in pair 1 only, the other eight in neither | reverted | none (never committed; diff in `target/phase35/attempts/A3b.patch`) | liquid-tumbler gained in both pairs (−7.6% / −7.3%) but D-11 fails on fountain (+1.7% / +1.8%), again a scene without a chain. The diagnostic ABBA (`A3b-diag-*`) did not reproduce it: fountain 1.150 / 1.131 vs base 1.140 / 1.133 |
| A4 | 35-05 | per-y-row binary search in `visit_sorted_tag_indices_in_aabb`: inside the unchanged `first..last` window, two ranged partition-point searches per tag row (`binary_partition_point_in`) replace the x-mask scan; the linear scan stays when rows · 2 · bit length of the window ≥ window. Equivalence tests against a verbatim copy of the scan (grid, 2,000 random tags × 200 boxes, empty, error cases) passed 4/4 | tesla-valve, stacked-drip, liquid-tumbler | base `spot-base-05` (pair 1 / pair 2): liquid-tumbler 24.604 (24.501-24.757) / 24.253 (24.162-24.363); stacked-drip 2.699 (2.572-2.748) / 2.634 (2.620-2.706); tesla-valve 4.103 (4.050-4.150) / 4.154 (4.019-4.504) | liquid-tumbler 24.776 / 24.946; stacked-drip 2.674 / 2.665; tesla-valve 4.146 / 4.188 | yes (25/25): `target/phase35/A4-full.jsonl` vs `before-full.jsonl` | confirmed: liquid-tumbler above the base max in both targeted pairs (24.776 > 24.757, 24.946 > 24.363). The full run `A4-base-full.jsonl` → `A4-full.jsonl` (load 9.9–11.3) listed liquid-tumbler and 15 other scenes; isolated ABBA `--runs 5` of those 15 (`A4-iso-*`): elastic-particles 0.576 / 0.571 vs base max 0.575 / 0.570, liquid-timer 0.801 / 0.802 vs 0.787 / 0.781, rigid-particles 0.271 / 0.273 vs 0.263 / 0.263, surface-tension 0.333 / 0.328 vs 0.329 / 0.318 in both pairs; six more in pair 1 only, liquid-bubbler in pair 2 only, four in neither | reverted | none (never committed; diff in `target/phase35/attempts/A4.patch`) | no target gained in either pair (tesla-valve +1.0% / +0.8%, stacked-drip −0.9% / +1.2%, liquid-tumbler +0.7% / +2.9%), and five scenes were above the base max in both pairs, three of them by 3–6% (liquid-timer +5.1% / +4.2%, rigid-particles +3.4% / +5.3%, surface-tension +5.7% / +3.9%). Likely cause: the per-row searches' scattered probes cost more than the sequential scan they skip for the box sizes these scenes query, so the profile's 13.7% self share in tesla-valve is the scan's cost, not wasted work |
| A5 | 35-05 | ordered bitset walk for body-contact candidate rows: when a fixture query visits 32 or more rows, `collect_candidate_rows` sets one bit per row in a `row_marks` buffer reused across one `generate` call and reads the marked words in ascending order with `trailing_zeros`, clearing each word as it goes, instead of `sort_unstable` + `dedup`; fewer rows keep the sort. Tests against the sort-plus-dedup reference passed on the old and new code | liquid-tumbler, stacked-drip, tesla-valve | base `spot-base-05b` (hard link of `spot-base-05`, engine A1; pair 1 / pair 2): liquid-tumbler 23.424 (23.315-23.538) / 23.698 (23.482-23.848); stacked-drip 2.524 (2.488-2.545) / 2.584 (2.531-2.641); tesla-valve 3.839 (3.809-3.900) / 3.920 (3.876-3.948) | liquid-tumbler 22.724 / 22.978; stacked-drip 2.482 / 2.546; tesla-valve 3.805 / 3.833 | yes (25/25): `target/phase35/A5-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A5-base-full.jsonl` → `A5-full.jsonl` listed fountain (1.138 > 1.134), stacked-drip (2.553 > 2.539) and washing-machine (2.076 > 2.057); isolated ABBA `--runs 5` (`A5-iso-*`): fountain 1.134 / 1.131 vs base max 1.149 / 1.157, stacked-drip 2.485 / 2.482 vs 2.576 / 2.612 (below the base min in both pairs), washing-machine 2.054 / 2.047 vs 2.074 / 2.416. Stacked-drip in targeted pair 2 (no gain) stayed below the base max (2.546 vs 2.641) | kept | `216de3929` | liquid-tumbler (−3.0% / −3.0%) and tesla-valve (−0.9% / −2.2%) below the base min in both targeted pairs; stacked-drip below the min in targeted pair 1 only (−1.7% / −1.5%; pair 2 2.546 vs min 2.531) and in both isolated pairs (−2.3% / −1.8%), but only the targeted pairs count toward the gain. The tesla-valve pair 1 margin is small (3.805 vs min 3.809) |
| A6 | 35-06 | conservative spatial query for moving fixtures at particle iteration 0: `moving_fixture_query_pad` bounds the start remap `S(p) = M p + t` (convex displacement, corner maximum `D0`, `L`, the inf-norm of `M - I`) by the closed form `max(motion, D0) / (1 - L)` with relative, absolute and per-meter rounding margins, a 5% overshoot and one f32 corner check, and `query_particles_for_fixture` uses that pad instead of the `0..n` scan. `None` (full scan) when `L >= 1`, an input is not finite, the check fails, or a candidate velocity is not finite. 7 unit tests compare filtered and full-scan hit lists bit for bit (rotating polygon, rotating and translating circle, chain plus edge, fast rotation, static and iteration 1, and a 0.3 rad swept edge whose hits lie outside the motion-only pad); a motion-only pad fails 3 of them | washing-machine | base `spot-base-06` (pair 1 / pair 2): washing-machine 2.024 (2.000-2.029) / 2.018 (2.006-2.053) | washing-machine 1.436 / 1.435. Shared path, non-targets (`A6-shared-*`, base → after): soup-stirrer 1.130 → 1.013 / 1.146 → 1.014, water-wheel 1.254 → 1.192 / 1.276 → 1.210, theo-jansen 0.195 → 0.147 / 0.198 → 0.144 (all three below the base min in both pairs) | yes (25/25): `target/phase35/A6-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A6-base-full.jsonl` → `A6-full.jsonl` listed hydraulic-fountain (0.797 > 0.797, base max 0.7968) and liquid-timer (0.720 > 0.713); isolated ABBA `--runs 5` (`A6-iso-*`): hydraulic-fountain 0.786 / 0.787 vs base max 0.793 / 0.811, liquid-timer 0.716 / 0.724 vs 0.723 / 0.732 | kept | `2256dd8cd` | washing-machine below the base min in both pairs (−29.0% / −28.9%); the full run also shows liquid-bubbler −18.5%, soup −11.7%, soup-stirrer −9.9% and theo-jansen −24.5%. FIXTURE_CONTACT_FILTER hook calls for moving fixtures at iteration 0 now run in query order instead of row order (hit list unchanged; matches the static path) |
| A8 | 35-07 | ungrouped-append fast path in particle creation: for `maybe_group == None`, `prepare_create` clones `group_records` and re-applies `retain_empty_after_member_removal` to the trailing empty records instead of cloning the group lane and running `rebuild_group_records_for_system` / `membership_ranges` (O(n) per call, twice per emitted particle). Targets `prepare_create` 8.79% inclusive in the 35-02 tesla-valve profile (`membership_ranges` 2.97% self, its group-lane `_platform_memmove` 3.30%). A `debug_assert_eq!` re-checks equality with the full rebuild; the grouped path and every error check are unchanged, and an explicit `i32::MAX` lane check keeps the rebuild's limit. A plain clone is not equal: the rebuild resets the cached statistics timestamp of an empty retained group, which a test reproduces. 4 tests compare storage (`ParticleStorage` `PartialEq`) and errors against a verbatim copy of the old `prepare_create` | tesla-valve (fountain, water-wheel, sparky reported) | base `spot-base-07` (hard link of `spot-A6`; pair 1 / pair 2): tesla-valve 3.895 (3.876-3.928) / 3.960 (3.928-4.015); fountain 1.147 (1.139-1.164) / 1.157 (1.152-1.163); water-wheel 1.229 (1.224-1.247) / 1.235 (1.218-1.254); sparky 0.134 (0.130-0.139) / 0.137 (0.132-0.139) | tesla-valve 3.593 / 3.477; fountain 0.915 / 0.918; water-wheel 1.012 / 0.987; sparky 0.136 / 0.135 | yes (25/25): `target/phase35/A8-full.jsonl` vs `before-full.jsonl` | none confirmed. Full run `A8-base-full.jsonl` → `A8-full.jsonl` (load 9.5–12.8, rising during the after run) listed color-mixer, dam-break, float-or-sink, impulse, liquid-timer, liquid-tumbler, particles, soup-stirrer, wave-machine and wave-tank; isolated ABBA `--runs 5` of all ten (`A8-iso-*`, load 6.3 → 13.1 during the a1/a2 runs): pair 1 listed dam-break 0.500 > 0.495, float-or-sink 0.482 > 0.478, impulse 0.742 > 0.733, liquid-tumbler 24.475 > 24.382, particles 1.558 > 1.546 and wave-tank 0.899 > 0.895; pair 2 listed none (dam-break 0.509 vs max 0.514, float-or-sink 0.475 vs 0.493, impulse 0.743 vs 0.774, liquid-tumbler 24.189 vs 24.985, particles 1.599 vs 1.625, wave-tank 0.904 vs 1.007). Sparky stayed below the base max in both targeted pairs | kept | `5695f4b39` | tesla-valve below the base min in both pairs (−7.8% / −12.2%); the other emitting scenes also gained in both pairs: fountain −20.2% / −20.7%, water-wheel −17.7% / −20.1%. Sparky (550 particles) did not change measurably |

35-03 run notes: binaries `target/phase35/bin/spot-base-03` (plan start `51041deb0`, engine equal to BEFORE_COMMIT), `spot-A1` (the A1 source, committed as `91b27a6d6`) and `spot-A2` (A1 plus `attempts/A2.patch`). A1 was timed 2026-10-08 05:53–05:57Z (load 7.8–9.2) and A2 11:26–11:30Z (load 6.2–8.2); `uptime` lines are in `target/phase35/A1.uptime`, `A1-iso.uptime`, `A2.uptime` and `A2-iso.uptime`. No cargo build or test ran during either window; queued test binaries sat at `_dyld_start` behind syspolicyd (no CPU). The A2 binary itself waited about 3 h 14 min at `_dyld_start` before its warmup launch, so A2 was timed about 5.5 h after A1. That is why the A1 numbers in the A2 row (`spot-A1` as base) sit about 1–3% below the A1 row's after numbers; each row compares only binaries run back to back.

35-04 run notes: binaries `target/phase35/bin/spot-base-04` (plan start `0cc0c0986`, byte-identical to `spot-A1`), `spot-A3` (the A3 source, committed as `0a2fe8cb8`; working-tree diff SHA-256 `2d0965a6…` checked at commit time) and `spot-A3b` (`7f0b36b18` plus `attempts/A3b.patch`). A3 was timed 2026-10-08 15:05–15:09Z (load 6.4–10.5) and A3b 21:25–21:28Z (load 7.2–9.5); `uptime` lines are in `target/phase35/A3.uptime`, `A3-iso.uptime`, `A3b.uptime`, `A3b-iso.uptime` and the `*-diag.uptime` files. No cargo build or test ran during either window. The diagnostic pairs `A3-diag-*` and `A3b-diag-*` (fountain and soup-stirrer, `--runs 5`, 21:28Z) were run after both decisions were fixed and do not change them. They do show that on this host a single isolated ABBA set can put a scene that does not take the changed path about 1.5% above the base max in both pairs. Across 7 and 12 re-checked scenes, the regression check fired once per attempt. The two chain-edge variants are recorded as reverted under D-11 as written.

35-05 run notes: binaries `target/phase35/bin/spot-base-05` (plan start `aed221fc0`; a hard link of `spot-base-04`, valid because `git diff aed221fc0 -- crates/` was empty, so the engine equals A1) and `spot-A4` (built from the A4 working tree, source digest in `target/phase35/A4.source-sha`). A4 was timed 2026-10-09 02:17–02:20Z (load 8.0–11.3) and its isolated pairs 02:24Z (load 8.5–12.1); `uptime` lines are in `target/phase35/A4.uptime` and `A4-iso.uptime`. No cargo build or test of ours ran during timing. Another repository's `cargo test` ran on the shared host just before the targeted run and again before the isolated run; timing waited until it exited each time. A4 failed D-11 decisively, so it was never committed. A commit-then-revert would have needed a full `cargo test` cycle (about 70 freshly linked test executables under syspolicyd stalls) for code that leaves the tree immediately, as with A3b. A5 used `spot-base-05b` (a hard link of `spot-base-05`, because A4 left the engine unchanged) and `spot-A5` (built from the A5 working tree, source digest in `target/phase35/A5.source-sha`, checked again at commit time). A5 was timed 2026-10-09 03:06–03:09Z (load 5.3–8.0) and its isolated pair at 03:09Z (load 7.0–7.5); `uptime` lines are in `target/phase35/A5.uptime` and `A5-iso.uptime`. The full checks ran after timing, as in 35-04 (03:10–09:16Z; `cargo test -p liquidfun --all-features` alone took 4 h 43 min under launch stalls), and the working-tree digest still matched when `216de3929` was committed.

35-06 run notes: binaries `target/phase35/bin/spot-base-06` (plan start `285259f4d`; a hard link of `spot-A5`, valid because `git diff 216de3929 285259f4d -- crates/` was empty) and `spot-A6` (built from the A6 working tree, source digest in `target/phase35/A6.source-sha`, checked again at commit time). A6 was timed 2026-10-09 09:52–09:54Z (load 5.1–5.8); `uptime` lines are in `target/phase35/A6.uptime`, `A6-shared.uptime`, `A6-full.uptime` and `A6-iso.uptime`. No cargo build or test ran during timing. Before timing, a mutation check replaced the pad with `motion` alone: the swept-edge, washing-machine and fast-rotation tests failed, so the tests detect a pad that ignores the start remap. A6 passed D-11, so A7 (per-fixture transform validation) was not tried. The full checks ran after timing (09:59–16:58Z; `cargo test -p liquidfun --all-features` alone took 5 h 51 min under launch stalls), and the working-tree digest still matched when `2256dd8cd` was committed.

35-07 run notes: binaries `target/phase35/bin/spot-base-07` (plan start `cba903103`; a hard link of `spot-A6`, valid because `git diff 2256dd8cd cba903103 -- crates/` was empty), `spot-A8` (built from the A8 working tree, source digest in `target/phase35/A8.source-sha`), `spot-base-07b` (a hard link of `spot-A8`) and `spot-A9` (built from the A8 plus A9 working tree, digest in `A9.source-sha`). A8 and A9 touch independent files, so both were built and timed before one combined full check cycle, and each was judged on its own pair of binaries. A8 was timed 2026-10-09 18:01–18:15Z and A9 18:26–18:29Z (load 6.3–13.1; the A8 isolated pair 1 after runs saw the load rise from 7.4 to 13.1); `uptime` lines are in `target/phase35/A8.uptime`, `A8-full.uptime`, `A8-iso.uptime`, `A9.uptime`, `A9-full.uptime` and `A9-iso.uptime`. Timing waited each time until another repository's `cargo` processes on the shared host had exited, and no cargo build or test of ours ran during timing. The first full check run failed clippy on two A9 doc-comment backtick lints and two test `assert!` equality lints (`A8A9-checks-1.log`); after fixing those, a release rebuild was byte-identical to `spot-A9` (SHA-256 `bd2b2eb5…`), so the timed binary matches the committed source (`A9-final.source-sha`). Both tests modules were run before timing: the A9 equivalence tests passed against the original resequence body, then against the new one (`A8-unit.log`, `A9-unit.log`).

## Target records

- liquid-tumbler: profile recorded (hot path `pressure::damping` 22.9%); attempts: A1 kept, A2 reverted, A3 reverted (chain edge hoist, −9.4% / −10.4% but a soup-stirrer regression under D-11), A3b reverted (no-branch variant, −7.6% / −7.3% but a fountain regression under D-11); neither regression reproduced in a later diagnostic pair; A4 reverted (per-row AABB query, +0.7% / +2.9%, no gain and regressions under D-11), A5 kept (candidate-row bitset walk, −3.0% / −3.0%); current median 22.7–23.0 ms/step (`spot-A5` in the 35-05 pairs, base 23.4–23.7); status: open
- tesla-valve: profile recorded (hot path `visit_sorted_tag_indices_in_aabb` 13.7%); attempts: A4 reverted (per-row AABB query, +1.0% / +0.8%, no gain), A5 kept (candidate-row bitset walk, −0.9% / −2.2%), A8 kept (ungrouped particle creation without the group-record rebuild, −7.8% / −12.2%); current median 3.48–3.59 ms/step (`spot-A8` in the 35-07 pairs, base `spot-A6` 3.90–3.96); status: open (A9 in progress)
- stacked-drip: profile recorded (hot path `pressure::damping` 17.1%); attempts: A1 kept, A2 reverted, A4 reverted (per-row AABB query, −0.9% / +1.2%, no gain), A5 kept (candidate-row bitset walk; gain in targeted pair 1 only, −1.7% / −1.5%); current median 2.48–2.55 ms/step (`spot-A5` in the 35-05 pairs, base 2.52–2.58); status: open
- washing-machine: profile recorded (hot path `push_fixture_particle_hit` 15.2%); attempts: A6 kept (conservative spatial query for moving fixtures at iteration 0, −29.0% / −28.9%); A7 not tried; current median 1.435–1.436 ms/step (`spot-A6` in the 35-06 pairs, base 2.018–2.024); status: open
- particles: profile recorded (hot path `pressure::damping` 22.8%); attempts: A1 kept, A2 reverted (particles regressed); current median 1.55–1.56 ms/step (A1 ABBA after, base 1.58–1.63); status: open
