---
phase: quick-261010-wkd
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/components/CanvasStage.tsx
  - web/tests/hud-sliders-toggle.test.ts
  - docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md
  - web/src/catalog/portrait-bounds.ts
  - web/tests/portrait-bounds.test.ts
autonomous: true
requirements: [QUICK-261010-wkd]
generated_by: gsd-plan-phase
lifecycle_mode: quick
phase_lifecycle_id: quick-261010-wkd
generated_at: "2026-10-11T04:29:29Z"

must_haves:
  truths:
    - "The bottom-toolbar sliders toggle shows a panel-with-chevron glyph that is visibly different from the top-right Controls drawer glyph"
    - "The chevron points down while the sliders are shown (label 'Hide sliders') and up while they are hidden (label 'Show sliders')"
    - "The toggle's aria-expanded, aria-controls, and accessible labels are unchanged"
    - "At 390x844 the Tesla valve renders larger than in the 261010-vzz after screenshot (more than ~98.5 px per metre) and roughly centred horizontally"
    - "Every Tesla valve wall stays inside the portrait frame, and hiding the sliders still grows the Tesla valve world frame"
  artifacts:
    - path: "web/src/components/CanvasStage.tsx"
      provides: "SliderPanelIcon used by HudSlidersToggle; SlidersIcon removed"
      contains: "SliderPanelIcon"
    - path: "web/src/catalog/portrait-bounds.ts"
      provides: "Narrowed TESLA_VALVE_PORTRAIT derived from the valve wall extent"
      contains: "TESLA_VALVE_PORTRAIT"
    - path: "web/tests/hud-sliders-toggle.test.ts"
      provides: "Guard that the toggle does not reuse ControlsIcon and that its chevron follows visibility"
    - path: "web/tests/portrait-bounds.test.ts"
      provides: "Tesla valve extreme wall points, tesla-valve width override, height threshold back toward 0.80"
  key_links:
    - from: "HudSlidersToggle"
      to: "SliderPanelIcon"
      via: "expanded prop bound to props.visibility === \"shown\""
      pattern: "<SliderPanelIcon expanded=\\{props\\.visibility === \"shown\"\\}"
    - from: "PORTRAIT_VIEW_BOUNDS[\"tesla-valve\"]"
      to: "safe-frame best fit at 390x844"
      via: "worldBoundsForViewport -> cameraFrame -> safeFrame"
      pattern: "\"tesla-valve\": TESLA_VALVE_PORTRAIT"
---

<objective>
Fix two owner review findings from quick task 261010-vzz.

1. The new sliders toggle in the bottom toolbar uses `SlidersIcon`, which reads the same as the `ControlsIcon` on the top-right Controls drawer trigger. Replace it with a panel-with-chevron glyph that reads as "collapse or expand the slider panel".
1. `TESLA_VALVE_PORTRAIT` is 2.7 m wide, but the valve walls span only about 1 m. The empty side margins collide with the FPS/debug column, so the safe-frame fit shrinks the valve and pushes it right of centre at 390x844. Narrow the frame to the real wall extent plus a small margin, derived from the scene source.

Purpose: The toggle becomes recognisable, and the phone view of the Tesla valve becomes large and centred again.
Output: Updated icon, guard tests, spec sentence, narrowed portrait frame, adjusted portrait tests, before/after screenshots kept outside git.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md
@docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md
@.planning/quick/261010-vzz-hud-safe-frame-slider-toggle-and-chrome-/261010-vzz-SUMMARY.md
@web/src/components/CanvasStage.tsx
@web/src/catalog/portrait-bounds.ts
@web/tests/portrait-bounds.test.ts
@web/tests/hud-sliders-toggle.test.ts
@web/e2e/hud-safe-frame.spec.ts

<interfaces>
From web/src/components/CanvasStage.tsx (current):
```tsx
// HudSlidersToggle (around line 226): button.hud-button.canvas-sliders-toggle with
// data-hud-obstacle, aria-label={hudSlidersToggleLabel(props.visibility)},
// aria-expanded={props.visibility === "shown"}, aria-controls={HUD_SLIDERS_ID}
//   ...
//   <SlidersIcon struck={props.visibility === "hidden"} />   // line ~242, the only SlidersIcon use

function ControlsIcon() {            // line ~380, used by the drawer trigger at line ~287
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 8h16M4 16h16M8 6v4M16 14v4" />
    </svg>
  );
}

function SlidersIcon(props: { readonly struck: boolean }) { ... }  // line ~388, delete it
```

From web/src/catalog/portrait-bounds.ts (current):
```ts
const TESLA_VALVE_PORTRAIT = { minX: -1.35, minY: -0.2, maxX: 1.35, maxY: 4.6 } as const;
// PORTRAIT_VIEW_BOUNDS: "tesla-valve": TESLA_VALVE_PORTRAIT
```

