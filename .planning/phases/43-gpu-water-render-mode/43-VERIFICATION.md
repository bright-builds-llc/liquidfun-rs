---
phase: 43-gpu-water-render-mode
verified: 2026-10-14T01:08:06Z
status: passed
human_approved_at: "2026-10-14T01:09:57Z"
human_approved_by: "user (repository owner) via chat: approved, please push it"
score: 4/4 roadmap success criteria verified (31/31 plan must-have truths verified)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: 2026-10-14T01:08:06Z
lifecycle_validated: true
overrides_applied: 0
human_verification:
  - test: "Open Dam Break, Fountain and the First Pour puzzle on a real iPhone, choose Particles > Water and play (D-20)."
    expected: "Water draws, not the Shaded blob or canvas fallback. It shows tinted water with highlights and white foam on fast jets. The cup, meter, target line, tools and scale legend sit above the water. Half-float targets work, or the view falls back to Shaded blob without a blank frame."
    why_human: "This needs a real mobile GPU. Chromium/SwiftShader e2e cannot reproduce iOS WebGL2 precision or extension behavior. Non-blocking under the PROJECT-SCOPE hobby profile."
  - test: "On a desktop GPU, compare Water with Shaded blob on Dam Break, Fountain and a puzzle level."
    expected: "Refraction of the walls, specular highlights and Beer-Lambert tint can be seen and look good. Foam reads as white spray on fast fronts and jets, and slow pooled water stays clear and tinted."
    why_human: "Look quality is subjective, and D-19 forbids screenshot-diffing shader output. The SwiftShader screens only confirm that foam and overlays are present."
---

# Phase 43: GPU Water Render Mode Verification Report

**Phase Goal:** Players can view water with refraction, highlights and foam in scenes and puzzles. Fallbacks never leave a blank canvas.
**Verified:** 2026-10-14T01:08:06Z
**Status:** passed. Every automated check passes. The two device and look items were approved by the repository owner in chat ("approved, please push it", 2026-10-14T01:09:57Z).
**Re-verification:** No. This is the initial verification, run against the final tree at HEAD `66181d36e`, after the code review fixes `60ada549b..95576cf7d`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Players can choose a "Water" render mode that shows refraction, highlights and foam on fast water, in scenes and puzzles, with the cup, meter and tool overlays on top. | ✓ VERIFIED | `mode.ts` adds `"water"` as the last "Surface" option, maps it to `"webgl-water"` and keeps `DEFAULT_RENDER_MODE = "shaded-blob"`. `RenderModeOptions.tsx` feeds both pickers: `PlayerPanel`, plus the row and sheet in `PuzzleOptionsDrawer` and `PuzzlePanel`. `water-shade.frag.glsl` is `precision highp float`. It refracts the backdrop by the field gradient and adds Blinn-Phong spec, an `exp(-uAbsorption*a)` tint in each particle's color, and foam from the MRT speed attachment. `scene-surface.ts drawSceneFrame` draws tools, labels and the glass cup on the 2D canvas after `presentSceneFrame`, and `clearFrame` makes that canvas transparent when WebGL drew. These e2e tests pass: dam-break, Tesla Valve worker, Fountain, the puzzle (2D overlay hash identical to Shaded blob) and the phone run with cup, target line and meter. |
| 2 | When the device cannot run Water, the view falls back to Shaded blob and then to canvas. A lost context falls back, then recovers when the context is restored. | ✓ VERIFIED | `present-frame.ts` runs one chain per frame: `drawWater`, then `drawShadedBlob`, then the canvas `paintMetaball` (not covered, opaque `CANVAS_COLOR` fill). It returns the `DrawnSurface`, which is written to `data-drawn-surface`. `drawWater` returns false when there is no context, no float or half-float target, fewer than 2 draw buffers, no highp, a failed link or an incomplete MRT. In `field-pass.ts`, reset listeners are registered before the `watchParticleSurface` repaint listeners, so the repaint rebuilds the held frame. These e2e tests pass: lost context gives `canvas` and restore gives `webgl-water` (both paused and while building), the unavailable picker draws `webgl`, and no float targets draws `canvas` with opaque alpha 255. |
| 3 | Exports keep their existing styles while Water mode is selected. | ✓ VERIFIED | `svgRenderMode("water")` takes the `rigidRenderMode` path and returns `"solid"`, the same as shaded-blob. `svg-export-worker.ts` calls `captureFrame()` with no argument. `git diff` from the phase base to HEAD is empty for `readme-svg/plans.ts`, `demo-media/capture.ts` and `docs/assets/readme`. The e2e test shows the 1 s dam-break SVG is byte-identical in Water and Shaded blob. |
| 4 | The foam speed data is produced only while Water mode is active. All 25 scene fingerprints are unchanged, and puzzle outcomes match in every render mode. | ✓ VERIFIED | Rust `capture_frame_with_speeds(flag)` attaches the lane only on request, and `capture_frame()` passes `false`. All three owners read `wantsParticleSpeeds(mode, cachedWaterSupport())` at each capture: the scene session, the puzzle session and the worker `advance`/`capture` `speeds` flag. Re-run here: `just playground-scene-spot` gives 25/25 identical to `target/phase43/base.txt` (stamp `2026-10-14T01-07-10Z`). Rust on/off proofs show bit-identical positions and puzzle status. The e2e First Pour RunFingerprint is the same for Water and Shaded blob. Other modes never request or read the lane. |

