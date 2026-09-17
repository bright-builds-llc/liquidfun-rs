# Phase 16 Independent AI Review

## Reviewer and scope

- `reviewer_identity`: separate Cursor `generalPurpose` AI reviewer
- `reviewer_agent_id`: `a3d7afad-57cb-4bde-a887-ea0e53c9ebd4`
- `reviewer_status`: AI, not human
- `model`: inherited model, GPT-5.6 Sol
- `implementing_executor`: no
- `subagents_used`: no
- `reviewed_at_utc`: `2026-09-17T03:50:45Z`
- `base`: `82556bb048e8b8c18261d4e6186a927860404bff`
- `head`: `4a9978b8e0a53b3e0ac0161e03c33a2614bbf6c7`
- `selected_attempt`: `target/phase16/closure-attempt-8`

I independently reinspected the complete non-planning implementation diff from
the exact base through the exact head, not only the corrective commit. The
review covered all changed source, tests, scripts, documentation, manifests,
and locks; the generated declaration; all Phase 16 threat registers; and every
regular file in selected closure attempt 8. Passing automation was treated as
evidence rather than as acknowledgment.

The materially applied guidance was the repo-local hobby-project and
independent-review policy in `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, `PROJECT-SCOPE.md`,
`standards/core/frontend-ui.md`, `standards/core/testing.md`,
`standards/core/verification.md`, the Rust and TypeScript/JavaScript standards,
all four Phase 16 plan threat models, and `16-UI-SPEC.md`.

## Decision

**APPROVED.**

There are no unresolved substantive findings and zero unresolved high-severity
findings.

## Findings and resolutions

1. **F-16-R1 — Medium — resolved.** The earlier review found that Canvas
   backing-store resizing and redraw after viewport changes were absent. Exact
   head adds one `ResizeObserver`; its callback exits when `stopped`, obtains
   current CSS dimensions, calls `resizeCanvasBackingStore` to rebuild the
   backing store and camera at capped DPR, stores that camera for future RAF
   draws, and redraws `maybePreviousFrame` without advancing or reconstructing
   physics. `stopResources` sets `stopped` and disconnects the observer before
   cancelling RAF and disposing the session, so queued callbacks are guarded
   and no resize callback can draw after failure, disposal, or cleanup. The
   Chromium test changes viewport width, waits for a changed backing width, and
   checks both backing dimensions against rounded CSS dimensions times
   `min(devicePixelRatio, 2)`. Its later screenshots show valid post-resize
   Rust frames, and proof metadata records `resizeRedrewLastFrame: true`.
1. **F-16-R2 — Informational — resolved.** Attempt 8 browser provenance,
   browser proof, and closure summary all identify exact head
   `4a9978b8e0a53b3e0ac0161e03c33a2614bbf6c7`. Each records only unrelated
   `.planning/config.json` as dirty.
1. **F-16-R3 — Informational — accepted scope.** The aggregate-check stderr log
   notes local CMake and C++ versions differ from optional canonical
   strict-qualification versions. `PROJECT-SCOPE.md` makes those profiles
   optional for this browser phase, and no browser/native/package result relies
   on them.

## Required inspection outcomes

### UI-SPEC and lifecycle

The approved dark single-panel contract is implemented: exact fixed copy,
semantic heading order, atomic polite status, runtime identity and counts in
DOM text, Canvas role/name/fallback text, visible caption, native Dispose
button, four lifecycle states, exact palette and typography, spacing tokens,
16:9 viewport, 16px world inset, DPR cap, focus ring, narrow containment, and
no Phase 17–19 controls or chrome. Running is reached only after WASM
initialization, session construction, frame validation, and first draw.

The corrected resize path rebuilds camera and backing store, redraws the last
Rust frame, does not step physics, and is disconnected and guarded before
failure/disposal cleanup. Disposal still cancels RAF before idempotent session
release and retains the final rendered frame.

### Generated declarations and copied ownership

`web/src/generated/liquidfun-wasm/liquidfun_wasm.d.ts` declares
`ProofFrame.free(): void`, `ProofSession.free(): void`, four required
`Float32Array` getters, and the required `Uint8Array` color getter. Rust exports
fresh cloned boxed slices. The TypeScript boundary calls each bulk getter once,
validates types, strides, bounds, finite values, and positive radii, frees each
temporary frame in `finally`, poisons failed sessions, and disposes the session
at most once. No application raw-memory, buffer, pointer, zero-copy, or manual
WebAssembly instantiation path was found.

### Dependency and lock integrity

- The private `liquidfun-wasm` package is unpublished, version `0.0.0`, and
  depends one-way on `liquidfun` plus exact `wasm-bindgen = "=0.2.128"`.
- `Cargo.lock` resolves wasm-bindgen, macro support, and shared components at
  `0.2.128` with registry checksums.
- `web/package.json` directly pins Bun `1.4.2`, SolidJS `1.9.15`, Playwright
  `1.63.0`, TypeScript `7.0.2`, Vite `8.3.0`, vite-plugin-solid `2.11.14`,
  Vitest `5.0.1`, and `@types/node` `22.20.3`.
- `web/bun.lock` agrees with those direct versions and records package
  integrity hashes. Frozen install passed without lock drift.

### Native, default, and package isolation

The workspace still has exactly
`default-members = ["crates/liquidfun"]`. The retained native tree contains
only `bitflags` as a normal dependency. Cargo metadata identifies `liquidfun`
as the sole default member. Package verification built and tested 238 entries
outside the repository, and the package list contains no wrapper, web source,
generated JavaScript/declaration/WASM, browser, C++, protocol, differential,
benchmark, or testbed artifact.

All 15 selected closure commands report passed with exit code zero. I inspected
every stdout and stderr log, including empty successful logs, plus the smoke
log, smoke summary, closure summary, provenance, last-run record, Playwright
JSON report, and all passing attachments. The complete base-to-head range also
passes an independent `git diff --check`.

### Browser artifacts

- Initial PNG: SHA-256
  `a5b3d7cf64e8d4819742654906d4e8b6aedca6ce7778f87995a224bf9a02ae18`,
  26,384 bytes, 784 by 442 pixels.
- Moving PNG: SHA-256
  `3196f5288d30e7765970dfb0c8ebf7dbea03acd70abc41f540bfa78c44573401`,
  24,501 bytes, 784 by 442 pixels.
- Disposed PNG: SHA-256
  `d69b460a8a1f7925c7eece6947f710c9ac4415970f51c83193d60ddaea24ec4e`,
  23,659 bytes, 784 by 442 pixels.

Those independently recomputed values exactly match `browser-proof.json` and
`closure-summary.json`. Visual inspection shows clear initial-to-moving
particle settling/compression and rigid-circle movement while the basin remains
fixed. The disposed PNG preserves a valid later frame. The browser assertion
then verifies unchanged step count, moved count, pixel hash, and recaptured PNG
bytes after two animation opportunities.

The single passing Playwright result has no retry, unexpected, or flaky result
and lists exactly the four required attachments: `canvas-initial.png`,
`canvas-moving.png`, `canvas-disposed.png`, and `browser-proof.json`. Every
attachment is byte-identical to its corresponding retained browser file.

## Threat review

### Plan 16-04

- **T-16-04-01 — mitigated.** One step per RAF, Rust's 1–4 step bound,
  512/16/8 frame caps, DPR cap 2, no catch-up loop, bounded-count browser
  assertions, corrected resize backing-store/camera reconstruction, and
  last-frame redraw are present.
- **T-16-04-02 — mitigated.** Checked Rust geometry feeds a TypeScript parser
  that rejects incorrect arrays, strides, counts, non-finite values, and
  non-positive radii. Projection is pure and no JavaScript physics or silent
  geometry clamp exists.
- **T-16-04-03 — mitigated.** Temporary frames are freed in `finally`; failure
  poisons ownership; disposal is idempotent; RAF and ResizeObserver are stopped
  before session release; queued resize callbacks guard on `stopped`; and
  Chromium verifies post-disposal stability.
- **T-16-04-04 — mitigated.** Rendering consumes JavaScript-owned copied arrays
  with no raw memory path. All three successful PNGs are hashed, bound to
  observations, attached to the passing result, and included byte-for-byte in
  this review manifest.
- **T-16-04-05 — mitigated.** Exact dependency/tool pins and locks,
  package-pinned Chromium, regenerate-before-smoke, fresh attempt allocation,
  exact-head provenance, complete command evidence, package isolation, and
  separate digest review are present.

### Inherited Phase 16 registers

- T-16-01-01 through T-16-01-05 are mitigated by checked step/allocation
  bounds, finite geometry validation, opaque generated owners, copied boxed
  slices, and exact Rust/wasm-bindgen/wasm-pack inputs.
- T-16-02-01 through T-16-02-05 are mitigated by bounded frame parsing,
  typed-array and geometry rejection, exact frame/session free behavior,
  absence of a raw-memory adapter, and exact frozen Bun dependencies.
- T-16-03-01 through T-16-03-05 are mitigated by bounded RAF/backing-store
  behavior including corrected resize handling, checked geometry, terminal
  lifecycle cleanup, copied arrays, and current-checkout build integrity.

## Fixed review manifest

The manifest includes every non-planning tracked path changed from exact base
through exact head, the generated declaration, and every regular file
recursively beneath selected attempt 8. It excludes this active review,
planning summaries, final verification, mutable acknowledgments, and the
preserved rejected review outside attempt 8. Paths are sorted by UTF-8 byte
order and each entry records the lowercase SHA-256 of exact file bytes.

- `.gitignore` `51f4a6ba44050b3ce2116cd67fc80cce0e52fa2d2cc76778bf69dd4ecc17794d`
- `Cargo.lock` `6f766ea4d10c814aa00248ee652e9a32e4e1f686e924407ca243c43a36e198a8`
- `Cargo.toml` `edae19c9af2748c6414c5e0275765b839e1ee3409d525c43a6ec70d40e6c03bb`
- `README.md` `2d72992eb8d55f068e7ab099de3197f1b9ae23c6bbd9bfc19b72e141f17fd8cb`
- `TESTING.md` `3c17bfaa7bf62a0d5ed663df27c86503f038df8c9e36f3e9799e46642ab84b61`
- `crates/liquidfun-wasm/Cargo.toml` `579efbfcfee42eb7b34eed1eb30275a9c0648a8b3ad86a3bb61e1a00e9ed0e0b`
- `crates/liquidfun-wasm/src/frame.rs` `492215b8062be21781c2e8a8f3258b44f7f3f3fa7ba0feffd0740bd50eda0314`
- `crates/liquidfun-wasm/src/lib.rs` `c9c0193a0e91dd3fd855fce4aef070eea87393507b83691cb0c397af64e9facb`
- `crates/liquidfun-wasm/src/scene.rs` `d818fb687e7dc77beb38371f98238eedaca69408a51f297a8183373ad5fd795f`
- `crates/liquidfun-wasm/src/session.rs` `79880de4ad26b2f5ddf97820ec9dbc949c38fad988dd1e7e9315cc98612808b9`
- `justfile` `b2ffc9a0ecbaeb10738b4847dad918e7880453149ffaba6441c2d2bc1eaaefef`
- `scripts/phase16-closure.ts` `968a039235dd3fb3fd3ed40f359ba2d3f71600ae25ff1510555789238209691f`
- `scripts/web-build.ts` `82cb278bff20efde40bf84403e65148af992c50e8f7bf47573d1b6ff66502fce`
- `target/phase16/closure-attempt-8/browser/browser-proof.json` `3822b54f6af53773c5be406b3451c0a19b75fb5258dd2812de8e9c53fcb9d869`
- `target/phase16/closure-attempt-8/browser/canvas-disposed.png` `d69b460a8a1f7925c7eece6947f710c9ac4415970f51c83193d60ddaea24ec4e`
- `target/phase16/closure-attempt-8/browser/canvas-initial.png` `a5b3d7cf64e8d4819742654906d4e8b6aedca6ce7778f87995a224bf9a02ae18`
- `target/phase16/closure-attempt-8/browser/canvas-moving.png` `3196f5288d30e7765970dfb0c8ebf7dbea03acd70abc41f540bfa78c44573401`
- `target/phase16/closure-attempt-8/closure/closure-summary.json` `0f9aa08cd3eec384cbc94ed145335d77ed88d525fa53b5134ce3b5c4ceb36793`
- `target/phase16/closure-attempt-8/closure/logs/01-cargo-fmt.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/01-cargo-fmt.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/02-wasm-tests.stderr.log` `f99b3a9134c89617640b30aa3a8daf12fbaa5e52092bdb58e79501d8d5fd0795`
- `target/phase16/closure-attempt-8/closure/logs/02-wasm-tests.stdout.log` `e6f40434efd4229dc18f5c53d94408eeaf049d927148abc632452cde2d824f1b`
- `target/phase16/closure-attempt-8/closure/logs/03-wasm-clippy.stderr.log` `4cf86278c1e6d659943a46a3d7e3da429ac15be90f0f17742e24ef85228e7d3e`
- `target/phase16/closure-attempt-8/closure/logs/03-wasm-clippy.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/04-web-build.stderr.log` `ceda4bda28e49c11c56ffc40cab7123c68217b62beb0bd4fdb05d97739650f52`
- `target/phase16/closure-attempt-8/closure/logs/04-web-build.stdout.log` `64e9d970c16d17a8946dddaf61467e3c131afdef760405059f55e35a8e41ea6f`
- `target/phase16/closure-attempt-8/closure/logs/05-native-build.stderr.log` `ee1d3fffa2022e962e63e0dbe4d35c9c198450a20c29fc09030f635dbb38bfcd`
- `target/phase16/closure-attempt-8/closure/logs/05-native-build.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/06-native-tests.stderr.log` `f8d74f086e11045435b64b3e172e1cf1f04bc54d2e3238f1e995cb9db01bab77`
- `target/phase16/closure-attempt-8/closure/logs/06-native-tests.stdout.log` `95882a4131c61869cb1475eea1c949e9621f4a98b5d039cff15770a9d4a1fb62`
- `target/phase16/closure-attempt-8/closure/logs/07-default-build.stderr.log` `a41051563a4cb5fc1b1bcdd9aa0ba28ca80f7736d629d90cc138218756e8e3a9`
- `target/phase16/closure-attempt-8/closure/logs/07-default-build.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/08-package-verify.stderr.log` `0bcc943793be5fa825121d142991bc618ec69adfaf553cc9fca6a969edc0deda`
- `target/phase16/closure-attempt-8/closure/logs/08-package-verify.stdout.log` `19503d775e93305ae5c3f03e44c97596dcd82690cbd7f87e0ff71ac08c46c995`
- `target/phase16/closure-attempt-8/closure/logs/09-aggregate-check.stderr.log` `f11419fc293794269c1d02fd7d549d4dfe6ce3bc2ab37433c6f84b6eec7637fd`
- `target/phase16/closure-attempt-8/closure/logs/09-aggregate-check.stdout.log` `f23a351daf8b90e294340b6c0e9ba866e115415d05c305b00ac5fe420cd52ec3`
- `target/phase16/closure-attempt-8/closure/logs/10-markdown-check.stderr.log` `4a8a653d600b5972ad1cb85ebb120b2f46c2daf444ff8428715f817634b35417`
- `target/phase16/closure-attempt-8/closure/logs/10-markdown-check.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/11-bright-builds.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/11-bright-builds.stdout.log` `b785fba08cf58ec5ffd21d618a921979ba583b633f527f583f4acf1a672f8473`
- `target/phase16/closure-attempt-8/closure/logs/12-git-diff-check.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/12-git-diff-check.stdout.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/13-native-tree.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/13-native-tree.stdout.log` `78f6733a334de5ac024895cbac2f984ce7459b9308a8e80bbc988f4ba723d83b`
- `target/phase16/closure-attempt-8/closure/logs/14-cargo-metadata.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/14-cargo-metadata.stdout.log` `cad757dba8f0d3cbb3f39f3799aaa4b625ea4572c77f1948f49e28b2f50940a9`
- `target/phase16/closure-attempt-8/closure/logs/15-package-list.stderr.log` `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `target/phase16/closure-attempt-8/closure/logs/15-package-list.stdout.log` `4e8bb0ccb739e15e8b7f1e0e59493d6549209a939174c06bf65bf4c786034da1`
- `target/phase16/closure-attempt-8/playwright-output/.last-run.json` `91d1c43004802cd49950d78eb11c8fa7d05da8ffffe219a8b13b2f561bc00903`
- `target/phase16/closure-attempt-8/playwright-output/rust-wasm-proof-runs-Rust--74a7c--and-freezes-after-disposal-chromium/attachments/browser-proof-json-ecd108d523150e7611069fc214dab0c16fed4fee.json` `3822b54f6af53773c5be406b3451c0a19b75fb5258dd2812de8e9c53fcb9d869`
- `target/phase16/closure-attempt-8/playwright-output/rust-wasm-proof-runs-Rust--74a7c--and-freezes-after-disposal-chromium/attachments/canvas-disposed-png-a48e9188ee0f08b25eb73f290aa48775932bb3e7.png` `d69b460a8a1f7925c7eece6947f710c9ac4415970f51c83193d60ddaea24ec4e`
- `target/phase16/closure-attempt-8/playwright-output/rust-wasm-proof-runs-Rust--74a7c--and-freezes-after-disposal-chromium/attachments/canvas-initial-png-fbaac8e3a745692566f4c148a61dd3508cbef620.png` `a5b3d7cf64e8d4819742654906d4e8b6aedca6ce7778f87995a224bf9a02ae18`
- `target/phase16/closure-attempt-8/playwright-output/rust-wasm-proof-runs-Rust--74a7c--and-freezes-after-disposal-chromium/attachments/canvas-moving-png-69ac4362c04044c2726708a6f82db724916f07d1.png` `3196f5288d30e7765970dfb0c8ebf7dbea03acd70abc41f540bfa78c44573401`
- `target/phase16/closure-attempt-8/playwright-report.json` `c3ddf49424b03d0330d4f6b51b5cdd9215269c1d656d85b133d46617d39caf20`
- `target/phase16/closure-attempt-8/provenance.json` `6a2ab3b910f2bce1b636a6994b2bc8df6ca1f1558802035a62cf767acc552ca6`
- `target/phase16/closure-attempt-8/smoke-summary.json` `86c902f7b3d38ac033b9f2cf26ede3929eac39434c5904575e6306c1d3140d67`
- `target/phase16/closure-attempt-8/smoke.log` `f42d9cecfe1bfef52de49acfe9b54b771cd81d9fff2fb7b65a483d659bd70a50`
- `web/bun.lock` `cbc2431a0292c61a019e7cb3390dac7e9e59a0b14b99ce147dfe4f812bc97d55`
- `web/e2e/rust-wasm-proof.spec.ts` `842a068482f15df82a2a491f85a319602e00446cc396d91dcc3d5976f73b6335`
- `web/index.html` `c6bcfc865bcd2c7eabf983a9bda108bb5ea0a600c7863c558afda7001e751c60`
- `web/package.json` `a6745f096ee0d2680b117074bc214eb31fe1b2cdbdb867d965be040812d55ce6`
- `web/playwright.config.ts` `96a9793befab5a01a2cbdb83a79e721caf87b55661644ba1061032cd47929ce7`
- `web/src/App.tsx` `d93b9aee7d6c788a4949041498c1e380ab05182e7a6661aaef7e402cda020181`
- `web/src/app.css` `8602ef0829e176188034d4d5c1db16d276a660fe777518cef31aaa0890bbb9c8`
- `web/src/generated/liquidfun-wasm/liquidfun_wasm.d.ts` `73e975ae94be331b866b78464b67a0e43b0308b94a86531cd106d53386d23338`
- `web/src/main.tsx` `b205f0093603713318ba7e3a292253e7e70da6bd379113a21f394baa8fc806d0`
- `web/src/physics/frame.ts` `a7af4d1f21fbeae262123c9a5b79dd95fcdf8978d62a56833086ac4dda0b9da5`
- `web/src/physics/loader.ts` `6911ff191d0c0d34e12e6d51cf0f549f5a9aa8b026c026642c5f7d16bd287c9d`
- `web/src/physics/session.ts` `a9f3c0a974c541758d73900855b4b29ed652ece58693420bbf38b950a56fe483`
- `web/src/render/camera.ts` `058087807ad2650817171988c7c3d3b717b98afa2b59e2fed391de7ed133dedd`
- `web/src/render/canvas.ts` `e290267d8e091e0c6263dfe9de05c9a8c47e76c66c58d44d23198cfcd1a71cc4`
- `web/tests/camera.test.ts` `c98b5c79d0a5a4ad96d64ecd11c681002a457cf79bb29e665b68607f7771d98a`
- `web/tests/frame.test.ts` `634f86ca4f434b420243e728cf0f77dc665f54a6911ca86514a5c5506b0b84ca`
- `web/tests/session.test.ts` `3c1ca37b00091d3eefd91155cccd6affb9e29d135f511bb8becd48055867ecb7`
- `web/tsconfig.json` `c4712ac1c789d8a64ef82b56ad0d9d377ed03940cfc189f1e323e83dd8b5e24c`
- `web/vite.config.ts` `e24e9925fa71bcf88c8027ea4bfcd48d7a9ceabbace1aaac41949eb53d930d52`
- `web/vitest.config.ts` `73105c1f5a2ca3e53da4ac4e965c441533529505dcd53bc700d08c356e09f7df`

Manifest entry count: `77`.

## Digest acknowledgment

For each manifest entry in the order above, I concatenated its path UTF-8
bytes, one NUL byte, the lowercase SHA-256 text, and one LF byte, then hashed
the complete concatenation with SHA-256.

`review_digest`:
`c37418b1bc9c331fb915e68a038b255562ab6bebb40a997c1b0fc39abc988917`

I, the separate Cursor `generalPurpose` AI reviewer with agent ID
`a3d7afad-57cb-4bde-a887-ea0e53c9ebd4`, explicitly acknowledge that exact
same digest,
`c37418b1bc9c331fb915e68a038b255562ab6bebb40a997c1b0fc39abc988917`,
as the implementation-and-evidence content I independently inspected and
approve. There are no unresolved substantive findings and zero unresolved
high-severity findings.

Any implementation, generated declaration, or selected-evidence byte change
invalidates this acknowledgment and requires a new fixed manifest, digest, and
independent review.

This review grants no package publication, tag, release, deployment, or other
release authority.
