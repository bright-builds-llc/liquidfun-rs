# Pitfalls Research

**Domain:** Adding twelve JavaScript LiquidFun testbed scenes to an existing native Rust engine and SolidJS playground (v1.3 Reference Testbed Scenes)
**Researched:** 2026-09-21
**Confidence:** HIGH for integration and policy pitfalls verified in this repository and the pinned JS testbed sources; MEDIUM for which exact engine gaps block each scene until a capability inventory is run against live APIs

## Executive Warning

v1.3 ports **recognizable** JS LiquidFun testbed scenes into the existing private WASM wrapper and SolidJS player. It is not a sealed C++/JS parity claim, not a public benchmark, and not crate publication. Dam Break stays; its recorded ≤ 3× native pair (`rust_over_cpp_ratio` ≈ 2.96 at stamp `2026-09-21T20-38-50Z`) must not be casually regressed by scene-only cheats. The playground still uses a **4-step-per-frame catch-up cap**. Commit `77fbd84` fixed force-buffer stacking after SolveForce; new force/impulse-heavy scenes can revive that class of bug.

Suggested phase kinds for planners (continue numbering after Phase 25):

1. **Engine capability** — missing or incomplete behavior a listed scene cannot run without
2. **Scene port** — geometry, particle recipes, and headless scene modules in `liquidfun-wasm`
3. **Player interaction** — SolidJS controls, pointer/keyboard mapping, catalog entries
4. **Honesty / docs** — credits, limitation language, no bit-exact or “Rust is N×” claims

Do **not** cut particle counts to fake smoothness. Do **not** treat visual resemblance as bit-exact or sealed differential parity.

## Critical Pitfalls

### Pitfall 1: Particle-Flag and Group-Flag Mismatches

**What goes wrong:**
A scene “runs” but behaves like ordinary water: elastic blobs melt, rigid clumps deform, barriers leak, powder sparks behave like fluid, or Drawing Particles modes paint the wrong material. Surface Tension looks like Color Mixer. Elastic Particles and Rigid Particles become indistinguishable.

**Why it happens:**
JS tests set precise `b2_*Particle` and `b2_*ParticleGroup` combinations. Ports reuse the nearest existing playground recipe (often `WATER`, or Jelly Drop’s `ELASTIC | SPRING` together) instead of the upstream bits. Group flags (`SOLID`, `RIGID`) are confused with particle flags (`ELASTIC`, `SPRING`, `TENSILE`). Drawing Particles also OR’s `REACTIVE` for wall/spring/elastic/barrier paints.

**How to avoid:**
Inventory each scene’s flag recipe from the pinned lfjs sources before coding. Map bit-for-bit to `ParticleFlags` / `ParticleGroupFlags` (already upstream-valued in `definition.rs` / `group/flags.rs`). Keep Elastic Particles’ three groups distinct: spring+solid, elastic+solid, elastic+solid with angular velocity. Rigid Particles use **group** `RIGID | SOLID`, not particle `ELASTIC`. Drawing Particles must preserve mode→flag tables including barrier combinations and reactive OR. Add a headless assert that created particles’ flag bits match the recipe.

**Warning signs:**
Jelly Drop flags copied into Elastic/Rigid scenes; Color Mixer (`COLOR_MIXING` only) reused for Surface Tension; `WATER` default left on Powder/Sparky groups; no unit check of `flags.contains(...)` after group create.

**Phase to address:**
**Engine capability** first if a required flag’s solver path is missing or no-op; otherwise **scene port** with recipe tests. **Honesty / docs** if a mode is deliberately incomplete.

---

### Pitfall 2: Confusing Color Mixing with Surface Tension

**What goes wrong:**
Surface Tension is shipped as a recolored Color Mixer: blobs mix hues but do not show tensile attraction/clustering. Or tensile-only particles are used without `COLOR_MIXING`, so the official three-color demo loses its mixing story. Visitors and docs treat the existing Color Mixer as “already covering” Surface Tension.

**Why it happens:**
Both demos use colored particle groups in a bowl-like basin. The playground already has Color Mixer with `COLOR_MIXING` and tunable mix strength. Upstream Surface Tension sets `b2_tensileParticle | b2_colorMixingParticle` on all three groups plus a small radius (`0.035`) and damping.

