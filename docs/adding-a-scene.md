# Adding a playground scene

This checklist lists every Rust and web site that a new playground scene touches, the gates it must pass, and how the protected scene fingerprint set grows afterwards. It describes the tree after Phase 45, where scene counts are derived from the catalog instead of typed by hand. Each row says what enforces it: `compile` (the build fails until the site is filled), `test` (a unit or e2e test fails), `compile + test`, `convention` (nothing fails; follow it anyway), or `optional` (only some scenes need it).

In the paths below, `<id>` is the kebab-case catalog id (for example `tesla-valve`), `<module>` is the same id in snake_case (`tesla_valve`), `<Variant>` is the `SceneId` variant (`TeslaValve`), and `<family>` is the catalog family of the new scene (`curiosity` for the v1.6 scenes).

## Before you start

- Use toy meter scale: particle radius about 0.04 to 0.06 m, geometry within about 10 m of the origin, thick overlapping polygon walls and default gravity 10 m/s².
- Keep the scene to about 3,000 particles or fewer. Record its particle count and native release ms/step (see [Gates](#gates)).
- Do not change `crates/liquidfun` by default. Build the mechanism as bridge code in `crates/liquidfun-wasm` on existing `World` APIs.
- Add no new dependencies and no new cue kinds. README cues support only `action` and `pointer-up`, and `SurveyCue` supports only `Default`, `Action` and `PointerUp`. Design the scene so it moves with no cue or one action cue.
- Ship the scene without touching `docs/benchmarks/scene-fingerprints.json`. The baseline grows later, in its own commit (see [Growing the protected fingerprint set](#growing-the-protected-fingerprint-set)).

## Rust bridge

All sites are in `crates/liquidfun-wasm/src`.

| #     | Site                                                                                                                                           | What to add                                                                                                                                                                                                                                                                                                 | Forced by      |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| R1    | `scene/<module>.rs` (plus `scene/<module>/tests.rs` and, if needed, `scene/<module>/geometry.rs`)                                              | The scene builder and its `SceneHooks`, in the `foo.rs` + `foo/` layout. The file named in the catalog credit must be the top-level `scene/<module>.rs`: `web/src/catalog/links.ts` rejects scene paths that contain `/`.                                                                                   | test           |
| R2    | `crates/liquidfun-wasm/src/scene.rs` module list                                                                                               | `mod <module>;`                                                                                                                                                                                                                                                                                             | compile        |
| R3/R5 | `crates/liquidfun-wasm/src/scene.rs` `SceneId` and `build_scene`                                                                               | A `SceneId::<Variant>` and its `build_scene` arm. The match is exhaustive.                                                                                                                                                                                                                                  | compile        |
| R4    | `crates/liquidfun-wasm/src/scene.rs` `parse_scene_id`                                                                                          | A `"<id>" => Ok(SceneId::<Variant>)` arm. A missing arm fails `all_scene_ids_match_the_web_catalog_order`.                                                                                                                                                                                                  | test           |
| R6    | `crates/liquidfun-wasm/src/scene.rs` `particle_iterations`                                                                                     | An arm only if the scene needs more than 2 particle substeps (fine grains, small vessels).                                                                                                                                                                                                                  | optional       |
| R7    | `crates/liquidfun-wasm/src/scene.rs` `ALL_SCENE_IDS`, plus `variant_index` in the tests of `crates/liquidfun-wasm/src/scene/gravity_slider.rs` | Append the variant to `ALL_SCENE_IDS` in catalog order, and add a `variant_index` arm. The order test checks the list; `variant_index` is an exhaustive match.                                                                                                                                              | compile + test |
| R8    | `crates/liquidfun-wasm/src/session/tests.rs` `parse_scene_id_maps_allowlisted_tokens`                                                          | Append the token. The R4 order test already covers it.                                                                                                                                                                                                                                                      | convention     |
| R9    | `crates/liquidfun-wasm/src/scene_spot.rs` `SURVEY_SCENES`                                                                                      | Append `("<id>", SceneId::<Variant>, SurveyCue::…)` in catalog order. The array length is `ALL_SCENE_IDS.len()`, and `survey_scenes_match_catalog_scene_ids` checks the order.                                                                                                                              | compile + test |
| R10   | `crates/liquidfun-wasm/src/scene_spot/tests.rs` `only_idle_scenes_are_scripted`                                                                | Update the scripted list only when the new scene uses a non-`Default` cue.                                                                                                                                                                                                                                  | test           |
| R12   | `crates/liquidfun-wasm/src/session/escape.rs` `EscapeLimits::for_scene`                                                                        | An arm only if the playfield exceeds 12 m of fall or 48 m sideways (the Wave Tank precedent).                                                                                                                                                                                                               | optional       |
| R13   | `scene/<module>/motion_tests.rs`                                                                                                               | Timing tests for a scene whose motion depends on release-mode physics (for example sink and rise windows). Mark each test `#[cfg_attr(debug_assertions, ignore = "release-only ...")]` so the debug suite lists them as ignored, and run them with `cargo test -p liquidfun-wasm --release --lib <module>`. | optional       |

## Web catalog and player

All sites are in `web/src/catalog` unless the path says otherwise.

| #     | Site                                                                                    | What to add                                                                                                                                                                                                                                                                                                                                                         | Forced by      |
| ----- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| W1    | `web/src/catalog/scenes.ts` `SCENE_IDS`                                                 | Append the id. This list is the source of truth for sidebar, README, survey and demo-media order. The Rust order test fails until R4 and R7 match it, and every `Record<SceneId, …>` stops compiling until it has the new key.                                                                                                                                      | test           |
| W2    | `scene-records-<family>.ts`                                                             | The scene record: `withGravitySlider([...])`, `WATCH_FIRST_HINT` or a written hint, `sceneSource("<module>.rs")`, `[SHOWCASE]` and `viewBounds`. For the first scene of a new family, create the module, export its array and append it to the `SCENES` spread in `web/src/catalog/scene-records.ts`, keeping catalog order. Checked by `web/tests/scenes.test.ts`. | test           |
| W3/W4 | `<id>-controls.ts`; `DEFAULT_PRESET_VALUES` in `web/src/components/scene-controls.ts`   | Only for live controls. Copy `web/src/catalog/washing-machine-speed.ts` (`surface: "hud"`, `widget: "spinner"`, `recreates: false`). Add preset defaults only for new `preset` controls. Control ids share one namespace across scenes, so pick unique ids.                                                                                                         | optional       |
| W5    | `previews-<family>.tsx` and the `ScenePreview` switch in `web/src/catalog/previews.tsx` | A `<Variant>Preview()` component and its `case`. The switch ends in a `never` guard, so a missing case fails to compile. Shared pieces are in `web/src/catalog/preview-kit.tsx`.                                                                                                                                                                                    | compile        |
| W6    | `web/src/catalog/portrait-bounds.ts` `PORTRAIT_VIEW_BOUNDS`                             | A phone frame sized to the scene's action, walls included (AGENTS.md "Playground canvas framing"). A custom `viewBounds` constant lives in `web/src/catalog/scene-view-bounds.ts`. To reuse it here, re-export it from `scene-records.ts` and then `scenes.ts`, because `portrait-bounds.ts` imports from `./scenes`.                                               | compile + test |

## Scripts, tests and e2e

| #   | Site                                                                                                   | What to add                                                                                                                                              | Forced by      |
| --- | ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| S1  | `web/scripts/readme-svg/plans.ts` `README_SVG_PLANS`                                                   | Append `{ id, cues }` in catalog order. `assertReadmeSvgPlanCoverage` (run by `web/tests/readme-svg.test.ts`) fails on a missing, extra or reordered id. | test           |
| S2  | `docs/assets/readme/<id>-10s.svg` and its WebP                                                         | Run `just readme-svg`. It writes both files and refreshes the README gallery. The post-merge Readme scene preview workflow runs the same command.        | convention     |
| S3  | `web/scripts/demo-media/model.ts` `SCENE_CAPTURE_PLANS`                                                | Append `{ id, title, route, interactionStep, action }`. The module-load assert and `web/tests/demo-media-model.test.ts` check it against `SCENE_IDS`.    | test           |
| T1  | `web/tests/scenes.test.ts` `UI_SPEC_DESCRIPTIONS`, `UI_SPEC_HINTS` and the explicit id and title lists | Add the entries. The two records are `Record<SceneId, string>`; the lists are UI-SPEC contracts.                                                         | compile + test |
| T2  | `web/tests/scene-catalog-controls.test.ts` `RECREATING_CONTROL_IDS`                                    | Append `"gravity"` and any other recreating control, in catalog order.                                                                                   | test           |
| T3  | `web/tests/scene-catalog-credits.test.ts` `expectedPaths`                                              | The `crates/liquidfun-wasm/src/scene/<module>.rs` credit path.                                                                                           | compile        |
| T4  | `web/tests/portrait-bounds.test.ts` `MIN_HEIGHT_FRACTION`                                              | A height fraction, plus an optional `MIN_WIDTH_FRACTION` entry and wall points.                                                                          | compile + test |
| T6  | `web/tests/controls.test.ts`                                                                           | A HUD control test for each new live control. Split the file before it reaches 628 lines.                                                                | convention     |
| E1  | `web/e2e/player-helpers.ts` `SCENE_HASH_PATHS`                                                         | The `#/…` route. `bun run typecheck` includes the e2e project.                                                                                           | compile        |
| E2  | `web/e2e/player.spec.ts` `POINTER_CONTROL`                                                             | `{ gesture, control }` for interactive scenes. It is a `Partial` record, so a missing entry silently drops the scene into the watch-first loop.          | convention     |
| E5  | `web/e2e/player-helpers.ts` `ALL_SCENE_TIMEOUT_MS`                                                     | Raise it only if the sequential all-scene loops in `web/e2e/player.spec.ts` time out.                                                                    | optional       |

## Already derived (no edit)

Phase 45 removed these per-scene edits. Leave them alone:

- `PAGE_SUMMARY` in `web/src/player/runtime.ts` names no count.
- `web/e2e/shell.spec.ts` derives the drawer focus cycle (`SCENE_IDS.length + 2`), the last link title (`SCENES.at(-1)`) and the preview count from the catalog.
- `web/tests/scenes.test.ts` and `web/tests/demo-media-model.test.ts` use `SCENE_IDS.length` and `SCENE_IDS.at(-1)`.
- The gravity slider `seen` arrays, the `SURVEY_SCENES` length and the puzzle catalog-length check use `ALL_SCENE_IDS.len()`.
- `web/e2e/hud-safe-frame.spec.ts` iterates `SCENE_IDS`, so each new scene gets its HUD safe-frame cases automatically.
- Hash routing, the sidebar, `ScaleLegend`, the gravity slider control (through `withGravitySlider`) and the scene list parsed by `tools/xtask/src/playground/spot.rs` all come from the catalog.
- `docs/benchmarks/scene-survey.md` is a dated record. Do not update its counts.

## Gates

Run these before the scene's phase closes:

- **Scene fingerprint gate:** `just scene-fingerprint-gate` runs a fresh full-catalog capture and compares it with `docs/benchmarks/scene-fingerprints.json` by scene id. Use `just scene-fingerprint-gate --stamp <utc>` or `just scene-fingerprint-gate --spot <path>` to read a saved capture. Every protected id must be unchanged (25 at Phase 45). The new id appears under "added (accepted)", which passes.
- **Baseline growth rule:** the protected set grows only in its own commit after the scene ships (next section).
- **README plan coverage:** `just readme-svg` for the assets, and `web/tests/readme-svg.test.ts` for `assertReadmeSvgPlanCoverage`.
- **Portrait frame:** `web/tests/portrait-bounds.test.ts` checks that every portrait frame fills an iPhone-sized canvas.
- **HUD safe frame:** `web/e2e/hud-safe-frame.spec.ts` (build first with `just web-build`, then `cd web && CI=true bunx playwright test e2e/hud-safe-frame.spec.ts`).
- **Scale legend:** `web/tests/scale-legend.test.ts`. The scene must show the shared legend and publish its camera scale in meters.
- **Credits path rule:** `web/tests/scene-catalog-credits.test.ts` and `web/src/catalog/links.ts` accept only a top-level `crates/liquidfun-wasm/src/scene/<module>.rs` path.
- **Particle budget and ms/step:** about 3,000 particles or fewer at toy meter scale. Record the native release ms/step, the `median_ms_per_step` that `cargo xtask playground scene-spot --scene <id>` writes to `target/dam-break-perf/<stamp>/scene-spot.json`, together with `start_particles` and `end_particles`, in the phase summary.
- **Repository checks:** `bun scripts/bright-builds-check.ts all`, which includes the 628-line file-length check.
- **Web unit tests and types:** `cd web && bun run typecheck && bun run test:unit` runs the unit gates above together.

## Growing the protected fingerprint set

The protected set grows only in its own commit, after the scene ships, and never in the same commit as a scene or shared-code change. The gate itself is the xtask. This is the manual path for adding a shipped scene to the protected set:

1. Ship the scene in its own commits with the gate passing: every protected id unchanged and the new id listed under "added (accepted)".
1. In a separate commit that changes nothing else, append `{ "id": "<id>", "fingerprint": "<16 hex>" }` to `scenes` in `docs/benchmarks/scene-fingerprints.json`, in catalog order, from a fresh full-catalog capture (`just playground-scene-spot`). Update `provenance` (`git_commit`, `scene_spot_stamp`, `captured_at`, and the host fields if they changed). For example, read the new fingerprint with `jq -r --arg id <id> '.scenes[] | select(.scene == $id) | .fingerprint' target/dam-break-perf/<stamp>/scene-spot.json`.
1. Rerun `just scene-fingerprint-gate --spot target/dam-break-perf/<stamp>/scene-spot.json`. It must report every protected id unchanged, including the new one.

Manual comparison (fallback, not the gate). Write sorted `id fingerprint` lines for the baseline and the capture, then:

```sh
jq -r '.scenes[] | "\(.id) \(.fingerprint)"' docs/benchmarks/scene-fingerprints.json | sort > base.txt
jq -r '.scenes[] | "\(.scene) \(.fingerprint)"' target/dam-break-perf/<stamp>/scene-spot.json | sort > after.txt
join base.txt after.txt | awk '$2 != $3 {print "CHANGED", $1; bad=1} END {exit bad}'
test "$(join base.txt after.txt | wc -l)" -eq "$(wc -l < base.txt)"
```

The line-count check catches a protected id that is missing from the capture, because `join` drops unmatched lines.

## Related guidance

- [README scene gallery](../AGENTS.md#readme-scene-gallery)
- [Playground canvas framing](../AGENTS.md#playground-canvas-framing)
- [Playground scale legend](../AGENTS.md#playground-scale-legend)
- [Playground scene spot-check](playground-scene-spot-check.md)
