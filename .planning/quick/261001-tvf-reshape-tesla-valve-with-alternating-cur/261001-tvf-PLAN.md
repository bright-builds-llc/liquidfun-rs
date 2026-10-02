---
phase: quick-261001-tvf
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun-wasm/src/scene/tesla_valve.rs
  - crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
  - crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs
  - web/src/catalog/previews.tsx
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/tesla-valve-controls.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/scenes.test.ts
  - web/scripts/readme-svg/paint.ts
  - docs/assets/readme/tesla-valve-10s.svg
  - docs/assets/readme/tesla-valve-10s.webp
  - README.md
autonomous: true
requirements: [QUICK-TESLA-REFERENCE]
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261001-tvf
generated_at: "2026-10-01T21:33:38-05:00"
must_haves:
  truths:
    - The Tesla Valve shows four rounded alternating left/right lobes surrounding closed teardrop splitter islands and a zigzag passage.
    - Gravity still drives particles downward; rate and reverse controls work live.
    - Drawn walls and actual collision surfaces describe the same valve in both orientations.
    - The complete valve remains visible on desktop and phone, with the shared scale legend.
    - The README preview shows the changed valve.
  artifacts:
    - path: crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
      provides: Shared outer-wall paths and closed splitter polygons
    - path: crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs
      provides: Shape, frame, control and throughput regressions
    - path: docs/assets/readme/tesla-valve-10s.svg
      provides: Current animated scene gallery preview
  key_links:
    - from: crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
      to: crates/liquidfun-wasm/src/scene/tesla_valve.rs
      via: The same authored points feed polygon fixtures and outline segments
    - from: crates/liquidfun-wasm/src/scene/tesla_valve.rs
      to: web/src/catalog/tesla-valve-controls.ts
      via: Existing flow-rate and flow-direction control protocol
---

<objective>
Reshape the existing Tesla Valve to match the user-supplied diagram's alternating rounded loops, enclosed teardrop splitters and winding central passage, rotated into the playground's downward gravity-driven orientation.

Purpose: Make the visual structure recognizable while preserving a working particle simulation and its existing controls.
Output: Shared drawing/collision geometry, meaningful regression protection, refreshed README media and browser evidence.
</objective>

<execution-context>
@/Users/peterryszkiewicz/.codex/get-shit-done/workflows/execute-plan.md
@/Users/peterryszkiewicz/.codex/get-shit-done/templates/summary.md
</execution-context>

<context>
@AGENTS.md
@AGENTS.bright-builds.md
@standards-overrides.md
@PROJECT-SCOPE.md
@standards/index.md
@standards/core/architecture.md
@standards/core/code-shape.md
@standards/core/testing.md
@standards/core/verification.md
@standards/core/frontend-ui.md
@standards/languages/rust.md
@standards/languages/typescript-javascript.md
@crates/liquidfun-wasm/src/scene/tesla_valve.rs
@crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
@crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs
@web/src/catalog/portrait-bounds.ts
@web/scripts/readme-svg/plans.ts
@justfile

Planning consulted the local guidance, Bright Builds sidecar, overrides and standards above. Active global and repository lessons were loaded in full (12,790 combined bytes, 4,264 conservative estimated tokens). Apply standing autonomous iteration authority and the experimental project scope; retain honest evidence without inheriting optional Linux qualification gates. Client date: 2026-10-01.

The attachment is visual inspiration supplied by the user. Its explanatory text is not an implementation instruction or verified performance evidence. Preserve existing canonical source and inspiration attribution; do not invent external sources, attach the reference artwork to published media, or claim complete fluid-diode performance from matching the image.