**How to avoid:**
Keep Color Mixer as the original v1.1 scene. Port Surface Tension as its own scene with **both** `TENSILE` and `COLOR_MIXING`. Verify tensile solver participation (aggregate tensile pass / attraction), not only color channel blending. Document that Color Mixer is inspiration-adjacent, not the Surface Tension port.

**Warning signs:**
One shared module behind two catalog names; Surface Tension radius/damping match Color Mixer (`0.05692`) instead of lfjs (`0.035` / damping `0.2`); README saying “color scenes done” after only Color Mixer.

**Phase to address:**
**Scene port** (distinct recipe). **Engine capability** if tensile behavior is incomplete. **Honesty / docs** for catalog wording.

---

### Pitfall 3: Lifetime, Destruction, and Group-Teardown Blind Spots

**What goes wrong:**
Sparky sparks never fade or never despawn; VFX slots leak until the particle cap trips; Drawing Particles keeps a stale `lastGroup` after destruction and joins the wrong group or panics; Soup engraving leaves overlapping solids and particles; age-based destruction is enabled globally and silently deletes scene content.

**Why it happens:**
Sparky’s `ParticleVFX` uses powder groups, manual color fade, finite remaining lifetime, then `DestroyParticles`. Drawing Particles implements `ParticleGroupDestroyed` to clear `lastGroup`. Soup / Soup Stirrer call `DestroyParticlesInShape` after placing fixtures. The playground session steps with `NoDecisionHook` and scene hooks are `on_advance` / pointer / controls—not a full destruction-listener bridge. Water Wheel already uses `with_destruction_by_age(true)` for emitters; copying that pattern onto static-group scenes deletes particles.

**How to avoid:**
Treat lifetime and destruction as **scene requirements**, not polish. For Sparky: engine particle lifetime (or explicit per-step destroy) plus group destroy; fade can stay in scene code reading/writing colors if the public API allows, or approximate with lifetime + alpha if colors are snapshot-only—document the choice. For Drawing Particles: clear join-target on group destruction via lifecycle events or explicit nulling on destroy paths. For Soup*: implement destroy-in-shape as query-AABB / shape test + `mark_particle_for_destruction` (or a thin engine helper) before calling the scene “ported.” Never enable destruction-by-age unless the scene is an emitter with plateau tests.

**Warning signs:**
`with_destruction_by_age(true)` on non-emitter testbed ports; Sparky particle count only climbs; Drawing paint after delete joins a destroyed group; no test that Soup fixtures carve a particle-free pocket.

**Phase to address:**
**Engine capability** for destroy-in-shape / lifecycle exposure needed by WASM scenes; **scene port** for Sparky VFX and Soup carving; **player interaction** if destruction must sync with UI join state.

---

### Pitfall 4: Theo Jansen / Wave Machine Without Live Revolute Motors

**What goes wrong:**
Wave Machine is a static tank of water. Theo Jansen is a collapsed sculpture or a free-floating chassis that never walks. Keyboard motor toggles appear in the UI but do nothing. Soft distance “suspension” is omitted, so legs jitter violently or lock.

**Why it happens:**
Both scenes need a **revolute motor that changes every step** (Wave Machine: `SetMotorSpeed(0.05 * cos(t) * π)`; Theo Jansen: enable/speed via keys). Water Wheel already creates a revolute joint **without** a motor—easy to copy the wrong pattern. Theo Jansen also needs many soft `DistanceJoint`s (`frequencyHz = 10`, `dampingRatio = 0.5`) plus filter `groupIndex = -1` on chassis/legs. Motor mutation APIs exist (`set_revolute_motor_enabled`, `set_revolute_motor_speed`, …) but must be wired through scene hooks and WASM controls.

**How to avoid:**
Capability-check motor create + per-step speed mutation before claiming Wave Machine. Port Theo Jansen legs with soft distance joints and collision filtering, not welded polygons. Map `a/s/d/m` (and limit key if kept) to labeled controls or keyboard handlers that call joint mutators on the live joint id. Drive Wave Machine speed from `on_advance` using simulated time (`n * dt`), not wall-clock alone, so pause/reset stay honest under the 4-step cap.

