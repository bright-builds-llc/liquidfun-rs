# Fuzz Corpus and Regression Handoff

The private `fuzz/` package is outside the production workspace. Its five
targets use exact `cargo-fuzz 0.13.2`, `libfuzzer-sys 0.4.13`, `arbitrary 1.4.2`, and `nightly-2026-07-15`.

## Reviewed targets and bounds

| Target             | Boundary                                                   | Hard limit                                    |
| ------------------ | ---------------------------------------------------------- | --------------------------------------------- |
| `protocol`         | Production strict protocol decoders                        | 1 MiB input                                   |
| `shapes_collision` | Checked shape construction and collision dispatch          | 256 operations                                |
| `world_mutation`   | Body, fixture, handle, and mutation APIs                   | 256 operations and 128 body/fixture creations |
| `particles`        | Particle creation, invalidation, compaction, and step APIs | 256 operations and 4,096 particle creations   |
| `groups_ownership` | Group handles and owned particle-buffer adoption/teardown  | 256 operations and 64 group creations         |

Inputs are fully converted to bounded typed operations before world effects.
Typed API rejections are ordinary target outcomes. Panics, sanitizer findings,
timeouts, non-finite committed state, and invariant failures are findings.

## Seed corpus

Store small reviewed seeds under `fuzz/corpus/<target>/`. Seeds must be safe to
publish, deterministic, and free of external paths or secrets. Reproduce them
with the exact checked-in nightly:

```console
cargo +nightly-2026-07-15 fuzz run <target> fuzz/corpus/<target> -- -runs=1000
```

## Minimized regression contract

Before promoting any finding into a named regression, minimize the exact input
with `cargo +nightly-2026-07-15 fuzz tmin` and record all fields below:

- target;
- SHA-256 of the exact minimized input;
- repository-relative exact-bytes path;
- exact generator and cargo-fuzz version;
- exact dated Rust toolchain;
- candidate commit;
- oracle identity and comparison-policy identity when applicable;
- exactly one classification: `Harness`, `PhysicsMismatch`, `Sanitizer`,
  `Timeout`, or `Schema`;
- fix commit, issue, or pull-request reference.

Physics mismatch is never inferred from a crash, typed rejection, timeout,
schema failure, or sanitizer diagnostic. The minimized bytes become an ordinary
deterministic regression only after the classification and provenance fields
are reviewed. CI artifacts remain confined to their target-specific bounded
evidence directory.
