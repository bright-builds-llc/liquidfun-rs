---
phase: 13-restore-evidence-and-replay-integrity
verified: 2026-07-27T15:03:58Z
status: passed
score: 7/7 success criteria verified
requirements:
  - FND-04
  - COMP-04
  - COMP-05
  - COMP-08
  - TEST-07
  - EXMP-03
---

# Phase 13 Verification

## Verdict

Phase 13 is complete. Canonical Linux acceptance run `30277799121` passed at exact accepted head `dbaa64819debc5da268d32fcd342da7632ac6370`, and the independently downloaded schema-v2 terminal identity validates the exact producer, bundle, review base, promotion, and accepted-head chain.

## Exact Identity

| Identity | Value |
| --- | --- |
| Producer P | `6e8261a66a67a05bf3fadb4ad9d818121c395324` |
| Bundle B | `fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2` |
| Review base R | `88aba114356cd84c9464d4e6ff62f1d6d3872af7` |
| Promotion Q | `9f3c7c3480a7e371b4d7c39f7050da3ed4a660e5` |
| Accepted head A | `dbaa64819debc5da268d32fcd342da7632ac6370` |
| Review SHA-256 | `58e41c6d754341f9dba8a9fbfb1a0c2d4dbc485fdf46129a680a62e2af5a5735` |

## Success Criteria

| # | Criterion | Result | Evidence |
| --- | --- | --- | --- |
| 1 | Reviewed live replay is strict, projection-aware, and shares production acquisition. | Pass | Focused Phase 13 and catalog failure suites passed 70/70; the canonical terminal records the exact reviewed live-check as its final successful step. |
| 2 | Projected physics and contract failures produce bounded replayable evidence in their designated roots. | Pass | Failure contracts verify exact request authority, comparison surface, confinement, captures where available, typed categories, and RFC 6901 first-divergence pointers. |
| 3 | Incremental schema-v2 promotion reviews all seven members and changes only the real subset. | Pass | Q's Git diff equals the three recorded changed paths; the other four members are byte-identical to R; all seven are unchanged from Q through A. |
| 4 | One fresh acknowledgment authorizes the complete review subject. | Pass | Reviewer `pRizz` acknowledged the exact schema-v2 review digest `58e41c6d...5735`; no previous acknowledgment was reused. |
| 5 | Acceptance proves P/B/R/Q/A, exact paths, immutable bytes, and ordered live checking. | Pass | Terminal identity schema v2 records all identities and path/content digests; Q has R as sole parent and exact trailers; all seven ordered steps succeeded. |
| 6 | Canonical Linux acceptance succeeds at exact A and terminal identity validates independently. | Pass | Run `30277799121` passed at A. Artifact `8657594142` was downloaded and independently checked against Git history, promoted bytes, and ordered steps. |
| 7 | Failed runs and original promotion history remain intact. | Pass | The Plan 13-04 summary retains original history and appends all recovery attempts, including failed runs `30211150612`, `30211470256`, `30211674242`, and `30277369306`. |

## Requirement Coverage

| Requirement | Result | Verification |
| --- | --- | --- |
| FND-04 | Verified | Schema-v2 manifest, receipt, source map, replay evidence, and provenance bind upstream paths, revision, notices, and alteration state. |
| COMP-04 | Verified | The shared out-of-process acquisition and reviewed replay operate on semantic protocol records without exposing C++ pointers or layouts. |
| COMP-05 | Verified | Sealed request authority and replay records bind scenario/schema, upstream and adapter identities, target/build context, seed, and tolerance profile. |
| COMP-08 | Verified | Typed bounded evidence identifies the first divergent RFC 6901 path and retains the exact replay request and comparison surface. |
| TEST-07 | Verified | The rigid-stack regression and its reviewed replay evidence remain named, tracked, reproducible, and provenance-bound. |
| EXMP-03 | Verified | The same catalog scenario definition drives native execution, oracle acquisition, regression replay, and differential acceptance. |

## Canonical Artifact

Terminal artifact `8657594142`, named `phase13-terminal-identity-30277799121-dbaa64819debc5da268d32fcd342da7632ac6370`, has provider digest `sha256:6e51b5f49937e283761ec9c805552af1de4da2a6cc28fe8c5f1b2e63fc02a304`. Its downloaded `identity.json` hashes to `46b42effa1def2a61095a25d88955c8fe7fcba677158039a09ef32208446d25f`.

The identity reports exactly seven successful ordered steps: identity, provenance, reviewed replay, diagnosis, regression, oracle build, and live replay. The failure upload was skipped and terminal publication occurred only after the aggregate gate passed.

## Disconfirmation Evidence

Acceptance run `30277369306` failed before terminal publication when the independent provenance parser rejected the new `digest_mode` field. It uploaded no identity. The acceptance-only repair added fail-closed mode validation outside both producer-affecting closures, after which run `30277799121` passed without changing the acknowledged P/B/R/Q evidence chain.

## Authority Limits

This verification establishes the Phase 13 Linux exact-head evidence and replay contract. It does not claim Windows particle-group compatibility; that work begins in Phase 14.
