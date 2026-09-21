# Project Research Summary

**Project:** liquidfun-rs — milestone v1.3 Reference Testbed Scenes
**Domain:** Porting twelve JavaScript LiquidFun testbed scenes into the existing Rust/WASM SolidJS playground
**Researched:** 2026-09-21
**Confidence:** HIGH

## Executive Summary

v1.3 completes the official LiquidFun JS testbed menu in the existing hobby playground: twelve missing scenes (Dam Break already ships) become allowlisted native Rust/WASM modules behind the current SolidJS player. Experts build this as scene factories on one physics world—not a second engine, not an lfjs embed, and not a frontend rewrite. The stack stays `liquidfun` + private `liquidfun-wasm` + SolidJS/Vite/Canvas 2D; scenes need recipes, joints, flags, and a few adapter seams (destroy-in-shape, post-step contact observation, raised rigid frame caps), not new crates or frameworks.

The recommended approach is capability-clustered ports: basin/watch-first scenes first, then elastic/rigid/tensile flag demos, then interaction seams (destroy-in-shape, group impulse, motors/joints), then Sparky and Drawing Particles. Acceptance is **recognizable layout and behavior** with honest credits to pinned lfjs + C++ sources—not sealed C++/JS parity. Drawing Particles ships with recognizable paint plus a small preset set; the full lfjs keyboard matrix is out of scope.

Key risks are wrong particle/group flag recipes, faking physics in JavaScript, Sparky contacts never firing under `NoDecisionHook`, Wave Machine / Theo Jansen motors never updating, unbounded particle growth, and gallery edits that regress Dam Break force-buffer or Medium recipe evidence. Mitigate with Rust-side helpers and headless asserts, post-step (or non-mutating session-hook) contact observation without FFI expansion, particle/VFX caps under the existing 4-step catch-up contract, and package isolation for any engine API additions.

## Key Findings

### Recommended Stack

No new production dependency, second physics engine, glam, Rayon/SIMD defaults, or frontend framework. Prefer capabilities already in `liquidfun` and the existing `liquidfun-wasm` + SolidJS player. Details: [STACK.md](STACK.md).

**Core technologies:**
- `liquidfun` (workspace path crate) — sole physics runtime — already exposes particle/group flags, joints, force/impulse ranges, split, filters, shapes
- `liquidfun-wasm` (private) — scene factories, stepping, pointer/control bridge — existing `SceneId` / `SceneHooks` / `ProofSession` pattern
- `wasm-bindgen` `=0.2.128` — Rust↔JS session API — keep exact pin
- SolidJS 1.9.15 + Vite 8.3.0 + Canvas 2D — catalog, controls, draw copied lanes — catalog growth only
- `@kobalte/core` 0.13.x + Playwright 1.63.0 — shell + Chromium smoke — extend allowlists, do not add a second E2E runner

**Stack additions for v1.3 (not installs):**
- Twelve `liquidfun-wasm` scene modules + `SceneId` / `web/src/catalog/scenes.ts` entries
- Thin shared **destroy-in-shape** helper for Soup, Soup Stirrer, and Drawing Particles (required). Prefer a private wasm/scene helper composing `query_aabb_with_particles` + `Shape::test_point` + mark/destroy; if public API cannot compose safely, add a narrow public `liquidfun` method. Do not leave this undecided or fake it in JS
- Raise `MAX_RIGID_CIRCLES` / `MAX_RIGID_SEGMENTS` for Theo Jansen (40 balls + legs) and Sparky (6 circles + VFX bodies)
- Sparky: observe contacts **after the step** (or a session hook that records without mutating mid-solve); spawn VFX in `on_advance`. Do **not** expand FFI / `WorldCommand` for mid-step create. Do **not** require mid-step world mutation
- Optional manual C++ oracle spot-check only — never a playground dependency

### Expected Features

Twelve missing JS-menu scenes plus catalog/player reuse. Full cards and priorities: [FEATURES.md](FEATURES.md).

