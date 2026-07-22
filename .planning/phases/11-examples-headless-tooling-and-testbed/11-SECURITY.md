---
phase: 11-examples-headless-tooling-and-testbed
phase_number: 11
audited_at: 2026-07-22T17:00:00-05:00
asvs_level: 1
block_on: open
status: secured
threats_total: 174
threats_closed: 174
threats_open: 0
unregistered_flags: 0
auditor: gsd-security-auditor
---

# Phase 11 Security Verification

## Result

**SECURED.** All 174 registered OWASP ASVS L1 / STRIDE threats from Plans
11-01 through 11-29 have disposition `mitigate` and are closed by implemented,
tested controls. No registered threat uses `accept` or `transfer`, no formal
`## Threat Flags` section exists in any Phase 11 summary, and no unregistered
summary flag requires follow-up.

The two renderer advisories are not reclassified as accepted threat-register
risks. They are a narrow implementation constraint inside the mitigated private
dependency boundary: only `RUSTSEC-2025-0035` and `RUSTSEC-2026-0192` are
ignored, the affected Macroquad graph is private, non-default, and unpublished,
the published `liquidfun` package rejects renderer dependencies, and Phase 12
must replace the renderer and remove both ignores before release-readiness
claims.

## Audit Method

1. Read every Plan 11-01 through 11-29 and every matching summary in full.
1. Extracted all 174 threat rows and classified their declared dispositions.
1. Read or hashed every implementation file named by plan frontmatter; the one
   superseded Plan 11-12 path, `tools/reference/src/adapter-inputs.txt`, is
   explicitly corrected by its summary to the implemented authority at
   `tools/reference/adapter-inputs.txt`.
1. Matched each mitigation to source and adversarial-test patterns in the plan's
   cited files, including strict decoding, bounds, identity checks, path
   confinement, process teardown, artifact topology, package isolation, and
   deterministic evidence attribution.
1. Checked all summaries for formal `## Threat Flags`; none were present.

## Disposition Summary

| Disposition | Count | Verification |
| --- | ---: | --- |
| mitigate | 174 | Implemented control and regression evidence found; all closed |
| accept | 0 | No accepted-risk-log entry required by the threat register |
| transfer | 0 | No transfer documentation required by the threat register |

## Evidence Packages

Each package below verifies the two plan-specific trust-boundary threats plus
the plan's four cross-cutting bounds, disclosure, isolation, and attribution
threats. The corresponding summary's Security Verification or Security Review
section supplies the plan-level review trail.

