# Feature Research

**Domain:** Place-then-play physics puzzle mode ("get the water into the cup") added to an existing SolidJS + Rust/WASM LiquidFun playground
**Researched:** 2026-10-11
**Confidence:** MEDIUM. Genre conventions are well established and consistent across sources (The Incredible Machine, Cut the Rope, Where's My Water, Poly Bridge, Sugar Sugar, Fantastic Contraption). Web search found few formal design write-ups, so many UX specifics come from general genre practice and are marked as such. Claims about this repo's code come from reading the source on 2026-10-11 (HIGH).

## How the reference games work (evidence summary)

| Game | Build/run model | Constraint | Scoring and stars | Progression | Teaching |
|------|-----------------|------------|-------------------|-------------|----------|
| The Incredible Machine | Place parts while stopped, then run. Stopping resets the machine to its start state **with your parts kept** for more edits. | Per-level parts bin with fixed types **and counts**, plus fixed scenery you cannot move. | None. Pass or fail. | Linear unlock. In TEMIM, 47 puzzles are open from the start; after that, each one unlocks the next. | Parts are introduced one at a time, then combined. The limited bin acts as scaffolding that keeps levels solvable. Criticized for pixel-perfect solutions and having **no fast-forward**. |
| Fantastic Contraption | Build in zero gravity inside a marked build zone, then press Go. | Build zone only. No piece or time limits. | None. | Linear. | Open-ended. Many valid solutions. |
| Poly Bridge | Build, then simulate. | A **budget** in currency. Each part has a price. | Under budget plus unbroken counts as fully complete. A separate half-budget achievement exists. | Worlds. Finishing everything under budget unlocks a secret world. | One new material or idea per level. |
| Cut the Rope | Live interaction. | None. | **3 stars collected inside the level**. You can pass with 0 stars. | Stars **gate the next box** (for example, 20 to 30 stars for box 2). | One new object per few levels. Almost no text. |
| Where's My Water | Live digging. | Level layout. | **3 ducks** that fill when enough water passes through them. In WMW2 the ducks are optional and unlock bonuses. | Chapters. | Very little tutorial. New props and hazards are introduced level by level and taught visually. |
| Sugar Sugar | Live drawing. | None. | A cup is full at **100 grains**. Some levels have color filters, striped cups and gravity flips. | Linear. | One new twist per level. |

**What this means for us:** our owner-chosen model is The Incredible Machine's loop: a per-level tool bin with counts, an untimed build phase, Play, then Stop to return to build mode with placements kept. Cut the Rope supplies the 3-star frame. Sugar Sugar supplies the "cup fills to N particles" goal. The two best-known criticisms of the genre, pixel-perfect solutions and slow repeated runs, both apply directly to us and drive several recommendations below.

## Feature Landscape

### Table Stakes (Users Expect These)

Missing any of these makes the mode feel broken or unfinished.

#### Goal, timer and win/lose

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| GOAL | **Cup goal zone that counts qualifying particles** and shows fill % live | It is the core loop. Sugar Sugar and Where's My Water both show a filling container. | MEDIUM | Count particles whose position is inside the cup's interior polygon or AABB, **filtered by level-declared particle groups** so the sponge's own particles, decorative particles and powder in a water level do not count. `crates/liquidfun/src/particle/query.rs` already has `query_aabb`. Counting should happen in the WASM session layer (`crates/liquidfun-wasm/src/session.rs`) after each step and be read-only, so all 25 scene fingerprints stay unchanged. |
| GOAL | **Fill denominator is a fixed per-level particle count** ("cup capacity"), not "% of all water emitted" | Players read % as "how full is the cup". Sugar Sugar uses a fixed 100 grains. | LOW | Display it as a percentage, but author it as an integer count. This avoids % drift when emitter totals change. |
| GOAL | **Splash-through protection**: a particle counts only while it is inside. Win requires the target to be **held** for a short window (about 0.5–1 s of sim time). | Without it, a spray that passes through the cup can trigger a false win. Players treat a false win as a bug. | LOW | It is a debounce over consecutive steps. It is deterministic because it is counted in steps. |
| TIME | **Timer runs only during Play and counts simulated time (steps × dt), not wall-clock time** | Building must be untimed. Every placement game lets you think. A wall-clock timer would make results depend on the device. | LOW | **Critical dependency.** `simulation-worker.ts` drops steps over `FRAME_STEP_BUDGET_MS` (budgeted advance), so on a slow phone simulated time falls behind wall time. A wall-clock timer would break "a working solution always reproduces". Show seconds derived from the step count. |
| WIN | **Clear win and fail end states**: a win overlay with stars, and a fail overlay with "Out of time: 54% of 80%" | Players need closure and to know how close they got. | LOW | Primary buttons: Win → Next level (also Replay, Levels). Fail → **Edit tools** (back to build mode with placements kept) and Retry. |
| WIN | **The run ends early once the outcome is settled** (the top star bar is reached and held, or the fill can no longer change) | Waiting out a 20 s timer after an obvious win is tedious. This is TIM's "no fast-forward" complaint. | LOW–MEDIUM | At minimum, end at the top fill tier or the timer, whichever comes first. See Differentiators for fast-forward. |
| METER | **Fill meter on or next to the cup with a visible target line**, plus a HUD readout `62% / 80%` and a countdown | Players watch the cup, not a sidebar. A target line answers "how much more?" at a glance. | LOW–MEDIUM | Canvas overlay drawn in world space at the cup, plus a HUD chip. The HUD chip must carry `data-hud-obstacle` on the leaf element (repo rule; `web/src/render/safe-frame.ts`). Must work in every render mode, including the future GPU mode. |

#### Tools and placement (build mode)

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| TOOL | **Per-level tool palette with a remaining count on each tool** (for example, Ramp ×2, Pin ×1) | TIM's parts bin is the genre default. Counts are the main difficulty and teaching lever. | MEDIUM | The level definition declares `allowedTools: {type, count}[]`. A tool with 0 left greys out. Removing a placed tool returns it to the bin. |
| TOOL | **Fixed tool sizes; each size is its own tool type** ("Short ramp" and "Long ramp", not a resize handle) | Keeps the solution space small and touch-friendly, and keeps the "fewer tools" star meaningful. | LOW | Resizing adds a dimension to the search space and a fiddly handle on phones. |
| PLACE | **Place**: drag from the palette onto the canvas (desktop), **and tap a tool then tap the canvas** (touch) | Drag-from-tray is expected on desktop. Tap-to-place is more reliable on phones, where dragging out of a scrolling tray conflicts with scroll. | MEDIUM | Reuse the CSS-bound camera unproject from `web/src/input/canvas-pointer.ts`. **The existing poke, drag and shove gestures must be off in puzzle mode** in build and run alike. They go through WASM `pointer_action`, which would push water during a run. |
| PLACE | **Move**: drag a placed tool. Offset is kept from the grab point so the tool does not jump to the finger. | It is the basic manipulation. A tool jumping to the finger is a well-known touch bug. | LOW–MEDIUM | Record the grab offset on pointerdown and apply it on move. Add a small extra lift on touch (`pointerType === "touch"`) so the finger does not hide the tool. |
| PLACE | **Rotate**: a visible rotation handle on the selected tool, **snapped to 15° steps**. Desktop also gets scroll wheel or Q/E. Shift (or a long hold) rotates freely only if a level needs it. | Ramps are useless without angles. Snapping makes touch rotation precise and avoids pixel-perfect angle hunting. | MEDIUM | One-finger handle rotation is enough. Two-finger twist is optional (see Differentiators). Angle math uses τ per repo convention. |
| PLACE | **Remove**: an × button on the selected tool, the Delete or Backspace key, and dragging back onto the palette | All three are common, and touch users need a visible target. | LOW | |
| PLACE | **Invalid placement feedback**: a red tint while overlapping fixed geometry, other tools, the cup interior, the initial water, or outside the level's build area. Release snaps back to the last valid spot. | Without it, players drop tools inside walls and the physics explodes on Play. | MEDIUM | Overlap test against static fixtures plus a per-level `noBuildZones` list (cup interior, emitter mouth, finale-critical regions). Can run in TS against level geometry. No engine query is needed while paused. |
| PLACE | **Selection state**: one selected tool at a time, highlighted, with handles. Tap or click on empty space deselects. | It is needed to show handles and to route keyboard nudges. | LOW | |
| PLACE | **Hit areas of at least 44×44 CSS px on touch** for tools, handles and palette items | Apple's HIG minimum. Small ramps at phone zoom are otherwise untappable. | LOW | Make the hit slop larger than the visual on small tools. Handles sit outside the tool body so the finger does not cover the tool. |

#### Build/run modes and reset

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| MODE | **Two explicit modes: Build (paused, editable) and Run (simulating, tools locked)**, with one primary Play / Stop button | This is the Incredible Machine and Fantastic Contraption contract. Editing during a run would break determinism and the "place-then-play" decision. | MEDIUM | Build mode shows a captured frame of the level's initial state without stepping (the `capture` worker op exists). Run mode rebuilds the world from **level definition + placements** and steps it. |
| MODE | **Stop / Edit returns to Build with placements kept** (owner decision: Reset returns to the player's placements) | It is TIM's exact reset loop. Losing your placements on reset is the most-hated failure in this genre. | LOW–MEDIUM | Implement Stop as a full deterministic rebuild from (level, placements). Do not try to rewind. The existing `generation` field on worker envelopes already supports discarding stale frames across rebuilds. |
| MODE | **"Clear tools"**, separate from Stop, which returns every tool to the bin, with confirm or one-step undo | Players sometimes want a clean slate. It must never happen by accident. | LOW | Keep it visually secondary. |
| MODE | **Placements survive leaving and returning to a level within the session**, and ideally across reloads via localStorage | Accidentally tapping "Levels" must not wipe a near-solution. | LOW | Store the last placement set per level next to progress. Treat it as a cache: if it fails to parse, drop it silently and log to the console. |

#### Stars, level select, progress

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| STAR | **3 stars with criteria visible before Play** (on the level intro card and as tick marks on the fill meter) | Cut the Rope and Angry Birds made this universal. Players expect to know how to earn each star. | LOW | Recommended rule (see "Star rule" below): ★1 reach target fill, ★2 reach bonus fill, ★3 win using ≤ par tools. Each criterion is independent and shown as a checklist on the result screen. |
| STAR | **Best result is kept**: a replay never lowers stored stars. Best fill and fewest tools are stored separately. | It is universal. Losing stars on replay feels punitive. | LOW | Merge rule: per criterion, take the OR of best values. |
| SELECT | **Level select grid**: 5–8 cards with number, title, "teaches X" subtitle, earned stars, and a lock state | It is the standard entry point. It also shows the teaching arc. | LOW–MEDIUM | New route `#/puzzle` (select) and `#/puzzle/<id>` (play), handled next to the existing `web/src/routing/hash.ts`. The 25 scenes and their routes stay unchanged. |
| SELECT | **Linear unlock: winning level N unlocks N+1**, with Level 1 always open | Matches the small level count and the teaching order. TIM uses the same scheme. | LOW | **Locks are soft**: a deep link to `#/puzzle/<id>` always opens the level (with a "skipped ahead" note). Storage-blocked visitors and README visitors must never hit a wall. |
| PROG | **Per-browser progress in localStorage** (stars, best fill, fewest tools, unlocked), **fully playable when storage is blocked** | Owner decision. Safari private mode and blocked storage throw on access. | LOW | Reuse the injected `() => window.localStorage` loader pattern from `web/src/player/playground-session.ts` and `CanvasStage.tsx`. Use one versioned key (`liquidfun-puzzles:v1`), schema-validate on read, and fall back to in-memory state. |
| NEXT | **"Next level" on win, and "Levels" reachable from the play screen at all times** | It is the core flow. | LOW | |

#### Teaching and onboarding

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| TEACH | **Each level introduces exactly one idea**, and its palette is the lesson (owner list: water, viscous, powder, elastic, buoyancy, valve, finale) | Matches TIM's "parts introduced one at a time, then combined" and Where's My Water's per-level props. | MEDIUM per level | **Level 1 must be close to trivial** (one ramp, generous target) so the controls are the only thing being learned. The finale combines at most 3 earlier ideas. |
| TEACH | **One-line level caption** shown on the intro card and in the HUD (for example, "Thick liquid flows slowly. Plan for the timer.") | Genre games use almost no text, but one sentence per level is the minimum that tells the player what is new. | LOW | No multi-page tutorials. |
| TEACH | **First-level control hint**: an animated ghost hand or arrow showing "drag a ramp here → press Play", shown once and dismissible | Without it, phone users do not discover drag-from-palette. | LOW–MEDIUM | Show only on Level 1, only until the first placement. Never block input. |

#### Phone / touch

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| PHONE | **Portrait frame per level** so the cup, emitter and build area fill an iPhone canvas (about 390×844) | Repo rule for scenes. The same contain-fit problem applies to levels. | LOW per level | Mirror `web/src/catalog/portrait-bounds.ts` plus its test (`web/tests/portrait-bounds.test.ts`) with a puzzle equivalent keyed by every level id. The HUD safe frame then handles chrome. |
| PHONE | **Bottom tool tray in portrait and side tray in landscape**, marked `data-hud-obstacle` per leaf | The palette must not cover the action. It has to be reachable by thumb. | MEDIUM | Repo rule: obstacle markers go on leaf elements with camera-independent sizes. The existing `web/e2e/hud-safe-frame.spec.ts` pattern should be extended to every level. |
| PHONE | **No page scroll, zoom or pull-to-refresh fighting with canvas drags** | It is the classic mobile web game failure. | LOW | Set `touch-action: none` on the canvas in puzzle mode. The playground likely already does this for pointer gestures; verify it. |
| PHONE | **Tilt gravity disabled in puzzles** | Tilt changes the outcome per device and would make stars depend on how the phone is held. | LOW | `web/src/input/tilt-gravity.ts` must not bind in puzzle routes. |

#### Sound (opt-in)

| ID hint | Feature | Why Expected | Complexity | Notes / Dependencies |
|---------|---------|--------------|------------|----------------------|
| SND | **Muted by default; one visible speaker toggle; the preference persists** | Owner decision. Browsers suspend AudioContext until a user gesture anyway. | LOW | Create or resume **one** AudioContext inside the toggle's click handler (Chrome autoplay policy; iOS shares one context underneath). Store the preference in localStorage. Even with a stored "on", the context must still be resumed on the first gesture. |
| SND | **Win chime, plus splash and impact cues during Run** | Owner decision. These are basic game feedback. | MEDIUM | Synthesize with WebAudio (no asset licensing). Drive cues from **per-frame aggregates** (new particles entering the cup this frame; `body_contact_count` already exists in `crates/liquidfun-wasm/src/frame.rs`). **Never feed the simulation** (SEED-005). Cap voices and rate-limit, or a dam of water becomes noise. |
| SND | **Audio suspends on a hidden tab and on Stop** | Expected etiquette. | LOW | Hook into the existing visibility handling in `web/src/player/scene-lifecycle.ts`. |
| SND | **UI ticks for place, rotate snap, remove and Play** (very quiet) | Tactile feedback on phones is otherwise absent. | LOW | Only when sound is on. |

### Differentiators (Competitive Advantage)

These align with the project's core value: fun experimentation that shows off a real particle engine. Pick a few. Do not try to do all of them.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Real particle materials as the lesson**: viscous, powder, elastic, tensile and buoyant behaviors from the actual engine | No casual puzzle game ships true LiquidFun material variety. Each level is a demo of an engine feature. | MEDIUM | Already available via existing scene builders. The main work is choosing level parameters that keep particle counts phone-friendly. |
| **Determinism promise surfaced to players**: "Your solution will always work the same way" | Turns an engine property into player trust. There are no flukes, so failures are always the player's to fix. | LOW | It is only true if the timer and win rule are step-based, tilt and pointer are off, and steps are not dropped from the simulation itself. Budgeted advance delays steps but must never skip them. **Verify this.** |
| **Fast-forward (2× / 4×) during Run** | Fixes TIM's biggest complaint. Iteration speed is the fun in place-then-play. | MEDIUM | Safe because the timer is step-based. On phones the worker may not keep up; then it caps at whatever rate is achievable, and the outcome is identical either way. |
| **Rising-pitch "plink" as particles enter the cup**, with pitch tied to fill % | An audio fill meter. Players hear progress without looking. Very cheap given the cup counter. | LOW | It is the best single sonification cue for this mode, and depends on GOAL and SND. |
| **On-demand hint ladder**: tier 1 is a text nudge; tier 2, unlocked after 2 failed runs, shows a ghost of one tool from the reference solution | Unblocks stuck players without spoiling the level. Nearly free, because every level should ship a reference solution for tests anyway. | LOW–MEDIUM | No currency or cost. Hint use does not reduce stars (keeps it friendly), or it is at most noted. |
| **Last-run water trail ghost** in build mode (a faint outline of where the water went on the previous run) | Shows where the water missed and speeds up iteration. | MEDIUM | Needs a sampled path from the previous run (for example, a downsampled particle-density image captured during the run). Defer unless cheap. |
| **Two-finger twist to rotate on touch** (in addition to the handle) | Feels native on phones. | MEDIUM | Pivot at the tool centre. It conflicts with pinch-zoom if puzzles allow zoom, so pick one. Recommendation: no zoom in puzzles; each level's frame fits. |
| **Keyboard placement parity** (Tab to cycle tools, arrows to nudge, R/Shift+R to rotate, Delete to remove, Space for Play/Stop) | Accessibility, in keeping with the repo's earlier labeled keyboard controls (WEB-07). Also speeds up desktop iteration. | MEDIUM | Selection model from PLACE is a prerequisite. |
| **GPU water render mode** (refraction, highlights, foam), closing phase per SEED-003 | Makes the cup filling satisfying, and improves every scene and the README too. | HIGH | It is a renderer-only change. It needs per-particle velocity in the frame for foam, and a canvas fallback. The fill meter and target line must draw on top in all modes. |
| **Glass-cup rendering**: the cup drawn as a translucent vessel with the target line etched on it | Puts the goal in the world rather than in UI chrome. | LOW–MEDIUM | Static-layer art in `web/src/render/static-layer.ts`. Gets better with GPU refraction later. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Poking, dragging or shoving water during Run** (the existing playground gesture) | It is already built and it is fun. | It breaks place-then-play, makes stars un-reproducible, and turns puzzles into skill tests. | Disable the canvas pointer actions in puzzle routes. Keep them in the 25 scenes. |
| **Live interaction tools in Run** (tap to open a valve, Where's My Water style) | Valves feel like they should be tapped. | Timing-dependent input breaks determinism and the owner's place-then-play decision. | **A valve is configured in Build**: either a timed gate with a delay chosen while building (for example, opens at 3 s) or a passive one-way flap (Tesla-valve-style). Flag for owner confirmation. |
| **Wall-clock timer** | It is the obvious implementation. | Slow devices drop steps under the frame budget, so the same solution would pass on desktop and fail on a phone. | A step-count timer (TIME). |
| **Star-gated unlocks** (Cut the Rope boxes) | Encourages replay. | With 5–8 levels it just blocks people. It also conflicts with "fully playable when storage is blocked". | Linear soft unlocks, with deep links always allowed. |
| **Lives, energy, hint currency, ads, daily rewards** | Mobile-genre habit. | Hostile in a hobby demo, and needs servers or economy design. | Free retries and free hints. |
| **Accounts, leaderboards, cloud sync** | Competition. | Owner excluded. Needs servers. Determinism also makes leaderboards trivially copyable. | Local best stars only. |
| **Level editor or shareable solution links** | Natural next step. | SEED-002 is explicitly out of scope. A share format also freezes the placement schema early. | Keep the placement data model serializable (it is needed for localStorage anyway) so SEED-002 can reuse it later. |
| **Freeform tool resizing and continuous rotation by default** | "More creative". | Blows up the search space, recreates TIM's pixel-perfect problem, and is fiddly on touch. | Fixed sizes as separate tool types, and 15° rotation snap. |
| **Unlimited tools** (Fantastic Contraption style) | Freedom. | Removes the teaching lever and makes the "fewer tools" star meaningless. | Per-level counts with par. |
| **Levels that need pixel-perfect placement** | Hand-tuned designer solutions. | TIM's most-cited frustration. Particle splashes make it worse. | Design rule: each level's reference solution must still win with ±0.25 m position and ±15° rotation jitter. Check this in a test. |
| **Win the moment fill touches the target** | Simple. | Splash-through false wins, and no room for a "higher fill" star. | Held-target debounce, and run until the top tier or the timer. |
| **Mandatory multi-step tutorial screens** | Onboarding. | Players skip text. The genre teaches through level design. | One caption per level, plus a single dismissible ghost hand on Level 1. |
| **Zoom and pan in puzzle levels** | The existing scenes allow it. | Conflicts with drag-to-place and two-finger rotate, and hides the cup. | A fixed per-level frame (landscape bounds plus a portrait frame), so the whole level is always visible. |
| **Elastic "sponge" that soaks up water** | The owner's list says "sponge". | LiquidFun elastic particle groups deform and bounce. They do not absorb liquid. A real soak mechanic would mean new engine behavior. | Teach elastic as a **squishy deflector or bouncer** (call it a "jelly pad" or keep "sponge" as a name only), or fake absorption by destroying water in a shape and counting it as stored. **Flag for owner confirmation.** |
| **Engine-side changes for goal counting** | It seems natural to put the counter in the engine. | Risks the 25 scene fingerprints and couples the engine to game rules. | Count read-only in the WASM session after each step, using existing queries. |

## Star rule (recommended, flag for owner confirmation)

The owner said "1–3 stars for fewer tools used or a higher fill". Three independent, pre-visible criteria satisfy both axes and match Cut the Rope's model of separate goals:

1. ★ **Win**: hold `targetCount` particles in the cup before the timer ends.
1. ★ **Brim**: hold `bonusCount` (higher fill, for example target + 15–20 percentage points) before the timer ends.
1. ★ **Thrifty**: win using ≤ `parTools` tools.

- The run ends when Brim is held, or when the timer ends. If Win was held at any point, the level is won, so later slosh-out does not cost the win. Stars are scored on **peak held fill** and on the number of tools placed.
- Stars from different runs merge (best of each), so a player can earn Brim with many tools and Thrifty in another run, like Cut the Rope stars across replays. **Owner choice:** merge across runs (friendlier, recommended), or require all three in one run (harder).
- **Authoring rule:** every level must be unwinnable with zero tools, and par must be achievable. A reference solution per level achieves 3 stars and is checked by a headless test. Because the engine is deterministic, this works as a regression guard against engine or level changes.

## Feature Dependencies

```
Level definition model (geometry, emitter, cup, allowedTools, par, targets, portrait frame)
    └──requires──> existing scene builder seam (crates/liquidfun-wasm/src/scene.rs, catalog records)

Deterministic rebuild from (level, placements)
    └──requires──> Level definition model
    └──requires──> worker op to build a level with a placement list (worker-messages.ts extension)

Cup goal counter (filtered, read-only, per step) ──> Fill meter / HUD readout
    └──requires──> Deterministic rebuild
    └──requires──> frame payload extension carrying cup count (frame.rs → physics/frame.ts)

Step-based timer + held-target debounce ──> Win/Fail state ──> Star scoring
    └──requires──> Cup goal counter

Build mode (capture without stepping) ──> Tool placement UX (place/move/rotate/remove/validate)
    └──requires──> camera unproject (input/canvas-pointer.ts), pointer gestures disabled
    └──requires──> Tool palette with counts

Star scoring ──> Progress store (localStorage, versioned, in-memory fallback) ──> Level select + soft unlocks

Hash routes #/puzzle and #/puzzle/<id> ──> Level select, play screen (reuses canvas, camera, HUD safe frame, scale legend)

Portrait frame per level ──> Phone layout + bottom tray (data-hud-obstacle)

Cup counter ──enhances──> Sound plinks (rising pitch)
Win state ──enhances──> Win chime
body_contact_count (exists) ──enhances──> Impact sounds
Reference solutions ──enhances──> Hint ladder (ghost tool), regression tests
Step-based timer ──enables──> Fast-forward
GPU water mode ──requires──> per-particle velocity in frame; must not hide fill meter/target line

Run-time pointer gestures ──conflicts──> place-then-play determinism
Tilt gravity ──conflicts──> reproducible stars
Zoom/pan ──conflicts──> drag-to-place + two-finger rotate
```

### Dependency Notes

- **Everything requires the level model and deterministic rebuild first.** Stop/Edit, stars, hints and regression tests all assume that (level, placements) → identical run. Build this as the first phase, together with the goal counter, before any placement UX. A level with hard-coded placements already proves the loop headlessly.
- **The timer and win rule must be step-based** because of the existing budgeted-advance behaviour in `web/src/physics/simulation-worker.ts`. This is a hard dependency for the determinism promise, fast-forward and phone fairness.
- **The placement UX depends on disabling existing input paths.** `canvas-pointer.ts` gestures, tilt and possibly the camera zoom tools in `ViewportTools.tsx` must be route-aware. This is a regression risk for the 25 scenes, so keep the scenes' code paths untouched and branch at the route level.
- **The progress store depends on a stable level id and star schema.** Version the storage key. If a level's targets change later, keep the stored stars (honest enough for a hobby project) rather than silently resetting them.
- **Sound depends on a new per-frame aggregate** (cup entries this frame). Sound must be a pure consumer of frame data.
- **GPU water conflicts with overlay legibility.** Draw the fill meter and target line in a layer above whichever render mode is active.
- **Repo rules carry over.** If levels reuse the catalog machinery, the catalog-scene rules apply: README SVG plan coverage via `assertReadmeSvgPlanCoverage`, portrait bounds keyed by every `SceneId`, and the scale legend. Recommendation: give **levels their own registry** (`LevelId`) with a parallel portrait-frame table and test. Do not add levels to `scenes.ts`, or the README gallery and coverage asserts will demand entries for every level.

## MVP Definition

### Launch With (v1.5 core)

- [ ] Level model plus deterministic rebuild from (level, placements). This is the foundation.
- [ ] Filtered cup counter, held-target debounce, step-based timer, and win/fail overlays. This is the core loop.
- [ ] Fill meter with target line and star ticks, plus HUD fill and timer chips (`data-hud-obstacle`). Players need feedback.
- [ ] Tool palette with counts, plus place/move/rotate (15° snap)/remove, invalid-placement feedback, and Build/Run with Stop keeping placements. This is the owner's place-then-play loop.
- [ ] 3-criterion stars (Win / Brim / Thrifty), best-of merge, and criteria visible before Play.
- [ ] `#/puzzle` level select plus `#/puzzle/<id>`, linear soft unlocks, and versioned localStorage with in-memory fallback.
- [ ] 5–8 teaching levels, each with a caption, a portrait frame and a reference solution that is test-verified for 3 stars and jitter tolerance.
- [ ] Touch: tap-to-place, 44 px hit areas, bottom tray, tilt and pointer gestures off, no zoom.
- [ ] Opt-in sound: toggle, single AudioContext resumed on gesture, win chime, cup plinks, impacts, voice cap.

### Add After Validation (in-milestone if time allows)

- [ ] Fast-forward 2×/4×. Add when play-testing shows waiting is the main friction.
- [ ] Hint ladder with a ghost tool from the reference solution. Add when a level shows players getting stuck.
- [ ] Keyboard placement parity. Add once the selection model is stable.
- [ ] Glass-cup art. Cheap polish before the GPU phase.
- [ ] GPU water render mode (owner-planned closing phase).

### Future Consideration (later milestones)

- [ ] Last-run water trail ghost. Needs run-sampling infrastructure.
- [ ] Two-finger rotate. Only if handle rotation tests poorly on phones.
- [ ] Level editor and share links (SEED-002). Out of scope now. Reuse the placement schema.
- [ ] Color-filter or multi-cup goals (Sugar Sugar twists). These are content for a second level pack.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Level model + deterministic rebuild | HIGH | MEDIUM | P1 |
| Filtered cup counter + debounce + step timer | HIGH | MEDIUM | P1 |
| Fill meter / HUD chips | HIGH | LOW–MEDIUM | P1 |
| Tool palette + placement UX + validation | HIGH | HIGH | P1 |
| Build/Run modes, Stop keeps placements | HIGH | MEDIUM | P1 |
| Stars (3 criteria) + result overlay | HIGH | LOW | P1 |
| Level select + soft unlocks + progress store | HIGH | LOW–MEDIUM | P1 |
| 5–8 teaching levels + reference solutions | HIGH | MEDIUM–HIGH (tuning) | P1 |
| Phone layout (portrait frames, tray, touch rules) | HIGH | MEDIUM | P1 |
| Opt-in sound (chime, plinks, impacts) | MEDIUM | MEDIUM | P1 (owner decision) |
| Fast-forward | MEDIUM | MEDIUM | P2 |
| Hint ladder | MEDIUM | LOW–MEDIUM | P2 |
| Keyboard placement | MEDIUM | MEDIUM | P2 |
| GPU water render mode | MEDIUM | HIGH | P2 (closing phase, owner decision) |
| Glass-cup art | LOW–MEDIUM | LOW | P2 |
| Last-run trail ghost | MEDIUM | MEDIUM–HIGH | P3 |
| Two-finger rotate | LOW | MEDIUM | P3 |

## Competitor Feature Analysis

| Feature | The Incredible Machine | Cut the Rope / Where's My Water | Poly Bridge | Our Approach |
|---------|------------------------|---------------------------------|-------------|--------------|
| Constraint | Parts bin with counts | Level layout only | Budget | Parts bin with counts, plus par for a star |
| Build vs run | Stopped build, Run, Stop keeps parts | Live | Build, then simulate | TIM model, untimed build |
| Reset | Back to start with parts kept | Restart level | Back to build | Stop is a deterministic rebuild with placements kept, plus a separate Clear tools |
| Scoring | Pass/fail | 3 collectibles (stars, ducks) | Under budget | 3 independent criteria (Win, Brim, Thrifty) |
| Unlocks | Linear | Star-gated boxes or chapters | Worlds | Linear soft locks; deep links always open |
| Teaching | One part at a time | Visual, almost no text | One material per level | One idea per level, one caption, Level 1 ghost hand |
| Iteration speed | No fast-forward (criticized) | n/a | Simulation speed controls | Early end on outcome, plus optional fast-forward |
| Determinism | No RNG in physics | n/a | Mostly deterministic | Fixed-step deterministic engine with a step-based timer |

## Open decisions to surface for requirements

1. **Valve semantics** in place-then-play: a timed gate configured in Build, or a passive one-way flap. Live tapping is not recommended.
1. **"Sponge" semantics**: an elastic deflector (honest to the engine), or simulated absorption via destroy-in-shape (new mechanic).
1. **Star merge**: best-of across runs (recommended), or all three stars in a single run.
1. **Soft locks**: show locked levels as playable through deep links only (recommended), or fully open with "recommended order" badges.
1. **Hints**: free and star-neutral (recommended), or hint use blocks the Thrifty star.

## Sources

- [Hardcore Gaming 101: The Incredible Machine / The Even More Incredible Machine](https://www.hardcoregaming101.net/the-incredible-machine-the-even-more-incredible-machine/): Stop resets the machine to its start state and allows edits again; no fast-forward; pixel-perfect placement criticism; linear unlock after 47 open puzzles. MEDIUM.
- [Andy Matuschak, notes on The Incredible Machine](https://notes.andymatuschak.org/zEYFLQ99qFwbSswAV6NaX9T): a limited toolbox as scaffolding for solvability; parts introduced gradually and then combined. MEDIUM.
- [Pixelated Arcade: The Incredible Machine](https://pixelatedarcade.com/games/the-incredible-machine): per-puzzle part types with fixed quantities. MEDIUM.
- [TechSpot: The Incredible Machine 2](https://www.techspot.com/downloads/5852-the-incredible-machine-2.html): fixed scenery objects; no RNG in physics. MEDIUM.
- [Wikipedia: Fantastic Contraption (2008)](https://en.wikipedia.org/wiki/Fantastic_Contraption_(2008_video_game)): build zone, then Go; no piece or time limits. MEDIUM.
- [Nintendo: Poly Bridge](https://www.nintendo.com/en-gb/Games/Nintendo-Switch-games/Game-1318070.html), [Gaming Nexus: Poly Bridge 2 review](https://gamingnexus.com/Article/6117/Poly-Bridge-2-Review): under-budget completion and secret-world unlock. MEDIUM.
- [Cut the Rope wiki: Fabric Box](https://cuttherope.fandom.com/wiki/Fabric_Box), [Arkadium Cut the Rope scoring](https://support.arkadium.com/en/support/solutions/articles/44002797498--cut-the-rope-how-to-play-tips-scoring): 3 stars per level; stars gate boxes; you can pass with 0 stars. MEDIUM.
- [Wikipedia: Where's My Water?](https://en.wikipedia.org/wiki/Where%27s_My_Water%3F), [Macworld: Where's My Water 2](https://www.macworld.com/article/221881/where-s-my-water-2-launches-with-new-puzzles-more-ducks-and-hard-challenges.html), [CDM: Exploring Where's My Water 2](https://blogs.thecdm.ca/techspeak/2024/04/20/exploring-wheres-my-water-2/): 3 ducks filled by water; optional in WMW2; minimal tutorial with props introduced per level. MEDIUM (sources partly disagree on whether ducks are required).
- [MobyGames: Sugar, Sugar](https://www.mobygames.com/game/52605/): a cup is full at 100 grains; filters, gravity flip and wraparound. MEDIUM.
- [Game Developer: porting to a mobile touch interface](https://www.gamedeveloper.com/design/porting-to-a-mobile-touch-interface): drag offset to avoid finger occlusion. LOW–MEDIUM (single developer account).
- [Microsoft: guidelines for rotation](https://learn.microsoft.com/en-us/windows/uwp/design/input/guidelines-for-rotation): touch rotation pivot conventions. MEDIUM.
- [Chrome: Web Audio, Autoplay Policy and Games](https://developer.chrome.com/blog/web-audio-autoplay): AudioContext must be resumed from a user gesture. HIGH.
- [WebKit bug 264473](https://bugs.webkit.org/show_bug.cgi?id=264473), [adactio audiosession links](https://adactio.com/links/tags/audiosession): iOS Web Audio defaults to `ambient` and obeys the silent switch; `navigator.audioSession.type = "playback"` overrides this. LOW–MEDIUM (no Apple docs found). The owner may prefer to respect the silent switch, which is the default and arguably correct for opt-in sound.
- Repo source read 2026-10-11: `web/src/physics/simulation-worker.ts` (budgeted advance under `FRAME_STEP_BUDGET_MS`), `web/src/physics/worker-messages.ts` (ops: init, advance, capture, control, action, pointer, gravity; `generation` envelope), `crates/liquidfun-wasm/src/frame.rs` (`body_contact_count`), `crates/liquidfun/src/particle/query.rs` (`query_aabb`), `web/src/player/playground-session.ts` (localStorage loader pattern), CLAUDE.md repo rules (portrait frames, `data-hud-obstacle`, README plan coverage, scale legend). HIGH.

---
*Feature research for: v1.5 Puzzle Mode (place-then-play water puzzles in the liquidfun-rs playground)*
*Researched: 2026-10-11*
