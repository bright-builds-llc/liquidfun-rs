---
phase: 21-playground-leftover-cleanup
verified: 2026-09-20T20:44:47Z
status: passed
score: 6/6 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-09-20T19-45-21
generated_at: 2026-09-20T20:44:47Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 21: Playground leftover cleanup Verification Report

**Phase Goal:** Remove dead playground chrome and proof leftovers, and bring the three oversized WASM scene files under the Bright Builds file-length gate without changing scene behavior.
**Verified:** 2026-09-20T20:44:47Z
**Status:** passed
**Re-verification:** No — initial verification

Provenance: `21-CONTEXT.md`, plans `21-01` through `21-04`, matching SUMMARYs, and this report share `lifecycle_mode: yolo` and `phase_lifecycle_id: 21-2026-09-20T19-45-21`. No `direct-fallback` markers.

Must-haves are the four ROADMAP success criteria plus Plan 04 product-smoke and independent-review truths. Plan 01–03 frontmatter truths restate those criteria (loader deletion, unknown-only fallback, named-file lengths plus `upstream_cli` split) and are folded into the evidence below. They did not reduce roadmap scope. Empty plan `requirements: []` arrays are correct: Phase 21 is leftover cleanup with no milestone requirement reassignment.

Locked decisions D-01 through D-11 in `21-CONTEXT.md` are honored. Passing `just web-player-smoke` is supporting evidence, not the independent-review acknowledgment.

## Goal Achievement

