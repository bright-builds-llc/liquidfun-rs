---
phase: 15-re-establish-candidate-evidence
verified: 2026-09-17T01:12:00Z
status: passed
score: 5/5 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 15-hobby-2026-09-17T00-43-37
generated_at: 2026-09-17T01:12:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 15: Hobby-project Wrap-up Verification Report

**Phase Goal:** Make the existing experimental library easy to use and honestly described, with a small release-preparation checklist and a bounded verification/completion record.
**Status:** passed
**Re-verification:** No — initial verification of the replacement hobby lifecycle, not the historical strict campaign.
**Verified implementation:** `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`, relative to planning baseline `92c536d`.

## Goal Achievement

The three roadmap success criteria are preserved below. Plan truths that restate them are merged; two additional execution truths retain the actual-results and native-regression requirements. All five are verified against document contents, command implementations, retained output, live CI identity and independent acknowledgment, rather than summary assertions alone.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Current scope, contributor and release guidance consistently describe experimental API, best-effort extra platforms and incremental parity, without changing the Rust development pin or declared compiler minimum. | VERIFIED | PROJECT-SCOPE.md accepted baseline, AGENTS.md current guidance/projection, README.md platform/API text and CONTRIBUTING.md agree. TESTING.md and BENCHMARKING.md distinguish current checks from historical strict profiles. Git comparison confirms unchanged Cargo metadata and rust-toolchain.toml. |
| 2 | A short experimental-release checklist reuses local Rust checks, relevant regressions, one native example, macOS Cargo CI and isolated package verification before publication; publication remains separately authorized. | VERIFIED | RELEASE.md experimental preparation has six steps and exact commands, source-bound macOS CI, notices and independent review. README links both scope and checklist. Package command traces to actual isolated extraction, license checks and build/test. |
| 3 | A source-identified completion record reports actual checks and limitations, preserves strict evidence semantics and distinguishes deferred certification from completed hobby work. | VERIFIED | 15-HOBBY-COMPLETION.md identifies full source, local platform/compiler, commands, CI and review. 15-REVIEW.md acknowledges the recomputed 28-file digest with actual separate AI identity/time. Strict validators, generated compatibility report and historical records are unchanged; strict status remains not release-ready. |
| 4 | The current experimental preparation checklist has actual local and macOS CI results. | VERIFIED | Ordered fmt/Clippy/build/test logs, package result, 43 docs tests and docs/Markdown/managed checks are retained under target/phase15-hobby/attempt-15-14-01. Independent live gh query confirms Cargo CI 35169132504 success at the exact implementation SHA, macOS success and optional Linux skipped. |
| 5 | Existing particle-group regressions and representative native behavior remain working. | VERIFIED | Focused logs show 30 passed including both persisted Windows seeds on macOS. Exact headless test executed once and passed; source executes eight representative scenarios and asserts nonempty semantic checkpoints and serialized capture bytes. |

**Score:** 5/5 truths verified. HOBBY-01 through HOBBY-03 are satisfied; no acceptance override was needed.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| PROJECT-SCOPE.md | Accepted hobby scope and compatibility promises | VERIFIED | Substantive accepted-policy table, current compiler meaning, independent review and separate publication boundary; linked from README, instructions and contributor guidance. |
| RELEASE.md | Experimental preparation checklist | VERIFIED | Six actionable steps and concise record schema ahead of retained strict process; command implementation is real. |
| README.md | Reachable current scope and preparation | VERIFIED | Links resolve to scope and RELEASE experimental section, with honest maturity/platform statements. |
| 15-HOBBY-COMPLETION.md | Source-bound results, review and limitations | VERIFIED | Completed local/hosted results and actual acknowledgment; no pending placeholder remains. |
| 15-REVIEW.md | Identified independent acknowledgment | VERIFIED | Separate AI /root/review15_hobby_docs, time 2026-09-17T01:10:03.370473+00:00, exact digest and all 28 bound entries. |

The artifact helper passed all three Plan 13 artifacts. Manual inspection additionally establishes substantive content and actual links for the completion/review records. COMPATIBILITY.md is wholly generated and was correctly preserved; public current guidance explains its optional strict meaning. This satisfies the goal without weakening its generator or validator.

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| README.md | PROJECT-SCOPE.md and RELEASE.md | Markdown links | WIRED | Current scope and experimental preparation links are present and targets exist. |
| RELEASE.md | tools/xtask/src/package.rs | cargo xtask package verify | WIRED | main.rs:130 dispatches package::run; verify checks metadata/archive/license, extracts outside the repository and calls build_and_test. The helper's literal-path search missed this command link; manual tracing resolves it without an override. |
| 15-HOBBY-COMPLETION.md | RELEASE.md | Experimental checklist link and result table | WIRED | Relative link resolves to repository RELEASE.md; all six checklist items mapped to actual results/limits. |
| 15-HOBBY-COMPLETION.md | .github/workflows/ci.yml | Run URL, headSha and retained identity | WIRED | Run 35169132504 belongs to bright-builds-llc/liquidfun-rs Cargo CI, matching implementation SHA. Workflow ordinary matrix contains macos-15 only. |
| Completion | Independent review | 15-REVIEW.md and fixed digest | WIRED | Recomputed every bound file SHA-256 and aggregate; no mismatches for 841c63faa79d3e615cdbf949e0c2c5bd4584a4692bd1b1e6558575f18d1c4ea7. |

