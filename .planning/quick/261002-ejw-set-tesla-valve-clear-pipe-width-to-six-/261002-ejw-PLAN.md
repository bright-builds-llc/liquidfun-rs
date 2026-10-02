---
phase: quick-261002-ejw
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
  - crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs
  - crates/liquidfun-wasm/src/scene/tesla_valve/flow_tests.rs
  - web/src/catalog/previews.tsx
  - web/src/catalog/scene-records.ts
  - web/tests/scenes.test.ts
  - web/tests/portrait-bounds.test.ts
  - docs/assets/readme/tesla-valve-10s.svg
  - docs/assets/readme/tesla-valve-10s.webp
autonomous: true
requirements: [QUICK-TESLA-SIX-CM-WIDTH]
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261002-ejw
generated_at: "2026-10-02T10:29:27-05:00"
must_haves:
  truths:
    - Tesla Valve straight runs and circular bends have a physical clear width of 0.06 m.
    - True circular bends, trunk position, 0.005 m particle radius and existing rates remain unchanged.
    - The inlet still emits up to 48 distinct particles per frame inside the narrower port.
    - Forward/reverse at default/maximum rates stay contained and satisfy existing meaningful directional checks.
  artifacts:
    - path: crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
      provides: Shared six-centimeter clear-width boundaries and fitting source grid
    - path: crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs
      provides: Independent width/radius/source-spacing assertions
  key_links:
    - from: crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
      to: crates/liquidfun-wasm/src/scene/tesla_valve.rs
      via: Shared geometry continues driving actual fixtures, outlines and emitter slots
---

<objective>
Set Tesla Valve clear pipe width to six centimeters while preserving the trunk centerline and bend radii, true circular curves, particle size and emission settings.
Output: Narrower checked geometry, a fitting 48-slot inlet, retained flow regressions and refreshed previews.
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
@standards/core/architecture.md
@standards/core/code-shape.md
@standards/core/testing.md
@standards/core/verification.md
@standards/languages/rust.md
@standards/languages/typescript-javascript.md
@crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs
@crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs

Client date: 2026-10-02. Apply the already loaded local guidance, Bright Builds sidecar/overrides, experimental scope and standing iteration authority. Active lessons were fully read (5,230 global + 8,295 repository bytes); no audit trigger applies. The parent reports clean synced main at 5a3dc13, whose incoming change is preview-bot WebP only.

Worker ownership is Rust Tesla geometry/tests, including the existing flow-test module if present. Parent ownership is thumbnail/gallery, fresh builds/browser checks, full validation, independent review, records and commits. Preserve other agents' edits. Scope six-centimeter width to constant-width runs and bends; existing short junction transitions may remain wider. No engine/rate changes are authorized without demonstrated failure and explicit scope justification.

<interfaces>
Set shared CLEAR_WIDTH to 0.06 m. With unchanged wall half-thickness 0.03 m, BORDER_HALF becomes 0.06 m, giving bypass nominal border radii 0.11/0.23 m around the unchanged 0.17 m centerline. Keep radius 0.005 m, speed 2 m/s, default rate 1440/s, maximum 2880/s, step 30 and maximum count unchanged. Replace the too-wide 16×3 inlet grid with 4 columns ×12 rows at pitch 0.0102 m; the downstream row is 0.15 m downstream of the inlet plane and the last row remains 0.0378 m downstream, inside the existing 0.20 m port. Derive the final source positions in the existing orthonormal inlet basis.
</interfaces>
</context>

