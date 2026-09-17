---
phase: 16-rust-wasm-browser-bridge
audited_at: 2026-09-17T04:57:48Z
status: secured
asvs_level: 1
block_on: high
security_enforcement: true
threats_total: 20
threats_closed: 20
threats_open: 0
unregistered_flags: 0
closure_attempt: target/phase16/closure-attempt-10
review_digest: 7b63ca2e7f1580a8a509e8265aca968667bc8443b9055651e285991a0cd3ff38
---

# Phase 16 Security Verification

## Scope and method

This audit verifies only the 20 threats declared in the four Phase 16
`<threat_model>` blocks. All have disposition `mitigate`; there are no accepted
or transferred risks to verify. Package publication, deployment,
authentication, backend, secret, and database risks are outside this phase.

The audit inspected the declared Rust, TypeScript, Canvas, build, closure, test,
browser-proof, review, and review-fix sources. Passing automation was used only
as supporting evidence after source inspection. Repository guidance materially
applied here was `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, `PROJECT-SCOPE.md`, and the architecture, code-shape,
testing, verification, frontend, Rust, and TypeScript standards.

## Complete threat register

| Threat ID | Category | Disposition | Status | Verification evidence |
| --- | --- | --- | --- | --- |
| T-16-01-01 | D | mitigate | CLOSED | Rust caps particles/segments/circles at 512/16/8, uses checked lane multiplication, accepts only 1–4 steps, and checks step-index addition before stepping (`frame.rs:5-7,46-64,79-96`; `session.rs:8,75-94`). Bounds and no-effect/overflow paths are exercised in `frame.rs:242-282` and `session.rs:234-265,314-326`. |
| T-16-01-02 | T/D | mitigate | CLOSED | Scene geometry uses fallible engine constructors (`scene.rs:47-184`); frame construction rejects non-finite values and non-positive radii without clamping (`frame.rs:61-66,100-123`). Focused rejection tests are at `frame.rs:304-355`. |
| T-16-01-03 | D | mitigate | CLOSED | `ProofSession` and `ProofFrame` are opaque wasm-bindgen owners with no public engine identities (`lib.rs:12-79`; `frame.rs:128-197`). Generated declarations include explicit `free()` for both classes (`liquidfun_wasm.d.ts:6-10,46-50`), while the TypeScript owner enforces exactly-once disposal. |
| T-16-01-04 | T/D | mitigate | CLOSED | All five Rust getters clone into `Box<[f32]>` or `Box<[u8]>` (`frame.rs:164-197`), and the crate forbids unsafe code (`lib.rs:3`). No adapter path retains or exports a raw pointer or live memory view. The copy-independence test mutates one returned lane and confirms a later getter is unchanged (`frame.rs:218-239`). |
| T-16-01-05 | T | mitigate | CLOSED | The wrapper pins `wasm-bindgen = "=0.2.128"` and is unpublished (`crates/liquidfun-wasm/Cargo.toml:1-19`); `Cargo.lock:4037-4084` resolves 0.2.128 and `rust-toolchain.toml` pins 1.97.0. `web-build.ts:13-16,119-170` checks exact Rust/wasm-pack/Bun versions and regenerates current-checkout output. |
| T-16-02-01 | D | mitigate | CLOSED | The TypeScript owner always calls `advance(1)` (`session.ts:42-54`). The parser caps 512/16/8 and checks safe integer lengths before iteration (`frame.ts:1-8,40-81,120-166`). Unit tests reject each count/stride boundary (`frame.test.ts:132-324`). |
| T-16-02-02 | T/D | mitigate | CLOSED | Runtime parsing requires exact typed-array constructors, lane lengths, finite floats, positive radii, and consistent rigid counts (`frame.ts:83-184`). Unit tests independently exercise wrong constructors, all five lane shapes, NaN/infinity, both radius classes, and rigid-count mismatch (`frame.test.ts:197-388`). |
| T-16-02-03 | D | mitigate | CLOSED | `createSceneSession` guards disposed state, frees every captured frame in `finally`, poisons and frees after failure, and makes `dispose()` idempotent (`session.ts:23-70`). Tests assert exact frame/session free counts, poisoning, idempotence, and use-after-dispose rejection (`session.test.ts:84-197`). |
| T-16-02-04 | T/D | mitigate | CLOSED | The parser returns generated boxed-slice copies and retains no Rust frame (`frame.ts:112-184`); the owner frees the wrapper immediately in `finally` (`session.ts:48-54`). No `.buffer`, pointer, manual instantiate, or `WebAssembly.Memory` use exists under `web/src/physics`. |
| T-16-02-05 | T | mitigate | CLOSED | `web/package.json:1-25` pins Bun and all seven direct packages exactly; `web/bun.lock:8-16` records the same pins. `web-build.ts:138-170` validates the deletion target, regenerates before frozen install/typecheck/tests/build, and `.gitignore` excludes generated/build outputs. |
| T-16-03-01 | D | mitigate | CLOSED | One RAF callback requests one frame and schedules only after completion (`App.tsx:301-339`); Rust remains bounded to one through four steps and frame caps remain enforced. Canvas DPR is capped at 2 (`canvas.ts:14,43-67`), and there is no catch-up loop or elapsed-time integration. |
| T-16-03-02 | T/D | mitigate | CLOSED | Rust checked geometry and the TypeScript parser gate all render data. Camera creation rejects non-finite or non-positive viewports (`camera.ts:33-49`), with focused invalid-viewport tests (`camera.test.ts:104-117`). Canvas projection/drawing consumes the validated frame and contains no physics integration. |
| T-16-03-03 | D | mitigate | CLOSED | Terminal cleanup disconnects resize effects, cancels RAF, then disposes once (`App.tsx:250-298,370-377,431`). Failure and disposal preserve the last observation; the session adapter frees temporary frames and poisons failures. No stopped path schedules another frame. |
| T-16-03-04 | T/D | mitigate | CLOSED | Canvas receives only `RenderFrame` typed-array copies (`canvas.ts:1,70-177`) after parser validation and frame-wrapper release. Source search found no raw Memory, buffer, pointer, or zero-copy adapter path. |
| T-16-03-05 | T | mitigate | CLOSED | Exact Cargo/Bun/tool pins and both lockfiles are present. `runCompleteBuild` verifies tools, regenerates WASM, then performs frozen install, typecheck, all unit tests, and Vite build in order (`web-build.ts:164-176`). Attempt 10 retains the successful `just web-build` result. |
| T-16-04-01 | D | mitigate | CLOSED | The inherited RAF, Rust, frame, and DPR bounds remain present. Chromium additionally asserted 192 particles and four rigid shapes (`rust-wasm-proof.spec.ts:199-215`; `browser-proof.json:45-46`) and retained a passing bounded run. |
| T-16-04-02 | T/D | mitigate | CLOSED | Checked Rust construction, strict TS parsing, camera validation, and terminal browser failure handling remain present. Chromium observed real Rust frame advancement and changed Canvas pixels (`browser-proof.json:51-52`); visual inspection of the retained initial and moving PNGs confirms visible movement. |
| T-16-04-03 | D | mitigate | CLOSED | App and adapter cleanup controls remain present. Chromium clicks the native disposal action, waits two animation opportunities, and asserts unchanged step count, movement count, pixel digest, and screenshot bytes (`rust-wasm-proof.spec.ts:279-312`; `browser-proof.json:54-55`). |
| T-16-04-04 | T/D | mitigate | CLOSED | Copied-array ownership remains source-verified. The browser test hashes three explicit PNGs and writes observation-bound metadata only after all assertions pass (`rust-wasm-proof.spec.ts:239-380`). Attempt-10 JSON and disk hashes match, all four attachments are byte-validated, and the exact bytes are included in the independent review manifest. |
| T-16-04-05 | T | mitigate | CLOSED | Exact tools, lockfiles, package-pinned Chromium, regenerate-before-smoke, unique `mkdir` attempt allocation, source identity, closure isolation, and digest review are implemented (`web-build.ts:179-199,214-283,410-451`; `source-identity.ts:122-178`; `closure-runner.ts:281-389`). Attempt 10 passed all 16 closure commands with no forbidden package entries, and the separate AI reviewer acknowledged the exact 85-entry digest with zero unresolved high-severity findings (`16-REVIEW.md:60-190,288-309`). |

## Cross-cutting control evidence

### Source and generated-artifact binding

- `captureSourceIdentity` hashes porcelain status, unstaged diff, staged diff,
  and sorted untracked path/content bytes (`source-identity.ts:122-178`).
- Closure compares current source to browser proof and provenance before checks,
  then compares it again after all commands (`closure-runner.ts:301-339`).
- Generated WASM output is confined to the canonical ignored path, removed only
  after path validation, and regenerated before every complete build
  (`web-build.ts:138-176`; `.gitignore`).
- Current hashes for the implementation, validators, generated declaration,
  browser proof, three PNGs, and closure summary match the fixed
  `16-REVIEW.md` manifest. No reviewed implementation path differs between
  candidate `80d4d7b54454eedf850da8f4e8c253a6d09e41d1` and current HEAD
  `12b18d8df7e71841c2911833813a1f8ec6ae1a65`; the pre-existing worktree change
  remains `.planning/config.json`.

### Evidence parsing, confinement, and immutability

- Browser proof parsing requires exact records, bounded JSON/report sizes,
  bounded integers/strings/dimensions, exact assertion/artifact sets, PNG
  signatures, dimensions, lengths, and hashes
  (`browser-evidence.ts:99-369,370-459`).
- Artifact and attachment paths are checked both lexically and after
  `realpath`; attachments require exact names, content types, and bytes
  (`browser-evidence.ts:392-416,463-577`).
- Smoke allocates the next unused attempt directory without deleting a prior
  attempt (`web-build.ts:179-199`). Closure logs use `wx`, and summary
  publication uses an identity-last hard link that cannot overwrite an
  existing record (`closure-runner.ts:70-85,88-125,251-279,344-360`).

### WR-01 through WR-03

| Review fix | Threat mapping | Closure evidence |
| --- | --- | --- |
| WR-01 — complete staged, unstaged, status, and untracked source identity | T-16-04-05 and inherited build-integrity threats | `source-identity.test.ts:49-87` rejects staged and untracked mutations; attempt 10 binds proof, provenance, and closure to working-tree digest `7c2270ade8e357c31bb1f987a7dd94f0317497614d817dad921ae511c5c29f1e`. |
| WR-02 — strict malformed-evidence and path-confinement validation | T-16-04-04 and T-16-04-05 | `browser-evidence.test.ts:188-260` rejects empty proof, traversal, missing files, and attachment-byte mismatch. |
| WR-03 — bounded immutable failure summaries | T-16-04-05 | `closure-runner.test.ts:70-134` verifies one bounded failure record and no overwrite on rerun. |

`16-REVIEW-FIX.md` records all three fixes as complete. The retained attempt-10
regression log reports 8 passed, 0 failed, and the independent reviewer
re-ran and inspected those regressions.

## Browser and review evidence

- Attempt 10 is source-bound to revision
  `80d4d7b54454eedf850da8f4e8c253a6d09e41d1` and working-tree digest
  `7c2270ade8e357c31bb1f987a7dd94f0317497614d817dad921ae511c5c29f1e`.
- `closure-summary.json` is `passed`; all 16 command entries are `passed`, and
  `forbiddenPackageEntries` is empty.
- `browser-proof.json` records package-pinned Playwright 1.63.0 / Chromium
  153.0.8010.12, 192 particles, four rigid shapes, real step/movement progress,
  changed pixels, resize redraw, stopped post-disposal frames, and preserved
  disposed pixels.
- The retained PNG SHA-256 values are:
  - initial: `cb8ba853b7c922eaac42e6b8ba1cb5271fb1f5ed094e254720adb9ba17c76775`
  - moving: `8cd1950f6c41feffc14bfc1ce1ec5f73f4ed6673161edb1a7d98b867a089445c`
  - disposed: `e0daf38cfb0ea0139d6a55c717b47f1ddfd6cb367964ef5e5dc01911351aee24`
- The separate identified GPT-5.6 Sol AI reviewer was not the implementing
  executor, inspected source and exact evidence, and explicitly acknowledged
  digest `7b63ca2e7f1580a8a509e8265aca968667bc8443b9055651e285991a0cd3ff38`
  at `2026-09-17T04:42:51Z`.

## Threat flags and audit trail

None of the four summaries contains a `## Threat Flags` section or an
unregistered threat flag. Their security-relevant deviations map to the
registered threats above; WR-01 through WR-03 are mapped explicitly and closed.

Audit-time verification:

- `cargo test -p liquidfun-wasm --lib` — 14 passed, 0 failed.
- `cd web && bun run test:unit` — 39 passed, 0 failed.
- SHA-256 recheck of the selected implementation and attempt-10 evidence —
  matched the fixed independent-review manifest.
- `git diff --name-only 80d4d7b...HEAD` over reviewed implementation paths —
  empty.
- A fresh `bun test scripts/phase16/*.test.ts` invocation exceeded the
  30-second foreground window and was stopped; it is not counted as a fresh
  pass. The source-matched retained attempt-10 run completed in 133.28 seconds
  with 8 passed and 0 failed, and the independent reviewer separately reran it
  successfully.

## Unregistered flags

None.

## SECURED

**Phase:** 16 — Rust WASM Browser Bridge  
**Threats Closed:** 20/20  
**Threats Open:** 0  
**ASVS Level:** 1

All declared mitigations are present, exercised, bound to retained current
evidence, and covered by an independent exact-digest review.
