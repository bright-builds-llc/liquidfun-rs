# Phase 4: Math, Settings, and Numerical Policy - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-11T04:16:20.259Z
**Phase:** 4-math-settings-and-numerical-policy
**Mode:** Yolo
**Areas discussed:** Public math and settings contract, Floating-point and compiler semantics, Comparison, determinism, and divergence policy

## Public math and settings contract

| Option | Description | Selected |
| --- | --- | --- |
| Literal Rust mirror | Maximum source familiarity, but preserves unsafe and accidental C++ surface details. | |
| External math crate plus adapters | Mature ergonomics, but loses exact operation-order and feature control. | |
| Purpose-built curated module | Native Rust consumer API with exact control, safe construction, and explicit upstream mapping. | ✓ |
| Private conversion-only types | Small semver surface, but fails the consumer-facing phase requirement. | |

**Agent's choice:** Purpose-built curated `liquidfun::math` module.
**Notes:** Preserve source behavior and expression order while replacing uninitialized values, unchecked indexing, macros, global allocator hooks, and public dense-index artifacts with safe Rust contracts.

## Floating-point and compiler semantics

| Option | Description | Selected |
| --- | --- | --- |
| Compiler defaults | Minimal configuration, but contraction and target behavior can vary invisibly. | |
| Strict floating environment | Strong exception/rounding semantics beyond what the Rust baseline requires. | |
| Explicit precise scalar baseline | Disables fast math and contraction, preserves IEEE classes and gradual underflow, and records effective provenance. | ✓ |
| Platform-native fast mode | Potentially faster, but invalid as canonical compatibility evidence. | |

**Agent's choice:** Explicit precise scalar IEEE baseline.
**Notes:** Run pure probes in debug and release, preserve exact float classes and bits at the transport boundary, and treat optimization-dependent changes as findings.

## Comparison, determinism, and divergence policy

| Option | Description | Selected |
| --- | --- | --- |
| Exact bits everywhere | Maximum sensitivity but unsuitable as a universal cross-platform or transcendental policy. | |
| Global epsilon | Simple but scale-dependent and capable of hiding structural defects. | |
| Absolute-relative everywhere | Useful for composite values but wrong for discrete, bit-sensitive, and local-kernel fields. | |
| ULP everywhere | Useful for finite local kernels but weak around zero, discontinuities, and long horizons. | |
| Typed hybrid by semantic field | Closed versioned policies preserve the narrowest defensible rule for each observable. | ✓ |

**Agent's choice:** Typed hybrid field-policy registry.
**Notes:** Signed zero is distinct by default, arithmetic NaN is a mismatch, ordering is semantic, horizons bound evidence rather than scale tolerances, and only the canonical tier may promote golden fixtures.

## Agent's Discretion

- Private module/file split and helper names.
- Exact typed error and schema names.
- Probe grouping, bounded corpus size, and initial evidence-derived numeric thresholds.
- Documentation layout that preserves the complete discoverability contract.

## Deferred Ideas

- Collision, shape, solver, and particle-specific tolerances and horizons belong to their implementation phases.
- Accelerated floating-point modes remain diagnostic experiments until separately reviewed.
