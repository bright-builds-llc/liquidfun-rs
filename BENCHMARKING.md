# Benchmarking

LiquidFun performance evidence is a paired, same-host comparison between native Rust and the pinned C++ oracle. It is scoped to the exact workload, resolved scenario bytes, build identities, hardware session, and statistical interval in an immutable report. Benchmark data is not compatibility, correctness, coverage, or parity evidence.

The machine authorities are:

- `protocol/benchmarks/phase12-v1.json` for the sealed case matrix
- `reference/performance/policy.json` for measurement and analysis rules
- `reference/performance/manifest.toml` for explicitly reviewed immutable reports
- `reference/upstream-lock.toml` for the C++ oracle revision

The reviewed-report manifest is currently empty. Therefore, the project currently publishes no accepted performance number.

## Workload matrix

The matrix contains 32 sealed cases across 14 workloads. Scalable workloads use the exact 128, 1,024, and 8,192 entity size points; fixed workloads use their reviewed scenario horizon.

| Workload                | Size points                |
| ----------------------- | -------------------------- |
| `world_step`            | fixed                      |
| `broad_phase`           | 128, 1,024, 8,192 entities |
| `narrow_phase`          | fixed                      |
| `contact_solve`         | fixed                      |
| `ccd`                   | fixed                      |
| `joints`                | fixed                      |
| `particle_lifecycle`    | 128, 1,024, 8,192 entities |
| `particle_contacts`     | 128, 1,024, 8,192 entities |
| `particle_sort`         | 128, 1,024, 8,192 entities |
| `particle_pressure`     | 128, 1,024, 8,192 entities |
| `large_particle_system` | 128, 1,024, 8,192 entities |
| `aabb_query`            | 128, 1,024, 8,192 entities |
| `ray_cast`              | 128, 1,024, 8,192 entities |
| `mixed_world`           | 128, 1,024, 8,192 entities |

Each case binds its workload and size point to a catalog hash, canonical resolved-scenario hash, timestep bits, solver iteration counts, logical horizon, engine roles, and ordered regions. A benchmark implementation must execute those resolved bytes; semantically similar ad hoc setup is not interchangeable evidence.

## Measurement method

Both engines run on the same host in scalar release mode. Rust and C++ samples are interleaved, and their first-run order alternates by sample ordinal to reduce order bias. The canonical timing authority is unprofiled wall-clock time.

One independent baseline run consists of:

1. setup outside the timed region;
1. one declared warm-up;
1. 30 paired samples of the exact measured actions and logical horizon;
1. teardown outside the timed region.

The workflow retains at least five independent baseline runs. Under the current policy, exactly five runs produce 150 raw sample pairs for every sealed case. Setup, catalog resolution, process construction, correctness comparison, report serialization, and teardown are excluded from the timed measured-actions region.

The runner records native and oracle process generations and reset epochs. A crash, timeout, malformed response, physics mismatch, identity mismatch, or incomplete case set is a harness or correctness failure, never a timing sample.

## Calibration and intervals

For each case, the analysis computes one relative Rust-versus-oracle delta for each of the five independent runs. It then reports the mean and a two-sided Student 95% confidence interval with four degrees of freedom. Raw run deltas remain in `calibration.json`; an interval never replaces its samples.

An observed change is practically meaningful only when its confidence interval clears `max(3%, calibrated noise floor)`. The 3% floor is the policy's 300-basis-point minimum, not permission to discard uncertainty. Results whose intervals cross the threshold are inconclusive.

Calibration is host- and workload-specific. It must not be reused to claim behavior for another CPU, operating system, toolchain, build mode, workload, size point, or scenario version.

## Profiling and optimization admission

Diagnostic profiles use the `phase12_v1` parent/child schema. They may identify allocation, cache, or scaling bottlenecks and may guide an optimization, but profiled timings are never public timing authority.

An optimization is admissible only when all of the following hold:

- the candidate uses reviewed `release_scalar` mode and unprofiled wall-clock timing;
- all 32 sealed cases and their identity hashes are present;
- the relevant interval clears `max(3%, calibrated noise floor)`;
- the candidate has at least a 10% diagnostic profile share or typed allocation, cache, or scaling bottleneck evidence;
- differential, determinism, safety, and public-API correctness hashes remain accepted;
- no protected workload regresses beyond the calibrated threshold.

The admission decision is computed by `cargo xtask performance optimization-check` from `target/phase12-performance/optimization-record.json`. Passing that command admits only the candidate described by the record; it does not create a public claim or update the reviewed-report manifest.

## Reproducing raw evidence

Prepare the pinned C++ release oracle and confirm the closed matrix before collecting measurements:

```console
cargo xtask upstream build --preset oracle-release
cargo xtask performance paired --check
```

Collect, calibrate, and validate through the confined workflow:

```console
bash scripts/phase12-performance.sh paired
bash scripts/phase12-performance.sh calibrate
bash scripts/phase12-performance.sh validate
```

Equivalent discoverable recipes are:

```console
just phase12-performance-paired
just phase12-performance-calibrate
just phase12-performance-validate
```

Raw case reports are written below `target/phase12-performance/raw/`; the paired summary, calibration, logs, and validation identity remain below `target/phase12-performance/`. These local files are explicitly unreviewed and non-claiming. They must not be copied into public documentation as accepted results.

The validation command checks exact policy and matrix hashes, the complete raw-report filename set, catalog and resolved-scenario hashes, Rust and oracle revisions, compiler and linker identities, target, compile and link flags, hardware identity, compatibility status, profile schema, baseline/run cardinality, and sample counts.

## Interpreting results

A positive relative delta means native Rust used less measured wall-clock time than the C++ oracle for that exact case; a negative delta means it used more. Interpretation must use the interval rather than only the mean.

- An interval wholly above the calibrated practical floor supports a workload-scoped Rust improvement.
- An interval wholly below the negative practical floor supports a workload-scoped Rust regression.
- An interval crossing either boundary is inconclusive at this sample size and host.
- A result for one size point says nothing about another size point unless each has its own admitted interval.
- A result for one workload cannot be aggregated into an engine-wide claim.

`d2_supported` records that the measured build ran on a supported platform tier. It does not promote D1 parity or prove compatibility. Compatibility remains governed by `COMPATIBILITY.md`, the differential authorities, and their tolerance profiles.

## Public claim rules

Every public number must link directly to one immutable report listed in `reference/performance/manifest.toml` and name:

- workload and size point;
- benchmark matrix and policy versions or hashes;
- Rust and oracle revisions;
- scalar optimization mode and compiler/linker flags;
- CPU, logical-core count, memory, operating system, and target;
- sample count, interval method, mean, and 95% interval;
- compatibility and correctness-gate status.

Claims are workload-only. Do not write “Rust is faster,” “LiquidFun is faster,” “up to N% faster,” or another universal summary from this matrix.

A compliant future claim has this bounded form:

> In immutable report `[report identity]`, native Rust measured `[interval]` relative to the pinned C++ oracle for `[workload]` at `[size point]` under `[matrix/policy hashes]`, `[hardware]`, and `[compiler/linker flags]`. This result is limited to that workload, size, host, build identity, and interval; it is not a generalized engine-wide performance claim.

This template is illustrative and contains no result. It becomes publishable only after every placeholder is replaced from one reviewed manifest-listed report.
