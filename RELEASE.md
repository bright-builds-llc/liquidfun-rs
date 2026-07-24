# Release process

This document defines the fail-closed path for a parity-bearing `liquidfun`
release. A checklist item is not evidence: every accepted result must be
machine-readable, reviewed, hash-bound, and tied to one frozen source candidate.
If any required step fails or remains unavailable, do not publish.

## Versioning and MSRV

Releases follow Semantic Versioning. Before 1.0, incompatible public API changes
may require a minor-version increment and must still be documented. Beginning
with v1.0.0, incompatible public API or behavior changes require a major-version
increment unless the affected contract is explicitly outside SemVer.

Rust 1.92.0 is fixed for the v1.0.x line. A higher MSRV requires a reviewed
policy change and the next permitted SemVer release; a patch release must not
raise it. Repository construction and evidence use the pinned Rust 1.97.0
development toolchain.

## Freeze the source candidate

Choose a clean, reviewed commit and record its full 40-hex source-candidate SHA.
Never use a moving branch, abbreviated SHA, later documentation commit, or
current `HEAD` substitution as candidate identity. Freeze the source tree before
creating package or evidence artifacts.

The release-candidate workflow checks out that exact commit detached, records
its source-tree hash, and binds every producer workflow, job, run, toolchain,
target, payload, and artifact to it. Evidence from another commit cannot be
merged into the candidate.

## Audit the frozen candidate

Run inexpensive source-bound gates first: ordered Rust checks, warning-denied
rustdoc and doctests, docs contracts, package isolation, notices and licenses,
upstream corpus closure, generated-report freshness, compatibility closure, and
the publication dry run.

Then gather the reviewed scheduled or release-candidate outputs for package,
MSRV, four durable platforms, conditional macOS disposition, canonical
differential evidence, Rust safety, C++ sanitizers, fuzzing, regressions, Rust
and C++ coverage, performance, documentation, notices, corpus closure, and
compatibility closure. The candidate must have zero unexplained applicable gaps.

Only the fail-closed audit authorizes readiness:

```bash
cargo xtask release audit --manifest reference/release/candidate-manifest.json --candidate <full-source-candidate-sha> --output human
```

The audit must reject missing or duplicate kinds, mixed commits, stale
conditional support, unreviewed records, wrong producer identities, artifact or
payload hash drift, advisory or unsafe waivers, package drift, incomplete
corpus outcomes, and nonzero compatibility gaps.

## Reuse the exact package

Create the `.crate` archive once with Rust 1.97.0 and bind its exact SHA-256,
byte size, package/version, `rust-version = "1.92"`, features, dependencies,
source inventory, legal files, scalar mode, compiler class, tolerance profile,
and source-candidate commit:

```bash
cargo xtask package create-artifact --archive target/release-candidate/liquidfun.crate --identity target/release-candidate/package-identity.json --candidate-commit <full-source-candidate-sha>
```

MSRV and every durable or conditional native lane must download and verify
those exact bytes. Do not rerun `cargo package`, rebuild a different archive, or
substitute an unpacked tree in a downstream lane. The archive must remain
Cargo-only and exclude the C++ oracle, protocol, reference data, benchmark,
renderer, testbed, and other private tooling.

## Review notices and evidence

Confirm the archive contains the root MIT `LICENSE` and that repository release
materials preserve all applicable LiquidFun, Box2D, derived-material, and
developer-dependency notices in `THIRD_PARTY_NOTICES.md` and the source map.
Unmapped derivation or unresolved license classification blocks publication.

Review the human report together with the machine manifest. Every public parity,
platform, safety, and performance sentence must name or link evidence no broader
than its workload, target, policy, and candidate. Coverage percentages,
diagnostic profiles, screenshots, testbed pixels, and D2 portability results do
not independently prove parity.

## Attest after the source freeze

Tracked release records may be committed after the frozen source candidate only
to attest that candidate. The later attestation commit must contain an
allowlisted documentation and attestation-only diff and must name both the
frozen source-candidate SHA and the attestation commit.

Validate the proposed worktree before committing, then validate the committed
range. Never relabel the attestation commit itself as the audited source
candidate. Tag the reviewed attestation commit only after both validations and
the frozen-candidate audit pass.

## Publication dry run

Run the dry run against the same reviewed source and confirm its generated
archive is byte-identical to the audited `.crate` identity:

```bash
cargo publish -p liquidfun --dry-run
```

Inspect package contents, metadata, README links, license, MSRV, feature surface,
and isolated all-feature build/test results. Do not use `--allow-dirty` for the
publication decision.

## Publish or do not publish

Publish only the already audited package after the source candidate,
attestation, tag, registry metadata, and expected archive checksum all agree.
Record the registry response and published checksum without modifying the
evidence that authorized the decision.

Publication has no technical rollback. If any pre-publication identity,
evidence, notice, dry-run, or audit check fails, stop and create a new source
candidate; do not waive the check. If a defect is discovered after publication,
yank the affected version when appropriate, document the reason, prepare a new
audited SemVer release, and never overwrite or reuse the published version.