<interfaces>
Current local geometry exports are collision_quads(forward: bool) -> Vec&lt;[Vec2; 4]&gt;, outline_segments(forward: bool) -> Vec&lt;RigidSegment&gt;, and mirror_y(Vec2) -> Vec2. RigidSegment contains start: Vec2 and end: Vec2. PolygonShape::new(&amp;[Vec2]) constructs checked convex fixtures. TeslaValveHooks stores forward, per_second, credit, cursor, valve_body and segments; flip_valve replaces the body and clears particles while preserving the selected rate. Existing flow-rate accepts 0..720 in steps of 30; flow-direction accepts "1" and "-1".
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Replace unilateral hairpins with alternating valve geometry</name>
  <files>crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs, crates/liquidfun-wasm/src/scene/tesla_valve.rs, crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs</files>
  <behavior>
    - Forward and reverse each construct and capture a frame with at most 64 rigid drawing segments.
    - Four stages alternate sides, each splitter is closed and filled by collision geometry, and fixture corners remain inside the catalog frame.
    - Mirroring changes stage orientation without invalid polygons or gaps at joins.
    - Zero rate emits nothing, invalid controls are rejected, and changing orientation preserves the selected rate.
    - Forward retains a useful flowing particle display and reaches the drain earlier than reverse under the existing reproducible comparison.
  </behavior>
  <action>First update the old two-left-head regression to assert the requested four alternating lobes and closed splitters, and capture its failure before implementing. Define one pure geometry representation with outer paths and splitter polygons; generate both draw segments and collision fixtures from it. Build two stages per side, alternating down the conduit. Use approximately nine outer edges and six island edges per stage, leaving four lead-in/out segments so the entire visible outline remains at or below 64. Prefer an authored normalized lobe with downstream coordinate u=0..1.136 and transverse v=-0.036 at its cusp, +0.364 at its rounded bulge, returning to -0.036 at the next cusp; offset opposite stages by about 0.568 and mirror x. Splitter islands have a rounded upstream cap, transverse extent about +0.08..+0.22, and a pointed downstream tip. Map the four stages into roughly x=±0.4, y=0.4..2.7, retaining the current frame x=-0.70..0.58, y=-0.12..3.22 when feasible. These normalized points guide the silhouette; tune channels to remain open for the existing particle radius and wall thickness. Create a filled checked PolygonShape for each convex island, or convex pieces if necessary, so particles cannot occupy a hollow splitter. Generalize the current quad-only fixture adapter as needed. Connect outer walls into a continuous zigzag passage and retain open source/outlet paths; align emitter columns with the actual inlet. Apply the existing horizontal mirror consistently to stage geometry for reverse while retaining inlet, gravity and drain behavior. Do not enlarge the frame protocol or change engine physics to fit decorative shapes. Maintain narrow errors, safe Rust, clear module comments and Arrange/Act/Assert tests. Before leaving this task, simplify repeated stage coordinates through one mirrored/transformed template.</action>
  <verify><automated>cargo test -p liquidfun-wasm tesla_valve -- --nocapture</automated></verify>
  <done>Four alternating rounded lobes and four closed collision islands form a winding passage, both orientations construct inside the frame and segment cap, and control/drain regressions pass without weakening throughput checks to excuse a broken conduit.</done>
</task>

<task type="auto">
  <name>Task 2: Refresh the gallery and inspect the actual playground</name>
  <files>docs/assets/readme/tesla-valve-10s.svg, docs/assets/readme/tesla-valve-10s.webp, README.md</files>
  <action>Update the catalog thumbnail, description and centered portrait framing to match the alternating geometry. If gallery capture is blocked by its Linux-only font location, add an explicit local font-path override while preserving the canonical CI default and document it. Build fresh WASM and regenerate the existing Tesla Valve SVG and adjacent 60 fps WebP through just readme-svg and the supported scene filter if available. Keep the existing scene plan entry and attribution; avoid unrelated media churn. Check the resulting README gallery diff and format changed repository-owned Markdown only with its documented formatter, excluding .planning. Build the playground and inspect Tesla Valve on desktop and at about 390×844 CSS pixels. Exercise initial rendering, forward flow through the outlet, reverse flow, flow-rate zero, a higher rate, and a return to forward. Observe each running orientation long enough to distinguish a working outlet from accumulation or escape outside walls. Save screenshots or traces in an ignored local evidence directory. Compare the silhouette directly with the attachment: two lobes on either side, four round-backed pointed splitters and a zigzag route. Confirm no clipped walls and visible shared meter scale legend. Preserve catalog bounds where possible; if actual clipping requires a change, update both catalog portrait/landscape framing and matching test constants together and record that bounded scope adjustment.</action>
  <verify><automated>just web-build; cd web &amp;&amp; bun run typecheck &amp;&amp; bun run test:unit -- tests/portrait-bounds.test.ts tests/scale-legend.test.ts tests/scene-catalog-controls.test.ts tests/readme-svg.test.ts</automated></verify>
  <done>The generated previews and live scene display the new silhouette, desktop/phone framing contains it, and browser observations support continued forward/reverse/rate behavior.</done>