**Warning signs:**
Revolute created with `enable_motor: false` and never updated; Wave Machine `on_advance` empty; Theo Jansen without distance joints; motor speed updated from JS rAF without going through the session step budget.

**Phase to address:**
**Engine capability** if soft distance or motor mutation is incomplete in the path scenes use; **scene port** for joint graph; **player interaction** for Theo Jansen controls.

---

### Pitfall 5: Sparky Contact Callbacks Never Fire in the Playground Session

**What goes wrong:**
Circles bounce; no spark VFX. Or sparks spawn every frame from permanent contacts. Or contact handling requires storing `World` borrows across steps and breaks the safe hook model.

**Why it happens:**
Upstream Sparky sets `world.SetContactListener(this)` and uses `BeginContactBody` with fixture/body `userData` tags. The WASM session always steps with `NoDecisionHook`. Scene `SceneHooks` run around advance but do not receive begin-contact events. Tagging bodies in user associations without reading contact transitions yields a silent no-op.

**How to avoid:**
Do not pretend Sparky works without a contact path. Prefer one of: (1) a scene-scoped `CollisionDecisionHook` / step hook that records begin-contact points for sparkable bodies into a side buffer consumed in `on_advance`, or (2) post-step inspection of contact transitions/observations already produced by the step, filtered by body tags. Fire VFX once per new contact (edge-trigger), not while sleeping contacts persist. Keep hooks free of re-entrant world mutation beyond deferred commands the engine already allows.

**Warning signs:**
Sparky port with only `on_advance` gravity and no contact buffer; `NoDecisionHook` left unconditional for all scenes; sparks while circles are at rest in persistent contact; `unsafe` or raw pointers added “to match C++ listeners.”

**Phase to address:**
**Engine capability** / WASM session hook plumbing first; **scene port** for VFX; **player interaction** only for camera/UI.

---

### Pitfall 6: Frame-Budget Blowups from Testbed-Scale Particle Counts

**What goes wrong:**
A scene opens, then the tab hitch-loops: physics falls behind, the 4-step cap drops time on the floor, Drawing Particles or Sparky VFX push counts toward the system maximum, and Dam Break or other scenes feel worse after shared hot-path edits. Someone “fixes” FPS by shrinking particle counts, raising `dt`, lifting `MAX_STEPS_PER_FRAME`, or skipping capture—masking cost and breaking comparability.

**Why it happens:**
JS testbed radii are often `0.025`–`0.035` (Wave Machine, Impulse, Liquid Timer, Soup, Elastic, Surface Tension, Particles), which pack far more particles per shape than Dam Break’s `0.06324555` Medium recipe. Drawing Particles can paint indefinitely. Sparky allocates up to 50 VFX groups. The player already caps catch-up at **4 steps/frame** (`web/src/physics/clock.ts`). Native Dam Break performance work is recent and fragile to silent recipe changes.

**How to avoid:**
Keep testbed radii and shapes recognizable; enforce an explicit **maximum particle count** and fail closed or stop emission when full (Fountain pattern). Do **not** reduce counts solely to look smooth. Do **not** raise the 4-step cap to hide cost. Prefer: plateau tests, emission/VFX budgets, honest stutter notes, and profiling shared kernels if many scenes are slow—not scene-local cheats that change Dam Break Medium. Spot-check Dam Break after shared engine edits (`just playground-scene-spot` / pair recipes already used in v1.2).

**Warning signs:**
PRs that change Dam Break column/row counts “for the new gallery”; `MAX_STEPS_PER_FRAME > 4`; Drawing Particles without a cap; Wave Machine radius quietly bumped to `0.06+` without documenting a deliberate playground adaptation; WASM vs C++ FPS tables.

**Phase to address:**
**Scene port** (caps, plateau tests); **player interaction** (keep clock contract); **honesty / docs** (stutter and adaptation notes); **engine capability** only for real shared hot-path fixes—not count cuts.

---

### Pitfall 7: Treating Visual Resemblance as Bit-Exact or Sealed Parity

**What goes wrong:**
Catalog copy, README, or milestone language claims “LiquidFun testbed parity,” “identical to the JS demo,” or sealed differential coverage because water sloshes and colors look similar. Elastic/Rigid ports inherit upstream’s own “This test is buggy” status but are marketed as correct. Failed oracle cases are waived because “it looks fine.”

