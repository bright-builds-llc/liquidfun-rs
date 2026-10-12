---
quick_id: 261012-eaw
mode: quick
files_modified:
  - web/scripts/readme-svg/encode.ts
  - web/tests/readme-svg-webp.test.ts
---

# Quick Task 261012-eaw: Make the missing-encoder WebP test deterministic

## Diagnosis

`startAnimatedWebpEncoder` returns as soon as `spawn()` returns. Node reports a
missing executable (ENOENT) through the child's `error` event on a later tick.
A 512-byte frame fits the stdin buffer, so `stdin.write` returns `true` and
`writeFrame` can resolve before `spawnFailure` rejects. The test then fails on
`rejects.toThrow`. This is a production race: callers cannot rely on the first
frame write reporting a missing encoder.

## Task 1: Wait for the encoder process before accepting frames

- **files:** `web/scripts/readme-svg/encode.ts`
- **action:** After wiring the existing `error`/`close` listeners, await the
  child's `spawn` or `error` event. On `error`, remove the temp directory and
  reject with `ffmpegSpawnError` (for example, "ffmpeg is required ..."). Keep the
  `spawnFailure` race in `writeFrame`/`finish` for later process errors.
- **verify:** `cd web && bun run typecheck`
- **done:** `startAnimatedWebpEncoder` never resolves for a missing command.

## Task 2: Assert the deterministic contract

- **files:** `web/tests/readme-svg-webp.test.ts`
- **action:** Make the missing-encoder case assert that starting the encoder
  rejects with "ffmpeg is required". No frame write is reached, so nothing can
  hang. Keep the Arrange/Act/Assert comments.
- **verify:** run `bunx vitest run tests/readme-svg-webp.test.ts` 20 times;
  `cd web && bun run test:unit`; `bun scripts/bright-builds-check.ts all`
- **done:** 20 of 20 runs pass and the full unit suite and the checks pass.
