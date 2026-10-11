---
phase: quick-261011-n6v
plan: "01"
subsystem: web-tests
tags: [vitest, flaky-test, timeout, static-layer, canvas]
generated_by: gsd-executor
lifecycle_mode: quick
phase_lifecycle_id: quick-261011-n6v
generated_at: "2026-10-11T21:45:07Z"
requires: []
provides:
  - pixelMismatch helper and identicalPixels contract for exact full-canvas comparisons in static-layer tests
affects:
  - web/tests/static-layer.test.ts
tech-stack:
  added: []
  patterns: [loop-based buffer summary asserted with toEqual on a small object instead of typed-array deep equality]
key-files:
  created: []
  modified:
    - web/tests/static-layer.test.ts
decisions:
  - "Fixed the root cause (typed-array deep equality cost) instead of raising any vitest timeout; vitest.config.ts is unchanged"
  - "Exact equality is preserved: matching buffer lengths and zero differing bytes. A failure now reports the differing-byte count and the first differing index instead of a 1.2M-element dump"
  - "The two former maxDifference loops now use the same helper, so all five comparisons share one contract"
metrics:
  duration: 2min
  completed: 2026-10-11
  tasks: 2
  files: 1
---

# Quick 261011-n6v: Fix flaky static-layer test timeout in Pages CI Summary

Vitest deep equality over two 1,228,800-byte `ImageData.data` buffers took about 1 s per test. In Pages run 38177252614, with 60 test files running in parallel, `stable wall foreground > applies current native transforms and stroke styles to reused paths` hit the 5000 ms default timeout. All five full-canvas comparisons now go through a loop-based `pixelMismatch` helper. Every comparison still requires exact equality, and the three formerly slow tests went from about 1000 ms to 4-10 ms.

## Tasks

| Task | Name | Commits | Files |
| ---- | ---- | ------- | ----- |
| 1 | Replace all five image comparisons with a fast pixelMismatch helper | 4e23b88cb | web/tests/static-layer.test.ts |
| 2 | Run the full web and repository checks, then commit atomically | 4e23b88cb | web/tests/static-layer.test.ts |

Both tasks touch the same single file and land in one atomic commit, as the plan specifies.

### Task 1

- I added `pixelMismatch(actual, expected)` after `factory()`. It returns `{ actualLength, expectedLength, differingBytes, firstDifferingIndex }`. I also added the `identicalPixels` constant: lengths 640 * 480 * 4, 0 differing bytes, first index -1.
- Three `expect(...data).toEqual(...data)` sites and two `maxDifference` loops now use `expect(pixelMismatch(actual, expected)).toEqual(identicalPixels)`. Exactly 5 call sites remain.
- Test names, the Arrange/Act/Assert comments, and the other assertions (`toHaveBeenCalledTimes`, `mock.calls.at(-1)`) are unchanged.
- **Tamper check (not committed):** I temporarily asserted the first test against `expected.slice()` with `tampered[1234] ^= 1`. The test failed (1 failed, 7 passed), and the diff showed `"differingBytes": 1` and `"firstDifferingIndex": 1234`. I restored the file from a scratchpad copy; `grep -c tampered` returns 0 and `git diff` shows only the intended edits.

### Per-test timings (`bunx vitest run tests/static-layer.test.ts --reporter=verbose`, local macOS)

| Test | Before | After |
| --- | --- | --- |
| applies current native transforms and stroke styles to reused paths | 1050 ms | 10 ms |
| detects camera values changed on the same camera object | 986 ms | 4 ms |
| invalidates stroke/mode changes and reset even when the geometry is identical | 1 ms | 1 ms |
| preserves DPR backing resolution and keeps current circle labels outside the cached layer | 19 ms | 6 ms |
| invalidates a lost/restored context and never shares a foreground across surfaces | 1 ms | 1 ms |
| reuses stable copied geometry and preserves overlapping stroke pixels | 8 ms | 6 ms |
| keeps moving rigid geometry on the direct path | 3 ms | 2 ms |
| invalidates an in-place geometry change and a camera change | 1045 ms | 5 ms |
| File duration | 3.45 s | 242 ms |

In a full-suite JSON-reporter run after the change, the slowest test in the whole web unit suite took 92 ms (`demo-media-artifacts`). The slowest static-layer test took 9 ms.

## Verification

| Check | Result |
| --- | --- |
| `bunx vitest run tests/static-layer.test.ts --reporter=verbose` (before change) | 8 passed; 3 tests at 986-1050 ms |
| Tamper run (temporary, reverted) | failed as intended with differingBytes 1, firstDifferingIndex 1234 |
| Task 1 automated verify (vitest verbose, no `maxDifference`/`.data).toEqual`, 5 helper call sites, `git diff --quiet -- web/vitest.config.ts`) | pass |
| `bun run typecheck` | pass (exit 0) |
| `bun run test:unit` (first run, pre-commit) | **FAIL (exit 1)**, cause not captured; see Deferred Issues |
| `bun run test:unit` (6 reruns pre-commit, including a JSON-reporter run) | pass every time, 60 files / 525 tests |
| `bun scripts/bright-builds-check.ts all` | pass, findings=0 |
| Task 2 automated verify after commit (typecheck, test:unit 60/525, bright-builds findings=0, Co-Authored-By trailer, HEAD touches only web/tests/static-layer.test.ts) | pass |
| Style review of `git diff` (2-space indent, double quotes, trailing commas, ~80-column wrapping) | pass; the one helper signature was wrapped by hand |

## Deviations from Plan

1. **[Style] Wrapped the helper signature.** The single-line `function pixelMismatch(actual: ArrayLike<number>, expected: ArrayLike<number>) {` from the plan is 82 columns. I wrapped it across lines in prettier style to match the file's ~80-column wrapping. Behavior is unchanged.

Otherwise the plan was executed exactly as written.

## Deferred Issues

- **One unexplained full-suite failure.** The first `bun run test:unit` run, made right after `bun run typecheck`, exited with code 1. I had piped the output through `tail -6`, so the failing test name was lost. The tail showed only vitest's transform-cache note and `error: script "test:unit" exited with code 1`. Seven later full-suite runs passed 60/60 files and 525/525 tests: six before the commit (one of them with the JSON reporter) and one after it. In the JSON-reporter run no test exceeded 92 ms. That run was not static-layer: by then all static-layer tests ran in 10 ms or less, and the static-layer file had passed alone just before. Another test in the suite may be intermittently flaky under local load. If Pages CI shows a unit failure in a file other than static-layer.test.ts, capture the full log and open a separate quick task.

## Known Stubs

None.

## Self-Check: PASSED

- File: web/tests/static-layer.test.ts present, contains `function pixelMismatch` and 5 `pixelMismatch(actual, expected)` call sites.
- Commit found: 4e23b88cb (touches only web/tests/static-layer.test.ts, ends with the Co-Authored-By trailer).
- web/vitest.config.ts unchanged.