**Why it happens:**
Hobby milestone success is visitor-facing recognition. lfjs Elastic Particles and Rigid Particles files literally comment that the tests are buggy. Differential infrastructure exists and invites overclaiming when a scene is merely demoable.

**How to avoid:**
Acceptance language: **recognizable port** on this engine in the existing player. Record known differences (timestep coupling, missing callback fidelity, soft-joint tuning, upstream-buggy demos). Use headless smoke and Chromium player tests for open/reset/interact—not bit-exact frame hashes against JS. Optional differential probes are evidence of specific behaviors, not a sealed matrix for twelve scenes.

**Warning signs:**
“Parity” in UI strings; comparing playground FPS to `oracle-release`; deleting failing tests because the canvas looks right; claiming Elastic Particles fixed upstream bugs without evidence.

**Phase to address:**
**Honesty / docs** throughout; verification criteria in **scene port** and **player interaction** phases.

---

### Pitfall 8: Scene-Only Cheats That Regress Dam Break Performance or Correctness

**What goes wrong:**
Gallery work reintroduces force-buffer stacking, always-on full-world clones on new code paths, extra per-particle WASM calls, or shared solver “simplifications.” Dam Break explodes again after ~1 minute, or the unprofiled pair jumps well above ~3×. Or Dam Break Medium is silently altered so old stamps look improved.

**Why it happens:**
Impulse / Soup Stirrer apply large group or body forces every interaction/step—the same force-lane family as the Dam Break wall-velocity bug. Shared particle kernels tempt “fast paths” under playground pressure. The ≤ 3× stamp is SHA-bound to pre-`77fbd84` HEAD; confusion about what may change is high.

**How to avoid:**
Preserve Dam Break Medium literals and the force-buffer zeroing invariant. Any `crates/liquidfun` change for scenes must re-run Dam Break headless regression and, for shared hot paths, consider an unprofiled pair or spot-check—not a scene FPS anecdote. Do not edit Dam Break counts/radius to make the gallery feel fair. Do not add default SIMD/Rayon or weaken `unsafe_code = "forbid"`.

**Warning signs:**
Force/impulse scenes without non-stacking tests; Dam Break constants edited in the same PR as Wave Machine; “temporary” HashMap neighborhoods; playground-only physics forks.

**Phase to address:**
**Engine capability** (correctness under new force/contact use); **honesty / docs** (performance claims stay stamp-bound); spot-check after shared edits.

---

### Pitfall 9: Missing Query/Destroy and Group Force APIs Faked in JavaScript

**What goes wrong:**
Soup looks like Particles-with-props because solids overlap fluid. Impulse clicks do nothing or apply force to one particle. Drawing Particles cannot erase. Ports push destruction and impulses into the SolidJS layer by inventing positions—violating “native Rust behavior.”

**Why it happens:**
lfjs relies on `DestroyParticlesInShape`, `ParticleGroup::ApplyForce` / `ApplyLinearImpulse`, and continuous destroy-and-paint. A grep-level gap: convenience destroy-in-shape is not a named public mirror of the JS helper; group-wide force/impulse may need world/group APIs already used internally. The temptation is to approximate in JS.

**How to avoid:**
Implement missing operations in Rust (engine or WASM scene helper using `query_aabb_with_particles` + mark-for-destruction, and group apply force/impulse). Keep SolidJS as controls and rendering only. Add unit tests: Impulse changes group momentum; Soup carve removes particles under fixtures; Drawing erase reduces count in the brush AABB.

**Warning signs:**
`apply_pointer` that only tweens mesh colors; TODO comments “destroy later”; Impulse implemented as camera shake.

**Phase to address:**
**Engine capability** for reusable APIs; **scene port** for Soup/Impulse/Drawing; never **player interaction** alone.

---

### Pitfall 10: Player Interaction Gaps for Drawing, Impulse, Stirrer, and Theo Jansen

**What goes wrong:**
Scenes are watch-only: Drawing Particles cannot paint; Impulse cannot poke; Soup Stirrer cannot toggle the prismatic joint; Theo Jansen cannot reverse. Or pointer coords are wrong after resize (v1.1 lesson), and paint spawns in the wrong place. Catalog Reset does not clear join state / VFX / motor time.

