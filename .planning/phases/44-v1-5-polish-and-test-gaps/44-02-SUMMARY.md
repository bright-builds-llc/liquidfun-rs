---
phase: 44-v1-5-polish-and-test-gaps
plan: "02"
subsystem: web-e2e-typecheck
tags: [typescript, e2e, playwright, typecheck, tooling]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 44-2026-10-14T01-34-05
generated_at: "2026-10-14T02:03:49.134Z"
requirements-completed: []
requires:
  - "44-01 e2e edits (puzzle-levels.spec.ts, puzzle-editor-helpers.ts), which typecheck clean"
provides:
  - "strict e2e typecheck project web/tsconfig.e2e.json"
  - "bun run typecheck covering the app and e2e projects"
  - "e2e-only window.teslaBenchmark declaration"
affects: [44-03]
tech-stack:
  added: []
  patterns:
    - "second tsconfig project that extends the strict base and overrides only types/include"
    - "e2e-only ambient .d.ts typed from the real export via import type + typeof"
key-files:
  created:
    - web/tsconfig.e2e.json
    - web/e2e/tesla-benchmark.d.ts
  modified:
    - web/package.json
    - web/playwright.config.ts
    - web/e2e/player-helpers.ts
    - web/e2e/canvas-stage.spec.ts
    - web/e2e/water-helpers.ts
    - web/e2e/water.spec.ts
    - web/e2e/worker-quality-fixture.ts
decisions:
  - "44-02: web/tsconfig.e2e.json extends the strict base and overrides only types (node, vite/client) and include (e2e/**/*.ts, playwright.config.ts); bun run typecheck runs tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json"
  - "44-02: all 16 e2e type errors fixed at the root with no any/ts-ignore; window.teslaBenchmark for e2e is declared in e2e/tesla-benchmark.d.ts as typeof compareWorkerDirect"
metrics:
  duration: 3min
  completed: 2026-10-14
  tasks: 2
  files: 9
---

# Phase 44 Plan 02: E2E typecheck Summary

`bun run typecheck` now checks `web/playwright.config.ts` and `web/e2e/**` through a strict `web/tsconfig.e2e.json`. All 16 errors this exposed are fixed at the root. No compiler flag was loosened, and no escape hatch was added.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | 42af8afef | build(44-02): typecheck e2e code and the Playwright config |
| 2 | d15e553f5 | fix(44-02): type the e2e helpers and specs without escape hatches |

## The 16 errors and their fixes

| # | File:line | Cause | Fix |
|---|-----------|-------|-----|
| 1 | `playwright.config.ts:26` | `forensicConfig` was inferred as `reporter: "line"[][]` | Annotated as `Pick<PlaywrightTestConfig, "outputDir" \| "reporter">`. The `as const` casts were removed. |
| 2-4 | `e2e/player-helpers.ts:160,168,169` | the `evaluate` element was `HTMLElement \| SVGElement` | Added an `element instanceof HTMLCanvasElement` guard that throws, so `getContext`, `width` and `height` are typed. |
| 5-7 | `e2e/canvas-stage.spec.ts:237` | implicit `any` on the overloaded `addEventListener` override | Annotated the parameters as `(type: string, listener: EventListenerOrEventListenerObject, options?: boolean \| AddEventListenerOptions): void`. The existing `typeof listener !== "function"` check narrows the listener. |
| 8-13 | `e2e/live-worker.spec.ts:38,41,55,57,73,76` | `window.teslaBenchmark` was not declared in the e2e project | Added the new `e2e/tesla-benchmark.d.ts`, which declares `teslaBenchmark: { readonly compareWorkerDirect: typeof compareWorkerDirect }`. The spec itself is unchanged. |
| 14 | `e2e/water-helpers.ts:62` | `nativeGetExtension.call(this, name)` resolved to the last overload | The saved native method is typed as `(this: WebGL2RenderingContext, name: string) => unknown`. |
| 15 | `e2e/water.spec.ts:451` | `openPuzzle`'s default parameter gave it the `DESKTOP` literal type | Added `type Viewport` and annotated the parameter as `viewport: Viewport = DESKTOP`. |
| 16 | `e2e/worker-quality-fixture.ts:47` | the `httpServer` union includes `Http2SecureServer` | The fixture destructures `httpServer` and guards the call with `"closeAllConnections" in httpServer`. The preview server here is a plain `http.Server`, so the call still runs. |

The Plan 01 e2e code (`puzzle-levels.spec.ts`, `puzzle-editor-helpers.ts`) reported no errors.

## Decision coverage

- **D-09:** `web/tsconfig.e2e.json` extends `./tsconfig.json`. It overrides only `types` (`node`, `vite/client`) and `include` (`e2e/**/*.ts`, `playwright.config.ts`), so it inherits `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` and the unused-variable checks. The `typecheck` script is now `tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json`. The gate at `scripts/web-build.ts:292` runs `bun run typecheck`, so it covers e2e too. In `package.json`, only the `typecheck` line changed.
- **D-10:** All 16 errors are fixed at the root. `grep -rnE "@ts-ignore|@ts-expect-error|@ts-nocheck|: any\b|as any\b|<any>" web/e2e web/playwright.config.ts` prints nothing.
- **D-11:** No behavior changed. The touched specs pass, and the specs that use the changed helpers pass too (see Verification below).

## SC4 (typecheck half)

The command that proves it is `cd web && bun run typecheck`, which runs `tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json` over both projects. It **exits 0**. `npx tsc --noEmit -p tsconfig.e2e.json 2>&1 | grep -c "error TS"` prints 0. After Task 1, the same count was 9, which matches the plan's expectation.

## Verification

- `cd web && bun run typecheck` exits 0.
- `bun run test:unit`: 119 files and 1843 tests pass, both after Task 1 and after Task 2.
- `bun run build:app` exits 0.
- `CI=true bunx playwright test e2e/canvas-stage.spec.ts e2e/live-worker.spec.ts e2e/water.spec.ts`: **30 passed** (37.8 s).
- `CI=true bunx playwright test e2e/player.spec.ts e2e/rust-wasm-proof.spec.ts`: 14 passed and 1 skipped. I ran these as an extra check because `player.spec.ts` uses `canvasPixelSha256`. The skipped test is the pre-existing opt-in `rust-wasm-proof` spec, which skips unless `PHASE16_CLOSURE_ATTEMPT_DIR` is set and has its own local hash function.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet 9e6efcc23 -- web/tsconfig.json web/bun.lock web/e2e/hud-safe-frame.spec.ts web/e2e/shell.spec.ts crates/` exits 0. No dependency was added, and no protected file or Rust file changed.

## Deviations from Plan

None. The plan was executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: web/tsconfig.e2e.json, web/e2e/tesla-benchmark.d.ts, web/package.json, web/playwright.config.ts, web/e2e/player-helpers.ts, web/e2e/canvas-stage.spec.ts, web/e2e/water-helpers.ts, web/e2e/water.spec.ts, web/e2e/worker-quality-fixture.ts
- FOUND commits: 42af8afef, d15e553f5
