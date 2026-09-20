---
phase: 21-playground-leftover-cleanup
plan: "04"
reviewed: 2026-09-20T20:38:33Z
independent_reviewed: 2026-09-20T20:38:33Z
reviewed_at: 2026-09-20T20:38:33Z
depth: deep
local_head: 6bf7ffc6128622aed4caffca1590ed658d36a2d5
phase_implementation_head: 8dbbe52362917da122a8cc5e42e9df66b60a5f60
origin_main_at_review: 4d008dca1b7017bacb7f65da5fb0a6364289a991
files_reviewed: 14
files_reviewed_list:
  - web/src/physics/loader.ts
  - README.md
  - TESTING.md
  - web/src/components/FallbackPanel.tsx
  - web/src/App.tsx
  - web/e2e/player.spec.ts
  - web/package.json
  - web/e2e/rust-wasm-proof.spec.ts
  - tools/xtask/tests/upstream_cli.rs
  - tools/xtask/tests/upstream_cli/verify.rs
  - tools/xtask/tests/upstream_cli/configure.rs
  - tools/xtask/tests/upstream_cli/build.rs
  - tools/xtask/tests/upstream_cli/failures.rs
  - .planning/phases/21-playground-leftover-cleanup/21-CONTEXT.md
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
decision: APPROVED
digest: 3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07
review_digest: 3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07
reviewer_identity: 85a64940-8a7f-4074-8c53-7bf520cedcc7
reviewer_invocation_id: f55e32a9-97cf-43da-80fe-9907a85f58a5
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
---

# Phase 21 Independent AI Review

## Decision

**APPROVED.**

There are no Critical, Warning, or Info findings. D-01 through D-11 leftover
honesty, the independently recomputed digest, and the inspected Chromium
player-smoke wrapper all match the Phase 21 leftover-cleanup contract. This
review does not mark requirement completion in planning state files and
grants no package publication, tag, release, or GitHub Pages deploy
authority. It does not claim a new GitHub Pages URL, crates.io publish, or
npm publish.

This is independent AI review, not human approval. Passing
`just web-player-smoke` is not this acknowledgment.

This document replaces the Task 1 smoke-evidence draft in
`21-REVIEW.md` that left reviewer identity and decision empty. That draft
was written by the implementing executor and is not a valid independent
acknowledgment. The implementing agent does not approve this digest.

## Reviewer and exact scope

- `reviewer_identity`: Cursor independent AI review subagent
  (`gsd-code-reviewer`), Task/agent-store identity
  `85a64940-8a7f-4074-8c53-7bf520cedcc7`
- `reviewer_invocation_id`: `f55e32a9-97cf-43da-80fe-9907a85f58a5`
- `reviewer_disclosure`: AI reviewer, not a human
- `model`: Cursor Grok 4.6
- `implementing_or_fixing_executor`: no
- `reviewed_at_utc`: `2026-09-20T20:38:33Z`
- `phase_implementation_head`: `8dbbe52362917da122a8cc5e42e9df66b60a5f60`
- `local_head_at_review`: `6bf7ffc6128622aed4caffca1590ed658d36a2d5`
- `origin/main_at_review`: `4d008dca1b7017bacb7f65da5fb0a6364289a991`

This reviewer is not implementing executor
`d37e67b3-b2d4-425b-ae9e-6026475fd36b` and is not conversation/subagent
`38b7d77e-cb5a-464c-abf8-7f683f9b7614`. This reviewer did not resume that
agent, its parent store, or any prior implementing or fixing executor.
The implementing agent does not acknowledge or approve this digest.

The reviewer independently inspected the complete relevant Phase 21
leftover-cleanup implementation (plans 21-01 through 21-03) ending at
`8dbbe52`, plus the Task 1 smoke-evidence commit `6bf7ffc` that does not
change the 14 hashed source bytes. Cross-file support inspected but not
hashed into the digest includes `web/src/routing/hash.ts`,
`web/e2e/shell.spec.ts`, `crates/liquidfun-wasm/src/scene/color_mixer.rs`,
`crates/liquidfun-wasm/src/scene/dam_break.rs`,
`crates/liquidfun-wasm/src/scene/water_wheel.rs`,
`.github/workflows/pages.yml`, `target/web-build/web-build.log`,
`web/test-results/.last-run.json`, `21-01-SUMMARY.md`, `21-02-SUMMARY.md`,
and `21-03-SUMMARY.md`. Material guidance was `AGENTS.md` Independent
review (2026-09-16 owner policy), `PROJECT-SCOPE.md` hobby scope,
`21-CONTEXT.md` D-01 through D-11, and `21-04-PLAN.md` threats
T-21-04-01 through T-21-04-06. Passing `just web-player-smoke` or other
automation is supporting context only and is not this acknowledgment.