**Why it happens:**
v1.1 scenes mostly use one gesture and preset selects. These twelve need richer keyboard and pointer modes (Drawing’s many material keys; Impulse `l`/`f`; Stirrer click-to-toggle joint; Theo Jansen motor keys). Shared `pointer_action` and control plumbing exist but each scene must map them.

**How to avoid:**
Reuse the shared CSS-bound unproject and `pointer_action` path. Give each interactive scene labeled controls and figcaption instructions. Reset must remount to defaults **and** drop Drawing `lastGroup`, Sparky VFX slots, Wave Machine `time`, and motor speeds. Extend Chromium smoke for at least one gesture per interactive scene without requiring a full browser matrix.

**Warning signs:**
Catalog entry with no figcaption gesture; Drawing only on keydown without mouse move paint; Reset leaves sparks or motor speed; new absolute pointer math forked per scene.

**Phase to address:**
**Player interaction** after the scene’s physics path works headlessly; **honesty / docs** for control credits vs upstream keys.

---

### Pitfall 11: Leaking WASM / C++ / Submodule Requirements into the Published Crate

**What goes wrong:**
`liquidfun` gains `wasm-bindgen`, path deps on `liquidfun-wasm`, or reference/CMake hooks “to share scene code.” Ordinary `cargo build -p liquidfun` needs the submodule. Package isolation fails.

**Why it happens:**
Twelve scenes create pressure to share code. Scenes already live correctly in private `liquidfun-wasm`.

**How to avoid:**
Keep scenes and playground glue in private crates/web. Engine additions needed by scenes stay native, headless, and free of WASM/C++ deps. Re-run package isolation checks when engine APIs expand.

**Warning signs:**
`wasm-bindgen` in `crates/liquidfun/Cargo.toml`; scenes under `crates/liquidfun/src`; build.rs probing `third_party/liquidfun`.

**Phase to address:**
**Engine capability** API design and packaging checks; reinforced in **honesty / docs**.

---

### Pitfall 12: Soup Stirrer “Inheritance” and Prismatic Joint Lifecycle Bugs

**What goes wrong:**
Soup Stirrer duplicates Soup incorrectly (missing carve, different damping). Toggling the prismatic joint leaks joints or leaves the stirrer unconstrained with unbounded force integration. Click-toggle and key-toggle disagree.

**Why it happens:**
lfjs builds Stirrer by constructing `TestSoup` then adding a stirrer, `DestroyParticlesInShape`, prismatic joint to `g_groundBody`, and per-step oscillatory `ApplyForceToCenter` while speed &lt; max. JS “inheritance” is ad hoc.

**How to avoid:**
Share Soup setup via a Rust helper used by both scenes, or build Stirrer by composing explicit steps—not by hoping catalog order initializes Soup first. On toggle, destroy/create the joint through typed APIs; keep force application in `on_advance` with the same in-soup and max-speed guards. Test toggle twice and Reset.

**Warning signs:**
Two divergent basin geometries; joint toggle only sets a bool without `destroy_joint`; force applied when joint is null without speed guard.

**Phase to address:**
**Scene port** (shared Soup foundation); **player interaction** for toggle; **engine capability** if prismatic create/destroy is incomplete.

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Reuse Color Mixer as Surface Tension | Fast catalog entry | Wrong physics story; docs lie | Never as the Surface Tension port |
| Copy Jelly `ELASTIC\|SPRING` into all soft scenes | One recipe | Elastic vs spring vs rigid demos collapse | Never for Elastic/Rigid/Drawing modes |
| Enable destruction-by-age on every particle system | Sparks eventually vanish | Static groups evaporate | Emitter scenes with plateau tests only |
| Lift 4-step catch-up or shrink particle counts for FPS | Looks smoother locally | Hides cost; breaks Dam Break comparability; false “perf win” | Never for milestone acceptance |
| Approximate destroy-in-shape / contacts in SolidJS | Ships UI sooner | Non-native physics; undebuggable drift | Never |
| Skip soft distance joints on Theo Jansen | Fewer joint APIs | Walker shreds or locks; not recognizable | Never if claiming Theo Jansen |
| Always `NoDecisionHook` for all scenes | Simple session | Sparky cannot spark | Only for scenes that need no contacts/listeners |
| Quiet Dam Break recipe edits for gallery fairness | Uniform FPS | Invalidates ≤ 3× evidence and Medium identity | Never |
| Market “parity” because canvas looks close | Easy messaging | Trust debt; fights sealed-oracle meaning | Never |
| Default SIMD/Rayon to survive Wave Machine | Faster on one laptop | Determinism/policy breach | Never in this milestone |

