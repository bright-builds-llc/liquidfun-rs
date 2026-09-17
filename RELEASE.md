# Release process

## Current project scope

The project is hobby-oriented; [PROJECT-SCOPE.md](PROJECT-SCOPE.md) defines the
current goals. Linux x64 qualification and a dedicated performance runner do
not block ordinary development or project completion. The strict parity-bearing
process below is optional and runs only when explicitly requested. A simpler
experimental publishing checklist is still under discussion; package publication
requires a separate user instruction.

The optional strict qualification described below defines the fail-closed path for a parity-bearing `liquidfun`
release. A checklist item is not evidence: every accepted result must be
machine-readable, reviewed, hash-bound, and tied to one frozen source candidate.
If any required step fails or remains unavailable, do not publish.

## Current release status

This checkout is **not release-ready**. It has no completed full-SHA
`release-candidate` workflow run, no retained complete evidence bundle from that
run, and no tracked source-candidate, manifest, and report records accepted by
`cargo xtask release attestation validate`. Local tests, generated compatibility
closure, or a clean worktree cannot replace those run-bound inputs.

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

## Retain candidate producer attempts

Check the bounded orchestration before freezing C:

```bash
scripts/phase15-candidate-evidence.sh check
```

The command modes require Python 3.11 or newer and can run on macOS. Independent
semantic validation additionally requires a prepared environment with Bash 4 or
newer, GNU find, jq, and SHA tools. Use exact repository, candidate, resolved
branch or tag, run, and run-attempt identities throughout.

Dispatch Oracle with its actual declared input; it does not accept
`candidate_sha`:

```bash
scripts/phase15-candidate-evidence.sh dispatch \
  --candidate <full-source-candidate-sha> \
  --attempt-root target/release-attempt/dispatch-oracle \
  --workflow oracle.yml --ref <exact-branch-or-tag> \
  --input evidence_phase=phase11
```

Supply all declared inputs explicitly for other workflows, including
`candidate_sha=C`. Performance requires the reviewed controlled-host parameters.
If dispatch returns uncertain results, reconcile the same attempt before
considering another dispatch:

```bash
scripts/phase15-candidate-evidence.sh reconcile \
  --candidate <full-source-candidate-sha> \
  --attempt-root target/release-attempt/dispatch-oracle
```

Reconciliation requires exactly one authoritative GitHub run URL in the
persisted `dispatch-response.txt`, then verifies that exact run ID against
fresh provider metadata. An absent or ambiguous URL remains blocked even when
a same-SHA run exists; time proximity and a unique search result do not authorize
redispatch or establish which run this attempt created.

Collect each of the seven producer stems (`platform`, `oracle`, `safety`,
`fuzz`, `regressions`, `coverage`, and `performance`) with its exact terminal
run and attempt. For example:

```bash
scripts/phase15-candidate-evidence.sh collect \
  --candidate <full-source-candidate-sha> \
  --attempt-root target/release-attempt/retained \
  --workflow oracle --ref <exact-branch-or-tag> \
  --run-id <run-id> --run-attempt <run-attempt>
```

Each producer gets a new subdirectory that is never overwritten. A failed
collection requires a new attempt root; preserve the failed records. At a
checkout of C, independently validate and restore the retained artifacts into
a fresh destination:

```bash
scripts/phase15-candidate-evidence.sh validate-retained \
  --candidate <full-source-candidate-sha> \
  --attempt-root target/release-attempt/retained \
  --destination target/release-attempt/fresh-downloads \
  --validator-bash bash
```

This checks the provider's current attempt and retained archive SHA-256 values,
restores the 21 artifact directories, and invokes the existing semantic
validator. It does not establish release readiness. Continue with the existing
release aggregation and audit, then the C/A attestation sequence below.

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

Independent review follows the [repository review policy](AGENTS.md#independent-review): an identified human or independent AI reviewer may review the evidence.

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

Source-readiness verification happens before freezing source commit C: finish
implementation, tooling, planning, documentation prerequisites, and local gates.
Final phase verification happens after the producers, retained evidence, and
both attestation validations pass. It must not be recorded as complete before
that evidence exists.

Tracked release records may be committed after the frozen source candidate only
to attest that candidate. Complete this sequence without reordering it:

1. Finish source-readiness verification before pushing the frozen source
   candidate C; keep public readiness non-ready.

1. Run every reviewed producer and the full-SHA `release-candidate` workflow
   against that exact candidate.

1. Download the complete retained bundle, including all evidence envelopes and
   the exact `.crate` archive referenced by its manifest. At a fresh checkout
   of C, restore every payload to its original repository-relative `target/`
   path; an archive inventory without its payload bytes cannot pass validation.

1. Materialize the source-candidate, manifest, and report records, then validate
   the proposed worktree:

   ```bash
   cargo xtask release attestation validate-worktree \
     --source reference/release/source-candidate.json \
     --manifest reference/release/candidate-manifest.json \
     --report reference/release/audit-report.json
   ```

1. Commit only the three JSON records as attestation commit A directly after C,
   then validate the explicit frozen-source-to-attestation range C..A:

   ```bash
   cargo xtask release attestation validate \
     --source reference/release/source-candidate.json \
     --manifest reference/release/candidate-manifest.json \
     --report reference/release/audit-report.json \
     --attestation-commit <full-attestation-commit-sha>
   ```

1. Only after both validations pass may public readiness status be projected
   in a later documentation commit D. Each of `README.md`, `COMPATIBILITY.md`,
   and `RELEASE.md` must carry `Status: **release-ready**`,
   `Source candidate: <full-source-candidate-sha>`, and
   `Attestation commit: <full-attestation-commit-sha>`, with the two SHA values
   individually enclosed in Markdown backticks. Check that projection using
   `cargo xtask docs check --attestation-commit <full-attestation-commit-sha>`.
   The default docs check keeps requiring non-ready copy; it never infers A
   from `HEAD` or a ready boolean. Complete final phase verification and the
   milestone audit against the accepted C/A evidence. Tags and package
   publication require separate release authority.

Keep planning, scripts, and ordinary documentation edits outside C..A; the
nine-path attestation allowlist remains unchanged. Reverted intervening edits
are still outside that boundary. Later metadata or documentation commits do not
change C or A: validation from D must still explicitly select A, read records
byte-identical to those committed at A, and restore the retained payload paths.
Missing, stale, or tampered records and payloads cannot authorize ready copy.

Cargo CI runs `bash scripts/phase15-docs-check.sh`. With no standalone
attestation marker in README, this runs the default non-ready docs check.
After projection, it reads the explicit C/A markers, verifies the three tracked
records against A before selecting the release run, and downloads the exact
`phase12-release-<run-id>-<source-candidate-sha>` archive. It restores the 19 audit
envelopes and exact package at `target/phase12-release/<source-candidate-sha>`
and runs the full attestation-backed docs check. No source or attestation SHA
is inferred from HEAD. The original ZIP and command/provider logs remain under
`target/phase15-docs-ci/`; an existing restoration destination is rejected.
Run this check in a fresh checkout with `contents: read` and `actions: read`
access. The release workflow currently retains that artifact for 90 days;
missing or expired evidence fails closed rather than relying on cached bytes.

Never relabel the attestation commit itself, current `HEAD`, or a documentation
projection commit as the audited source candidate.

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
