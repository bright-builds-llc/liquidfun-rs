# Phase 47 Deferred Items

Out-of-scope discoveries logged during execution (not fixed here).

## From 47-01

- `crates/liquidfun/src/particle/solver/boundary/support.rs:98`: `warning: unreachable expression` (an early `return Ok(())` at line 79 makes the trailing `Ok(())` unreachable). It appears only in `--release` builds of the `liquidfun` engine crate, which this phase must not edit. It predates Phase 47 and does not affect the debug clippy gate.
