---
phase: 42-opt-in-sound
plan: "05"
subsystem: puzzle-sound-verification
tags: [playwright, e2e, sound, puzzle, phase-gate, fingerprints, determinism]
requires:
  - "42-01 sound bridge scalars and target/phase42/{phase-base.txt,base.txt}"
  - "42-02 pure sound core and SOUND_STORAGE_KEY"
  - "42-03 sound engine shell, puzzle-sound glue and SOUND_CONTROL_SELECTOR"
  - "42-04 HUD toggle (Sound), drawer section (Sound effects) and form-factor row"
provides:
  - "web/e2e/sound-helpers.ts: installFakeAudio, installUnavailableAudio, storeSoundPreference, readStoredSound, readAudioLog, AudioLog"
  - "web/e2e/puzzle-sound.spec.ts: 14 Chromium tests for D-13 and the UI-SPEC naming and fixed-box rules"
  - "test:puzzle runs puzzle-sound.spec.ts"
  - "target/phase42/checks-05.{sh,results,log}: 32 phase gates, all exit=0"
  - "target/phase42/after-final.txt: 25 post-phase fingerprints, identical to base.txt"
affects:
  - "/gsd-verify-work for Phase 42: automated criteria 1-4 are evidenced; listening and iPhone UAT remain manual"
tech-stack:
  added: []
  patterns:
    - "Recording fake AudioContext installed by addInitScript, read back through window.__liquidfunAudioLog"
    - "Storage seeding guarded by a sessionStorage marker so reloads observe the app's own writes"
key-files:
  created:
    - web/e2e/sound-helpers.ts
    - web/e2e/puzzle-sound.spec.ts
  modified:
    - web/package.json
decisions:
  - "42-05: the sound on/off run comparison plays both pages concurrently in separate browser contexts at 15 Hz and 4x; both reach outcomeStep 201, decidedStep 262, peakHeld 811, heldHash 0cf8c2703dcd8fac, with 16 source starts on the sound-on page"
  - "42-05: the fixed-box and drawer-mirror checks are one test per viewport (loop), so each viewport fails independently"
  - "42-05: Prettier is not part of this repo's toolchain (no config; existing e2e files do not match its defaults), so the JS pre-commit format step was not applied; typecheck, build, Bright Builds and Playwright were"
metrics:
  duration: 20min
  completed: 2026-10-13
  tasks: 2
  files: 3
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: "2026-10-13T20:07:43.952Z"
---

# Phase 42 Plan 05: Fake-audio browser proofs and final phase gates Summary

Phase 42 is now proven in a real browser and gated on the final tree. A recording fake `AudioContext` shows that no context exists before opt-in or, with a stored "on", before a trusted gesture. It also shows that the choice survives a reload in both directions, that the context suspends when the tab is hidden or the player leaves the puzzle, and that the first-pour solution reaches an identical run fingerprint with sound on and off. All 32 phase gates passed on the first full run. The 25 scene fingerprints are identical to the pre-phase baseline.

## Performance