## Findings

No Critical findings. No Warning findings. No Info findings.

## Required inspection outcomes

### Digest

The reviewer recomputed the digest independently from current file bytes
using this exact command (concatenated file bytes in this listed order,
then SHA-256):

```bash
cat \
  web/src/physics/loader.ts \
  README.md \
  TESTING.md \
  web/src/components/FallbackPanel.tsx \
  web/src/App.tsx \
  web/e2e/player.spec.ts \
  web/package.json \
  web/e2e/rust-wasm-proof.spec.ts \
  tools/xtask/tests/upstream_cli.rs \
  tools/xtask/tests/upstream_cli/verify.rs \
  tools/xtask/tests/upstream_cli/configure.rs \
  tools/xtask/tests/upstream_cli/build.rs \
  tools/xtask/tests/upstream_cli/failures.rs \
  .planning/phases/21-playground-leftover-cleanup/21-CONTEXT.md \
  | shasum -a 256
```

All 14 paths exist, are tracked, and were hashed. No path was skipped.
All 14 per-file SHA-256 hashes match the implementer-recorded values.
`git diff 8dbbe52 --` on those 14 paths is empty, so HEAD `6bf7ffc` source
bytes match phase implementation HEAD `8dbbe52`. The independently
recomputed digest is exactly:

`3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`

That value matches the implementer-computed digest. Per-file SHA-256
(supporting, not the digest):

- `web/src/physics/loader.ts` `c71f43bb00e8c83293bae734a3a581d621f4c28330f8e843d1864deb21979d2d`
- `README.md` `1fca44461bb7608f0b5711dffae4f9e16f4500addbef71b9b483b49ec497a240`
- `TESTING.md` `e0d89cc7168e6aa245dd864c4d29e6c19eb608eebeea0a5441136cadef5931d1`
- `web/src/components/FallbackPanel.tsx` `a921807ef05ddaf441ac6a51772bc85d5f2663211275b1e8e27b3aa7782255f6`
- `web/src/App.tsx` `5847b9b7cde20ee42c6d4a90e90f3b4928ad87692c26f033e9b0b86ba2f0ebbd`
- `web/e2e/player.spec.ts` `2e112adc5354825bc285d1f3f5f43a9f2ec0fb4d5eefe189ad4101a9a0ff2c95`
- `web/package.json` `d18baa1f372cae139f4ae1a13d4dac44f346acb752bca57a6921edb8568d2d87`
- `web/e2e/rust-wasm-proof.spec.ts` `122a25fc43a4a26e4c079ccc3ef8e2dd1d1f205bcadc904e972baf8aa044de77`
- `tools/xtask/tests/upstream_cli.rs` `05af11aba604a78265dd709ad94f70ab1bfd3feadc48d6ee186e9ceb2333e3c1`
- `tools/xtask/tests/upstream_cli/verify.rs` `560b0d08390f2786efd775cf3756e5edbb8863bc5b26e2dbe68178c5d9b2cba9`
- `tools/xtask/tests/upstream_cli/configure.rs` `a5deb863d302842f9813497a789030775decb7cfde9c29dea547b1f8455f8693`
- `tools/xtask/tests/upstream_cli/build.rs` `3afa06ee89dca764ad3a72dc6caf69da3ad8e4f8d13d1224d84ef002ee1f5bb0`
- `tools/xtask/tests/upstream_cli/failures.rs` `775f6563ed70b36caf75360e2cf3702c28da41ecb9d5762566761548167ffc23`
- `.planning/phases/21-playground-leftover-cleanup/21-CONTEXT.md` `8edda6b4b35e7e79c34c609829a1aec02e6629cfb65771c721a78bae4dc59c55`