Dead Dam Break `loadProofSession` is gone. The player still constructs WASM only through `loadSceneSession(sceneId)` returning generated `ProofSession`. `rust-wasm-proof.spec.ts` stays opt-in behind `PHASE16_CLOSURE_ATTEMPT_DIR` and is omitted from `test:player`. `FallbackPanel` is a no-prop unknown-only panel; empty hashes still replace to Dam Break. Named WASM scene parents remain 346 / 360 / 426 lines and were not edited in this phase. Bright Builds `file-lengths` and `all` exit 0 after splitting `upstream_cli` as `foo.rs` plus `foo/`. Dam Break headless speed versus pinned C++ stays out of v1.1. Independent AI review is bound to digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`.

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1   | `loadProofSession` is removed or used; the opt-in `rust-wasm-proof.spec.ts` is either folded into ordinary documented smoke or kept as an explicit opt-in with no unused helper. | ✓ VERIFIED | `web/src/physics/loader.ts` exports only `loadSceneSession(sceneId)` and returns `new ProofSession(sceneId)` after `init`. `rg -n loadProofSession web/src web/tests web/e2e README.md TESTING.md` is empty. `App.tsx` imports and awaits `loadSceneSession(id)` inside `startScene`. `web/e2e/rust-wasm-proof.spec.ts` still `test.skip`s when `PHASE16_CLOSURE_ATTEMPT_DIR === undefined`. `web/package.json` `test:player` is the four-file Chromium allowlist and does not name `e2e/rust-wasm-proof.spec.ts`. D-01/D-02. |
| 2   | Dead `FallbackPanel` empty/not-ready branches are removed or made reachable from a real unknown/empty hash path. | ✓ VERIFIED | `FallbackPanel` is `export function FallbackPanel()` with no props. Heading `Scene not found`, locked body, and `href="#/scene/dam-break"` Open Dam Break. `web/src` has zero `FallbackKind`, `FallbackPanelProps`, `fallbackProps`, `EMPTY_HEADING`, `NOT_READY_BODY`, `Choose a scene`, `is not ready yet`, `innerHTML`. App mounts `fallback={<FallbackPanel />}`. Unknown hashes never call `loadSceneSession` (`onHashChange` `abandonScene`s when `maybeReadySceneId` is undefined). Empty hashes still parse `kind: "empty"` then `normalizeSceneRoute` replaces to `{ id: "dam-break" }` plus `maybeReplacementHash: "#/scene/dam-break"`. All six `SCENES` are `ready: true`. Playwright asserts Scene not found, locked body, absent Choose a scene, then Open Dam Break. D-04/D-05/D-06. |
| 3   | `color_mixer.rs`, `dam_break.rs` and `water_wheel.rs` satisfy Bright Builds `file-lengths` without changing public scene behavior, controls, or particle recipes. | ✓ VERIFIED | Live `wc -l`: color_mixer 346, dam_break 360, water_wheel 426 (all ≤ 628). `git log 71d2ef5^..HEAD --` those three paths is empty, so Phase 21 did not retune recipes, controls, gravity, joints, pointer magnitudes, or catalog/WASM ids (D-07/D-08). `bun scripts/bright-builds-check.ts file-lengths` prints `findings=0`; named scene files are absent from findings. Remaining checker debt was `tools/xtask/tests/upstream_cli.rs`; it is now a 225-line parent plus `verify.rs` / `configure.rs` / `build.rs` / `failures.rs`, no `mod.rs`, no `.bright-builds-rules-checks.tsv`. `bun scripts/bright-builds-check.ts all` prints `findings=0`. D-09. |
| 4   | Playground Dam Break headless speed versus pinned C++ remains out of this phase and out of v1.1 definition of done. | ✓ VERIFIED | ROADMAP SC 4 and CONTEXT deferred list still exclude C++ timing. Phase 21 did not edit `docs/playground-dam-break-timing.md`. `build_accepts_the_registered_playground_dam_break_bench` remains a fake-cmake registration test (`upstream build --preset oracle-release --target playground-dam-break-bench`). `just playground-dam-break-bench` was not used as a gate. No later v1.1 phase claims this work. D-09/D-10. |
| 5   | `just web-player-smoke` remains the product Chromium gate on leftover-cleanup HEAD, with `PHASE16_CLOSURE_ATTEMPT_DIR` unset and `rust-wasm-proof.spec.ts` omitted from `test:player`; `just web-smoke` is documented as historical Phase 16 forensic chrome, not the v1.1 product gate. | ✓ VERIFIED | `justfile` `web-player-smoke` is `bun scripts/web-build.ts player-smoke`, which runs `bun run test:player`. `test:player` lists only `e2e/player.spec.ts e2e/demo-media-clock.spec.ts e2e/shell.spec.ts e2e/reset-honesty.spec.ts`. README and TESTING both contain `historical Phase 16 forensic chrome`, `not the v1.1 product gate`, `not expected to pass against current Play/Pause/Reset chrome`, Dispose-session, PNG-hash, `PHASE16_CLOSURE_ATTEMPT_DIR`, and `Ordinary playground proof is just web-player-smoke`. Plan 04 recorded `just web-player-smoke` exit 0 with 37 Chromium passes on implementation HEAD `8dbbe52`. `web/test-results/.last-run.json` is `"status": "passed"` and `"failedTests": []`. This verifier did not re-run Playwright. Passing smoke is not the review acknowledgment. D-02/D-03/D-10. |
| 6   | A separate identified AI reviewer acknowledges the Phase 21 leftover-cleanup diff at an exact SHA-256 digest; the implementing executor does not approve its own work; no crate/npm publish, release tag, or new GitHub Pages URL. | ✓ VERIFIED | `21-REVIEW.md` binds digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07` to reviewer `85a64940-8a7f-4074-8c53-7bf520cedcc7` at `2026-09-20T20:38:33Z`, `reviewer_disclosure: AI reviewer, not a human`, `implementing_or_fixing_executor: no`, decision **APPROVED**. Reviewer is not implementing executor `d37e67b3-b2d4-425b-ae9e-6026475fd36b`. This verifier concatenated the 14 listed files in 21-04 order and recomputed SHA-256; the digest matches live bytes. `git tag --contains HEAD` is empty. `web/package.json` stays `"private": true`. `.github/workflows/pages.yml` has no Playwright job. Review grants no publication or new Pages URL. D-11. |

**Score:** 6/6 truths verified

### Required Artifacts