| Evidence | Implemented mitigation evidence |
| --- | --- |
| E11-01 | Strict 4 MiB/depth-32 parsing at `tools/xtask/src/inventory/corpus.rs:11`; revision, count, uniqueness, and strict raw records at `tools/xtask/src/inventory/corpus/model.rs:443` and `:684`; traversal/duplicate/limit regressions at `tools/xtask/tests/corpus_model.rs:58`. |
| E11-02 | Source file/byte budgets and pinned-checkout verification at `tools/xtask/src/inventory/corpus/discovery.rs:20`, `:194`, and `:428`; canonical in-memory checking and atomic publication at `:61`; adversarial discovery coverage at `tools/xtask/tests/corpus_discovery.rs:199`. |
| E11-03 | Strict catalog records at `crates/liquidfun-test-protocol/src/catalog/model.rs:369`; canonical-byte hashing, size enforcement, hash verification, and re-encoding equality at `crates/liquidfun-test-protocol/src/catalog/resolve.rs:285` and `:313`; protocol crate remains unpublished at `crates/liquidfun-test-protocol/Cargo.toml:9`. |
| E11-04 | Stable, evidence-backed, deterministic rigid definitions at `crates/liquidfun-test-protocol/src/catalog/scenarios/rigid.rs:377`; exact replay tamper rejection at `crates/liquidfun-test-protocol/src/catalog/scenarios/rope.rs:135`; all definitions pass through checked catalog constructors. |
| E11-05 | Particle/group/query definitions use checked stable IDs and evidence mappings at `crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs:30` and `crates/liquidfun-test-protocol/src/catalog/scenarios/queries_callbacks.rs:12`; deterministic and tampered-owner regressions at `queries_callbacks.rs:204` and `:268`. |
| E11-06 | Bounded mapping authority and fail-closed duplicate/missing/unknown/stale/contradictory validation at `crates/liquidfun-test-protocol/src/catalog/mapping.rs:204`; adversarial registry tests at `crates/liquidfun-test-protocol/tests/catalog_registry.rs:64`, `:94`, `:117`, and `:144`. |
| E11-07 | Closed session state/command types and pre-effect validation at `crates/liquidfun-differential/src/session.rs:480` and `:515`; one in-flight command and checked logical counters at `:490` and `:712`; duplicate-command no-effect regression at `crates/liquidfun-differential/src/session/tests.rs:232`. |
| E11-08 | Explicit reviewed observation maxima and stable owned identities at `crates/liquidfun/src/world/observation.rs:22`; diagnostic durations are separate and non-hashable at `crates/liquidfun/src/world/observation/profile.rs:58`; exact/one-over and profile-separation tests at `crates/liquidfun/tests/world_observations.rs:126` and `:176`. |
| E11-09 | Primitive/vertex/text limits and typed failures at `crates/liquidfun/src/debug_draw/collector.rs:15`; inert labels without renderer commands at `crates/liquidfun/src/debug_draw/primitive.rs:321`; stable finite output and private-identity exclusion at `crates/liquidfun/tests/debug_draw.rs:67` and `:102`. |
| E11-10 | Strict run request records and exact byte/hash identity at `crates/liquidfun-test-protocol/src/catalog/wire.rs:18` and `:68`; finite/order/bounds validation at `crates/liquidfun-test-protocol/src/checkpoint.rs:332` and `:406`; unknown/duplicate/wrong-hash regressions at `crates/liquidfun-test-protocol/tests/checkpoint_protocol.rs:195`. |
| E11-11 | Canonical bytes are verified before session creation and failed sessions are discarded at `crates/liquidfun-differential/src/catalog_native/executor.rs:64` and `:95`; panic boundaries at `:141` and `:163`; wrong-hash pre-session rejection at `crates/liquidfun-differential/tests/catalog_native.rs:115`. |
| E11-12 | Bounded JSONL record functions at `tools/reference/src/protocol.hpp:150` and `tools/reference/src/protocol.cpp:1247`; repeated fresh-session/reset and private-field exclusion at `tools/reference/tests/protocol_tests.cpp:190` and `:221`; all behavior-affecting inputs are digest-bound in `tools/reference/adapter-inputs.txt:1` and CMake consumes that authority at `tools/reference/CMakeLists.txt:217`. |
| E11-13 | Comparison verifies identity before an exhaustive semantic walk at `crates/liquidfun-differential/src/comparison_model/diff.rs:21` and `:33`; bounded/redacted records at `crates/liquidfun-differential/src/comparison_model.rs:21`; private-path and diagnostic-redaction regressions at `crates/liquidfun-differential/tests/comparison_model.rs:390` and `:481`. |
| E11-14 | Process failures carry bounded stderr and kill/reap proof at `crates/liquidfun-differential/src/supervisor/catalog.rs:59` and `:144`; confined, non-clobbering failure writes at `crates/liquidfun-differential/src/failure_bundle.rs:293` and `:349`; timeout/reap and symlink regressions at `crates/liquidfun-differential/tests/catalog_failures.rs:111` and `:147`. |
| E11-15 | Closed headless CLI argument forwarding is exercised at `tools/xtask/tests/catalog_cli.rs:63` and `:95`; `tools/xtask/src/differential.rs` builds subprocesses with explicit argument arrays and validated scenario/preset/profile values; renderer-free catalog routing is exposed by `tools/xtask/src/main.rs:23`. |
| E11-16 | Replay validates confined canonical directories, required entries, IDs, profiles, hashes, and bounds before execution at `crates/liquidfun-differential/src/fixtures/replay.rs:44`; duplicate/hash/symlink regressions at `crates/liquidfun-differential/tests/catalog_regressions.rs:214` and `:231`. |
| E11-17 | Criterion is confined to an unpublished package at `crates/liquidfun-benchmarks/Cargo.toml:9`; cases validate fixed identities, horizons, and expected checkpoints before timing at `crates/liquidfun-benchmarks/tests/catalog_equivalence.rs:7` and `:49`; dependency isolation is asserted at `:88`. |
| E11-18 | Headless catalog coverage at `crates/liquidfun-differential/tests/headless_catalog.rs:111`; CI clears display variables at `.github/workflows/ci.yml:71`; package archive traversal/content/default-member controls at `tools/xtask/src/package.rs:177` and `tools/xtask/tests/package_cli.rs:236`. |
| E11-19 | Executable corpus validation rejects missing, unknown, duplicate, stale, circular, contradictory, renderer, and private claims at `crates/liquidfun-differential/tests/phase11_corpus.rs:79` and `:107`; tracked mappings are content-addressed by `crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json`. |
| E11-20 | Evidence paths enforce 16 MiB JSON, 32 MiB archives, 64 files, depth 4, regular files, no symlinks/traversal/case collisions at `tools/xtask/src/phase11_evidence/paths.rs:13`; exact authority rejects denied run/artifact IDs and mixed same-run pairs at `tools/xtask/src/phase11_evidence/authority.rs:131` and `:184`; adversarial topology tests at `tools/xtask/tests/phase11_evidence_cli.rs:65`. |
| E11-21 | Evidence shell is fail-fast, fixed-mode, symlink-safe, temporary-file based, validates content before identity-last publication at `scripts/phase11-evidence.sh:2`, `:38`, `:57`, `:270`, and `:344`; workflow uses read-only permissions and pinned actions at `.github/workflows/oracle.yml:22`, `:44`, and `:369`; routing assertions at `tools/xtask/tests/phase11_evidence_cli/workflow.rs:99`. |
| E11-22 | Fresh authority metadata binds one successful dispatch/run, two distinct successful jobs, two artifacts, and immutable SHA in `target/phase11-evidence/run.json:65`; failed run/artifacts are permanently denied by the reusable command at `TESTING.md:940` and `:992`; archives were inspected before isolated extraction at `TESTING.md:984`. |
| E11-23 | Exact authority fixes SHA, jobs, artifact names/digests, semantic hashes, and denied identities in `reference/artifacts/phase11/exact-ref.json:3`; validator hard-codes and rechecks the authority/digests at `tools/xtask/src/inventory/validation/phase11.rs:56` and `:192`; mutation/diagnostic-claim tests at `tools/xtask/tests/inventory_cli/phase11.rs:26` and `:118`. |
| E11-24 | Testbed package is unpublished at `crates/liquidfun-testbed/Cargo.toml:9`; capability output is confined and rejects links at `crates/liquidfun-testbed/src/capability.rs:129`; the capability test proves regular outputs and no session effects at `crates/liquidfun-testbed/tests/capability.rs:18` and `:47`. |
| E11-25 | About links are fixed/validated HTTPS targets with literal unavailable states at `crates/liquidfun-testbed/src/ui/about.rs:10`, `:43`, and `:116`; presentation state remains separate from typed effects in `crates/liquidfun-testbed/src/app/state.rs`; shell regressions in `crates/liquidfun-testbed/tests/app_shell.rs` enforce typed-command-only effects and bounded display text. |
| E11-26 | Editing suppresses shortcuts at `crates/liquidfun-testbed/src/input.rs:104`; controller adapter prevents duplicate in-flight submission and emits typed `SessionCommand` values at `crates/liquidfun-testbed/src/controller_adapter.rs:153`; viewport validates finite geometry and confined non-linked PNG paths at `crates/liquidfun-testbed/src/ui/viewport.rs:35` and `:553`; no-tick/capture and traversal tests at `crates/liquidfun-testbed/tests/controller_ui.rs:187` and `:542`. |
| E11-27 | Screenshot output is explicitly diagnostic-only at `crates/liquidfun-testbed/src/screenshot.rs:12`; difference and inspector state consume canonical comparison records without redefining policy; `crates/liquidfun-testbed/tests/visual_contract.rs:88`, `:218`, and `:301` prove canonical projection, presentation-only resize, and bounded retained diagnostics. |
| E11-28 | Closure validates exact revision, authority headers, digests, counts, joins, terminal evidence, duplicates, and unresolved rows at `tools/xtask/src/inventory/corpus/validation.rs:67` and `:251`; deterministic report exposes unresolved count at `tools/xtask/src/inventory/corpus/report.rs:37`; closure tests reject unresolved/unknown/duplicate/unmapped items at `tools/xtask/tests/corpus_closure.rs:97`; report records 388 items and 221/127/40 terminal totals at `UPSTREAM-CORPUS.md:10`. |
| E11-29 | Root defaults only to `liquidfun` at `Cargo.toml:10`; metadata rejects extra publishable packages and renderer/private dependencies at `tools/xtask/src/package/metadata.rs:129` and `:246`; archive inspection rejects non-regular, oversized, traversing, duplicate, private, graphical, and native content at `tools/xtask/src/package.rs:177`; focused contracts at `tools/xtask/tests/package_cli.rs:183` and `:215`. |
| X-ADVISORY | `deny.toml:6` documents and lists exactly `RUSTSEC-2025-0035` and `RUSTSEC-2026-0192`; `tools/xtask/tests/package_cli.rs:183` asserts the exact set, no Macroquad dependency in `liquidfun`, and unpublished testbed scope; `TESTING.md:1494` and `.codex/tasks/todo.md:128` require Phase 12 replacement and waiver removal. |

