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

Pending (Task 2).

## Target profiles

### liquid-tumbler

Pending (Task 2).

### tesla-valve

Pending (Task 2).

### stacked-drip

Pending (Task 2).

### washing-machine

Pending (Task 2).

### particles

Pending (Task 2).

## Attempts

| ID | Plan | Change | Targets | Base median (min-max) | After median (ABBA pair 1 / pair 2) | Fingerprints equal | Regression (any scene) | Decision | Commit | Reason |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A0 | research | branchless particle damping loop (select_unpredictable; two variants incl. signed-zero safe) | liquid-tumbler | 21.2-22.1 | 24.0-25.3 | yes | not measured | rejected | none | slower: unconditional stores lengthen the dependent store-to-load chain; damping is a dependent scatter whose float order D-07 locks |

## Target records

- liquid-tumbler: profile pending; attempts: none yet; status: open
- tesla-valve: profile pending; attempts: none yet; status: open
- stacked-drip: profile pending; attempts: none yet; status: open
- washing-machine: profile pending; attempts: none yet; status: open
- particles: profile pending; attempts: none yet; status: open
