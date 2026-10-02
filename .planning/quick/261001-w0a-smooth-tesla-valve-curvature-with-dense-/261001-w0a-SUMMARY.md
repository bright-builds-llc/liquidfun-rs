---
phase: quick-261001-w0a
plan: "01"
status: complete
generated_by: gsd-execute-plan
generated_at: "2026-10-02T05:25:32Z"
source_commit: 488105771bdc87dee87438943e26f4b6faf6f9db
source_diff_sha256: 19d2e4360323388816757c79f4d9058ae12c8bba30240c2713645f3e2c94858a
---

# Quick task 261001-w0a: Smooth Tesla Valve and finer water

The valve now draws 300 segments per orientation, versus 60 before, using analytic cubic outer curves and rounded solid circle-plus-tangent-triangle islands. Maximum centerline chord error is 0.438 mm for the cubic walls and 0.179 mm for the island arcs. Existing framing and widened neck parameters are retained; actual physical wall-box clearance exceeds 60 mm, enough for the new 10 mm particle diameter.

Particles have radius 5 mm instead of 14 mm. The source defaults to 1440 particles/s instead of 180; the slider reaches 2880 in steps of 30. A 24×2 grid at 14 mm spacing emits 24 default or 48 maximum distinct particles per frame at 60 Hz. The cursor stays within its 48-slot cycle. Capacity is bounded at 16384, and fractional credit, zero rate, invalid tokens and live direction changes are tested. Rust and TypeScript frame bounds both accept at most 512 segments, with dense-limit acceptance and overflow rejection tests.

## Collision design and retained failures

Thin sampled chains and rounded cusp variants failed exact pressure containment. Those attempts are preserved under `target/tesla-smooth-validation/native-attempt*-failure.txt` and `strict-containment-attempt1.txt`; initial camera-only measurements do not establish conduit containment. No tolerance was accepted to hide the failures.

The final wall uses 196 checked `PolygonShape::oriented_box` fixtures derived from the exact displayed curve segments. Each box uses its segment's midpoint, angle and length, the original 30 mm physical half-thickness, and 10 mm endpoint overlap. The checked box constructor supports short positive lengths without the general polygon hull constructor's welding threshold. Drawing presents the shared wall centerlines, as the earlier thick-wall representation did. This restored thickness protects sharp junctions without cusp beads, engine tolerance changes or extra substeps. The four circle/triangle islands remain solid; the union's external outline is sampled from the same analytic data.

Pure geometry/emitter tests live in `tests.rs`; sustained behavior checks are in the new `flow_tests.rs`. Temporary filesystem diagnostics were removed. No production physics-engine files changed.

## Runtime evidence

Exact native and production-WASM conduit checks pass every four steps over four seconds at both rates and in both directions. The inlet remains intentionally open. Final WASM snapshots showed zero particles outside the camera frame as well. Counts are actual backend observations; the new analytic geometry produces small native/WASM differences, so no exact-bit or cross-runtime parity claim is made.

| Four-second case | Native live / discharged | WASM live / discharged | WASM average / p95 step time |
| --- | --- | --- | --- |
| 1440/s forward | 1950 / 3810 | 1940 / 3820 | 7.9 / 9.9 ms |
| 1440/s reverse | 5747 / 13 | 5740 / 20 | 14.1 / 27.8 ms |
| 2880/s forward | 3976 / 7544 | 3984 / 7536 | 15.7 / 19.4 ms |
| 2880/s reverse | 11256 / 264 | 11279 / 241 | 29.3 / 57.9 ms |

Evidence: `target/tesla-smooth-validation/native.json`, `geometry.json`, `wasm-performance.json`, `measure.ts` and `wasm-performance-final.log`. These are local diagnostics under concurrent desktop load, not a controlled hardware benchmark or a general 60 fps promise. Maximum reverse flow is heavier because water backs up. The existing default render cap can subsample large particle fills while simulation continues.

## Verification

- Required core checks passed in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features` (1032 tests). Native checks use the proven isolated `CARGO_TARGET_DIR=target/main-pull-resolution/native-check`.
- Strict affected-package Clippy/build and `cargo test -p liquidfun-wasm --all-features` passed all 292 tests in 274.78 seconds. Logs: `target/tesla-smooth-validation/core-*.log` and `wasm-*.log`.
- Fresh WASM/web builds, TypeScript checks and all 404 unit tests passed. Dense frame tests first failed against the old 64-segment bound, then passed with matched 512 limits.
- The initial browser run passed 43 tests but missed an existing startup timing assertion: Fountain advanced to step 9 while the test expected fewer than 8, during the heavy native run. The failed log/trace/context are preserved as `browser-attempt1-failure*`. The complete fresh rerun passed all 44 tests in 39.1 seconds; `browser-test-final.log` is the passing evidence. No threshold was weakened.
- Managed Bright Builds, repository Markdown and diff checks passed. Gallery regeneration recorded 600 frames at 60 fps for ten seconds, updating only Tesla Valve SVG/WebP; WebP size is 6,026,688 bytes. The existing explicit local Arial font setting is unchanged.
- The existing in-app preview was restored after its server had stopped, and refreshed to show the new curves, 1440/s controls and smaller stream. Final preview refresh follows publication so its provenance matches HEAD.

## Independent review and finalization

Separate AI reviewer `/root/tesla_review` acknowledged the exact source digest `19d2e4360323388816757c79f4d9058ae12c8bba30240c2713645f3e2c94858a` at actual review time 2026-10-02 05:17:04 UTC, with no actionable findings. Review covers all eleven source/media files, including the new flow-test module, final passing evidence and failed attempts. Record: `target/tesla-smooth-validation/review.md`.

Source commit: `488105771bdc87dee87438943e26f4b6faf6f9db` (`feat(playground): smooth Tesla Valve and refine particle flow`). Ordinary main publication uses AGENTS.md standing authorization. Prior wider-neck publication integrated the preview bot's compatible asset updates through `c5bc0a3`; no history was rewritten. The final chat reports the actual push result. The local guidance, Bright Builds sidecar/overrides and current hobby scope informed the work; this is experimental scene evidence, not a package release or fluid-diode engineering certification.
