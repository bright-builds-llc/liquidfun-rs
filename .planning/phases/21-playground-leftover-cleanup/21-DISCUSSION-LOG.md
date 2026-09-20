# Phase 21: Playground leftover cleanup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-20
**Phase:** 21-playground-leftover-cleanup
**Mode:** Yolo
**Areas discussed:** Unused proof leftover, Dead FallbackPanel branches, Scene file-length cleanup, Verification boundary

---

## Unused proof leftover

| Option | Description | Selected |
|--------|-------------|----------|
| Remove `loadProofSession` | Delete the unused Dam Break wrapper; keep `loadSceneSession` | ✓ |
| Re-wire `loadProofSession` | Call it again from App or tests so the helper is used | |
| Fold forensic spec into player smoke | Run Dispose-session / PNG-hash checks in `just web-player-smoke` | |
| Keep forensic spec as explicit opt-in | `PHASE16_CLOSURE_ATTEMPT_DIR` / `just web-smoke`; do not fold into product smoke | ✓ |

**User's choice:** Yolo recommended defaults — remove the unused helper; keep the forensic spec opt-in and historical.
**Notes:** `loadProofSession` has no live callers. The forensic spec does not import it. Product chrome no longer has Dispose session.

---

## Dead FallbackPanel branches

| Option | Description | Selected |
|--------|-------------|----------|
| Remove empty and not-ready kinds | All six scenes are ready; empty hashes already normalize to Dam Break | ✓ |
| Make empty/not-ready reachable | Undo empty-hash Dam Break replacement or add a not-ready catalog id | |
| Keep unknown fallback | Scene not found plus Open Dam Break remains the real visitor path | ✓ |

**User's choice:** Yolo recommended defaults — delete dead branches; keep unknown; do not undo empty-hash normalization.
**Notes:** Phase 17 wanted distinct empty vs unknown copy. Later shell work made empty hashes redirect to Dam Break. Leftover cleanup follows the live visitor path.

---

## Scene file-length cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm named scene files under 628 | Split further only if they exceed the gate again | ✓ |
| Change particle recipes to shrink files | Out of scope; would change public scene behavior | |
| Split xtask test if file-lengths must exit 0 | Prefer `foo.rs` plus `foo/` over a TSV exception | ✓ |

**User's choice:** Yolo recommended defaults — no behavior change; named files already pass; xtask only if the phase gate requires `file-lengths` to exit 0.
**Notes:** Live scout 2026-09-20: `color_mixer.rs` 346, `dam_break.rs` 360, `water_wheel.rs` 426, `upstream_cli.rs` 642.

---

## Verification boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Existing local player smoke plus unit/file-length checks | `just web-player-smoke`, wasm tests, Bright Builds file-lengths | ✓ |
| New Pages URL / browser matrix / C++ timing | Out of phase and out of v1.1 definition of done | |
| Independent AI review, no self-approval | 2026-09-16 owner policy | ✓ |

**User's choice:** Yolo recommended defaults.
**Notes:** No crate/npm publish, no release tag, no new GitHub Pages URL.

## Claude's Discretion

- Exact helper filenames and test-module splits
- Unknown fallback component shape after kind removal
- Historical `just web-smoke` documentation wording
- Exact unknown-hash Playwright assertions

## Deferred Ideas

- Dam Break headless speed versus pinned C++
- Rewriting Phase 16 forensic smoke onto current player chrome
- MysticUI/Tailwind migration
- Package publication and a new Pages URL
