# Phase 45: Curiosity Groundwork and Feasibility Spikes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-14
**Phase:** 45-curiosity-groundwork-and-feasibility-spikes
**Mode:** Yolo
**Areas discussed:** Fingerprint baseline and gate, Count de-hardcoding, File splits, Per-scene checklist, Spike conduct, Siphon spike, Hourglass spike

---

## Fingerprint baseline and gate

| Option | Description | Selected |
|--------|-------------|----------|
| Committed JSON baseline + tested xtask gate + just recipe | Pure join function with unit tests; provenance in the file | ✓ |
| Gitignored base file + shell join/awk | Research ARCH option; loses provenance and protection across clones | |
| Plain diff of a 25-line file | v1.5 procedure; fails as soon as scene 26 exists | |

**User's choice:** [auto] Committed JSON baseline + tested xtask gate (recommended default)
**Notes:** Matches the v1.6 STATE decision and the research reconciliation (commit the list).

## Count de-hardcoding

| Option | Description | Selected |
|--------|-------------|----------|
| `ALL_SCENE_IDS` + `SCENE_IDS.length`, number-free copy | One edit-free path per new scene; meets "no copy hard-codes twenty-five" | ✓ |
| Derive a number-word from the count | Needs a number-to-words helper for copy | |
| Keep the literal copy and edit per scene | Violates success criterion 2 | |

**User's choice:** [auto] `ALL_SCENE_IDS` and number-free copy (recommended default)

## File splits

| Option | Description | Selected |
|--------|-------------|----------|
| Split existing records and previews into cohesive modules, pure move | Keeps order and imports | ✓ |
| Add empty curiosity modules only | Leaves the existing files at the limit | |

**User's choice:** [auto] Pure-move split (recommended default)

## Per-scene checklist

| Option | Description | Selected |
|--------|-------------|----------|
| `docs/adding-a-scene.md` + AGENTS.md pointer | Discoverable by humans and agents, mdformat-checked | ✓ |
| Planning artifact only | Hidden from contributors | |

**User's choice:** [auto] Repository doc with AGENTS.md pointer (recommended default)

## Spike conduct

| Option | Description | Selected |
|--------|-------------|----------|
| Ignored lib tests, committed then removed (40-08 precedent) | No new test binary; code restorable from history | ✓ |
| Keep spike code in tree | Dead code in the bridge | |
| New integration test binary | ~2 min macOS scan per binary | |

**User's choice:** [auto] 40-08 precedent (recommended default)

## Siphon spike

| Option | Description | Selected |
|--------|-------------|----------|
| Six candidates in research order, earliest pass wins, go/reshape/drop by natural vs stand-in | Matches SUMMARY reconciliation | ✓ |
| Stop after natural candidates | Skips honest stand-ins | |

**User's choice:** [auto] Full ordered candidate list (recommended default)

## Hourglass spike

| Option | Description | Selected |
|--------|-------------|----------|
| Stick-slip → freeze-to-WALL → rigid clumps; drop to seed on fail; owner confirms | Matches SUMMARY and the owner's 2026-10-14 rule | ✓ |
| Ship as "sand pours" on fail | Rejected by the owner decision | |

**User's choice:** [auto] Ordered candidates with owner-confirmed decision (recommended default)

## Claude's Discretion

- Split module names, baseline file name and JSON shape, xtask subcommand name, spike geometry and sampling details, plan breakdown.

## Deferred Ideas

- SIPHON-03, SAND-03, animated hourglass rotation, raising the 48-circle frame cap.