**Score:** 4/4 roadmap criteria verified. All 31 plan must-have truths across plans 01 to 06 are verified.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/session/speeds.rs` | `particle_speeds`, moved `max_particle_speed` | ✓ VERIFIED | Both present. 12 speeds tests pass. |
| `crates/liquidfun-wasm/src/frame/speeds.rs` | `set_particle_speeds`, `particleSpeeds` getter | ✓ VERIFIED | Fail-closed checks for length, non-finite and negative values (IN-02 variants). |
| `web/src/physics/frame-speeds.ts` | `parseParticleSpeeds` | ✓ VERIFIED | Wired from `parseRenderFrame` through an optional spread. |
| `web/src/render/field-pass.ts` | Shared surface, half-float, loss/restore | ✓ VERIFIED | 390 lines. Accepts `EXT_color_buffer_float` or `EXT_color_buffer_half_float`. |
| `web/src/render/gl-program.ts` | `linkProgram`, `requireBuffer` | ✓ VERIFIED | Frees shaders on failure (IN-03). |
| `web/src/render/webgl-water.ts` | `drawWater` | ✓ VERIFIED | 393 lines. MRT `createFieldTarget(gl, 2)`, speed buffer or constant 0, backdrop upload gated by revision. |
| `web/src/render/water-backdrop.ts` | `createWaterBackdrop` | ✓ VERIFIED | Revision-gated raster of the stable walls. |
| `web/src/render/water-capability.ts` | `waterSupportFrom`, `probeWaterSupport`, `cachedWaterSupport` | ✓ VERIFIED | Probes a detached canvas, then calls `loseContext` in `finally`. |
| `web/src/render/water-shaders/water-shade.frag.glsl` | highp shade | ✓ VERIFIED | `precision highp float`. |
| `web/src/render/water-uniforms.ts` | Look constants | ✓ VERIFIED | Contains `WATER_FOAM_SPEED_LOW`. Background derives from `CANVAS_RGB` (IN-04). |
| `web/src/render/mode.ts` | water mode, label, `wantsParticleSpeeds`, `renderModeOptionView` | ✓ VERIFIED | Contains `WATER_UNAVAILABLE_LABEL = "Water (unavailable)"`. |
| `web/src/render/present-frame.ts` | Fallback chain returning `DrawnSurface` | ✓ VERIFIED | — |
| `web/src/components/RenderModeOptions.tsx` | Shared optgroups | ✓ VERIFIED | Used by `PlayerPanel` and `PuzzleOptionsDrawer`, which `PuzzlePanel` and `PlaygroundStage` render. |
| `web/e2e/water.spec.ts`, `web/e2e/water-helpers.ts` | D-19 proofs | ✓ VERIFIED | 13 tests. Re-run here: 13/13 passed. |
| `target/phase43/checks-06.results`, `after-final.txt` | Gate evidence | ✓ VERIFIED | 27 gates, all `exit=0`. `after-final.txt` has 25 lines. |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| `lib.rs ProofSession::capture_frame` | `capture_frame_with_speeds` | `maybe_particle_speeds.unwrap_or(false)` | ✓ WIRED |
| `frame.ts parseRenderFrame` | `parseParticleSpeeds` | optional spread | ✓ WIRED |
| `webgl-particles.ts drawShadedBlob` | `particleSurfaceFor` | shared surface and pass slot | ✓ WIRED |
| `webgl-water.ts` | `field-pass.ts` | `createFieldTarget(gl, 2)` | ✓ WIRED |
| `webgl-water.ts` | `frame.maybeParticleSpeeds` | `bindSpeeds` | ✓ WIRED |
| `water-capability.ts` | `linkWaterPrograms` | `programsLinked` fact | ✓ WIRED |
| `present-frame.ts` | `drawWater` | first link of the D-12 chain | ✓ WIRED |
| `scene-surface.ts drawSceneFrame` | `dataset.drawnSurface` / `speedLane` | return value | ✓ WIRED |
| `scene-surface.ts assignParticleSurface` | `watchParticleSurface` | held-frame repaint | ✓ WIRED |
| `App.tsx` | `cachedWaterSupport` | `waterAvailable` prop to both pickers | ✓ WIRED |
| `scene-lifecycle.ts` | `wantsParticleSpeeds` | accessor to `createSceneSession` and `createWorkerSession` | ✓ WIRED |
| `puzzle-host.ts` | `createPuzzleSession(generated, id, …)` | third-argument accessor | ✓ WIRED |
| `simulation-worker.ts sendFrame` | `session.captureFrame(speeds)` | `request.speeds` | ✓ WIRED |
| `worker-messages.ts frameTransferables` | lane buffer | pushes `maybeParticleSpeeds` | ✓ WIRED |
| `package.json test:water` | `e2e/water.spec.ts` | script | ✓ WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `webgl-water.ts` foam | `frame.maybeParticleSpeeds` | Rust `particle_speeds(view.velocities())`, then the wasm getter, then `parseParticleSpeeds` (and the worker transfer) | Yes. The e2e `data-speed-lane="present"` holds in scene, worker and puzzle runs, and the screens show white foam on spray. | ✓ FLOWING |
| Picker Water option | `waterAvailable` | `cachedWaterSupport()` probe in `App.tsx` | Yes. The e2e forced-unavailable case shows "Water (unavailable)" disabled. | ✓ FLOWING |
| `data-drawn-surface` | `presentSceneFrame` return | the actual draw result | Yes. All of `webgl-water`, `webgl` and `canvas` were seen in e2e. | ✓ FLOWING |

### Behavioral Spot-Checks (run by the verifier on HEAD `66181d36e`)

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| TS types | `cd web && npx tsc --noEmit` | exit 0 | ✓ PASS |
| Unit tests | `cd web && npx vitest run` | 1840/1840 passed | ✓ PASS |
| Wasm crate tests | `cargo test -p liquidfun-wasm --all-features` | 600 passed, 0 failed, 25 ignored; 12 `speeds` tests ok | ✓ PASS |
| 25/25 fingerprints | `just playground-scene-spot`, then diff with `target/phase43/base.txt` | 25 lines, identical | ✓ PASS |
| Wasm rebuild | `just web-wasm` | exit 0 | ✓ PASS |
| Puzzle replay (wasm) | `just puzzle-replay-wasm` | 6 `LEVELHASH … pass`, no fail | ✓ PASS |
| Release replay | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` | 34 passed | ✓ PASS |
| Browser proofs | `bun run build:app && CI=true bun run test:water` | 13/13 passed | ✓ PASS |
| Repo standards | `bun scripts/bright-builds-check.ts all` | findings=0 | ✓ PASS |
| Protected paths | `git diff --stat <phase-base>..HEAD` on `crates/liquidfun`, lockfiles and export pins | empty | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| WATER-01 | 01, 03, 04, 05, 06 | Players can choose a "Water" render mode with refraction, highlights and foam on fast-moving water | ✓ SATISFIED | Truth 1. The look quality itself is a human item. |
| WATER-02 | 02, 03, 04, 06 | Falls back to shaded blob, then canvas, on an incapable device or lost context; recovers on restore | ✓ SATISFIED | Truth 2 |
| WATER-03 | 01, 04, 05, 06 | Works in scenes and puzzles; exports keep their styles | ✓ SATISFIED | Truths 1 and 3 |