<tasks>
<task type="auto" tdd="true">
  <name>Task 1: Narrow shared channels and refit the source grid</name>
  <files>crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs, crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs, crates/liquidfun-wasm/src/scene/tesla_valve/flow_tests.rs if present</files>
  <behavior>
    - Independently derived physical faces of straight trunk/bypass runs and circular bends are separated by 0.06 m, including wall thickness.
    - Bypass nominal border radii are 0.11/0.23 m; unchanged world-space centerline radius is 0.17 m and bends remain semicircular.
    - The real maximum-rate emission path uses 48 distinct fitting slots with minimum spacing 0.0102 m; particles remain radius 0.005 m.
    - Both directions at default and maximum rates preserve strict conduit containment, drain behavior and existing directional thresholds.
  </behavior>
  <action>First update focused tests to assert the requested six-centimeter clear width and its independently derived border/physical-face distances; capture failures against current geometry. Set the one shared width constant to 0.06 m so all existing straight and circular boundary calculations derive the new offsets while retaining bend radii, trunk placement, tangencies and the existing short junction transitions. The existing bypass anchor/center expressions depend on width and may shift coherently with the new offsets; do not freeze them artificially. Keep wall thickness, fixture robustness and circle/island construction. Replace the source grid with the specified 4×12 layout in inlet coordinates, verifying each slot has particle-radius clearance from physical walls and remains inside the port; use the actual frame emission path to test spacing and count. Preserve rate, velocity magnitude, particle radius and all engine settings. Update old width/radius/source assertions from independently calculated expected geometry, then run existing strict containment/directional regressions in all four direction/rate cases. Do not weaken asserted directional thresholds, replace conduit checks with rectangular frame checks or change engine physics to hide failures. Investigate geometry/source errors first; report a concrete conflict if the requested width cannot satisfy existing behavior. Keep the implementation to shared width/grid parameters and clear derived geometry.</action>
  <verify><automated>cargo test -p liquidfun-wasm tesla_valve -- --nocapture</automated></verify>
  <done>Physical straight/bend channels measure six centimeters, the 48-slot source fits without overlap, circles/positions/settings remain correct and existing meaningful flow checks pass.</done>
</task>

<task type="auto">
  <name>Task 2: Refresh previews and validate the complete narrower valve</name>
  <files>web/src/catalog/previews.tsx, docs/assets/readme/tesla-valve-10s.svg, docs/assets/readme/tesla-valve-10s.webp</files>
  <action>The parent rebuilds WASM/web, refreshes affected Tesla Valve thumbnail/gallery media through the supported generator and inspects fresh desktop/phone scenes. Confirm visibly narrower straight runs and circular bends, intact source/walls/islands, readable metric legend and default/max forward/reverse flow. Record actual runtime/drain evidence for the six-centimeter configuration without treating prior wider-channel measurements as current evidence. Run the required Rust checks in order, affected WASM tests, web typecheck/unit/build/browser checks, managed checker and Markdown check; review scoped diffs and obtain a separate identified review. Parent creates the quick summary and finalizes records/commits; this planner writes only the PLAN.</action>
  <verify><automated>cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; cargo test -p liquidfun-wasm tesla_valve &amp;&amp; bun scripts/bright-builds-check.ts all &amp;&amp; just markdown-check &amp;&amp; just web-build &amp;&amp; (cd web &amp;&amp; bun run typecheck &amp;&amp; bun run test:unit &amp;&amp; bun run test:player) &amp;&amp; git diff --check</automated></verify>
  <done>Fresh previews and browser observations show the requested width, full checks pass, and actual evidence plus independent review are recorded.</done>
</task>
</tasks>

<threat-model>
| Boundary | Description |
| --- | --- |
| Channel/source geometry to particles | Narrowing can place particles inside solids or create leaks/pinches. |

| Threat ID | Category | Component | Disposition | Mitigation plan |
| --- | --- | --- | --- | --- |
| T-WIDTH-01 | T | Shared channel offsets | mitigate | Independent physical-face width/radius assertions and existing solid/containment tests. |
| T-WIDTH-02 | D | Inlet and flow | mitigate | Verify actual 48-slot spacing/clearance and strict default/max direction cases without rate changes. |
</threat-model>

<verification>
Prove actual physical clear width, not merely the shared constant. Retain circles, source clearance, collision containment and meaningful directional thresholds. Fresh browser/media checks verify visible integration; normal experimental local checks apply.
</verification>

<success-criteria>
Straight runs and bends have six-centimeter clear channels with preserved true circles/trunk/settings, fitting inlet emissions and passing existing behavior checks; current previews and review evidence support the change.
</success-criteria>

<output>
Parent creates .planning/quick/261002-ejw-set-tesla-valve-clear-pipe-width-to-six-/261002-ejw-SUMMARY.md after execution. Do not edit STATE or commit during planning.
</output>