- **Duration:** about 20 min
- **Started:** 2026-10-13T19:47:49Z
- **Completed:** 2026-10-13T20:07:43Z
- **Tasks:** 2 (1 source commit; Task 2's evidence lives in gitignored `target/phase42/`)
- **Files:** 3 (2 created, 1 modified)

## Browser proofs (`web/e2e/puzzle-sound.spec.ts`, 14 tests, 12 `test(` sites)

| Test | Proves |
| --- | --- |
| keeps sound off and builds no audio context by default | Button `aria-pressed="false"`; after 60+ run steps `constructed 0`, `started 0`, nothing stored (D-04) |
| waits for a trusted gesture before building the context for a stored on | Stored "on" shows pressed with `constructed 0`; a `page.mouse.click` on the level title gives `constructed 1`, last state `running` (D-03, D-04, D-17) |
| creates one context from the toggle and suspends it when turned off | On: constructed 1, running, stored "on". Off: last state suspended, constructed still 1, `closed 0`, stored "off" (D-04) |
| keeps sound on across a reload | Reload keeps pressed, stored "on", and builds no context before a gesture (D-03) |
| keeps sound off across a reload after turning it off | Seeded "on", turned off, reload: not pressed, stored "off" (D-03) |
| suspends while the tab is hidden and resumes when it is visible | `setDocumentHidden(true)` gives suspended; `false` gives running; one context (D-10) |
| suspends when the player leaves the puzzle and shows no toggle on scenes | `#/puzzles` gives suspended with `closed 0`; `#/scene/dam-break` has no "Sound" button (D-01, D-10) |
| reaches the same run with sound on and off | Identical `RunFingerprint`; sound-on page `started 16`; sound-off page `constructed 0`, `started 0` (D-08, D-13) |
| disables both controls when Web Audio is unavailable and still plays | Disabled "Sound unavailable in this browser" button; disabled unchecked "Sound effects"; unavailable line; Play reaches running (D-05) |
| mirrors the toggle in the options drawer at 1280x800 / 390x844 | One exact "Sound" button with `title` = hint and `aria-describedby="puzzle-sound-help"`; checkbox drives the button and the button drives the checkbox; section shows the hint (D-02, D-11, D-18) |
| keeps every HUD box fixed when sound toggles at 390x844 / 844x390 | `readHudGeometry` obstacles identical before and after; no leaf or world-frame overlaps (D-01) |
| shows the checkbox and hint in the form-factor row without a HUD toggle | `.control-row` section has "Sound effects" and the hint; no "Sound" button (D-18) |

Run fingerprints (15 Hz synthetic clock, 4x, recorded solution through the test seam):

| Page | outcomeStep | decidedStep | peakHeld | heldHash | Audio |
| --- | --- | --- | --- | --- | --- |
| Sound on | 201 | 262 | 811 | `0cf8c2703dcd8fac` | constructed 1, started 16 |
| Sound off | 201 | 262 | 811 | `0cf8c2703dcd8fac` | constructed 0, started 0 |

Screenshots in `target/phase42/screens/`: `portrait-off.png`, `portrait-on.png`, `landscape-off.png`, `landscape-on.png` (building phase), `portrait-drawer.png` (drawer open), `form-factor.png`. I viewed `portrait-on.png` and `portrait-drawer.png`. They show the pressed speaker beside the options trigger, and the Sound section with the hint below Particles.

## Phase gates (`target/phase42/checks-05.results`)

All 32 named gates have `exit=0`, and `grep -c ' exit='` is 32. No attempt failed, so there are no preserved attempt files. Run window: 2026-10-13T19:51:36Z to 20:07:20Z, on HEAD `a29c88370`, against phase base `bd685b08f`.

| # | Gate | Result and evidence |
| --- | --- | --- |
| 1 | web-wasm | exit=0 |
| 2 | fingerprints | exit=0; scene-spot stamp `target/dam-break-perf/2026-10-13T19-52-13Z/`; **25/25 identical** (`diff base.txt after-final.txt` empty, 25 lines) |
| 3 | engine-untouched | exit=0 (`git diff --stat $BASE -- crates/liquidfun` empty) |
| 4 | puzzle-fixtures-unchanged | exit=0 (fixtures and `level-hashes.txt`) |
| 5 | scenes-untouched | exit=0 |
| 6 | protected-tests-unedited | exit=0 |
| 7 | other-preferences-unchanged | exit=0 |
| 8 | no-new-dependencies | exit=0 (lockfiles unchanged; only the `test:puzzle` line of `web/package.json` changed) |
| 9 | cargo-fmt | exit=0 |
| 10 | cargo-clippy | exit=0 |
| 11 | cargo-clippy-wasm | exit=0 |
| 12 | cargo-build-wasm | exit=0 |
| 13 | cargo-test | exit=0; 75 test binaries, 1084 passed, 0 failed |
| 14 | wasm-crate-tests | exit=0; 585 passed, 25 ignored |
| 15 | sound-tests | exit=0; 11 passed, 1 ignored (probe) |
| 16 | release-replay | exit=0; 34 passed |
| 17 | puzzle-replay-wasm | exit=0; 6 `LEVELHASH … pass`, no failure lines |
| 18 | typecheck | exit=0 |
| 19 | vitest | exit=0; 113 files, 1718 tests |
| 20 | bright-builds | exit=0; 0 findings |
| 21 | player-smoke | exit=0; 45 passed |
| 22 | hud-safe-frame | exit=0; 64 passed |
| 23 | puzzle-e2e | exit=0; 95 passed (81 before plus 14 sound) |
| 24 | markdown | exit=0; no non-GSD Markdown changed since base |
| 25 | grep-no-wall-clock | exit=0 |
| 26 | grep-storage-allowlist | exit=0 (`localStorage` only in CanvasStage, first-hint-preference, playground-session, sound-preference) |
| 27 | grep-no-innerhtml | exit=0 |
| 28 | grep-no-hud-obstacle-on-controls | exit=0 |
| 29 | grep-no-audio-session | exit=0 |
| 30 | grep-no-audio-libraries | exit=0 |
| 31 | grep-audio-context-confined | exit=0 (only `web/src/audio/sound-engine.ts`) |
| 32 | grep-sound-key-and-scope | exit=0 |

Before the full run, I ran the fast diff and grep gates once from a scratchpad copy of the script to catch quoting mistakes. Only the full run in `target/phase42/checks-05.results` counts as evidence.

## Decision coverage (D-01 to D-18)

| Decision | Plan | Files | Proof |
| --- | --- | --- | --- |
| D-01 HUD toggle, 44 px, `aria-pressed`, puzzle routes only | 04 | `PuzzleSoundToggle.tsx`, `PuzzleHud.tsx`, `puzzle.css` | `puzzle-chrome.test.ts` sound checks; e2e fixed-box tests at 390x844 and 844x390; e2e scene route has no "Sound"; gates hud-safe-frame, grep-no-hud-obstacle-on-controls, grep-sound-key-and-scope |
| D-02 drawer "Sound" section mirrors the toggle | 04 | `PuzzleSoundToggle.tsx` (`PuzzleSoundSection`), `PuzzlePanel.tsx` | e2e "mirrors the toggle in the options drawer" at two viewports |
| D-03 `liquidfun.puzzle-sound.v1`, exact `"on"`, blocked storage keeps memory | 02 | `sound-preference.ts` | `sound-preference.test.ts` (20); e2e reload tests in both directions; gates grep-sound-key-and-scope, grep-storage-allowlist, other-preferences-unchanged |
| D-04 no context before opt-in and gesture; off suspends; never closed | 03 | `sound-engine.ts`, `puzzle-sound.ts` | `sound-engine.test.ts`, `puzzle-sound.test.ts`; e2e default-off, stored-on gesture and toggle tests (`closed 0`); gate grep-audio-context-confined |
| D-05 unavailable or throwing context gives a disabled toggle and a working game | 03, 04 | `sound-engine.ts`, `puzzle-sound.ts`, `PuzzleSoundToggle.tsx` | `puzzle-sound.test.ts` construction failure; e2e "disables both controls when Web Audio is unavailable" |
| D-06 four synthesized voices, no audio library | 02, 03 | `voices.ts`, `sound-engine.ts` | `sound-voices.test.ts` (27), `sound-engine.test.ts`; gates grep-no-audio-libraries, no-new-dependencies |
| D-07 signals: plink from goalCount, chime from outcome, read-only splash and impact scalars | 01, 02 | `puzzle/sound.rs`, `frame/puzzle.rs`, `frame-sound.ts`, `sound-signals.ts`, `triggers.ts` | Rust `puzzle::sound` (11); `frame-sound.test.ts`, `sound-signals.test.ts`, `sound-triggers.test.ts`; gates engine-untouched, fingerprints |
| D-08 pure consumer after dispatch and draw; failure turns sound off | 03 | `puzzle-loop.ts`, `puzzle-run.ts`, `puzzle-sound.ts` | `puzzle-loop.test.ts` fold, `puzzle-sound-wiring.test.ts`; e2e identical sound on/off fingerprint |
| D-09 6-voice cap, per-kind intervals, chime steals, compressor and 0.5 master, run-only | 02, 03 | `voice-pool.ts`, `triggers.ts`, `sound-engine.ts` | `sound-voice-pool.test.ts` (16), `sound-triggers.test.ts` (24), `sound-engine.test.ts` |
| D-10 hidden and leave suspend; Stop/Edit/Retry stop voices; generation guard | 03 | `puzzle-sound.ts`, `puzzle-host.ts`, `puzzle-lifecycle.ts` | `puzzle-sound.test.ts`, `puzzle-sound-wiring.test.ts`; e2e hidden/visible and leaving tests |
| D-11 silent-switch hint in drawer and as toggle `title`; no `audioSession` | 04 | `PuzzleSoundToggle.tsx`, `PuzzlePanel.tsx` | `puzzle-chrome.test.ts` copy strings; e2e drawer test checks `title` and section text; gate grep-no-audio-session |
| D-12 read-only scalars; replays, determinism and 25 scene fingerprints unchanged | 01, 05 | `puzzle/sound.rs`, `session.rs` | Rust tally on/off bit-identity and determinism tests; gates release-replay, puzzle-replay-wasm, puzzle-fixtures-unchanged, fingerprints (25/25) |
| D-13 Playwright fake `AudioContext` proofs | 05 | `e2e/sound-helpers.ts`, `e2e/puzzle-sound.spec.ts` | the 14 tests above; gate puzzle-e2e |
| D-14 Vitest pure core with injectable context factory | 02, 03 | `web/tests/sound-*.test.ts`, `sound-fake-context.ts` | gate vitest (1718 tests) |
| D-15 burst-onset impact on levels without dynamic rigid bodies | 02 | `triggers.ts` | `sound-triggers.test.ts` burst rules (`BURST_QUIET_SECONDS` 0.4) |
| D-16 failure turns sound off in memory; preference not overwritten | 03 | `puzzle-sound.ts` | `puzzle-sound.test.ts` failure tests (stored value unchanged, `failed` set) |
| D-17 unlock on `pointerup`, `keydown`, mouse `pointerdown`; not touch `pointerdown` | 03, 05 | `puzzle-sound.ts` | `puzzle-sound.test.ts` gesture rules; e2e stored-on mouse unlock |
| D-18 form-factor control row; distinct names "Sound" and "Sound effects" | 04, 05 | `PuzzlePanel.tsx`, `PuzzleSoundToggle.tsx` | e2e form-factor test and drawer exact-name count |

## Manual UAT left for `/gsd-verify-work`

These need human ears or a device. Automation cannot judge them.

1. Listen to the splash, impact, plink and chime balance at 1x, 2x and 4x on First Pour (fluid only) and Float Up (rigid impacts). Check that bursts do not clip and that 4x does not become noise.
1. Confirm that the win chime plays once and that the settle tail is silent after it.
1. On an iPhone, with the ring/silent switch on (ring): turn sound on and play. Sounds should play.
1. On an iPhone, with the silent switch engaged: confirm that sound stays silent and that the hint "No sound on iPhone? Check the silent switch." is visible in the drawer and as the toggle title.
1. On an iPhone with a stored "on": after a reload, the first tap (touch `pointerup`) should start sound.

## Task Commits

1. **Task 1:** `a29c88370` (test): fake-audio helpers, `puzzle-sound.spec.ts` and the `test:puzzle` entry.
1. **Task 2:** no source commit. The gate script, results, log, `after-final.txt` and the replay and sound logs are gitignored evidence in `target/phase42/`, per the plan facts. No gate failed, so no `fix(42-05)` commit was needed.

## Deviations from Plan

### Interpretations

**1. One test per viewport for the drawer and fixed-box checks**

- The plan lists the drawer mirror at DESKTOP and the fixed-box check at two phone sizes. Each now loops over its viewports and creates one test per viewport. The drawer test also runs at 390x844 so it can take the required `portrait-drawer.png`. The spec has 14 tests and 12 `test(` call sites, which meets the "at least 11" criterion.

**2. `readStoredSound` helper**

- Added to `sound-helpers.ts` beside the four planned helpers. The reload and toggle tests use it to read the stored token without repeating the storage key in the spec.

**3. JS format step**

- The global pre-commit list includes a format step "if available". This repo has no Prettier config, and existing files do not match Prettier's defaults. Applying Prettier would have produced style drift, so I skipped it. Typecheck, build, Bright Builds (0 findings) and the Playwright suites ran.

No Rule 1-4 fixes were needed. The app behaved as specified in every browser check.

## Known Stubs

None.

## Threat Flags

None. The fake audio and storage seeding are init scripts under `web/e2e/` only. T-42-15 to T-42-19 are covered by gates 2-8 and 13-23, the sound on/off fingerprint test, gates 8 and 30, gate 31 with the zero-construction tests, and a single full run with 32 passes recorded in `checks-05.results`.

## Self-Check: PASSED

- `web/e2e/sound-helpers.ts` and `web/e2e/puzzle-sound.spec.ts` exist.
- Commit `a29c88370` is in `git log`.
- `target/phase42/checks-05.results` has 32 `exit=0` lines and 32 `exit=` lines. `after-final.txt` has 25 lines and matches `base.txt`. Six screenshots exist.