gsd-tools `verify artifacts` passed 12/12 planned paths across plans 21-01 through 21-04.

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `web/src/physics/loader.ts` | `loadSceneSession` only | ✓ VERIFIED | 10 lines. Exports `loadSceneSession`. Returns `new ProofSession(sceneId)`. No alias. |
| `README.md` | Historical `just web-smoke` wording | ✓ VERIFIED | Locked phrases present. Ordinary proof is `just web-player-smoke`. No `loadProofSession`. |
| `TESTING.md` | Historical `just web-smoke` plus Phase 16 forensic section | ✓ VERIFIED | Same locked phrases. Clean-checkout list still includes opt-in `just web-smoke` as required by D-03. |
| `web/src/components/FallbackPanel.tsx` | No-prop unknown-only fallback | ✓ VERIFIED | 17 lines. Fixed text nodes. Hash-only Open Dam Break. No `maybeRaw`. |
| `web/src/App.tsx` | `Show fallback={<FallbackPanel />}` after `normalizeSceneRoute` | ✓ VERIFIED | 601 lines (≤ 628). Init and hashchange still `replaceState` empty hashes. `routeIdentity` keeps `"empty"` for exhaustiveness and does not render empty chrome. |
| `web/e2e/player.spec.ts` | Unknown-hash Scene not found plus Open Dam Break | ✓ VERIFIED | Existing test strengthened with locked body and Choose a scene count 0. Uses `UNKNOWN_SCENE_PATH`. |
| `tools/xtask/tests/upstream_cli.rs` | Fixtures, FakeTools, helpers, child mods | ✓ VERIFIED | 225 lines. `#[path = "upstream_cli/{verify,configure,build,failures}.rs"]`. `CARGO_BIN_EXE_xtask` plus the same fake-tool env keys. |
| `tools/xtask/tests/upstream_cli/verify.rs` | `verify_*` tests | ✓ VERIFIED | 90 lines. Five former names present. |
| `tools/xtask/tests/upstream_cli/configure.rs` | `configure_*` tests | ✓ VERIFIED | 133 lines. Six former names present. |
| `tools/xtask/tests/upstream_cli/build.rs` | Build/registration tests including playground-dam-break-bench | ✓ VERIFIED | 174 lines. `build_accepts_the_registered_playground_dam_break_bench` unchanged argv. |
| `tools/xtask/tests/upstream_cli/failures.rs` | LF gitattributes and clang tests | ✓ VERIFIED | 37 lines. Both former names present. |
| `21-REVIEW.md` | Independent exact-digest acknowledgment | ✓ VERIFIED | AI reviewer `85a64940-8a7f-4074-8c53-7bf520cedcc7`. Digest matches live bytes. |

### Key Link Verification