From web/tests/portrait-bounds.test.ts (current):
- `IPHONE = { width: 390, height: 844 }`.
- `MIN_HEIGHT_FRACTION["tesla-valve"] = 0.75`, with the comment "The tight valve frame no longer pads below the drain for HUD chrome."
- "fills most of an iPhone canvas for every scene" asserts `fraction.width > 0.85` for every scene. There is no per-scene width override.
- The "keeps each scene's walls inside the iPhone frame" tesla-valve list has inlet/outlet wall ends at x -0.183/-0.003, leg vertices at x ±0.46, and island points at x 0.19. The outer bypass arcs, which reach further out, are not listed.

Tesla valve wall geometry, from crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs. It is only mirrored in y when the flow direction flips, so the x extent is the same both ways:
- `WALL_HALF_THICKNESS = 0.03`, `BORDER_HALF_WIDTH = 0.06`, `BYPASS_RADIUS = 0.17`, `MIRROR_Y = 2.15`.
- Bypass centres are at x ±0.1899. The outer border radius is 0.23, so the centerline reaches about ±0.42 and the outer wall face about ±0.45. Leg centerline endpoints are at about ±0.46.
- Planner estimate of the wall extent including thickness: x about -0.49..0.49 and y about 0.0..4.32. The `#[cfg(test)]` constants `FRAME_MIN_X=-0.70`, `FRAME_MAX_X=0.58`, `FRAME_MIN_Y=-0.12`, `FRAME_MAX_Y=4.32` are loose containment bounds, not the extent. The drain sensor (x ±0.7, y -0.36..0.08) is not a wall.
- `outline_segments(forward: bool) -> Vec<RigidSegment>` returns the drawn wall centerlines for either direction.