## Complete Threat Verification

Every threat ID appears once below. Category is encoded by its column; the row's
`mitigate`, `CLOSED`, and evidence package apply to all six IDs in that row.

| Plan | Tampering | Spoofing | Denial of service | Information disclosure | Elevation of privilege | Repudiation | Disposition | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 11-01 | T-11-01-01 | T-11-01-02 | T-11-01-03 | T-11-01-04 | T-11-01-05 | T-11-01-06 | mitigate | CLOSED | E11-01, E11-29 |
| 11-02 | T-11-02-01 | T-11-02-02 | T-11-02-03 | T-11-02-04 | T-11-02-05 | T-11-02-06 | mitigate | CLOSED | E11-02, E11-29 |
| 11-03 | T-11-03-01 | T-11-03-02 | T-11-03-03 | T-11-03-04 | T-11-03-05 | T-11-03-06 | mitigate | CLOSED | E11-03, E11-29 |
| 11-04 | T-11-04-01 | T-11-04-02 | T-11-04-03 | T-11-04-04 | T-11-04-05 | T-11-04-06 | mitigate | CLOSED | E11-04, E11-29 |
| 11-05 | T-11-05-01 | T-11-05-02 | T-11-05-03 | T-11-05-04 | T-11-05-05 | T-11-05-06 | mitigate | CLOSED | E11-05, E11-29 |
| 11-06 | T-11-06-01 | T-11-06-02 | T-11-06-03 | T-11-06-04 | T-11-06-05 | T-11-06-06 | mitigate | CLOSED | E11-06, E11-29 |
| 11-07 | T-11-07-01 | T-11-07-02 | T-11-07-03 | T-11-07-04 | T-11-07-05 | T-11-07-06 | mitigate | CLOSED | E11-07, E11-29 |
| 11-08 | T-11-08-01 | T-11-08-02 | T-11-08-03 | T-11-08-04 | T-11-08-05 | T-11-08-06 | mitigate | CLOSED | E11-08, E11-29 |
| 11-09 | T-11-09-01 | T-11-09-02 | T-11-09-03 | T-11-09-04 | T-11-09-05 | T-11-09-06 | mitigate | CLOSED | E11-09, E11-29 |
| 11-10 | T-11-10-01 | T-11-10-02 | T-11-10-03 | T-11-10-04 | T-11-10-05 | T-11-10-06 | mitigate | CLOSED | E11-10, E11-29 |
| 11-11 | T-11-11-01 | T-11-11-02 | T-11-11-03 | T-11-11-04 | T-11-11-05 | T-11-11-06 | mitigate | CLOSED | E11-11, E11-29 |
| 11-12 | T-11-12-01 | T-11-12-02 | T-11-12-03 | T-11-12-04 | T-11-12-05 | T-11-12-06 | mitigate | CLOSED | E11-12, E11-29 |
| 11-13 | T-11-13-01 | T-11-13-02 | T-11-13-03 | T-11-13-04 | T-11-13-05 | T-11-13-06 | mitigate | CLOSED | E11-13, E11-29 |
| 11-14 | T-11-14-01 | T-11-14-02 | T-11-14-03 | T-11-14-04 | T-11-14-05 | T-11-14-06 | mitigate | CLOSED | E11-14, E11-29 |
| 11-15 | T-11-15-01 | T-11-15-02 | T-11-15-03 | T-11-15-04 | T-11-15-05 | T-11-15-06 | mitigate | CLOSED | E11-15, E11-29 |
| 11-16 | T-11-16-01 | T-11-16-02 | T-11-16-03 | T-11-16-04 | T-11-16-05 | T-11-16-06 | mitigate | CLOSED | E11-16, E11-29 |
| 11-17 | T-11-17-01 | T-11-17-02 | T-11-17-03 | T-11-17-04 | T-11-17-05 | T-11-17-06 | mitigate | CLOSED | E11-17, E11-29 |
| 11-18 | T-11-18-01 | T-11-18-02 | T-11-18-03 | T-11-18-04 | T-11-18-05 | T-11-18-06 | mitigate | CLOSED | E11-18, E11-29 |
| 11-19 | T-11-19-01 | T-11-19-02 | T-11-19-03 | T-11-19-04 | T-11-19-05 | T-11-19-06 | mitigate | CLOSED | E11-19, E11-29 |
| 11-20 | T-11-20-01 | T-11-20-02 | T-11-20-03 | T-11-20-04 | T-11-20-05 | T-11-20-06 | mitigate | CLOSED | E11-20, E11-29 |
| 11-21 | T-11-21-01 | T-11-21-02 | T-11-21-03 | T-11-21-04 | T-11-21-05 | T-11-21-06 | mitigate | CLOSED | E11-21, E11-29 |
| 11-22 | T-11-22-01 | T-11-22-02 | T-11-22-03 | T-11-22-04 | T-11-22-05 | T-11-22-06 | mitigate | CLOSED | E11-22, E11-29 |
| 11-23 | T-11-23-01 | T-11-23-02 | T-11-23-03 | T-11-23-04 | T-11-23-05 | T-11-23-06 | mitigate | CLOSED | E11-23, E11-29 |
| 11-24 | T-11-24-01 | T-11-24-02 | T-11-24-03 | T-11-24-04 | T-11-24-05 | T-11-24-06 | mitigate | CLOSED | E11-24, E11-29, X-ADVISORY |
| 11-25 | T-11-25-01 | T-11-25-02 | T-11-25-03 | T-11-25-04 | T-11-25-05 | T-11-25-06 | mitigate | CLOSED | E11-25, E11-29, X-ADVISORY |
| 11-26 | T-11-26-01 | T-11-26-02 | T-11-26-03 | T-11-26-04 | T-11-26-05 | T-11-26-06 | mitigate | CLOSED | E11-26, E11-29, X-ADVISORY |
| 11-27 | T-11-27-01 | T-11-27-02 | T-11-27-03 | T-11-27-04 | T-11-27-05 | T-11-27-06 | mitigate | CLOSED | E11-27, E11-29, X-ADVISORY |
| 11-28 | T-11-28-01 | T-11-28-02 | T-11-28-03 | T-11-28-04 | T-11-28-05 | T-11-28-06 | mitigate | CLOSED | E11-28, E11-29 |
| 11-29 | T-11-29-01 | T-11-29-02 | T-11-29-03 | T-11-29-04 | T-11-29-05 | T-11-29-06 | mitigate | CLOSED | E11-29, X-ADVISORY |

## Threat Flags

No Phase 11 summary contains a formal `## Threat Flags` section. Incidental
summary statements about threat review, renderer isolation, path confinement,
and authority recovery map to their registered plan threats and evidence
packages above. There are no unregistered flags.

## Accepted Risks

None. The threat register contains no `accept` disposition. The temporary
renderer advisory waiver remains governed by the mitigated private dependency
boundary and the mandatory Phase 12 removal condition documented in
X-ADVISORY.

## Open Threats

None.

## Audit Conclusion

Phase 11 satisfies the configured `block_on: open` policy: `threats_open` is
zero, every registered mitigation has implementation and regression evidence,
and the renderer advisory waiver cannot silently expand or enter the published
consumer graph.