## Integration Gotchas

Common mistakes when connecting new scenes to **this** repo’s engine, WASM session, and SolidJS player.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `SessionCore` + `NoDecisionHook` | Expect Sparky contacts without a hook | Scene-scoped contact capture or transition drain |
| `SceneHooks::on_advance` | Put Wave Machine motor speed in rAF only | Update motors inside the session step / `on_advance` with sim time |
| `pointer_action` + camera | New per-scene unproject | Reuse shared CSS-bound unproject from v1.1 |
| `MAX_STEPS_PER_FRAME = 4` | Raise cap for heavy scenes | Keep cap; budget particles/VFX; document stutter |
| Dam Break Medium constants | Tweak rows/radius while porting Wave Machine | Leave Medium literals alone; new scenes get own constants |
| Force buffer after SolveForce | ApplyForce paths that re-stack forces | Preserve zero-on-consume; add Impulse/Stirrer regressions |
| Package isolation | Move scenes into `liquidfun` for reuse | Keep scenes in `liquidfun-wasm` / web |
| Catalog Reset | Remount presets only | Also clear join targets, VFX, motor time, joints |
| Credits | “Based on LiquidFun” without scene source links | Per-scene implementation vs inspiration vs notice links (v1.1 pattern) |
| `just web-player-smoke` | Only Dam Break still covered | Extend smoke for new scene open/reset/one gesture |
| Upstream lfjs “buggy” Elastic/Rigid | Silent “fixes” marketed as parity | Port recognizably; note upstream caveat in honesty docs |
| Prismatic / revolute motors | Create-only, no mutation API wiring through WASM | Expose control/action → `set_revolute_motor_*` / joint destroy |

## Performance Traps

Patterns that work for current six scenes but fail on these twelve. Thresholds refer to this playground’s catch-up and caps unless noted.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Radius `0.025`–`0.035` packed boxes | Multi-thousand particles; hitch under 4-step cap | Caps + plateau tests; honest stutter notes; no silent radius inflation | First Wave Machine / Soup open on mid-tier laptops |
| Drawing Particles unbounded paint | Count → maximum; then errors or frozen fluid | Max count; stop paint when full; optional brush budget | Continuous drag in under a minute |
| Sparky 50 VFX groups | Spikes on multi-contact frames | Ring buffer already in JS—keep it; destroy before overwrite | Ball pile contacts |
| Raising catch-up above 4 | Tab jank “fixed,” physics races ahead after hide | Keep `MAX_STEPS_PER_FRAME = 4`; pause hidden tabs | Hidden-tab return (v1.1) |
| Shrinking Dam Break to match new scenes | Fake gallery FPS; pair evidence meaningless | Separate scene constants; never edit Medium for cosmetics | Any Dam Break timing comparison |
| Per-particle WASM calls for VFX color | Frame time dominated by bridge | Batch color updates in Rust; keep copied frame lanes | Sparky fade loops |
| Shared kernel “simplification” for gallery | Dam Break regresses toward old 100×-class cost | Spot-check Dam Break after shared edits | After first shared solver PR |
| Impulse/Stirrer force stacking bug revival | Exploding velocities ~tens of seconds in | Non-stacking force-buffer tests + scene soak | Impulse spam or long Stirrer runs |

## Security Mistakes

Domain-specific issues for this scene-port milestone (not generic web OWASP).

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting raw pointer/world coordinates from the page without finite checks | WASM trap / session death | Validate finite inputs at WASM boundary (v1.1 pattern) |
| Expanding cdylib surface with arbitrary scriptable joint/memory APIs | Harder audit; accidental memory footguns | Keep narrow control/action/pointer APIs; no raw pointers in frames |
| Copying large upstream assets or proprietary traces into the repo | License/notice drift | Link pinned lfjs sources; retain existing notice discipline |
| Committing machine-specific perf traces as “scene proof” | Path/PII leakage; false claims | Gitignore profiles; honesty docs cite stamps carefully |