**Must have (table stakes):**
- **Particles** — baseline splash + ball drop
- **Liquid Timer** — tensile|viscous slab through zig-zag shelves into columns
- **Wave Machine** — motorized rocking tank (`motorSpeed = 0.05 * cos(t) * π` each sim step)
- **Impulse** — click shoves whole group; force vs linear-impulse toggle
- **Elastic Particles** / **Rigid Particles** — spring/elastic+solid vs rigid|solid clumps + falling ball (recognizable motion; upstream JS notes buggy)
- **Surface Tension** — `TENSILE | COLOR_MIXING` three-color beading (distinct from Color Mixer)
- **Soup** then **Soup Stirrer** — broth with carved fixtures; prismatic paddle + joint toggle
- **Theo Jansen** — walker with soft distance + motorized revolute + particle slab (follow **JS**, not particle-free C++ header); direction/motor controls
- **Sparky** — begin-contact powder VFX that fade/destroy
- **Drawing Particles** — drag paint with destroy-then-create; recognizable drawing + **small preset set** (water + a few materials). Full lfjs key matrix is **not** required
- Catalog entry, credits (pinned JS+C++), shared player, reset remount, particle/VFX bounds per interactive/heavy scene

**Should have (competitive):**
- Labeled presets/controls mapped from JS keys (Drawing modes, Impulse l/f, Stirrer toggle, Theo Jansen direction)
- Cross-links (Elastic↔Jelly Drop, Surface Tension↔Color Mixer, Soup↔Stirrer)
- Sparky color fade in the existing color lane
- Optional pointer nudge on watch-first scenes

**Defer (v1.3.x / later):**
- Full Drawing Particles keyboard/material matrix
- C++ particle-parameter chrome and sealed differential matrices
- Box2D-only lfjs tests; LiquidFun Paint–class editor
- Claiming bit-exact / sealed parity

### Architecture Approach

One allowlisted `SceneId` module per scene under `liquidfun-wasm`, building a native `World` and implementing `SceneHooks`; SolidJS only catalogs and draws copied `ProofFrame` lanes. Engine stays renderer-free; add the smallest missing helpers only when blocked. Details: [ARCHITECTURE.md](ARCHITECTURE.md).

**Major components:**
1. `liquidfun` — native physics; optional narrow destroy-in-shape if private composition is unsafe
2. `liquidfun-wasm` scene modules + `SessionCore` — factories, hooks, frame caps, post-step Sparky contact observation
3. `web/` catalog + player — titles, controls, credits, Canvas draw; no physics

**Locked integration decisions (researcher disagreements resolved):**
- **Destroy-in-shape:** shared thin helper required; prefer private wasm composition of public queries; narrow public method acceptable if needed
- **Sparky:** post-step / non-mutating observe → deferred VFX; no FFI expansion; no mid-step mutation
- **Drawing UX:** paint + small presets; not full keyboard matrix
- **Theo Jansen particles:** follow JS water slab for playground recognition; keep `BuiltScene.particle_system` (may be empty only if a scene truly has none—Theo Jansen should not)

### Critical Pitfalls

Top risks from [PITFALLS.md](PITFALLS.md):

1. **Flag / group-flag mismatches** — Inventory lfjs recipes bit-for-bit; assert flags after create; never copy Jelly Drop `ELASTIC|SPRING` into Rigid or Color Mixer-only into Surface Tension
2. **Missing destroy / force / contact paths faked in JS** — Implement destroy-in-shape and group impulse in Rust; Sparky needs post-step contact observation, not SolidJS approximations
3. **Motors never live** — Wave Machine and Theo Jansen must mutate revolute motor speed/enable from `on_advance` / controls with sim time; soft distance joints + `groupIndex = -1` for the walker
4. **Lifetime / VFX / lastGroup leaks** — Sparky ring-buffer destroy; Drawing clear join target on group destroy; never enable destruction-by-age on static-group ports
5. **Frame-budget / Dam Break regression** — Keep `MAX_STEPS_PER_FRAME = 4`; cap paint/VFX; do not slash counts or edit Dam Break Medium for gallery FPS; preserve force-buffer non-stacking after Impulse/Stirrer work
6. **Overclaiming parity** — Catalog and docs say recognizable ports; note Elastic/Rigid upstream “buggy” caveats

