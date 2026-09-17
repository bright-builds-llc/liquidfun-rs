---
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 15-hobby-2026-09-17T00-43-37
generated_at: 2026-09-17T00:51:14Z
---

# Phase 15 hobby-wrap-up implementation findings

Discovery level 0: existing local commands and documentation patterns suffice; no dependency, external API design, physics feature or new UI is needed. Previous strict research is retained under archive/strict-2026-09-14 and is not current execution authority.

## Existing boundaries

- `tools/xtask/src/package.rs` already exposes `cargo xtask package verify`, checking archive contents and native extracted consumer behavior. Reuse it without introducing another release system.
- `tools/xtask/src/docs/contracts/readiness.rs` requires `not release-ready` in README, COMPATIBILITY and RELEASE absent validated attestation. Keep that label scoped to optional strict qualification. Do not relax validators to make hobby checks certify parity.
- `.github/workflows/ci.yml` provides the current macOS ordinary job and explicit optional Linux input. No CI implementation change is needed.
- `crates/liquidfun-differential/tests/headless_catalog.rs::every_representative_family_executes_and_captures_without_a_display` is an existing native demonstration across representative families, avoiding GUI or C++ requirements. Use its exact test command.
- `crates/liquidfun/tests/particle_group_properties.rs` retains both audited and current Windows-seed regressions. Run locally and report the actual platform; do not claim a new Windows result.
- Cargo declares `rust-version = "1.92"`; the development toolchain is 1.97.0. Preserve both. An experimental wrap-up is not publication, and it must disclose the outstanding pre-publication minimum-compiler verification.
- `just check` calls a distinct xtask aggregate. It does not replace the local ordered precommit commands in owner instructions.

## Scope and risks

The hardest boundary is honest separation of experimental usability from strict release readiness. Retain existing machine report regions and validate docs contracts after prose edits. Other risks are stale public support promises and choosing a green CI run for a different source; reconcile current prose and require the actual implementation commit's macOS result.

Local AGENTS hobby scope, independent-review policy, sidecar, overrides, verification/testing/Rust standards and current active lessons informed this plan. No managed blocks, strict validators, platform registry or engine code need modification. All changes are reversible documentation changes.

## Decision coverage

| Decision | Plan/task | Coverage |
| --- | --- | --- |
| D-10 | 13/3 and 14/2 | Full |
| D-11 | 13/1 | Full |
| D-12 | 13/1–2 and 14/2 | Full |
| D-13 | 13/3 and 14/1–2 | Full |
| D-14 | 13/3 and 14/1 | Full |
| D-15 | 13/1–3 and 14/2 | Full |
| D-16 | 13/1–2 | Full |
| D-17 | 13/1–2 and 14/1 | Full |
| D-18 | 13/1–2 | Full |
| D-19 | 13/1–3 and 14/1 | Full |
| D-20 | Archive isolation, 13/1–3 and 14/2 | Full |
| D-21 | 13/1–2 and 14/1 | Full |

Wave 1 owns policy/documentation; wave 2 consumes committed documentation and owns only the completion record. No file conflicts or new interface skeletons exist. The simplification pass chose existing documents and checks rather than new tools, a schema or a certification workflow. Deferred ideas remain excluded.