### Data-Flow Trace (Level 4)

Not applicable to this documentation/evidence phase: no dynamic UI or new data-rendering component was introduced. The existing native test calls execute_catalog_native and asserts real semantic capture rather than an empty placeholder. Package verification invokes extracted consumer build/test and reports success only after those operations return successfully.

### Behavioral Spot-Checks and Evidence

No additional runtime spot-check was necessary for documentation-only implementation; inspected the already-executed required checks and their actual code paths. Independent read-only verification included live CI lookup and digest recomputation.

| Behavior | Command or evidence | Result | Status |
| --- | --- | --- | --- |
| Local ordered baseline | 01-fmt through 04-test logs | Formatting, Clippy, build and all-features tests passed | PASS |
| Retained particle regressions | 05-regressions.log | 10 + 8 + 12 tests passed; both persisted seeds executed | PASS |
| Native representative execution | 06-headless.log and headless_catalog.rs:192 | 1 executed/passed; eight scenario families with substantive capture assertions | PASS |
| Isolated native package | 07-package.log and package.rs verify/build_and_test | 238 entries built/tested outside repository; license and isolation checks run first | PASS |
| Public docs contracts | 08-contracts.log and 09-docs.log | 43 tests passed; all public document contracts verified | PASS |
| Markdown and managed checks | 10-markdown.log and 11-managed.log | Passed; zero managed findings | PASS |
| Hosted source identity | gh run view 35169132504 -R bright-builds-llc/liquidfun-rs --json headSha,conclusion,workflowName,url,jobs | Exact source, overall/macOS success, Linux skipped | PASS |
| Independent evidence binding | Recompute review-manifest.json file hashes and aggregate | 28 matches; acknowledged digest matches | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| HOBBY-01 | 15-13 | Experimental scope, truthful support and accurate compiler metadata | SATISFIED | Truth 1; unchanged manifests/pin and aligned guidance. |
| HOBBY-02 | 15-13, 15-14 | Bounded checklist, native/package checks and separate publication | SATISFIED | Truths 2, 4 and 5; six-step checklist and actual results. |
| HOBBY-03 | 15-14 | Source-bound completion, limitations and independent review preserving strict deferral | SATISFIED | Truth 3; completed record and exact independent acknowledgment. |

No orphaned active Phase 15 requirements. PLAT-01, PLAT-05 and DOCS-09 explicitly belong to the historical strict campaign and remain deferred; this report does not mark them satisfied. No later phase is being used to conceal an active gap.

### Anti-Patterns and Disconfirmation

No blocking placeholder, missing artifact, unwired command or false passing claim was found. Historical pending statements in TESTING.md are scoped to earlier phase records rather than active checklist placeholders.

Three possible false-positive success claims were checked directly:

1. Local 1.97 tests do not establish the declared 1.92 minimum: package.rs explicitly selects 1.97 on this macOS host, and the completion record preserves 1.92 verification as a pre-publication obligation.
1. A passing headless test does not certify GUI behavior or full upstream parity: its assertions prove native semantic execution/capture only, exactly as reported.
1. Historical strict or other-platform results cannot be presented as current source results: the record distinguishes historical debug follow-ups from the exact macOS result and retains strict non-readiness.

No new runtime/error-handling branch was introduced by this phase. Existing GUI limitations and nonparticle capture omissions are disclosed rather than repaired or certified here. Raw logs remain ignored local evidence; the tracked review preserves their hashes without promising permanent raw-log retention.

### Lifecycle and Guidance

CONTEXT, both active PLAN files and both SUMMARY files share lifecycle_mode yolo and phase_lifecycle_id 15-hobby-2026-09-17T00-43-37; this report copies that provenance. Neither active summary is marked direct-fallback. No previous verification report exists for this replacement lifecycle.

Applied AGENTS.md hobby/standing-authorization/independent-review guidance, AGENTS.bright-builds.md, standards-overrides.md, standards index and verification/testing/Rust standards, plus active global and repository lessons. Active lessons totaled 12,790 bytes and 4,264 conservative estimated tokens and were read completely; no archive input was loaded. No project skill directories were present.

### Human Verification Required

None for the accepted documentation and bounded native-check scope. GUI appearance, performance feel, other-platform coverage, minimum-compiler verification and package publication are not claims made by this phase. They remain explicitly separate future work, not hidden human gates on current completion.

### Gaps Summary

No gaps in the replacement hobby goal. Experimental preparation is evidenced; strict qualification remains **not release-ready**. No package release, version selection or tag is authorized by this report.

_Verifier: /root/verify15_hobby_goal, independent gsd-verifier agent._
