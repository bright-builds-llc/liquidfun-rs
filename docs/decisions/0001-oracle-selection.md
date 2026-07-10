# ADR 0001: Select the Official Post-Release LiquidFun Commit as the Oracle

- **Status:** Accepted
- **Date:** 2026-07-09

## Context

`liquidfun-rs` needs one immutable behavioral reference before its public API,
tests, and translated implementation begin to accumulate compatibility
assumptions. The official `v1.1.0` release is the natural baseline, but the
official repository contains material fixes after that release.

The tag and commit identities are deliberately recorded separately:

| Identity | Value |
| --- | --- |
| Repository | `https://github.com/google/liquidfun.git` |
| Release tag | `v1.1.0` |
| Annotated tag object | `d15bcf1879144bf2a4c8ebcc73f6418186756fb2` |
| Release commit | `f38db7c627c3dc5ec879d726e16fa5a12ad6e478` |
| Selected official commit | `7f20402173fd143a3988c921bc384459c6a858f2` |
| Release commit date | 2014-07-16 |
| Selected commit date | 2018-01-10 |

The pinned release notes identify LiquidFun 1.1.0 as based on Box2D 2.3.0,
revision 280. That historical lineage, rather than current Box2D behavior,
defines the compatibility family for this project.

## Release-to-Candidate Delta Audit

The official range
`f38db7c627c3dc5ec879d726e16fa5a12ad6e478..7f20402173fd143a3988c921bc384459c6a858f2`
contains the following relevant classes of change.

### Material native behavior and correctness changes

- The particle-group split path fixes a memory error caused by retaining a
  reference into the particle flags buffer across particle cloning and buffer
  growth.
- Particle-versus-fixture collision solving honors
  `b2_fixtureContactFilterParticle` and calls the fixture/particle contact
  filter. The range also adds a `ParticleCollisionFilter` testbed scenario that
  exercises fixture and particle filtering.
- The growable-buffer integration is corrected by including
  `Common/b2GrowableBuffer.h` in the Box2D CMake header set; the related source
  comment is also corrected to name `b2ParticleAssembly.neon.s` accurately.

### Build and warning fixes

- CMake gains Xcode and thread-discovery fixes, including Windows thread
  discovery.
- C++11 undefined signed-left-shift behavior is removed from particle proxy tag
  construction.
- Preprocessor checks use `defined(...)`, avoiding warnings for undefined
  `SWIG` and `LIQUIDFUN_UNIT_TESTS` macros.
- Android build-path handling is corrected.

### Bindings and documentation changes

The range also expands and fixes JavaScript bindings, including collision
filtering, contact enablement, and body gravity-scale accessors; regenerates
JavaScript output; and updates README links and project documentation. These
changes help inventory the supported surface but are not, by themselves, the
reason to prefer the post-release commit as the native behavioral oracle.

## Decision

Use official commit `7f20402173fd143a3988c921bc384459c6a858f2` as the
canonical behavioral oracle. Pin it as the detached gitlink at
`third_party/liquidfun`; never resolve behavior from a moving branch or tag at
test time.

Treat `v1.1.0` and its peeled release commit as release context, not as the
selected gitlink. The native correctness fixes and fixture/particle filtering
behavior make the selected commit a more complete and defensible reference
than the 2014 release commit while preserving the documented Box2D
2.3.0/revision-280 ancestry.

The upstream checkout is read-only. Build compatibility must be implemented in
repository-owned wrappers first. If an upstream source patch ever becomes
unavoidable, it must live outside the submodule and carry preimage and patch
hashes, an alteration record, and a build-only or behavior-affecting
classification before it can replace the current `patch_set = "none"` policy.

## Consequences

- Compatibility claims and reference artifacts must cite the selected full
  commit, not `master`, `v1.1.0`, or a short SHA.
- The machine-readable authority is `reference/upstream-lock.toml`; the gitlink
  and documentation must agree with it.
- Later inventory work must include the post-release native fixes, tests,
  bindings, and documentation while distinguishing behavioral evidence from
  non-native surface changes.
- Updating the oracle is an intentional review event that repeats the delta,
  ancestry, provenance, license, and evidence audit.

## References

- [`reference/upstream-lock.toml`](../../reference/upstream-lock.toml)
- [`UPSTREAM.md`](../../UPSTREAM.md)
- Pinned release notes:
  `third_party/liquidfun/liquidfun/ReleaseNotes.md`
- Pinned Box2D license:
  `third_party/liquidfun/liquidfun/Box2D/License.txt`
