# Project Milestones: liquidfun-rs

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
