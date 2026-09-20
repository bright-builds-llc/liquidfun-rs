---
phase: 21-playground-leftover-cleanup
plan: "04"
draft: task-1-smoke-evidence
local_head: 8dbbe52362917da122a8cc5e42e9df66b60a5f60
smoke_command: just web-player-smoke
smoke_exit: 0
smoke_tests_passed: 37
vite_git_sha: 8dbbe52362917da122a8cc5e42e9df66b60a5f60
vite_build_id: 2026-09-20T20:29:47.311Z
phase16_closure_attempt_dir: unset
# reviewer_identity, reviewed_at, reviewer_disclosure, implementing_or_fixing_executor,
# digest, and decision are left for Task 2 independent gsd-code-reviewer.
---

# Phase 21 smoke evidence (Task 1 draft)

This file is a Task 1 draft written by the implementing executor. It is **not**
an independent review acknowledgment. Passing `just web-player-smoke` is not
an acknowledgment. The implementing executor does not approve this work.

A separate identified `gsd-code-reviewer` must inspect the complete Phase 21
leftover-cleanup diff and this smoke evidence, independently recompute the
digest, and replace this draft with the bound acknowledgment.

## Smoke

Command (`PHASE16_CLOSURE_ATTEMPT_DIR` unset; confirmed empty in the shell
environment before and after the run):

```bash
unset PHASE16_CLOSURE_ATTEMPT_DIR
just web-player-smoke
```

Exit: `0`

Started: `2026-09-20T20:29:42Z`
Completed: `2026-09-20T20:32:49Z`

Playwright: `37 passed` (Chromium, 3.0m). Wrapper `bun scripts/web-build.ts
player-smoke` ran `bun run test:player`, which listed only
`e2e/player.spec.ts`, `e2e/demo-media-clock.spec.ts`, `e2e/shell.spec.ts`,
and `e2e/reset-honesty.spec.ts`. `e2e/rust-wasm-proof.spec.ts` was not
invoked.

Ignored log `target/web-build/web-build.log` starts with
`start player-smoke` and ends with `complete player-smoke`. Injected
`VITE_GIT_SHA=8dbbe52362917da122a8cc5e42e9df66b60a5f60` and
`VITE_BUILD_ID=2026-09-20T20:29:47.311Z`. The log does not mention
`PHASE16_CLOSURE_ATTEMPT_DIR`.

`web/test-results/.last-run.json` records `"status": "passed"` and
`"failedTests": []`.

Unit prelude: `vitest run` reported 183 passed / 22 files.

Allowlist check:

```bash
rg -n "e2e/player.spec.ts e2e/demo-media-clock.spec.ts e2e/shell.spec.ts e2e/reset-honesty.spec.ts" web/package.json
rg -n "e2e/rust-wasm-proof.spec.ts" web/package.json
```

The four-file `test:player` line matched. The forensic spec path is empty in
`web/package.json`.

Leftover helper check:

```bash
rg -n "loadProofSession" web/src web/tests web/e2e README.md TESTING.md
```

No matches.

Pages workflow:

```bash
rg -n "playwright" .github/workflows/pages.yml
```

No matches. No Playwright job was added. `git tag --contains HEAD` is empty.
No push, package publish, HOST-EVIDENCE, `just web-smoke`, or
`just playground-dam-break-bench`.

This draft grants no package publication, tag, release, or Pages deploy
authority. Do not claim Firefox or Safari coverage.