Manifest entry count: `14`.

### D-01 unused proof helper

`web/src/physics/loader.ts` exports only `loadSceneSession(sceneId)`, which
initializes the generated package and returns `new ProofSession(sceneId)`.
There is no `loadProofSession` function, no compatibility alias, and no
rename of generated `ProofSession`. `App.tsx` constructs worlds only
through `await loadSceneSession(id)` inside `startScene`. `rg -n
"loadProofSession" web/src web/tests web/e2e README.md TESTING.md` is
empty.

### D-02 / D-03 forensic spec stays opt-in

`web/e2e/rust-wasm-proof.spec.ts` still `test.skip`s when
`PHASE16_CLOSURE_ATTEMPT_DIR` is undefined and still looks for
Dispose-session, PNG-hash, and closure-attempt artifacts. It is not on
`web/package.json` `test:player`. That allowlist remains
`e2e/player.spec.ts e2e/demo-media-clock.spec.ts e2e/shell.spec.ts
e2e/reset-honesty.spec.ts`. `rg -n "e2e/rust-wasm-proof.spec.ts"
web/package.json` is empty. This review does not cite
`rust-wasm-proof.spec.ts` as D-10 product evidence.

README.md and TESTING.md document `just web-smoke` as historical Phase 16
forensic chrome that allocates `target/phase16/closure-attempt-N`, sets
`PHASE16_CLOSURE_ATTEMPT_DIR`, and still looks for Dispose-session and
PNG-hash selectors. They state it is not the v1.1 product gate and is
not expected to pass against current Play/Pause/Reset chrome. Ordinary
playground proof is `just web-player-smoke`.

### D-04 / D-05 / D-06 FallbackPanel leftover honesty

`FallbackPanel` is a no-prop unknown-only component. Empty and not-ready
kinds, copy, `FallbackKind`, `FallbackPanelProps`, `fallbackCopy`, and
App `fallbackProps` are gone. `Show` mounts `fallback={<FallbackPanel />}`.
Unknown hash `#/scene/not-a-scene` still shows `Scene not found`, the
locked body, and hash-only `Open Dam Break`. Player smoke asserts that
`Choose a scene` is absent. All six catalog ids are `ready: true`.

`normalizeSceneRoute` still replaces parser `kind: "empty"` with Dam
Break and `maybeReplacementHash: "#/scene/dam-break"`. Parser kind empty
may exist internally (`maybeParseSceneRoute`, `routeIdentity`'s `"empty"`
branch) and does not render empty FallbackPanel chrome.

### D-07 / D-08 / D-09 file-lengths

Independent `wc -l` on this tree:

- `crates/liquidfun-wasm/src/scene/color_mixer.rs` 346
- `crates/liquidfun-wasm/src/scene/dam_break.rs` 360
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` 426

Each parent stays under the Bright Builds 628-line gate and already
declares `#[cfg(test)] mod tests;`. Named scene modules were left
unedited. `tools/xtask/tests/upstream_cli.rs` is a 225-line parent plus
`#[path = "upstream_cli/{verify,configure,build,failures}.rs"]` child
modules, matching `inventory_cli`, not a TSV exception. All 19 former
`#[test]` names remain. The playground Dam Break bench stays a
fake-cmake registration test; this review did not run
`just playground-dam-break-bench`.

This reviewer re-ran `bun scripts/bright-builds-check.ts file-lengths`
and observed `findings=0`. Plan 03 recorded `bun
scripts/bright-builds-check.ts all` exit 0. This reviewer did not run
`just web-smoke`.

### Player smoke (supporting context, not the acknowledgment)

The reviewer inspected `target/web-build/web-build.log`. It starts
`start player-smoke` and ends `complete player-smoke`. Injected
`VITE_GIT_SHA=8dbbe52362917da122a8cc5e42e9df66b60a5f60` and
`VITE_BUILD_ID=2026-09-20T20:29:47.311Z`. `PHASE16_CLOSURE_ATTEMPT_DIR`
is not referenced. `web/test-results/.last-run.json` records
`"status": "passed"` and `"failedTests": []`. The wrapper log does not
embed Playwright's per-test count; the Task 1 draft reported 37 Chromium
passes. `just web-player-smoke` was re-run on implementation HEAD
`8dbbe52` with `PHASE16_CLOSURE_ATTEMPT_DIR` unset. That automation
result is supporting context only and is not this acknowledgment.
`e2e/rust-wasm-proof.spec.ts` was not invoked.