## UX Pitfalls

Honesty and interaction pitfalls for visitors (not a visual redesign).

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Watch-only “Drawing Particles” | Feature feels broken | Pointer paint + mode controls + instructions |
| Surface Tension labeled but only mixes colors | Misleading science demo | Tensile+mixing recipe or honest limitation label |
| No mention of stutter on dense scenes | Users blame their device or the whole engine | Short note: catch-up capped; dense testbed radii |
| “Complete LiquidFun testbed” gallery title | Overclaim | “Reference testbed scenes” / recognizable ports |
| Reset leaves motors/sparks running | Confusing second run | Full session remount semantics |
| Keyboard-only Theo Jansen on mobile | Unusable walker | On-screen motor controls + optional keys |
| Catalog duplicates Dam Break under a new name | Noise | Keep one Dam Break; add only the twelve missing lfjs scenes |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Flag recipes:** Scene renders — verify particle/group flag bits match lfjs (including Drawing mode table and Surface Tension `TENSILE\|COLOR_MIXING`)
- [ ] **Elastic vs Rigid vs Jelly:** Soft blobs move — verify Elastic uses spring/elastic **particle** flags; Rigid uses **group** `RIGID\|SOLID`; Jelly Drop unchanged
- [ ] **Wave Machine:** Water present — verify revolute motor speed updates each sim step
- [ ] **Theo Jansen:** Assemblage visible — verify soft distance joints, motor enable/speed controls, filter group −1
- [ ] **Sparky:** Circles collide — verify begin-contact edge triggers VFX; powder + lifetime/destroy; ring buffer reuse
- [ ] **Soup carve:** Fixtures visible — verify particles under solids were destroyed (destroy-in-shape or equivalent)
- [ ] **Soup Stirrer:** Inherits Soup — verify shared foundation + prismatic toggle + oscillatory force guards
- [ ] **Impulse:** Click ripples — verify group ApplyForce vs ApplyLinearImpulse mode, not camera shake
- [ ] **Drawing Particles:** Paint works — verify destroy-in-brush, join/lastGroup clear on destroy, reactive OR rules
- [ ] **Liquid Timer / Particles:** Looks wet — verify tensile+viscous (Liquid Timer) vs plain water (Particles) recipes and edge fixture layout
- [ ] **Catch-up cap:** Scene “optimized” — verify `MAX_STEPS_PER_FRAME` still 4; counts not silently slashed for FPS
- [ ] **Dam Break:** Untouched Medium literals — verify no drive-by recipe edits; force-buffer non-stacking still tested
- [ ] **Parity language:** README excited — verify “recognizable port,” no bit-exact / sealed / “Rust is N×” claim
- [ ] **Package isolation:** Engine API added — verify `liquidfun` still free of WASM/C++ deps
- [ ] **Reset / smoke:** Catalog entry exists — verify remount clears scene state; Chromium smoke covers open/reset/gesture

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong flags shipped | MEDIUM | Fix recipes + flag asserts; do not “tune radius” to hide melting |
| Sparky without contacts | MEDIUM | Add session hook/transition path; keep VFX code, wire edge triggers |
| Wave Machine static tank | LOW | Enable motor + `on_advance` speed update; retest pause/reset |
| Theo Jansen collapse | HIGH | Restore soft distance joints and filters; retest motor controls |
| FPS “fixed” by cutting counts / raising catch-up | MEDIUM | Revert cheats; restore caps; document stutter; profile shared paths if needed |
| Dam Break explode or ≫3× after gallery PR | HIGH | Bisect shared force/solver edits; restore force-buffer zeroing; re-run Dam Break regression/pair |
| Overclaim parity in docs | LOW | Rewrite to recognizable-port language; remove sealed/bit-exact wording |
| Destroy-in-shape faked in JS | MEDIUM | Move carve/erase into Rust; delete JS physics approximations |
| Package isolation broken | HIGH | Move scenes back to private crates; restore isolation checks before merge |
| Stirrer joint leak | LOW | Fix destroy/create toggle; add double-toggle + Reset tests |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls. Use phase **kinds** (numbering continues after 25).

