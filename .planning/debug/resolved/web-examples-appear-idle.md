---
status: resolved
trigger: "Investigate issue: web-examples-appear-idle — Recently implemented web-based examples of the LiquidFun Rust/WASM library appear to do nothing when opened in a browser."
created: 2026-09-18T21:56:00-05:00
updated: 2026-09-18T22:37:00-05:00
---

## Current Focus

hypothesis: resolved — fractional frame time is retained between callbacks
test: objective UAT independently reran the repository-native web smoke
expecting: satisfied — controlled 120 Hz step and canvas pixel progression passed
next_action: archive the resolved session and commit the scoped fix

## Symptoms

expected: Opening a ready example should visibly run/animate the selected LiquidFun physics scene in the browser.
actual: The user reports that nothing seems to be happening after opening it in the browser.
errors: No error message has been supplied; inspect browser console, runtime state, network loading, and DOM/canvas evidence.
reproduction: Use the repository's documented local web workflow and/or the deployed Pages URL if applicable, then open an example and observe whether simulation frames and canvas pixels change.
started: The examples were recently implemented; current project state says Phase 19 web interaction/browser verification just completed.

## Eliminated

- hypothesis: JS or WASM assets fail to load because of a local/deployed base-path or MIME problem
  evidence: Development, production preview, and deployed Pages all loaded WASM with HTTP 200 and application/wasm; production/deployed JavaScript also returned 200.
  timestamp: 2026-09-18T22:21:00-05:00
- hypothesis: reduced-motion preference intentionally pauses the scene
  evidence: Every controlled probe explicitly used reducedMotion no-preference and the DOM reported data-playback=playing.
  timestamp: 2026-09-18T22:21:00-05:00
- hypothesis: one scene's physics implementation is static or broken
  evidence: At controlled 120 Hz, all six scenes remained at step 1 with unchanged scene-specific canvas hashes while requestAnimationFrame callbacks continued.
  timestamp: 2026-09-18T22:24:00-05:00
- hypothesis: requestAnimationFrame is not firing
  evidence: The frozen 120 Hz probes recorded 37–79 callbacks while both the step index and canvas hash stayed unchanged.
  timestamp: 2026-09-18T22:24:00-05:00
## Evidence

- timestamp: 2026-09-18T22:11:00-05:00
  checked: Phase 19 browser smoke and hosted evidence
  found: The smoke asserts Playing and increasing data-step-index, but explicitly chose DOM attributes instead of PNG hashes; live Pages evidence sampled data-step-index=1 only and did not wait for a later step or compare canvas pixels.
  implication: Existing evidence proves assets and the first Rust/WASM frame load, but does not prove visibly continuing animation.
- timestamp: 2026-09-18T22:11:00-05:00
  checked: web/src/App.tsx scheduleFrame and web/src/physics/clock.ts acceptedStepCount
  found: Each animation callback overwrites maybeLastTimestamp before calling a stateless Math.floor(elapsedSeconds / (1/60)); when the result is zero, it schedules another frame without retaining elapsed remainder.
  implication: A steady requestAnimationFrame cadence faster than 60 Hz can produce zero accepted steps forever despite the UI remaining Playing.
- timestamp: 2026-09-18T22:11:00-05:00
  checked: common bug patterns and recent Phase 19 history
  found: The symptom matches Async/Timing; Phase 19 changed pointer and presentation paths but did not change the Phase 17/18 frame clock.
  implication: Recent interaction work is not itself evidence of causation; the older frame-budget design remains the primary falsifiable timing hypothesis.
- timestamp: 2026-09-18T22:18:00-05:00
  checked: production preview with controlled requestAnimationFrame timestamps
  found: At 120 Hz, JS and application/wasm returned 200 with no console or page errors, 74 callbacks ran, status stayed playing, but step remained 1 and the canvas SHA-256 remained ef34b269...590f1. At 17 ms cadence, 74 callbacks advanced step 1 to 74 and changed canvas SHA-256 to f730d984...ff2c. Native headless Chromium also advanced step 1 to 45 and changed pixels.
  implication: Controlled refresh cadence alone deterministically toggles the visible-idle symptom; loading, WASM initialization, scene selection, rendering, and browser execution are healthy.
- timestamp: 2026-09-18T22:21:00-05:00
  checked: Vite development mode and deployed GitHub Pages with the same controlled timing probe
  found: Both environments reproduced the exact 120 Hz freeze at step 1 with identical canvas pixels and no page errors; both advanced to step 74 with changed pixels at 17 ms cadence.
  implication: The defect is shared application timing logic, not generated assets, production bundling, GitHub Pages routing, MIME, or caching.
