# Phase 15: Re-establish Candidate Evidence - Discussion Log

> Audit trail only. Decisions are captured in CONTEXT.md.

**Mode:** Yolo
**Date:** 2026-09-14T16:43:10.303691+00:00

## Candidate freeze

**Question:** When should the candidate be frozen?

**Alternatives:** Freeze current HEAD immediately; finish repairs before freeze; reuse prior D2 proof.

**Auto-selected recommendation:** After all source, tooling, planning and prerequisite repairs pass; preserve one full SHA for all accepted producers.

## Evidence and recovery

**Question:** How should evidence be collected?

**Alternatives:** Staged fresh production; all producers immediately; reuse independently validated same-candidate outputs.

**Auto-selected recommendation:** Stage prerequisite checks before expensive producer fan-out; retain complete same-candidate artifacts and exact run/attempt identities in separate directories.

## Acceptance and infrastructure

**Question:** What happens when a producer is unavailable?

**Alternatives:** Preserve required evidence; substitute shared-host trend; waive performance.

**Auto-selected recommendation:** Continue independent work, record the missing controlled performance runner/identity prerequisite, and keep acceptance non-ready until every required producer passes.

## Attestation and status

**Question:** How should readiness be recorded?

**Alternatives:** Strict frozen-source attestation; infer readiness from green CI.

**Auto-selected recommendation:** Validate both frozen-source worktree and committed-range attestation, then project readiness and repeat milestone audit; no package release or waiver.

Recommendations were synthesized from prior phase decisions, current tooling and independent advisor research. No new human approval is asserted. Infrastructure clarification remains pending.

## Scope revision — 2026-09-16

The owner requested: “refactor and remove the requirement for Linux x64 testing” and clarified that this is “more of a fun project, not a super high quality implementation.”

Confirmed: hobby/experimental goal; Linux x64 and its controlled benchmark host no longer gate ordinary completion. Explicit Linux Cargo checks and manually selected oracle modes remain available. Preserve existing evidence and strict validators rather than claim missing certification passed. The previous source freeze does not block scope changes.

Two preference questions were presented: local checks plus one hosted CI job versus local-only; and optional manual expensive checks versus a small scheduled sample versus removing unused workflows. No answers have been received yet. Broader CI layout, schedules, platform promises, and a simpler publishing checklist remain proposals in PROJECT-SCOPE.md. No broader answer is inferred.

## Accepted lighter baseline — 2026-09-16

The owner answered both pending questions: "Local checks + one CI job (recommended)" and "Optional manual checks (recommended)". Use one macOS Cargo smoke job and make expensive suites manual-only. This supersedes the earlier pending status for these two questions; release paperwork and compatibility promises remain discussion topics.