Safe frame, from web/src/render/safe-frame.ts. Candidates are rectangles bounded by obstacle edges. Ties within 0.5% of the best score go to the rectangle whose centre is nearest the viewport centre. The world contain-fits inside the chosen rectangle. In 261010-vzz, tesla-valve at 390x844 got frame 108-374 x 138-611 (266x473) for a 2.7 x 4.8 m world, about 98.5 px/m, and the scale legend read 1 m.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Replace the toggle's slider glyph with a panel-with-chevron icon</name>
  <files>web/src/components/CanvasStage.tsx, web/tests/hud-sliders-toggle.test.ts, docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md</files>
  <behavior>
    - Test A: the `HudSlidersToggle` function body renders `<SliderPanelIcon expanded={props.visibility === "shown"} />` and contains neither `<ControlsIcon` nor `<SlidersIcon`.
    - Test B: the set of `d="..."` path strings inside `function SliderPanelIcon` and the set inside `function ControlsIcon` do not intersect, and `SliderPanelIcon` draws a rect/panel element. This guards against the toggle reusing the drawer glyph.
    - Test C: `SliderPanelIcon` has two distinct chevron paths, a down chevron for `expanded` and an up chevron otherwise, chosen by `props.expanded`.
    - Test D: `function SlidersIcon` no longer exists in CanvasStage.tsx.
  </behavior>
  <action>
    RED: In web/tests/hud-sliders-toggle.test.ts, add a `describe("HUD sliders toggle icon", ...)` block with tests A-D. Use the file's existing source-text pattern (`source("components/CanvasStage.tsx")`). Add a small helper `functionBody(text, name)` that slices from `function ${name}(` to the next top-level `\nfunction ` (or end of file). Each test uses explicit `// Arrange`, `// Act`, `// Assert` comments. Run the suite and confirm the new tests fail. Commit `test(quick-261010-wkd): add failing sliders toggle icon guard`.

    GREEN: In web/src/components/CanvasStage.tsx, replace `SlidersIcon` with `SliderPanelIcon(props: { readonly expanded: boolean })`. Base it on the Lucide panel-bottom-close/open shapes; the existing icons already use the Lucide style (24x24 viewBox, stroke paths).
    - `<rect x="3" y="3" width="18" height="18" rx="2" />` for the panel, plus `<path d="M3 15h18" />` for the slider-card strip along its bottom.
    - Inside a Solid `<Show when={props.expanded} fallback={<path d="m9 12 3-3 3 3" />}>`, render `<path d="m9 9 3 3 3-3" />`. The down chevron means "Hide sliders", which collapses the bottom panel. The up chevron means "Show sliders". Keep both chevrons above the strip so they stay legible at hud-button size.
    - Pass `expanded={props.visibility === "shown"}` from `HudSlidersToggle`.
    - Delete `SlidersIcon` entirely; it has no other users.
    - Do NOT change `aria-label`, `aria-expanded`, `aria-controls`, `data-hud-obstacle`, or the button class.
    - Check `web/src/styles/canvas-stage.css`: if `.hud-button svg` styles set `fill: none` and stroke for `path` only, make sure `rect` also renders unfilled and stroked. Inspect the existing selector before editing; edit the CSS only if the rect would render filled. If you edit it, add the CSS file to the commit and to the SUMMARY.
    - Run the tests to green.

    Spec: in docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md (Component 1, around lines 58-59), replace "The icon is a slider glyph, struck through when hidden." with a sentence saying the icon is a panel with a chevron that is distinct from the Controls drawer glyph, pointing down while the sliders are shown and up while they are hidden. Format it with mdformat 1.0.0 under Python 3.13 (for example `uvx --python 3.13 mdformat==1.0.0 docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md`; check `scripts/markdown-check.sh` for the exact invocation). Then run `just markdown-check`. Never run mdformat on `.planning/**`.

    Commit `feat(quick-261010-wkd): give the sliders toggle a distinct panel chevron icon`.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/web && bunx vitest run tests/hud-sliders-toggle.test.ts && bun run typecheck && cd .. && just markdown-check</automated>
  </verify>
  <done>New icon tests failed before the change and pass after it. `SlidersIcon` is gone. The toggle renders `SliderPanelIcon` with a chevron that follows visibility. ARIA attributes are unchanged. The spec sentence is updated and markdown-check passes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Narrow the Tesla valve portrait frame to its walls and verify at 390x844</name>
  <files>web/src/catalog/portrait-bounds.ts, web/tests/portrait-bounds.test.ts</files>
  <behavior>
    - The tesla-valve wall-endpoint list includes the measured extreme-x wall points for both flow directions, at the outer bypass arcs and trunk corners. All of them lie inside the narrowed frame.
    - "fills most of an iPhone canvas" uses a per-scene `MIN_WIDTH_FRACTION` override, defaulting to 0.85, only for tesla-valve. The narrowed valve world is about 0.25 aspect against the phone's 0.46, so height becomes the binding fill and width cannot exceed about 0.6.
    - `MIN_HEIGHT_FRACTION["tesla-valve"]` returns to 0.80, or higher if the measured value supports it.
  </behavior>
  <action>
    Step 0, preserve the evidence: before any Playwright run, which wipes `web/test-results/`, copy `web/test-results/hud-safe-frame-visual/after-tesla-valve-*.png` to `<scratchpad>/wkd/baseline/`.

    Step 1, derive the wall extent from the scene source instead of guessing. Add a temporary `#[test] fn tesla_extent_probe()` to crates/liquidfun-wasm/src/scene/tesla_valve/tests.rs. It prints min/max x and y over every segment endpoint of `outline_segments(true)` and `outline_segments(false)`, and the extreme-x endpoint coordinates. Run `cargo test -p liquidfun-wasm tesla_extent_probe -- --nocapture`. A new debug test binary can stall for about 2 minutes on the macOS security scan, so use a generous timeout. Then remove the probe; `git diff crates/` must be empty. Wall faces sit `WALL_HALF_THICKNESS` (0.03) outside the centerlines. Cross-check the result against the planner estimate (x about ±0.49 with thickness). If they disagree by more than 0.05, trust the probe and note it.

    Step 2, tests first (RED where applicable):
    - In web/tests/portrait-bounds.test.ts, add the probe's extreme-x centerline endpoints (rounded outward to 3 decimals) to the "tesla-valve" wall list.
    - Add `const MIN_WIDTH_FRACTION: Partial<Record<SceneId, number>>`. Its only entry is "tesla-valve", with a comment saying the valve is tall and narrow, so height is the binding fill on a phone and the side margins were removed to keep it clear of the HUD stats column (quick 261010-wkd). Use `MIN_WIDTH_FRACTION[fraction.id] ?? 0.85` in the width assertion. Set the tesla-valve value after measuring in step 3, about 0.05 below the measured width fraction.
    - Set `MIN_HEIGHT_FRACTION["tesla-valve"]` to 0.80 and update its comment.

    Step 3, frame:
    - In web/src/catalog/portrait-bounds.ts, set `TESLA_VALVE_PORTRAIT.minX`/`maxX` to the probe's wall-face extent (centerline extreme ± 0.03) plus a small margin of about 0.08-0.10 m, rounded outward to 2 decimals. Expect roughly ±0.58. Keep it symmetric unless the probe shows a clearly asymmetric extent.
    - Keep `minY: -0.2` and `maxY: 4.6` for now.
    - Update the doc comment: the rectangle hugs the valve's walls on all sides, derived from the scene geometry, and the chrome-aware fit places it clear of the HUD.
    - Run `bunx vitest run tests/portrait-bounds.test.ts`, read the measured tesla-valve width and height fractions (temporarily log them if needed), set the overrides as above, and get to green.

    Step 4, browser check at 390x844:
    - Kill any stale `vite preview` on port 4173, because Playwright reuses an existing server. Build with provenance: `just web-build` (scripts/web-build.ts sets `VITE_BUILT_AT`; the wasm is unchanged).
    - With a throwaway Playwright script in `<scratchpad>/wkd/`, or a temporary unchecked spec that you delete afterwards, open tesla-valve while playing at 390x844 and at 1280x800. Read `data-world-frame` and compute pixels per metre = min(frameW / worldW, frameH / worldH) using the portrait bounds at 390x844. Read the scale legend label. Save screenshots to `<scratchpad>/wkd/after-tesla-valve-{390x844,1280x800}.png`, and optionally to `web/test-results/` (gitignored). Nothing goes into git.
    - Acceptance at 390x844: px/m is clearly above the 261010-vzz value of about 98.5 (target at least 5% larger), and the world-frame horizontal centre is within about 30 px of x=195.
    - If narrowing X alone does not make it larger (for example, the fit is still capped at 473 px of height between the title and the slider card), tighten Y as well: set `maxY` to the probe's top wall face plus about 0.08 and `minY` to the probe's bottom wall face minus about 0.1. Keep the walls inside, re-run step 3 and step 4, and say so in the SUMMARY.
    - If it still is not larger, stop and report the measured numbers honestly. Do not change safe-frame.ts in this task.
    - 1280x800 uses the landscape `worldBoundsForScene`, so it should be unchanged from the 261010-vzz after image. Confirm visually.

    Step 5, browser suites: `cd web && bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts`. The toggle round-trip test needs tesla-valve's frame area to grow when the sliders hide. A narrower, height-limited world should still grow, because hiding the card frees vertical space. If it does not, the geometry has legitimately changed: switch the round-trip to a scene whose frame grows, explain this in the spec comment, and record it as a deviation. Do not weaken the assertion.

    Commit `fix(quick-261010-wkd): fit the Tesla valve portrait frame to its walls`.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/web && bun run test:unit && bun run typecheck && cd .. && just web-build && cd web && bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts && cd .. && bun scripts/bright-builds-check.ts all && just markdown-check && cargo fmt --all --check && git diff --quiet -- crates/</automated>
  </verify>
  <done>`TESLA_VALVE_PORTRAIT` is derived from the probed wall extent plus a small margin. Every tesla-valve wall point, including the extremes, is inside the frame. The tesla-valve height threshold is 0.80 or higher, and its width override is justified in a comment. At 390x844 the valve's px/m beats 98.5 and its frame centre is within about 30 px of x=195, with the numbers recorded. The 1280x800 view is unchanged. Both Playwright specs pass, and no Rust changes are committed.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| localStorage -> HUD toggle | The persisted `liquidfun.hud-sliders.v1` value is read on load. This task does not change that path. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-wkd-01 | T | hud-sliders-preference read | accept | Unchanged. The existing try/catch and unknown-value-to-"shown" handling from 261010-vzz still applies. The task only changes static SVG and constant bounds. |