- timestamp: 2026-09-18T22:24:00-05:00
  checked: all six ready scene routes under controlled 120 Hz timestamps
  found: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel each stayed at step 1 and preserved its initial pixel hash across 37–38 additional animation callbacks.
  implication: The shared animation shell prevents every native scene from stepping on high-refresh timing.
- timestamp: 2026-09-18T22:25:00-05:00
  checked: clock unit coverage and git history
  found: clock.test.ts verifies isolated elapsed values but no accumulation across consecutive sub-step deltas. The pure stateless clock entered in 9c1921d; App.tsx began resetting the timestamp on every callback in 94a9eba.
  implication: Existing tests validate clamping but cannot detect discarded fractional frame time; the regression predates Phase 19 and Phase 19 evidence did not exercise high-refresh timing or pixel progression.
- timestamp: 2026-09-18T22:28:00-05:00
  checked: strict TDD red unit regression
  found: bun run test:unit -- tests/clock.test.ts failed only the new repeated-120-Hz case because accumulateStepTime did not exist; the five pre-existing clock tests passed.
  implication: The pure regression requires an explicit accumulation seam and was observed failing before production changes.
- timestamp: 2026-09-18T22:28:00-05:00
  checked: strict TDD red Chromium regression
  found: bun run test:player -- --grep "controlled 120 Hz" failed after 5 seconds with expected step > 1 but received 1; the same test also contains the required canvas SHA-256 change assertion.
  implication: The integration regression deterministically catches the original visible-idle behavior before the fix.
- timestamp: 2026-09-18T22:31:00-05:00
  checked: focused green unit and Chromium regressions
  found: The repeated-120-Hz unit case passed with all six clock tests, then the rebuilt production app passed the controlled-120-Hz browser test including step-index growth and canvas SHA-256 change.
  implication: The minimal accumulator corrects both the pure timing behavior and the visible browser integration that previously failed.
- timestamp: 2026-09-18T22:34:00-05:00
  checked: full web and repo-native verification
  found: bun run typecheck passed; 13 unit files with 120 tests passed; rebuilt full Chromium player suite passed 12/12 including pause/play/reset, hidden-tab max-four recovery, resize and all six scenes; just web-player-smoke rebuilt WASM and production assets and repeated the same passing suites.
  implication: The fix preserves the adjacent timing, interaction, lifecycle, and production-subpath behavior covered by the repository.
- timestamp: 2026-09-18T22:35:00-05:00
  checked: original controlled 120 Hz local production reproduction after the fix
  found: Across 73 controlled callbacks, data-step-index advanced from 1 to 36 and canvas SHA-256 changed from ef34b269...590f1 to 5cbec431...0129.
  implication: The exact previously frozen local behavior now advances simulation state and visible canvas pixels.
- timestamp: 2026-09-18T22:35:00-05:00
  checked: final diff hygiene and managed standards check
  found: git diff --check passed; App.tsx remains exactly 628 lines. Bright Builds reported only the three pre-existing deferred scene-module file-length findings (color_mixer.rs, dam_break.rs, water_wheel.rs), none touched by this fix.
  implication: The scoped diff is clean and introduces no new managed-check finding; the unrelated known file-length debt remains.
- timestamp: 2026-09-18T22:37:00-05:00
  checked: independent objective UAT confirmation
  found: Fresh just web-player-smoke passed typecheck, all 120 unit tests, production build, and all 12 Chromium player tests including controlled 120 Hz step and pixel progression. Bright Builds again reported only the same three untouched pre-existing file-length findings.
  implication: The original browser-idle behavior is confirmed fixed under the repository's Agent-Performed Simple UAT policy.
## Resolution

root_cause: web/src/App.tsx updates maybeLastTimestamp on every requestAnimationFrame callback before web/src/physics/clock.ts floors only that callback's elapsed time. Any cadence below the 1/60-second threshold, including 120 Hz at about 8.33 ms, returns zero forever because fractional elapsed time is discarded instead of accumulated. The UI remains Playing after the initial nextFrame call at step 1, but no later WASM advance or canvas redraw occurs.
fix: Added accumulateStepTime to retain fractional elapsed seconds while capping accumulated time at four ticks. App keeps the remainder between animation callbacks and clears it automatically whenever the timestamp anchor is reset. Added pure repeated-120-Hz and Chromium step-plus-pixel regressions.
verification: Red-green TDD recorded. Focused clock test passed 6/6; full web unit suite passed 120/120; typecheck and production build passed; full Chromium player suite passed 12/12; repo-native just web-player-smoke passed; original controlled 120 Hz probe advanced step 1 to 36 and changed canvas pixels. Independent objective UAT repeated just web-player-smoke with the same passing 120/120 unit and 12/12 Chromium results and confirmed the fix.
files_changed: [web/src/physics/clock.ts, web/src/App.tsx, web/tests/clock.test.ts, web/e2e/player-helpers.ts, web/e2e/player.spec.ts]