| Pitfall | Prevention phase kind | Verification |
|---------|----------------------|--------------|
| 1. Flag / group-flag mismatches | Engine capability (if solver no-op) → Scene port | Flag-bit asserts per scene recipe |
| 2. Color mixing vs surface tension | Scene port + Honesty/docs | Distinct Surface Tension module with `TENSILE\|COLOR_MIXING` |
| 3. Lifetime / destruction / lastGroup | Engine capability → Scene port | Sparky despawn; Drawing join clear; Soup carve tests |
| 4. Revolute motors (Theo Jansen / Wave Machine) | Engine capability → Scene port → Player interaction | Per-step motor speed; keyboard/controls mutate joint |
| 5. Sparky contact callbacks | Engine capability (session hook) → Scene port | Edge-triggered VFX on tagged body contacts |
| 6. Frame-budget blowups | Scene port + Player interaction + Honesty/docs | Caps held; catch-up still 4; no count-slash acceptance |
| 7. Visual resemblance as parity | Honesty/docs (all phases) | Copy review; no sealed/bit-exact claims |
| 8. Dam Break perf/correctness regression | Engine capability + Honesty/docs | Dam Break regression + optional pair/spot-check after shared edits |
| 9. Missing destroy/force APIs | Engine capability → Scene port | Impulse/Soup/Drawing headless tests |
| 10. Interaction gaps | Player interaction | Smoke: paint/poke/toggle/motor per interactive scene |
| 11. WASM/C++ in published crate | Engine capability | Package isolation check green |
| 12. Soup Stirrer composition / prismatic | Scene port → Player interaction | Shared Soup helper; joint toggle + Reset |
| Credits / upstream buggy demos | Honesty/docs | Per-scene inspiration links; Elastic/Rigid caveat noted |

## Sources

- Pinned JS testbed scenes under `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/` (`testDrawingParticles.js`, `testElasticParticles.js`, `testImpulse.js`, `testLiquidTimer.js`, `testParticles.js`, `testRigidParticles.js`, `testSoup.js`, `testSoupStirrer.js`, `testSparky.js`, `testSurfaceTension.js`, `testTheoJansen.js`, `testWaveMachine.js`) — HIGH confidence for required flags, joints, contacts, and interactions
- Official testbed listing: [LiquidFun JS testbed](http://google.github.io/liquidfun/testbed/index.html) — HIGH for scene inventory
- Repository playground constraints: `web/src/physics/clock.ts` (`MAX_STEPS_PER_FRAME = 4`); `crates/liquidfun-wasm/src/session.rs` (`NoDecisionHook`); `crates/liquidfun-wasm/src/scene.rs` (six-scene catalog and `SceneHooks`)
- Existing recipes: `color_mixer.rs` (`COLOR_MIXING` only); `jelly_drop.rs` (`ELASTIC | SPRING`); `water_wheel.rs` (revolute **without** motor; destruction-by-age emitter)
- Engine APIs: `ParticleFlags` bit values in `crates/liquidfun/src/particle/definition.rs`; revolute motor mutators in `crates/liquidfun/src/world/joint/revolute.rs`; soft `DistanceJointDef::with_frequency` / `with_damping_ratio`
- Force-buffer incident: `.planning/debug/knowledge-base.md` (`dam-break-wall-velocity`); commit `77fbd84`; PROJECT.md v1.2/v1.3 notes on ≤ 3× stamp vs later HEAD — HIGH for regression risk
- Prior milestone pitfalls (integration patterns only; not copied wholesale): `.planning/research/v1.1/PITFALLS.md` (catch-up, pointer unproject, disposal); `.planning/research/v1.2/PITFALLS.md` (Dam Break pair honesty, no WASM-vs-C++, no count cheats as perf wins)
- Hobby scope: `PROJECT-SCOPE.md`, `.planning/PROJECT.md` v1.3 goal (recognizable ports; not sealed parity or crate release)

---
*Pitfalls research for: v1.3 Reference Testbed Scenes — adding twelve JS LiquidFun scenes to liquidfun-rs*
*Researched: 2026-09-21*