### D-10 / D-11 Pages, publication, and hobby scope

Product proof is `just web-player-smoke`. This review claims no
Firefox/Safari/WebKit matrix, no screenshot-hash oracles, no Playwright
on Pages CI, no Linux qualification, and no Dam Break C++ timing.
`rg -n "playwright" .github/workflows/pages.yml` is empty. `git tag
--contains HEAD` is empty; this review creates no tag. `web/package.json`
stays `"private": true`. This review grants no crates.io publish, npm
publish, or new GitHub Pages URL. Hobby scope in `PROJECT-SCOPE.md`
applies. Publication remains unauthorized.

## Threat review

- **T-21-04-01 mitigated:** this separate AI reviewer
  `85a64940-8a7f-4074-8c53-7bf520cedcc7` binds approval to the exact
  digest below. The implementing executor
  `d37e67b3-b2d4-425b-ae9e-6026475fd36b` and conversation
  `38b7d77e-cb5a-464c-abf8-7f683f9b7614` did not write this
  acknowledgment.
- **T-21-04-02 mitigated:** player-smoke was re-run against implementation
  HEAD `8dbbe52` with `PHASE16_CLOSURE_ATTEMPT_DIR` unset; hashed source
  bytes at review HEAD `6bf7ffc` are identical.
- **T-21-04-03 mitigated:** this review does not treat `just web-smoke`
  as the product gate. Forensic smoke remains historical Phase 16 chrome.
- **T-21-04-04 mitigated:** this document discloses AI reviewer, not a
  human, and is not human approval.
- **T-21-04-05 accepted:** publication remains unauthorized; this review
  creates no tag, publishes no crate or npm package, and claims no new
  GitHub Pages URL.
- **T-21-04-06 mitigated:** `PHASE16_CLOSURE_ATTEMPT_DIR` was unset for
  player-smoke; `test:player` still omits `e2e/rust-wasm-proof.spec.ts`.

## Digest acknowledgment

`review_digest`:
`3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`

The independent result exactly matched all 14 current file entries and
the required digest.

> I, the Cursor independent AI review subagent identified above as
> Task/agent identity `85a64940-8a7f-4074-8c53-7bf520cedcc7`,
> independently inspected the listed Phase 21 leftover-cleanup
> implementation-and-evidence bytes and explicitly acknowledge exact
> digest
> `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`
> as the content I inspected and approve. I am an AI reviewer, not a
> human. I am not the implementing or fixing executor.

Any implementation, loader, fallback, player, docs, xtask-test, or
listed-context byte change invalidates this acknowledgment and requires
a new digest and independent review.

This review grants no package publication, tag, release, Pages deploy,
new GitHub Pages URL, crates.io publish, npm publish, or other release
authority. Passing `just web-player-smoke` alone is not this
acknowledgment.

## GSD Source Code Review

**Reviewed:** 2026-09-20T20:38:33Z
**Depth:** deep
**Files Reviewed:** 14
**Status:** clean

### Summary

Deep independent review confirmed deletion of unused `loadProofSession`
with `loadSceneSession(sceneId)` as the only constructor; unknown-only
no-prop `FallbackPanel` with locked Scene not found copy and Open Dam
Break; empty-hash Dam Break normalization retained; named WASM scene
parents under 628 lines; `upstream_cli` split as `foo.rs` plus `foo/`
with `#[path]` like `inventory_cli`; Chromium `test:player` omit of
`rust-wasm-proof.spec.ts`; historical `just web-smoke` wording; and no
Playwright job in `pages.yml`. The independently recomputed digest is
`3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`.
No source-code issues were found.

All reviewed files meet the Phase 21 leftover-cleanup quality bar for
D-01 through D-11. This is not human approval.

---

_Reviewed: 2026-09-20T20:38:33Z_
_Reviewer: Cursor AI (gsd-code-reviewer), Task/agent identity 85a64940-8a7f-4074-8c53-7bf520cedcc7_
_Depth: deep_

GSD_SOURCE_CODE_REVIEW_COMPLETE