No requirements are orphaned. REQUIREMENTS.md maps only WATER-01, WATER-02 and WATER-03 to Phase 43, and plans claim all three.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `web/src/render/webgl-water.ts` | 38 | `PLACEHOLDER_TEXEL` | ℹ️ Info | A legitimate 1×1 initial backdrop texel set to the canvas color. It is not a stub. |
| `web/src/render/water-capability.ts` | 126-134 | The probe caches a transient `false` for the page lifetime (review IN-05, skipped on purpose) | ℹ️ Info | The picker can show "Water (unavailable)" until a reload after a transient probe failure. This follows the locked decisions UI-D48 and UI-D50. When availability is false, the lane is not requested, so the runtime chain still draws Water when a stored `water` preference exists, but without foam. This does not block the goal. |

There are no TODO or FIXME markers, and production Rust has no `unwrap()`.

### Human Verification Required

#### 1. Real iPhone viewing check (D-20, non-blocking)

**Test:** Open Dam Break, Fountain and the First Pour puzzle on an iPhone, then choose Particles > Water.
**Expected:** Water draws with tint, highlights and foam on fast jets. The overlays sit on top. If half-float or MRT is unsupported, the view falls back to Shaded blob or canvas without a blank frame.
**Why human:** This needs a real mobile GPU. SwiftShader cannot represent iOS WebGL2.

#### 2. Real-GPU look review

**Test:** On a desktop GPU, compare Water with Shaded blob on Dam Break, Fountain and a puzzle level.
**Expected:** Refraction, highlights and tint are visible and pleasant. Foam reads as white spray on fast water, and slow pools stay clear.
**Why human:** Look quality is subjective, and shader screenshot diffs are out of scope by D-19.

### Gaps Summary

No gaps were found. All four roadmap success criteria and all plan must-haves are verified against the final tree, including after the review fixes. The verifier independently re-ran tsc, vitest, the wasm crate tests, the 25/25 scene fingerprint gate, the wasm and release puzzle replays, the app build and the 13-test Water Playwright suite. The only remaining items are the D-20 device check and the subjective look review. Both are non-blocking under the hobby profile, the verifier recorded `human_needed`; the repository owner approved both items in chat on 2026-10-14T01:09:57Z, so the status is `passed`.

Lifecycle provenance is consistent. CONTEXT, all 6 PLANs and all 6 SUMMARYs carry `lifecycle_mode: yolo` and `phase_lifecycle_id: 43-2026-10-13T22-55-49`, and no artifact is marked `direct-fallback`.

---

_Verified: 2026-10-14T01:08:06Z_
_Verifier: Claude (gsd-verifier)_