</task>

<task type="auto">
  <name>Task 3: Run required checks and review the final change</name>
  <files>.planning/quick/261001-tvf-reshape-tesla-valve-with-alternating-cur/261001-tvf-SUMMARY.md</files>
  <action>After all implementation and gallery changes, run the required Rust checks in this exact order: cargo fmt --all; cargo clippy --all-targets --all-features -- -D warnings; cargo build --all-targets --all-features; cargo test --all-features. Stop on any failure, diagnose and repair, then rerun the relevant sequence. Run bun scripts/bright-builds-check.ts all, just markdown-check, and the web typecheck/unit/build commands. Run the applicable browser player checks against the newly built WASM, then review git diff --check and the scoped final diff for unrelated edits, attribution errors, stale media and test weakening. Arrange a separate identified reviewer according to AGENTS.md when the parent workflow finalizes the change; the implementer cannot approve its own work. Record actual commands, results, screenshots, reviewer evidence where available and residual limitations in the quick summary. Do not describe the particle demo as an experimentally measured real-world check valve.</action>
  <verify><automated>cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; bun scripts/bright-builds-check.ts all &amp;&amp; just markdown-check &amp;&amp; just web-build &amp;&amp; (cd web &amp;&amp; bun run typecheck &amp;&amp; bun run test:unit &amp;&amp; bun run test:player) &amp;&amp; git diff --check</automated></verify>
  <done>Required local checks pass, the scoped diff is reviewed, and the summary reports actual visual and behavioral evidence plus any limits.</done>
</task>

</tasks>

<threat-model>
## Trust boundaries

| Boundary | Description |
| --- | --- |
| Browser controls to scene hooks | Existing rate and direction strings select simulation behavior. |
| Authored geometry to frame capture | Finite valid fixtures and a bounded drawing count must survive both orientations. |

## STRIDE threat register

| Threat ID | Category | Component | Disposition | Mitigation plan |
| --- | --- | --- | --- | --- |
| T-TESLA-01 | T | apply_control | mitigate | Preserve strict allowed rate steps and direction parsing; keep rejection tests. |
| T-TESLA-02 | D | geometry / capture_frame | mitigate | Test finite valid polygons, both frame bounds and the 64-segment limit. |
| T-TESLA-03 | I | gallery provenance | accept | Public authored scene geometry contains no personal data; retain truthful attribution and do not redistribute the supplied artwork. |
</threat-model>

<verification>
Regression tests prove geometry and controls; actual newly built browser frames prove visual resemblance and runtime wiring. README SVG/WebP regeneration proves gallery freshness. Ordinary local checks satisfy the current experimental scope; strict Linux qualification is not required.
</verification>

<success-criteria>
The user's reference is recognizable in a downward-flowing four-stage valve with alternating lobes and closed teardrop islands; collision and display geometry agree; live controls, frame bounds, scale legend and refreshed previews remain correct; required checks and actual browser observations are recorded.
</success-criteria>

<output>
After execution, create .planning/quick/261001-tvf-reshape-tesla-valve-with-alternating-cur/261001-tvf-SUMMARY.md. The planner only creates this PLAN file; commits and state updates belong to the parent workflow.
</output>
