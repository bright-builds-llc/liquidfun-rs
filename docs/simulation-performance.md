# Simulation performance sample

**Unreviewed local sample.** These numbers are one Linux host, one Dam Break
Medium recipe, and one compiler. They are not a public speed claim, not
compatibility evidence, and not Phase 12 evidence. Do not copy them into
`reference/performance/manifest.toml`.

The scalar deterministic step is unchanged. This crate still forbids `unsafe`,
and the default step does not use Rayon, explicit SIMD intrinsics,
`-C target-cpu=native`, or fast-math. A later sample, measured against the
tree that produced the tables below, is at the end of this page.

## Workload and host

Reproduce the wall-clock rows with the native bench, from a `--release` build:

```console
cargo run -p liquidfun-wasm --release --bin dam-break-bench -- --warmup 20 --steps 80
cargo run -p liquidfun-wasm --release --bin dam-break-bench -- --warmup 60 --steps 600
```

- Workload: playground Dam Break Medium, 1920 particles (48 by 40), radius
  `0.06324555`, spacing `0.101193`, origin `(-4.7, 0.4)`, dynamic circle
  radius `0.75` at `(2.5, 5.5)`, gravity `y = -10`, step `1/60`, velocity
  iterations 8, position iterations 3, particle iterations 2.
- OS/arch: Linux / `x86_64`
- CPU: Intel Xeon, 4 logical cores, advertised AVX2 and AVX-512 (KVM guest)
- Compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`, LLVM 22.1.6
- Target: default `x86_64-unknown-linux-gnu` (SSE2 baseline)
- Baseline tree: `fe5e95cc` (`origin/main` before this change)
- Compared builds were `--release` on the same host, back to back

The 60/600 recipe matches the locked playground pair length. This sample did
not rebuild the C++ oracle.

## Unprofiled wall clock

Short recipe: 20 warmup steps, 80 measured steps, three runs each, in the
order they were taken.

| Build | Run | Wall ms | ms/step | steps/s |
| ----------- | --- | --------- | -------- | ----------- |
| baseline | 1 | 75.195759 | 0.939947 | 1063.889787 |
| baseline | 2 | 74.961892 | 0.937024 | 1067.208923 |
| baseline | 3 | 74.951079 | 0.936888 | 1067.362886 |
| this change | 1 | 57.248884 | 0.715611 | 1397.407153 |
| this change | 2 | 57.143498 | 0.714294 | 1399.984299 |
| this change | 3 | 57.407811 | 0.717598 | 1393.538590 |

Median ms/step moved from `0.937024` to `0.715611` (about 1.31 times as many
steps per second on this short sample).

Long recipe: 60 warmup steps, 600 measured steps.

| Build | Wall ms | ms/step | steps/s | realtime factor |
| ----------- | ---------- | -------- | ----------- | --------------- |
| baseline | 578.837726 | 0.964730 | 1036.559943 | 17.275999 |
| this change | 478.873452 | 0.798122 | 1252.940620 | 20.882344 |
| this change | 476.143242 | 0.793572 | 1260.124994 | 21.002083 |

On the 600-step sample the faster of the two new runs is about 1.22 times the
baseline step rate (`0.964730 / 0.793572`). The short and long ratios differ;
report the recipe with the number.

## Same Dam Break state

A local probe hashed particle positions and velocities in storage order after
600 steps with no warmup. Each `f32` bit pattern was folded with a wrapping
multiply by `0x9E3779B97F4A7C15`. Both builds produced `80e38b2ad07795f6`.
The same probe also matched after 40 steps (`21b595b9abb320ff`) and 120 steps
(`d6301dc928cca29b`). The probe is not part of the bench output. `cargo test -p liquidfun --lib` passed (432 tests), including a contact-scan test that
compares contact weights and normals to the public neighborhood path by
`f32` bits.

## Where the time went

A temporary atomic timer around solver passes, removed before the wall-clock
table, attributed one 100-step window (20 warmup + 80 measured, two particle
iterations, so most passes ran 200 times). Those milliseconds include the
timer and are not the wall-clock authority.

| Pass | Before, ms | After, ms |
| ------------------ | ---------- | --------- |
| particle contacts | 30.671 | 24.072 |
| fixture collision | 23.019 | 8.682 |
| damping | 9.120 | 8.550 |
| body contacts | 4.808 | 4.925 |
| pressure | 3.932 | 3.129 |
| weight | 1.194 | 1.072 |
| integrate | 0.484 | 0.500 |
| rigid solve | 0.261 | 0.250 |
| gravity | 0.123 | 0.110 |

Inside the new contact pass, tag rebuild plus the in-place sort was about
8.4 ms and the distance walk was about 16.3 ms, again with the timer
installed. Neighbor windows averaged about 3.17 proxies (2,378,617 proxies
across 749,907 windows in that same 200-pass window).

## What changed

The step path no longer builds a neighborhood pair list and then walks it.
It follows scalar `FindContacts_Reference` from the pinned LiquidFun tree
`7f20402173fd143a3988c921bc384459c6a858f2`: write proxy tags, sort that buffer
in place by tag then particle row, test distance while scanning the right
and bottom tag windows, and append only real contacts. Contact and proxy
buffers are reused. The proxy buffer is cleared when the step commits, so a
between-step equality check still sees an empty scratch list.

Fixture collision for a static shape queries those sorted tags. The query
expands the fixture bounds by the fastest particle's motion plus `1e-3`,
which covers the static-transform round trip, and then the existing travel
box and ray cast still run. Moving fixtures use the same query when
`particle_iteration != 0`, because that iteration casts from the current
position. Iteration 0 still tests every particle against a moving fixture:
the ray start is remapped through the previous body transform, so a
current-position window would drop hits. Accepted hits are stable-sorted by
particle index so a later hit still overwrites velocity in the old
particle-major order.

## SIMD and parallelism

A four-wide distance kernel was written so LLVM could autovectorize
`dx * dx + dy * dy` without `unsafe`. On this host it did not beat the scalar
scan. Windows are short (about three proxies), so groups of four are the
exception. Forcing the kernel out of line made the traced contact pass slower
(about 30 ms versus 25 ms). Disassembly of the inlined form showed packed
`mulps` / `subps` on `Vec2` pairs, which is the existing two-wide layout, not
a four-particle distance. The kernel was removed.

The pinned x86 `oracle-release` path does the same. `FindContacts_Simd`,
`UpdateProxies_Simd`, and `CalculateTags_Simd` are compiled only for NEON.
The scalar oracle does not use them. Upstream's SIMD check allows approximate
equality; this sample keeps bit-identical contacts instead. Upstream's own
comment on `SortProxies` calls the sort a hot spot and points at a SIMD
mergesort. That sort is visible here (about 8 ms of the traced contact
window) and is still the ordinary stable tag-then-index sort.

There is no OpenMP in the upstream particle solve. A Dam Break contact pass
is about 0.12 ms and a collision pass is now under 0.05 ms, so a thread pool
would spend the pass on spawn and join. Damping and pressure scatter into
velocities in contact order; parallel writes would change that order. Rayon
stays out of `liquidfun`.

Per-pass velocity clones were left in place. Gravity's traced cost is about
0.1 ms. The damping cost is the contact loop, which must keep its order.

## C++ techniques not taken

- NEON contact and proxy-tag kernels. They are not in the scalar oracle, and
  a faithful port needs `unsafe` plus an approximate-equality policy.
- A threaded solver. The reference particle step is single-threaded aside
  from optional NEON.
- Replacing the velocity clone with in-place mutation. The profile did not
  put that clone on the critical path.
- `-ffast-math`, FMA contraction, or `-C target-cpu=native`. Those change
  IEEE results and would make this sample incomparable with the scalar
  oracle.

## Follow-up: proxy body contacts and in-place velocities

**Unreviewed local sample.** Same host, compiler, and Dam Break Medium recipe
as above. The parent binary is `9d4acadb`. The follow-up binary is
`18f5887e`. Release builds, back to back, no sampler attached. Do not copy
these numbers into `reference/performance/manifest.toml`.

### Unprofiled wall clock

Short recipe: 20 warmup steps, 80 measured steps, three runs each, in the
order they were taken.

| Build | Run | Wall ms | ms/step | steps/s |
| --- | --- | --- | --- | --- |
| parent `9d4acadb` | 1 | 56.940889 | 0.711761 | 1404.965771 |
| parent `9d4acadb` | 2 | 56.725460 | 0.709068 | 1410.301477 |
| parent `9d4acadb` | 3 | 56.653878 | 0.708173 | 1412.083388 |
| follow-up `18f5887e` | 1 | 56.082844 | 0.701036 | 1426.461183 |
| follow-up `18f5887e` | 2 | 55.808327 | 0.697604 | 1433.477839 |
| follow-up `18f5887e` | 3 | 55.819169 | 0.697740 | 1433.199409 |

Median ms/step moved from `0.709068` to `0.697740` (about 1.016 times as
many steps per second). Every follow-up run was faster than every parent
run. A later extra follow-up run, taken after the long samples, was
`0.699519` ms/step and stayed in that cluster.

Long recipe: 60 warmup steps, 600 measured steps, two runs each.

| Build | Wall ms | ms/step | steps/s | realtime factor |
| --- | --- | --- | --- | --- |
| parent `9d4acadb` | 484.279834 | 0.807133 | 1238.953097 | 20.649218 |
| parent `9d4acadb` | 479.032110 | 0.798387 | 1252.525640 | 20.875427 |
| follow-up `18f5887e` | 474.330277 | 0.790550 | 1264.941390 | 21.082357 |
| follow-up `18f5887e` | 475.437131 | 0.792395 | 1261.996510 | 21.033275 |

The faster follow-up run is about 1.010 times the faster parent run
(`0.798387 / 0.790550`). The two ranges do not overlap. The gain is small:
the traced body-contact pass was a few milliseconds across a 100-step
window, the floor still distance-tests a wide band, and the velocity copy
was already a small part of that window.

### Same Dam Break state

The same golden-ratio fold as the first sample (storage order, position bits
then velocity bits, wrapping multiply by `0x9E3779B97F4A7C15`) was run with
warmup 0. Both binaries produced:

| Steps | Checksum |
| --- | --- |
| 40 | `21b595b9abb320ff` |
| 120 | `d6301dc928cca29b` |
| 600 | `80e38b2ad07795f6` |

Those are the hashes already recorded against `fe5e95cc`. The probe was
removed before this note was added. `cargo test -p liquidfun --lib` passed
(434 tests). The new tests compare proxy-filtered body contacts with a full
scan by particle identity and by weight, normal, and mass bits, including
the strict contact path.

### What changed

Body contacts query the sorted contact proxies with the same tag window used
for fixture rays. Candidate rows are sorted back into particle order, then
the existing expanded-AABB point test and `distance_to_point` test still
run. A proxy-length mismatch or a failed tag query scans every particle.

Proxy rebuild uses `sort_unstable_by` on tag then row. That comparison is a
total order, so the sequence matches the previous stable sort.

Gravity, force, pressure, damping, and extra damping add into the live
velocity lane with the same `+=` and `-=` operand order. Debug builds still
reject a non-finite velocity before committing it. The release bench does
not take that debug copy.
