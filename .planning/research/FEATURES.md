# Feature Research

**Domain:** LiquidFun JavaScript testbed scene ports for the existing SolidJS playground
**Milestone:** v1.3 Reference Testbed Scenes
**Researched:** 2026-09-21
**Confidence:** HIGH for scene setup and recognizable behavior (pinned JS + C++ sources); MEDIUM for engine-gap severity until each scene is built against the live `liquidfun` public API

**Already shipped (do not re-scope as new work):** Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel; shared SolidJS player (play/pause/reset), catalog previews, hash routes, pointer interaction, GitHub Pages. Catalog source: `web/src/catalog/scenes.ts`.

**Authority for missing scenes:** Official JS menu in `third_party/liquidfun/liquidfun/Box2D/lfjs/index.html` (same thirteen entries as [google.github.io/liquidfun](https://google.github.io/liquidfun/) `lfjs/index.html`). Behavior described from `lfjs/testbed/tests/test*.js` with C++ Testbed headers as cross-checks.

## Feature Landscape

### Table Stakes (Users Expect These)

Features visitors assume exist once the milestone claims “every JS testbed scene we do not already have.” Missing these = incomplete ports or broken catalog UX.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Drawing Particles** scene | In the official JS menu; interactive particle painting is the flag showcase | HIGH | Empty U-shaped box; drag draws radius-0.2 circle groups; destroy-under-cursor then create. Mode keys (E/P/R/S/T/V/W/B/H/N/M/F/C/Z/X) select flags/groups. Sources: `testDrawingParticles.js`, `DrawingParticles.h`. Recognizable: paint with at least water + a few material modes (elastic/rigid/wall/color); joining consecutive strokes. Full keyboard palette is chrome. |
| **Elastic Particles** scene | Menu entry; shows spring vs elastic solids | MEDIUM | Basin + walls; red spring circle, green elastic circle, blue elastic box (angled, spinning); falling dynamic circle. Sources: `testElasticParticles.js`, `ElasticParticles.h`. Recognizable: three colored soft blobs that bounce/recover differently when hit. JS file notes upstream buginess — recognizable motion beats bit-exact soft response. |
| **Impulse** scene | Menu entry; teaches group force/impulse | LOW–MEDIUM | Loop chain box; one particle box group; click inside applies force (default) or linear impulse toward click from box center. Keys `l`/`f` toggle. Sources: `testImpulse.js`, `Impulse.h`. Recognizable: click sloshes the blob as a unit. Impulse vs force toggle is table-stakes control; C++ particle-parameter UI is chrome. |
| **Liquid Timer** scene | Menu entry; memorable “hourglass” flow | MEDIUM | Loop walls; tensile+viscous slab at top; edge baffles forming a vertical gap and zig-zag shelves into four bottom columns. Sources: `testLiquidTimer.js`, `LiquidTimer.h`. Recognizable: liquid drains through the gap and zig-zags into columns. Particle-parameter switching (C++ only) is chrome. |
| **Particles** scene | Menu entry; baseline “blob falls, ball falls” | LOW | Floor + slanted side walls; large red water circle (r=2); dynamic circle above. Sources: `testParticles.js`, `Particles.h`. Recognizable: water splatters in the basin while a rigid ball drops into it. Damping 0.2 and C++ particle-type picker are optional. |
| **Rigid Particles** scene | Menu entry; rigid-group counterpart to Elastic | MEDIUM | Same basin layout as Elastic; three RGB rigid+solid groups (circles + spinning box); falling ball. Sources: `testRigidParticles.js`, `RigidParticles.h`. Recognizable: colored clumps move as rigid bodies and collide with the ball. Same “JS buggy” caveat as Elastic. |
| **Soup** scene | Menu entry; liquid with floating “ingredients” | MEDIUM | Basin; water box group; carve holes then place circle + two squares + three edge “noodles” with mass data. Sources: `testSoup.js`, `Soup.h`. Recognizable: broth with bobbing solids. `DestroyParticlesInShape` at spawn is part of the look. |
| **Soup Stirrer** scene | Menu entry; animated stir of Soup | MEDIUM–HIGH | Builds on Soup; damping 1.0; dynamic circle stirrer; prismatic joint (x-axis); rotating force each step; click/`t` toggles joint. Sources: `testSoupStirrer.js`, `SoupStirrer.h`. Recognizable: auto-stirring paddle; joint unlock lets it roam. Requires Soup + prismatic + per-step force. |
| **Sparky** scene | Menu entry; collision sparks | HIGH | Tall chamber; six large dynamic circles stacked; on body contact spawn powder particle VFX that expand then fade/destroy. Sources: `testSparky.js`, `Sparky.h`. Recognizable: colliding balls throw short-lived colorful particle bursts. Contact listener + group destroy + color fade are table stakes; exact VFX pool size (50) is chrome. |
| **Surface Tension** scene | Menu entry; tensile + color mixing | MEDIUM | Basin; three tensile+colorMixing groups (red/green circles, blue box); falling ball; color buffer updates. Sources: `testSurfaceTension.js`, `ParticlesSurfaceTension.h`. Recognizable: blobs bead and mix color on contact (same family as Color Mixer, different flags/layout). |
| **Theo Jansen** scene | Menu entry; walking machine + particles | HIGH | Ground + end walls; 40 balls; chassis/wheel + six legs (distance + revolute); motorized revolute; particle slab on top. Keys a/s/d/m(/l). Sources: `testTheoJansen.js`, `TheoJansen.h`. Recognizable: walker walks (or tries) while particle rain sits atop. Direction/motor controls are table stakes; limit toggle is chrome. |
| **Wave Machine** scene | Menu entry; default JS selection; iconic slosh | MEDIUM | Motorized revolute box (four thin walls); particle fill; `Step` sets `motorSpeed = 0.05 * cos(t) * π`. Sources: `testWaveMachine.js`, `WaveMachine.h`. Recognizable: rocking tank that keeps making waves. No pointer required. C++ particle-parameter UI is chrome. |
| Catalog entry per new scene | Matches existing six-scene UX | LOW | Title, description, SVG preview, hash route, `ready`, credits — extend `web/src/catalog/scenes.ts` pattern. |
| Shared player reuse | Visitors already know play/pause/reset | LOW | Do not fork a second player; remount on reset like current scenes. |
| Per-scene credits | Honest inspiration/implementation links | LOW | Point at pinned `lfjs/testbed/tests/test*.js` and matching `Testbed/Tests/*.h` under commit `7f204021…`. |
| One meaningful interaction (where JS has one) | Existing playground promise | LOW–MEDIUM | Drawing: drag-paint. Impulse: click-push. Soup Stirrer: click-toggle joint. Theo Jansen: direction/motor. Sparky: watch collisions (optional poke). Passive/Liquid Timer/Particles/Rigid/Soup/Surface Tension/Wave Machine: watch-first is enough; optional pointer is differentiator. |
| Bounded lifecycle | Prevent WASM hang after long runs | MEDIUM | Cap particle counts; Sparky must destroy expired VFX groups; Drawing must not unbounded-grow without destroy. |

### Differentiators (Competitive Advantage)

Not required for “recognizable port,” but valuable for this hobby playground vs a raw testbed dump.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Preset controls mapped from JS keys | Discoverable without memorizing E/P/R keys | LOW–MEDIUM | Drawing Modes as labeled presets; Impulse force vs impulse; Theo Jansen direction; Soup Stirrer joint on/off. Prefer playground control patterns over freeglut key legends. |
| Honest “inspired by JS testbed” copy | Sets expectation vs sealed parity | LOW | Aligns with PROJECT.md: recognizable ports, not certification. |
| Keep original six scenes alongside | Catalog stays distinctive | LOW | Fountain / Float or Sink / Color Mixer / Jelly Drop / Water Wheel are not in the JS menu; Dam Break is shared. |
| Cross-links between related scenes | Teaching path | LOW | Elastic ↔ Jelly Drop; Surface Tension ↔ Color Mixer; Soup ↔ Soup Stirrer; Particles as baseline. |
| Sparky color fade in WASM color lane | Matches showcase spectacle | MEDIUM | Existing frame already carries particle colors (Color Mixer). |
| Optional pointer nudge on watch-first scenes | Consistency with v1.1 pointer work | LOW | Nice; not required for recognizability. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Sealed C++ / bit-exact parity for every scene | “Real port” prestige | Out of milestone scope; JS itself is already imperfect (Elastic/Rigid marked buggy); blocks shipping | Recognizable layout + flag behavior; document known gaps |
| Copying the old lfjs UI chrome (Three.js dropdown, freeglut particle-parameter bar, debug HUD) | Pixel familiarity | Conflicts with SolidJS player; accessibility and Pages stack already chosen | Same scenes, existing shell |
| Replacing or removing the six original scenes | Cleaner “only official menu” catalog | Deletes unique work (Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel) and Dam Break continuity | Append the twelve; keep all eighteen |
| Porting every commented-out Box2D-only lfjs test | Completeness | Explicitly “not related to LiquidFun” in `lfjs/index.html` | Out of scope |
| Full Drawing Particles keyboard matrix on day one | Match C++ demo | Many flags (barrier/reactive/repulsive/zombie) may need extra engine surface and UX | Ship paint + core materials first; expand modes if needed |
| In-process JS LiquidFun / Emscripten oracle in the browser | Instant official behavior | Violates native-Rust playground value; duplicates github.io | Link official showcase for comparison |
| Scene editor / freeform particle paint product | LiquidFun Paint nostalgia | Product scope explosion | Drawing Particles as a bounded demo only |
| Claiming “complete JS testbed” without engine gaps filled | Marketing | Some scenes need joints, contact hooks, destroy-in-shape, group impulse | Gate `ready` until the scene actually runs |

## Scene recognition cards (setup → see)

Concrete “must see” for each port. **Table-stakes behavior** vs **optional chrome**.

### Drawing Particles — `testDrawingParticles.js` / `DrawingParticles.h`
- **Setup:** Static ground forming an open box (floor, left/right walls, ceiling strip). Particle system radius 0.05. Starts empty.
- **Interaction:** Mouse down+move destroys particles in a circle then creates a group (radius 0.2) with current flags/groupFlags; consecutive strokes join the same group; mouse up clears `lastGroup`. Keyboard selects material (elastic, powder, rigid, spring, tensile, viscous, wall, barriers, repulsive, colorMixing, zombie) or move mode (`X`).
- **Must see:** Empty vessel; painting leaves colored particle strokes that persist and interact; at least one soft/rigid mode visibly differs from water.
- **Chrome:** Full parameter enum / C++ particle-parameter restart wiring; every exotic flag combo.

### Elastic Particles — `testElasticParticles.js` / `ElasticParticles.h`
- **Setup:** Basin; spring+solid red circle; elastic+solid green circle; elastic+solid blue box with angle/angularVelocity; dynamic circle at y=8.
- **Must see:** Three distinct soft clumps + falling ball that deforms them.
- **Chrome:** Exact recovery stiffness; fixing upstream JS bugs.

### Impulse — `testImpulse.js` / `Impulse.h`
- **Setup:** Chain loop box; particle box group; damping 0.2.
- **Interaction:** MouseUp inside box → normalize(click − center) → `ApplyForce` (default) or `ApplyLinearImpulse` (`l`/`f`).
- **Must see:** Click shoves the whole blob.
- **Chrome:** Runtime particle-type parameter (C++).

### Liquid Timer — `testLiquidTimer.js` / `LiquidTimer.h`
- **Setup:** Loop container; tensile|viscous top slab; edge fixtures: top shelf with ~0.1 gap, vertical throat, two diagonal shelves, four bottom columns.
- **Must see:** Liquid slowly routes through the timer geometry into columns.
- **Chrome:** Alternate particle parameter sets.

### Particles — `testParticles.js` / `Particles.h`
- **Setup:** Floor + two angled side walls; water group circle r=2 at (0,3); dynamic circle r=0.5 at (0,8).
- **Must see:** Classic splash + ball drop.
- **Chrome:** Particle-type picker; damping tweak.

### Rigid Particles — `testRigidParticles.js` / `RigidParticles.h`
- **Setup:** Same basin as Elastic; three rigid|solid colored groups; falling ball.
- **Must see:** Clumps that do not stretch like jelly; collide as solids.
- **Chrome:** Bit-perfect rigid solver vs C++.

### Soup — `testSoup.js` / `Soup.h`
- **Setup:** Basin; water box; destroy-in-shape carve-outs; circle, two boxes, three edge noodles with custom mass.
- **Must see:** Broth with floating bits.
- **Chrome:** Exact noodle mass data fidelity.

### Soup Stirrer — `testSoupStirrer.js` / `SoupStirrer.h`
- **Setup:** Soup + circle stirrer (particles cleared under it) + prismatic joint + oscillating `ApplyForceToCenter` in `Step`.
- **Must see:** Continuous stirring; click/`t` frees or re-rails the paddle.
- **Chrome:** Exact oscillation constants.

### Sparky — `testSparky.js` / `Sparky.h`
- **Setup:** Tall walls; six sparkable dynamic circles; particle system radius 0.25; contact → `ParticleVFX` (powder group, outward velocities, color fade, destroy).
- **Must see:** Collisions erupt into fading particle sparks.
- **Chrome:** maxVFX=50 ring buffer exactness; Three.js `updateColorParticles` flag.

### Surface Tension — `testSurfaceTension.js` / `ParticlesSurfaceTension.h`
- **Setup:** Basin; three tensile|colorMixing groups (RGB); falling ball; dampingStrength 0.2.
- **Must see:** Surface-tension beading + color bleed; ball impact.
- **Chrome:** Extra particle types.

### Theo Jansen — `testTheoJansen.js` / `TheoJansen.h`
- **Setup:** Ground; 40 balls; chassis + wheel + six CreateLeg assemblies (polygons, distance joints soft, revolutes); motor joint; particle box on top (radius 0.2).
- **Must see:** Legged machine + particle load; motor direction controls.
- **Chrome:** Limit enable (`l`); perfect gait vs C++.

### Wave Machine — `testWaveMachine.js` / `WaveMachine.h`
- **Setup:** Dynamic hollow box on revolute motor; particle fill; motor speed follows `0.05 * cos(time) * π`.
- **Must see:** Continuous rocking waves without user input.
- **Chrome:** Particle-parameter overlay.

## Feature Dependencies

```
Shared player + catalog credits
    └──requires──> Existing SolidJS/WASM session (v1.1)

Particles
    └──requires──> Water groups + static polygons + dynamic circle  (already in Dam Break / Float or Sink)

Elastic Particles
    └──requires──> SPRING + ELASTIC + SOLID groups  (Jelly Drop proves elastic)
    └──enhances──> Jelly Drop cross-link

Rigid Particles
    └──requires──> RIGID | SOLID group flags + rigid group solver

Surface Tension
    └──requires──> TENSILE + COLOR_MIXING  (Color Mixer proves color mixing)

Impulse
    └──requires──> Particle group ApplyForce / ApplyLinearImpulse APIs
    └──requires──> Chain/loop or equivalent sealed box

Liquid Timer
    └──requires──> TENSILE | VISCOUS + many static EdgeShape fixtures

Soup
    └──requires──> DestroyParticlesInShape (or equivalent carve) + mixed dynamic fixtures

Soup Stirrer
    └──requires──> Soup
                       └──requires──> PrismaticJoint + per-step body force

Wave Machine
    └──requires──> RevoluteJoint motor + dynamic compound box  (Water Wheel proves revolute + particles)

Drawing Particles
    └──requires──> DestroyParticlesInShape + CreateParticleGroup under pointer
    └──requires──> Subset of flags: WALL, SPRING, ELASTIC, POWDER, TENSILE, VISCOUS,
                   BARRIER, REACTIVE, REPULSIVE, COLOR_MIXING, ZOMBIE + RIGID/SOLID groups

Sparky
    └──requires──> Body contact begin callback + powder groups + destroy group + color buffer writes
    └──conflicts──> Unbounded particle growth without VFX lifetime

Theo Jansen
    └──requires──> RevoluteJoint (motor) + DistanceJoint (soft) + collision filter groupIndex
    └──requires──> Many dynamic bodies + particle group overhead
    └──enhances──> Joint-stress coverage beyond Water Wheel
```

### Dependency Notes

- **Soup Stirrer requires Soup:** JS constructs `new TestSoup()` then adds stirrer (`testSoupStirrer.js`); share one soup builder module.
- **Drawing Particles requires destroy-in-shape:** Stroke flow is destroy-then-create (`MouseMove`); without it, paint stacks forever.
- **Impulse requires group impulse/force:** Scene is meaningless if only body forces exist.
- **Sparky requires contact hooks:** Sparks are contact-driven, not timed emitters.
- **Theo Jansen requires distance + revolute:** Highest joint complexity; schedule after Wave Machine / Water Wheel patterns.
- **Elastic / Rigid / Surface Tension / Particles share basin geometry:** Factor a shared “LF basin” helper to avoid drift.
- **Do not conflict with original six:** New IDs must not replace `dam-break` … `water-wheel` in `SCENE_IDS`.

## MVP Definition

### Launch With (v1.3)

Minimum to claim the milestone goal: visitors can open every listed JS menu scene not already present.

- [ ] **Particles** — lowest-risk baseline splash; proves catalog expansion path
- [ ] **Wave Machine** — iconic default JS demo; motorized tank
- [ ] **Impulse** — click interaction + group force/impulse
- [ ] **Elastic Particles** and **Rigid Particles** — soft vs rigid group contrast (reuse Jelly Drop knowledge)
- [ ] **Surface Tension** — tensile + color mixing showcase
- [ ] **Liquid Timer** — memorable static geometry showcase
- [ ] **Soup** then **Soup Stirrer** — inheritance pair
- [ ] **Drawing Particles** (core paint + essential modes) — interactive flag lab
- [ ] **Sparky** — contact VFX
- [ ] **Theo Jansen** — joint-heavy walker (hardest; may land last but still in-scope)
- [ ] **Catalog + credits + reset** for each — shared visitor capabilities

### Add After Validation (v1.3.x)

- [ ] Drawing Particles full keyboard/material matrix — after core paint works
- [ ] Impulse / Liquid Timer particle-type presets — if public API already supports flags cheaply
- [ ] Pointer polish on watch-first scenes — consistency pass

### Future Consideration (later milestones)

- [ ] Sealed differential evidence per scene — optional qualification, not playground
- [ ] Remaining Box2D-only lfjs tests — explicitly out of LiquidFun JS menu
- [ ] LiquidFun Paint–class editor — product, not demo

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Catalog entries + credits for 12 scenes | HIGH | LOW | P1 |
| Particles | HIGH | LOW | P1 |
| Wave Machine | HIGH | MEDIUM | P1 |
| Impulse | HIGH | LOW–MEDIUM | P1 |
| Elastic Particles | HIGH | MEDIUM | P1 |
| Rigid Particles | HIGH | MEDIUM | P1 |
| Surface Tension | HIGH | MEDIUM | P1 |
| Liquid Timer | HIGH | MEDIUM | P1 |
| Soup | MEDIUM–HIGH | MEDIUM | P1 |
| Soup Stirrer | MEDIUM–HIGH | MEDIUM–HIGH | P1 |
| Drawing Particles (core) | HIGH | HIGH | P1 |
| Sparky | HIGH | HIGH | P1 |
| Theo Jansen | HIGH | HIGH | P1 |
| Drawing full flag matrix | MEDIUM | HIGH | P2 |
| Extra pointer nudges | LOW–MEDIUM | LOW | P2 |
| Sealed C++ parity | LOW (this milestone) | VERY HIGH | P3 / anti |
| lfjs UI chrome clone | LOW | HIGH | P3 / anti |

**Priority key:**
- P1: Must have for v1.3 launch
- P2: Should have once P1 scenes run
- P3: Nice to have / future or anti-feature

## Competitor Feature Analysis

| Feature | Official lfjs testbed | Official C++ Testbed | Our Approach |
|---------|----------------------|----------------------|--------------|
| Scene set | 13 LiquidFun-focused tests in dropdown | Larger Box2D+LiquidFun suite | Port the 12 missing JS menu scenes; keep Dam Break + 5 originals |
| Shell | Three.js + `<select>` | freeglut + particle parameter bar | Existing SolidJS player |
| Interaction | Mouse + letter keys | Mouse + keys + parameter UI | Labeled presets/actions + pointer where needed |
| Authority | Emscripten/JS bindings | Native C++ | Native Rust/WASM; credits cite JS+C++ sources |
| Claims | Live demo | Reference implementation | Recognizable ports; no sealed parity |

## Engine dependency risks (implementation flags)

| Capability | Scenes needing it | Evidence of prior use in playground |
|------------|-------------------|-------------------------------------|
| Water particle groups + basin | Particles, Soup, … | Dam Break |
| ELASTIC / SPRING + SOLID | Elastic, Drawing | Jelly Drop |
| COLOR_MIXING | Surface Tension, Drawing | Color Mixer |
| TENSILE / VISCOUS | Liquid Timer, Surface Tension | Flags exist in `particle/definition.rs`; scene-level proof thinner |
| RIGID \| SOLID groups | Rigid, Drawing | Group flags exist; no dedicated playground scene yet |
| Revolute motor | Wave Machine, Theo Jansen | Water Wheel |
| Prismatic joint | Soup Stirrer | Joint module present; unused in WASM scenes |
| Distance joint (soft) | Theo Jansen | Joint module present; unused in WASM scenes |
| DestroyParticlesInShape | Soup, Soup Stirrer, Drawing | Must confirm/expose for WASM scenes |
| Group ApplyForce / ApplyLinearImpulse | Impulse | Jelly Drop applies poke impulse locally — verify group API |
| Body contact begin | Sparky | May need WASM-facing contact hook |
| Barrier / reactive / repulsive / zombie | Drawing (full matrix) | Flags defined; treat as P2 unless paint requires them |

## Sources

- Pinned submodule `third_party/liquidfun` @ `7f20402173fd143a3988c921bc384459c6a858f2`
- JS menu: `liquidfun/Box2D/lfjs/index.html`
- JS scenes: `liquidfun/Box2D/lfjs/testbed/tests/testDrawingParticles.js`, `testElasticParticles.js`, `testImpulse.js`, `testLiquidTimer.js`, `testParticles.js`, `testRigidParticles.js`, `testSoup.js`, `testSoupStirrer.js`, `testSparky.js`, `testSurfaceTension.js`, `testTheoJansen.js`, `testWaveMachine.js`
- C++ cross-checks: `liquidfun/Box2D/Testbed/Tests/DrawingParticles.h`, `ElasticParticles.h`, `Impulse.h`, `LiquidTimer.h`, `Particles.h`, `RigidParticles.h`, `Soup.h`, `SoupStirrer.h`, `Sparky.h`, `ParticlesSurfaceTension.h`, `TheoJansen.h`, `WaveMachine.h`
- Existing playground catalog: `web/src/catalog/scenes.ts`
- Project scope: `.planning/PROJECT.md` (v1.3 goal)
- Prior feature research pattern: `.planning/research/v1.1/FEATURES.md`

---
*Feature research for: v1.3 Reference Testbed Scenes (JS LiquidFun menu ports)*
*Researched: 2026-09-21*
*Research version: v1.3*