## Implications for Roadmap

Based on research and locked build order, suggested phase structure (continue numbering after Phase 25; kinds: Engine capability → Scene port → Player interaction → Honesty/docs):

### Phase A: Catalog Scale + Basin Watch-First
**Rationale:** Validates allowlist/catalog/player growth with almost no new engine surface.
**Delivers:** Scene registration pattern + **Particles** + **Liquid Timer** running headlessly and in the player.
**Addresses:** Particles, Liquid Timer, catalog/credits/reset baseline.
**Avoids:** Overbuilding Drawing/Sparky first; frame-protocol churn.

### Phase B: Elastic / Rigid / Tensile Flag Cluster
**Rationale:** Reuses Color Mixer / Jelly Drop patterns; proves tensile and rigid groups in WASM before interaction-heavy seams.
**Delivers:** **Surface Tension**, **Elastic Particles**, **Rigid Particles**.
**Addresses:** Flag showcase table stakes; Elastic↔Jelly / Surface Tension↔Color Mixer cross-links.
**Avoids:** Confusing Color Mixer with Surface Tension; flag mismatches; destruction-by-age on static groups.

### Phase C: Interaction Seams (Destroy, Impulse, Motors)
**Rationale:** Lands the shared destroy-in-shape helper once, then ports scenes that need carve, group shove, or live motors/joints.
**Delivers:** Destroy-in-shape helper; **Soup** → **Soup Stirrer**; **Impulse**; **Wave Machine**; **Theo Jansen** (JS particle slab + soft distance + motor controls); raised rigid frame caps as needed for Theo Jansen.
**Addresses:** Soup/Stirrer pair, Impulse, Wave Machine, Theo Jansen; prismatic/revolute/distance wiring through hooks/controls.
**Avoids:** JS destroy/force fakes; Water Wheel–style motorless revolute copied into Wave Machine; Soup Stirrer divergent geometry; package isolation leaks.

### Phase D: Sparky + Drawing Particles
**Rationale:** Highest integration cost (contacts + paint UX); depends on destroy-in-shape and stable session/hook patterns from earlier phases.
**Delivers:** Post-step Sparky contact observation + powder VFX lifecycle; **Drawing Particles** paint + small material presets; smoke for paint/poke/contact spectacle.
**Addresses:** Sparky, Drawing Particles (core).
**Avoids:** Mid-step mutation / FFI expansion; full keyboard matrix scope creep; unbounded paint; `NoDecisionHook` left unconditional for Sparky.

### Phase E: Honesty, Caps, and Regression Guardrails (cross-cutting, finalize with D)
**Rationale:** Milestone credibility depends on wording, catch-up contract, and Dam Break safety after shared edits.
**Delivers:** Recognizable-port copy; stutter/cap notes; Chromium smoke growth; Dam Break spot-check after shared engine PRs.
**Addresses:** Credits, anti-feature avoidance, bounded lifecycle.
**Avoids:** Parity marketing; count cheats; ≤3× stamp misuse; WASM deps in publishable `liquidfun`.

### Phase Ordering Rationale

- Locked order: **basin/watch-first → elastic/rigid/tensile → interaction seams (destroy, impulse, motors) → Sparky and Drawing Particles**
- Capability clusters share one seam (destroy-in-shape once; motor `on_advance` once; post-step contacts once)
- Soup Stirrer requires Soup; Drawing and Soup* require destroy-in-shape; Sparky requires session contact plumbing without mid-step mutation
- Keeps existing six scenes untouched except allowlist/catalog growth

### Research Flags

