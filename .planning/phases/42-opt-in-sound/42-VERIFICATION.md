---
phase: 42-opt-in-sound
verified: 2026-10-13T20:50:32Z
status: human_needed
score: 4/4 roadmap success criteria verified (30/30 plan must-have truths)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: 2026-10-13T20:50:32.810Z
lifecycle_validated: true
overrides_applied: 0
human_verification:
  - test: "Listen to First Pour (fluid only) and Float Up (rigid impacts) with sound on at 1x, 2x and 4x"
    expected: "Splash, impact and plink voices are audible and balanced; bursts do not clip; plinks audibly rise as the cup fills; 4x does not become a wall of noise"
    why_human: "Loudness, balance and clipping are subjective and need real audio output; automation only proves node graphs and parameters against a fake context"
  - test: "Win First Pour with sound on and listen through the settle tail"
    expected: "The three-note chime plays once at the win; no splash, impact or plink starts after it"
    why_human: "Unit tests prove the trigger rule; only a listener can confirm the audible result in a real browser"
  - test: "On an iPhone with the ring/silent switch on ring, turn sound on and play"
    expected: "Sounds play"
    why_human: "Requires a physical iOS device"
  - test: "On an iPhone with the silent switch engaged, turn sound on and play"
    expected: "No sound; the hint 'No sound on iPhone? Check the silent switch.' is visible in the Puzzle options drawer and is the toggle's title"
    why_human: "Silent-switch behaviour is a device and OS property"
  - test: "On an iPhone with a stored 'on', reload a puzzle and tap the canvas"
    expected: "The first tap (touch pointerup) starts sound"
    why_human: "Real touch user-activation semantics differ from Playwright's synthetic mouse and keyboard input"
---

# Phase 42: Opt-in Sound Verification Report

**Phase Goal:** Players who turn sound on hear the puzzle: splashes, impacts, plinks that rise as the cup fills, and a win chime.
**Verified:** 2026-10-13T20:50:32Z
**Status:** human_needed
**Re-verification:** No. This is the initial verification, run after the code-review fix pass (commits 6772b0980..8cbf2587c, report 003656428).

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Sound is off by default, and no audio starts until the player turns it on. The toggle is remembered across visits. | VERIFIED | `sound-preference.ts`: only the exact `"on"` under `liquidfun.puzzle-sound.v1` means on. `createPuzzleSound` never builds a context. `sound-engine.ts` calls the factory only from `ensureRunning`, which runs only from the toggle, the checkbox and the trusted-gesture handlers. The e2e tests "keeps sound off and builds no audio context by default" (`constructed 0`, `started 0`), "waits for a trusted gesture…stored on", "keeps sound on across a reload" and "keeps sound off across a reload…" pass in the post-fix `puzzle-e2e` gate (99 passed). |
| 2 | With sound on, the player hears splash and impact sounds, plinks that rise in pitch as the cup fills, and a win chime, with a cap on simultaneous sounds. | VERIFIED (automated); listening is in human UAT | `sound-engine.ts` builds a noise buffer, a bandpass filter (splash), a pitch-dropping triangle (impact), a sine blip (plink), a three-note sine chime, a 0.5 master gain and a `DynamicsCompressorNode`. `voices.ts` maps fill to an 11-step pentatonic scale from C5 ("never falls as the cup fills"). `voice-pool.ts` sets `MAX_VOICES = 6` with 80/60/50 ms intervals, and the chime evicts the oldest voices. `triggers.ts` plays the chime once on the first won frame and starts nothing after it. The e2e sound-on run started 14 sources, and the sound-off run started 0. Rust tests `first_pour_solution_splashes` and `float_up_solution_reports_a_rigid_impact_with_impulse` pass. |
| 3 | Sound pauses when the tab is hidden or the player leaves the puzzle, and a hint mentions the phone silent switch. | VERIFIED | `puzzle-sound.ts` `onVisibilityChange` suspends when the tab is hidden. When it becomes visible, `resumeWithoutGesture` runs only if sound is on, and listeners exist only while a puzzle route is attached. `leave()` removes the listeners and suspends (`puzzle-lifecycle.ts:153`). `incrementGeneration` stops voices (`puzzle-host.ts:46`). `SOUND_HINT = "No sound on iPhone? Check the silent switch."` is the toggle's `title` and is shown in the drawer section (screenshot `portrait-drawer.png`). The e2e tests "suspends while the tab is hidden…" and "suspends when the player leaves the puzzle…" (`closed 0`) pass. |
| 4 | The impact data the bridge reports for sound does not change the simulation: 25/25 scene fingerprints are unchanged, and puzzle outcomes are the same with sound on and off. | VERIFIED | `diff target/phase42/base.txt target/phase42/after-postfix.txt` is empty (25 lines), and `base.txt` matches `target/phase41/after-final.txt`. `git diff --stat bd685b08f -- crates/liquidfun` is empty. `SoundTally::observe` borrows `&World` and `&StepReport` only. `removing_the_tally_leaves_positions_and_status_bit_identical` and `solution_sound_scalars_repeat_across_runs` pass (14 passed in the `puzzle::sound` filter). Release replay: 34 passed. Wasm replay: 6 `LEVELHASH … pass`. The e2e sound-on and sound-off runs produce the same fingerprint `{201, 262, 811, 0cf8c2703dcd8fac}`. |