gsd-tools `verify key-links` auto-verified 8/9. The FallbackPanel Show-fallback pattern failed because YAML escaped the JSX braces (`fallback=\\{<FallbackPanel />\\}`). Live `App.tsx` line 567 is `fallback={<FallbackPanel />}`. Manual grep confirms wiring.

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `web/src/App.tsx` | `web/src/physics/loader.ts` | `loadSceneSession` import and await | WIRED | Tool pass. `import { loadSceneSession }` and `await loadSceneSession(id)` in `startScene`. Ready ids only. |
| `web/package.json` | `web/e2e/rust-wasm-proof.spec.ts` | `test:player` omit | WIRED | Tool matched `test:player`. Manual: forensic spec is absent from the four-file allowlist; `test:browser` remains whole-tree `playwright test`. |
| `web/src/App.tsx` | `web/src/components/FallbackPanel.tsx` | `Show fallback={<FallbackPanel />}` | WIRED | Tool false negative (escaped regex). Import is `FallbackPanel` only; mount has no props. |
| `web/src/App.tsx` | `web/src/routing/hash.ts` | `normalizeSceneRoute` on init and hashchange | WIRED | Tool pass. Both call sites `replaceState` when `maybeReplacementHash` is set. |
| `web/e2e/player.spec.ts` | `web/e2e/player-helpers.ts` | `UNKNOWN_SCENE_PATH` `/liquidfun-rs/#/scene/not-a-scene` | WIRED | Tool pass. Helper export and `page.goto`. |
| `tools/xtask/tests/upstream_cli.rs` | `tools/xtask/tests/upstream_cli/build.rs` | `mod build` plus playground-dam-break-bench | WIRED | Tool pass. `#[path]` plus `mod build;`. |
| `tools/xtask/tests/upstream_cli.rs` | `CARGO_BIN_EXE_xtask` | `RepositoryFixture::command` | WIRED | Tool pass. Env keys `LIQUIDFUN_XTASK_{GIT,CMAKE,NINJA,CXX}` unchanged. |
| `21-REVIEW.md` | digest | Separate reviewer identity and SHA-256 | WIRED | Tool pass. 64-hex digest plus reviewer id. |
| `justfile` | `scripts/web-build.ts` | `web-player-smoke` → `player-smoke` | WIRED | Tool pass. Recipe is `bun scripts/web-build.ts player-smoke`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `FallbackPanel.tsx` | Locked heading/body/CTA | D-05 contract constants | Yes — fixed visitor copy by design, not `maybeRaw` | ✓ FLOWING |
| `App.tsx` Show fallback | `maybeCurrentSceneId()` | Live hash → `normalizeSceneRoute` → `maybeReadySceneId` | Yes — unknown `#/scene/not-a-scene` is undefined and mounts FallbackPanel; ready ids start WASM | ✓ FLOWING |
| `loader.ts` | `sceneId` | `startScene` from ready catalog ids | Yes — `new ProofSession(sceneId)` after WASM init | ✓ FLOWING |
| `21-REVIEW.md` | `digest` | Concatenated listed-file bytes | Yes — independently recomputed `3f4e1620…` | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| No `loadProofSession` in product/docs | `rg -n loadProofSession web/src web/tests web/e2e README.md TESTING.md` | empty | ✓ PASS |
| Named scenes and `upstream_cli` under 628; checker clean | `wc -l` named files; `bun scripts/bright-builds-check.ts file-lengths`; `bun scripts/bright-builds-check.ts all` | 346/360/426; both checkers `findings=0` | ✓ PASS |
| Review digest still matches live bytes | `cat` 14 listed paths `\| shasum -a 256` | `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07` | ✓ PASS |
| Forensic spec omitted from product smoke | `rg e2e/rust-wasm-proof.spec.ts web/package.json` | empty; `test:player` four-file list | ✓ PASS |
| Re-run `just web-player-smoke` | skipped | Already recorded exit 0 / 37 passed on `8dbbe52`; last-run.json `passed`; keep verification fast | ? SKIP |

### Requirements Coverage

Phase 21 plans declare `requirements: []`. ROADMAP: "none — leftover cleanup with no milestone requirement reassignment." REQUIREMENTS.md maps all 22 IDs to Phases 16–20. `grep "Phase 21" .planning/REQUIREMENTS.md` is empty. No orphaned WEB/WASM IDs.

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| _(none)_ | 21-01..21-04 | No milestone requirement reassignment | ✓ SATISFIED | Empty arrays are the contract, not a gap. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `web/tests/hash.test.ts` | 65 | Test title `parses a known not-ready id as a scene` | ℹ️ Info | Fountain is `ready: true`. This is parser coverage, not FallbackPanel `not-ready` chrome. Chesterton's fence: keep. Does not revive dead fallback copy. |
| `TESTING.md` | 122 | Clean-checkout still lists `just web-smoke` | ℹ️ Info | Required by D-03: forensic invocation stays documented as opt-in historical chrome, not the product gate. |

No blocker stubs. `web/src` has no TODO/FIXME/PLACEHOLDER, no `innerHTML`, and no empty/not-ready FallbackPanel branches.

### Human Verification Required

None. Unknown-hash copy is locked and asserted in Chromium `player.spec.ts`. Named scene modules were not edited, so public scene behavior needs no visual re-judgment. Independent AI review already exists in `21-REVIEW.md` and is distinct from smoke. Hobby scope plus D-10/D-11 do not require Firefox/Safari, Pages redeploy, or a human reviewer.

### Gaps Summary

No actionable gaps. Phase 21 is the last v1.1 phase; Dam Break C++ timing is an explicit out-of-DoD success criterion, not a deferred later-phase item.

Documented Plan 03 deviation (`#[path]` child modules instead of implicit `mod verify;`) is required for Cargo integration-test crate roots and matches `inventory_cli`. Layout remains `foo.rs` plus `foo/` with no `mod.rs`. Not a goal failure.

---

_Verified: 2026-09-20T20:44:47Z_
_Verifier: Claude (gsd-verifier)_
