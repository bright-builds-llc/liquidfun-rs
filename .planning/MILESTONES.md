# Project Milestones: liquidfun-rs

## v1.1 Web Playground (Archived: 2026-09-20)

**Delivered:** A playful SolidJS GitHub Pages playground with six native Rust/WASM physics demos, shareable scene hashes, honest Reset labels, and automatic same-checkout Pages delivery. The v1.1 label identifies planning history; the crate was not published or tagged as a release.

**Completed:** 6 phases, 39 plans, 81 task counts from plan summaries.

### Accomplishments

- Compiled this repository's Rust engine to WebAssembly with copied typed frame arrays, so visitors see real particle/rigid motion without a C++ runtime or external JavaScript physics engine.
- Shipped a shared SolidJS player with play/pause/reset, unknown-hash fallback, bounded hidden-tab stepping, and OIDC GitHub Pages delivery on every push to main.
- Built six native scenes (Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel) with labeled bounded controls, host-locked credits, and Pointer Events interaction.
- Restored in-app SVG catalog previews inside the Kobalte shell and remounted live controls so Reset matches documented initial physics and labels.
- Verified the hosted path and local Chromium `just web-player-smoke` gate, then cleaned leftover proof helpers and Bright Builds file-length debt.

### Scope and known gaps

- All 22 v1.1 requirements are Complete. Audit status `passed` at HEAD `d3ed8b955a03643a5976f92b152434093b37cdf6`.
- Remaining notes are accepted phase warnings: paused-pointer redraw (WR-01), deferred multi-touch (IN-02), jelly construction token defaults (IN-01), and the Phase 19 hosted SHA record `d3d8688`.
- Playground Dam Break speed versus pinned C++ is out of v1.1 definition of done.
- Strict native qualification, crate publication, and release tags remain separately authorized.

### Statistics and provenance

- Timeline: 2026-09-16 to 2026-09-20 in Git author dates; archive date 2026-09-20 UTC.
- Git range: `4224bdeaa0ed132363e821a05f44ecb622cb81bf` (define v1.1) through the archive commits on this date.
- About 299 paths changed over `9469ca7..d3ed8b9` after v1.0 archive; first-party playground/WASM sources are on the order of 8k physical lines under `web/src` plus `crates/liquidfun-wasm`.
- Live origin recorded in Phase 19: `https://bright-builds-llc.github.io/liquidfun-rs/`. HOST-01 still publishes later main revisions.

### Archives

- [Roadmap](milestones/v1.1-ROADMAP.md)
- [Requirements and outcomes](milestones/v1.1-REQUIREMENTS.md)
- [Milestone audit](milestones/v1.1-MILESTONE-AUDIT.md)
- [Completion state](milestones/v1.1-STATE.md)

**Next:** No new milestone scope chosen. Use `/gsd-new-milestone` when ready; define fresh requirements and continue phase numbering after 21. Phase directories remain in place for stable historical references. No Git version tag was created: archive completion is not release-tag authorization.

---

## v1.0 Experimental Foundation (Archived: 2026-09-17)

**Delivered:** A usable experimental native Rust physics library and a lighter development/preparation workflow. The v1.0 label identifies planning history; the crate was not published or tagged as a release.

**Completed:** 16 phases, 252 active plans, 434 task blocks in completed plan files. The archive CLI recognizes 377 task entries from summary formats; these are different counting methods, not additional tasks. Original strict Phase 15 history is excluded from current-plan totals.

### Accomplishments

- Built an independent native Rust engine covering math, shapes, rigid bodies, joints and particles, with safe handles and transactional mutation.
- Established a pinned upstream reference, semantic comparison tooling and regression fixtures without adding a C++ runtime requirement for consumers.
- Added renderer-independent scenarios, headless tooling and a private visual testbed.
- Repaired particle-group invariants and reference capture issues while retaining source-bound evidence and regression protection.
- Reframed the project for enjoyable experimentation: local checks, one macOS CI job and optional manual expensive validation.
- Completed the experimental preparation checklist with extracted-package checks, native regressions, exact-source macOS CI and independent AI review.

### Scope and known gaps

- HOBBY-01, HOBBY-02 and HOBBY-03 are complete.
- PLAT-01 (strict Linux x86_64 qualification), PLAT-05 (strict Windows x86_64 qualification) and DOCS-09 (full parity-bearing release audit) remain unchecked and deferred.
- Strict qualification remains **not release-ready**. Declared Rust 1.92 minimum verification is still required before publication; current GUI operation was not validated in this wrap-up.
- Three older debug records are retained as historical bookkeeping, with their later evidence classified in Phase 15 completion. No missing evidence was converted to success.
- The July audit is preserved unchanged in the archive; the current hobby archive assessment explains the scope change separately.

### Statistics and provenance

- Timeline: 2026-07-09 to 2026-09-16 in Git author dates; archive date 2026-09-17 UTC.
- Git range: `0eff86971475665b95247a82a42b3ef55a917456` through `9469ca70487e0919061d752158e017ff96ed7e20`; 1033 repository commits at completion.
- 1836 net changed paths over that range; 883 tracked first-party Rust files with 259639 physical lines, including tests and private tooling (not production-only LOC).
- Checked implementation: `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`; Cargo CI run `35169132504` passed. Later commits record results and planning history.

### Archives

- [Roadmap](milestones/v1.0-ROADMAP.md)
- [Requirements and outcomes](milestones/v1.0-REQUIREMENTS.md)
- [Historical strict audit](milestones/v1.0-MILESTONE-AUDIT.md)
- [Current hobby archive assessment](milestones/v1.0-HOBBY-ARCHIVE-AUDIT.md)
- [Completion state](milestones/v1.0-STATE.md)

**Next:** No new milestone scope chosen. Use `/gsd-new-milestone` when ready; define fresh requirements and continue phase numbering after 15. Phase directories remain in place for stable historical references. No Git version tag was created: archive completion is not release-tag authorization.