| T-wkd-02 | I | screenshots/probe output | mitigate | Screenshots stay in the scratchpad or gitignored `web/test-results/`. The Rust probe is removed before commit, enforced by `git diff --quiet -- crates/`. |
</threat_model>

<verification>
- `cd web && bun run test:unit` passes, with the new icon guards and portrait-bounds changes.
- `bun run typecheck` is clean. web/package.json has no lint script.
- `just web-build`, then `bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts`, all pass.
- `bun scripts/bright-builds-check.ts all` reports findings=0.
- `just markdown-check` passes.
- The before/after tesla-valve screenshots at 390x844 and 1280x800 are in the scratchpad. The 390x844 numbers (px/m, frame centre x, legend label) are recorded in the SUMMARY.
</verification>

<success_criteria>
- The sliders toggle glyph is a panel with a direction-aware chevron, not the slider/Controls glyph. Tests guard against reusing ControlsIcon.
- The Tesla valve portrait frame hugs the probed walls. At 390x844 the valve is larger than in 261010-vzz and roughly centred.
- All listed checks pass. Any test-premise change is reported as a deviation.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-wkd-distinct-hud-slider-toggle-icon-and-tigh/261010-wkd-SUMMARY.md`. Its frontmatter must include `generated_by: gsd-executor`, `lifecycle_mode: quick`, `phase_lifecycle_id: quick-261010-wkd`, and `generated_at` taken from a real `date -u +%Y-%m-%dT%H:%M:%SZ`. Record the probe extents, the chosen bounds, the measured width and height fractions, the 390x844 px/m, frame centre, and legend label before and after, any Y tightening, any e2e premise change, and the screenshot paths.
</output>