**Score:** 4/4 roadmap criteria verified. All 30 plan-frontmatter truths (6 per plan × 5 plans) have supporting code and passing checks. One plan truth was refined by a review fix (see the note under Anti-Patterns).

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/puzzle/sound.rs` | `SoundSignals`, `SoundTally` | VERIFIED | 161 lines. Membership-only `HashSet` with `BuildHasherDefault<DefaultHasher>`. Step-1 baseline (IN-02). Finite-impulse filter (IN-01). |
| `crates/liquidfun-wasm/src/frame/puzzle.rs` | three `ProofFrame` getters | VERIFIED | `js_name = splashContacts` / `impactCount` / `impactImpulse` at lines 95, 102 and 109. |
| `crates/liquidfun-wasm/src/session.rs` | tally wiring | VERIFIED | Two `tally.observe(` sites (278, 327) with failure detail; `set_puzzle_sound` at 511; `without_sound_tally` at 534. File is 602 lines, under 610. |
| `web/src/physics/frame-sound.ts` | `parsePuzzleSound` fail closed | VERIFIED | Called from `frame.ts:255`; `frame-sound.test.ts` passes. |
| `web/src/player/sound-preference.ts` | versioned key, exact `"on"` | VERIFIED | `export const SOUND_STORAGE_KEY = "liquidfun.puzzle-sound.v1";` |
| `web/src/audio/{sound-signals,voices,voice-pool,triggers}.ts` | pure core | VERIFIED | `MAX_VOICES = 6`, `PENTATONIC_SEMITONES` (11), `decideVoices`, `trackWithoutClock`, `foldSoundSignals`. |
| `web/src/audio/sound-engine.ts` | factory-injected Web Audio shell | VERIFIED | Only file that references the browser constructors (gate 31). |
| `web/src/player/puzzle-sound.ts` | shared state, gesture unlock, visibility, present | VERIFIED | 317 lines; the browser instance is used only by `playground-session.ts:128`, and the runtime defaults to disabled. |
| `web/src/components/PuzzleSoundToggle.tsx`, `web/src/styles/puzzle-sound.css` | HUD toggle and drawer section | VERIFIED | Rendered via `PuzzleHud.tsx:99` (`props.soundToggle`) inside the `puzzle-options-slot` obstacle, and `PuzzlePanel.tsx:394,404` (drawer and form-factor). |
| `web/e2e/puzzle-sound.spec.ts`, `web/e2e/sound-helpers.ts` | D-13 browser proofs | VERIFIED | 515 and 210 lines; listed in the `test:puzzle` script. |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| `session.rs` advance / advance_profiled | `SoundTally::observe` | after `on_after_step` | WIRED |
| `session.rs` annotate_frame | `FrameData::set_puzzle_sound` | after `set_puzzle_status` | WIRED |
| `frame.ts` parseRenderFrame | `parsePuzzleSound` | `maybeSound` spread | WIRED |
| `puzzle-loop.ts` presentSteps | `foldSoundSignals` | line 90; fold passed as the 4th `publish` argument | WIRED |
| `puzzle-run.ts` publish | `sound.present(frame, signals, state)` | line 132, after `publishState` / `setView` | WIRED |
| `puzzle-host.ts` incrementGeneration | `sound.stopVoices()` | line 46 | WIRED |
| `puzzle-lifecycle.ts` | `sound.attach/pause/leave` | lines 83, 142, 153 | WIRED |
| `PuzzleSoundToggle.tsx` | `controller.setSoundEnabled` | lines 57, 96 | WIRED |
| `triggers.ts` | `voices.ts plinkFrequency` | `maybePlink` | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `triggers.ts` splash/impact | `signals` (folded `maybeSound`) | Rust `SoundTally` from `ParticleSystemView::body_contacts` and `StepReport` solves | Yes. The probe shows per-step splash peaks of 86-153 and 5 Float Up impact steps. | FLOWING |
| `triggers.ts` plink | `frame.maybePuzzle.goalCount` | existing puzzle status getter | Yes | FLOWING |
| `triggers.ts` chime | `state.status.outcome` / `latch.outcome` | `PuzzleState` reducer | Yes | FLOWING |
| `sound-engine.ts` play | `decision.requests` | `decideVoices` | Yes. 14 sources started in the e2e sound-on run. | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| TS types | `cd web && bun run typecheck` | clean | PASS |
| Sound and glue units | `bunx vitest run` on the 11 sound, frame-sound, loop and chrome files | 11 files, 218 tests passed | PASS |
| Phase gates (post-fix) | `target/phase42/checks-05-postfix.results` at HEAD 003656428 | 32/32 `exit=0` (20:36-20:48Z) | PASS |
| Scene fingerprints | `diff base.txt after-postfix.txt` | identical, 25 lines | PASS |
| Lifecycle provenance | `gsd-tools verify lifecycle 42 --require-plans --raw` | `valid` | PASS |

Post-fix gate counts from `checks-05-postfix.log`: `cargo test --all-features` passes. The `liquidfun-wasm` run had 588 passed and 25 ignored. The `puzzle::sound` filter had 14 passed and 1 ignored (the probe). Release replay had 34 passed. Vitest had 113 files and 1725 tests. Player smoke had 45, hud-safe-frame 64 and puzzle e2e 99.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SOUND-01 | 02, 03, 04, 05 | Off by default; a remembered toggle | SATISFIED | Truth 1 |
| SOUND-02 | 01, 02, 03, 05 | Splash, impact, rising plinks, win chime, voice cap | SATISFIED (automated); listening UAT pending | Truth 2 |
| SOUND-03 | 03, 04, 05 | Pause on hidden or leave; silent-switch hint | SATISFIED (automated); device UAT pending | Truth 3 |

Every plan ID is in REQUIREMENTS.md. REQUIREMENTS.md maps no other ID to Phase 42, so no requirement is orphaned. The checkboxes in REQUIREMENTS.md are still `[ ]` / Pending, so the orchestrator ticks them at phase completion.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `web/src/player/sound-preference.ts` | 20, 30 | `catch {}` with a fallback | Info | Required by D-03, which says blocked storage means off and keeps the in-memory choice. It mirrors `first-hint-preference.ts` and has a comment. |
| `crates/liquidfun-wasm/src/puzzle/sound.rs` | 139-144 | drops non-finite impulses | Info | Deliberate IN-01 trade-off, so sound-only data cannot fail a run (D-08). Documented in the code. The impact still counts. |
| `web/src/components/PuzzleSoundToggle.tsx` | 49 | unavailable uses `aria-disabled`, not `disabled` | Info | IN-03 refinement of the plan 04 truth "renders a disabled button". It keeps focus, and the e2e tests for disabled controls and focus retention pass. |

The phase files contain no TODO, FIXME or placeholder markers, and the new Rust code has no `unwrap()`.

### Human Verification Required

1. **Listening balance at 1x, 2x and 4x.** Play First Pour and Float Up with sound on. Expected: splash, impact and plink are audible and balanced, plinks rise as the cup fills, nothing clips, and 4x is not noise.
1. **Win chime and silent tail.** Win First Pour with sound on. Expected: one chime, then silence through the settle tail.
1. **iPhone with the switch on ring.** Expected: sounds play.
1. **iPhone with the silent switch engaged.** Expected: silence, with the hint visible in the drawer and as the toggle title.
1. **iPhone with a stored "on".** Reload, then tap. Expected: the first touch `pointerup` starts sound.

**Why the status is `human_needed` and not `passed`:** The automated evidence covers all four success criteria, so nothing is a gap. But the phase goal is about what players hear. Every automated check uses a recording fake context: it proves which nodes start and with which parameters, not what comes out of a speaker. The silent-switch criterion is also a device property. Under the Agent-Performed Simple UAT policy, these items do not fail the phase. The verifier's rule still allows `passed` only when this section is empty. The orchestrator can treat the phase as complete for automated purposes and record the five items as pending manual UAT.

### Gaps Summary

None. The code-review findings (WR-01, WR-02, IN-01 to IN-05) are fixed and have regression tests. The full 32-gate suite was re-run on the post-fix tree with every gate at exit 0, and the 25 scene fingerprints are unchanged.

---

_Verified: 2026-10-13T20:50:32Z_
_Verifier: Claude (gsd-verifier)_