Phases likely needing deeper research during planning:
- **Phase C (destroy-in-shape exact API shape):** Confirm whether private wasm composition of public queries is sufficient or a narrow public method is required—decision criterion locked; implementation detail still needs a quick API inventory
- **Phase C (Theo Jansen):** Soft distance joint tuning + filter `groupIndex` + draw-cap sizing under Canvas 2D
- **Phase D (Sparky):** Exact post-step contact transition surface vs thin session adapter—must not mutate mid-solve or expand FFI
- **Phase D (Drawing presets):** Which small preset set is “recognizable enough” without the full matrix

Phases with standard patterns (skip research-phase):
- **Phase A (Particles, Liquid Timer, catalog):** Dam Break / existing scene registration patterns
- **Phase B (Elastic, Rigid, Surface Tension):** Jelly Drop / Color Mixer recipe adaptations
- **Phase C (Impulse, Wave Machine controls):** Jelly Drop poke + Water Wheel revolute (add motor mutation)
- **Honesty/docs and smoke extension:** v1.1 catalog/credits/smoke patterns

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | In-tree APIs + pinned lfjs/C++; no new frameworks needed; frame-cap numbers MEDIUM until counted per scene |
| Features | HIGH | Official JS menu + pinned sources; Drawing matrix deferred by lock |
| Architecture | HIGH | Existing `SceneHooks` / session seams inspected; Sparky/destroy paths locked above |
| Pitfalls | HIGH | Repo constraints (4-step cap, `NoDecisionHook`, Dam Break force-buffer) + upstream scene requirements |

**Overall confidence:** HIGH

### Gaps to Address

- **Exact rigid frame cap numbers** for Theo Jansen / Sparky — count draw lists during implementation; raise caps before claiming ready
- **Destroy-in-shape placement** (private helper vs narrow public) — try private composition first per lock; promote only if unsafe/incomplete
- **TENSILE / RIGID playground proof** thinner than Color Mixer / Jelly Drop — Phase B should include headless flag/solver participation checks
- **Impulse group API verification** — confirm contiguous `member_ids()` + range force/impulse match Jelly Drop poke semantics
- **Sparky color fade** — scene color writes vs lifetime-only approximation; document choice
- **Optional Dam Break pair/spot-check** after shared hot-path edits — not a per-scene gate

## Sources

### Primary (HIGH confidence)
- Pinned upstream @ `7f20402173fd143a3988c921bc384459c6a858f2` — lfjs `test*.js` + Testbed `Tests/*.h` for all twelve scenes
- Official JS menu / testbed — [google.github.io/liquidfun](https://google.github.io/liquidfun/) and [testbed](http://google.github.io/liquidfun/testbed/index.html)
- In-repo: `crates/liquidfun` particle flags/joints/queries; `crates/liquidfun-wasm` session/frame/scene; `web/src/catalog/scenes.ts`; `web/src/physics/clock.ts` (`MAX_STEPS_PER_FRAME = 4`)
- Prior research: `.planning/research/v1.1/*`, `.planning/research/v1.2/PITFALLS.md`; `.planning/PROJECT.md` v1.3 goal; `PROJECT-SCOPE.md`
- This milestone research: [STACK.md](STACK.md), [FEATURES.md](FEATURES.md), [ARCHITECTURE.md](ARCHITECTURE.md), [PITFALLS.md](PITFALLS.md)

### Secondary (MEDIUM confidence)
- Exact WASM frame-cap headroom until per-scene draw lists are counted
- Whether tensile/rigid solver paths need engine work beyond recipe wiring (prove in Phase B)
- Optional `@kobalte/core` 0.13.14 polish bump — not scene-blocking

### Tertiary (LOW confidence)
- None material for roadmap structure; remaining items are implementation measurements

---
*Research completed: 2026-09-21*
*Milestone: v1.3 Reference Testbed Scenes*
*Ready for roadmap: yes*
*Commit: deferred per orchestrator instruction*
