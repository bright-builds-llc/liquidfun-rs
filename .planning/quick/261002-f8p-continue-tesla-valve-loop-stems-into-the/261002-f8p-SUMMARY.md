---
phase: quick-261002-f8p
plan: "01"
status: complete
generated_by: gsd-execute-plan
generated_at: "2026-10-02T17:02:38Z"
source_commit: 3de2929f095d8b03de7fb63de846398823044d72
source_diff_sha256: 029d8f1f54dfc142c4fae6df4578a97562aaa55da94d1346c5a5903352c5b759
---

# Quick task 261002-f8p: Loop returns become the main pipe

Each of the four outer loop returns now continues directly along the next main-pipe wall. The obsolete return-to-old-leg segment and its separate loop-side corner are removed from the actual shared drawing/collision geometry. Branch angle 40° matches the next ±20° leg, and anchor L−2R/sin40° places both return borders exactly at the next leg's ±0.06 m nominal offsets. Physical clear width remains 0.06 m after accounting for 0.03 m wall half-thickness. True 0.17 m semicircular centerlines and solid circle/stem islands remain.

The final return also feeds a real outgoing leg. Symmetric 0.18 m inlet/outlet stubs and 20° vertical-port adapters prevent overlapping turns and preserve reverse-mode source alignment. An initial 40° inlet adapter uses both borders; four post-loop turns retain their opposite boundary only. Source plane is approximately (−0.093124,4.272043), outlet (−0.093124,0.027957), with mirror axis y=2.15. Drain top moves to 0.08 m, maintaining the previous roughly five-centimeter outlet gap. Radius 5 mm, speed 2 m/s, 48-slot 4×12 source grid, rates 1440/2880, gravity, substeps and capacity are unchanged. No engine files or dependencies changed.

Simplification pass: one analytic stage template establishes both the shared line and tangent, and only forward-collinear nodes are collapsed. The output has 468 segments within the existing 512 bound. A new topology-test module avoids growing the existing geometry tests. Actual fixture tests retain six-centimeter straight/bend clearances and solid island/outer-union checks.

Landscape top is 4.32; final portrait bounds [−1.35,−1.75,1.35,4.60] keep the expanded ports clear of controls. The thumbnail now draws actual outer/island boundaries instead of independently overlaid branch centerlines; the scene copy and gallery reflect the same topology.

## Verification and evidence

- The new continuity test first failed against the old return direction; retained `attempt1-topology-red.log` proves the previous shape did not meet the request. Fifteen focused debug geometry/source/topology/port/drain tests and strict Clippy pass. Intermediate unused-test-field/literal-style diagnostics and final green logs remain under `target/tesla-continuing-stems-validation`.
- Ordered core format, strict all-target/all-feature Clippy, build and tests pass (1032 including doctests) in the proven isolated target `target/main-pull-resolution/native-check`; `core-*.log` and `precommit-*.log` record results.
- Strict affected-package Clippy/build and all 299 native WASM-package release tests pass in 15.67 seconds. Unchanged arrival, >300 directional margin, >1000 sustained forward drainage and source-off guards pass (`wasm-test-release.log`).
- Production WASM checks exact outer containment and island exclusion every four steps over four seconds at both rates/directions, plus finite/drain state. The upstream opening is excluded using its actual plane; no distance tolerance or rectangular-envelope substitute is added.
- Fresh production WASM/web builds, typechecking and all 404 web unit tests pass. Gallery regeneration records 600 frames at 60 fps over ten seconds; WebP is 4,873,304 bytes. The temporary thumbnail diagnostic initially requested 180 steps in one call, which correctly failed the existing 1–4 WASM bound; its log remains preserved, and the producer was corrected to 45 bounded four-step calls. No production change was needed for that error.
- Two broad browser attempts observed the previous 800 px canvas height immediately after resizing to 1024 px, while the isolated test passed. Logs and traces remain preserved. The resize helper now waits at most five seconds for the same original non-null, width and height assertions; no final criterion is weakened. The full normal rerun passes 53 tests in 40.1 seconds, with one existing optional forensic profile uninvoked (`browser-attempt3.log`).
- Managed Bright Builds, Markdown and diff checks pass. Fresh existing-browser views show all four continuous stems, symmetric ports and the metric legend in normal and phone layouts, in both orientations. An initial phone frame left the outlet too near the HUD; the final small bottom-margin/width adjustment clears it. Forward 1440/s and the normal viewport are restored.

| Four-second production case | Live / discharged | First drain step | Average / p95 step |
| --- | --- | --- | --- |
| 1440/s forward | 2855 / 2905 | 116 | 18.1 / 24.3 ms |
| 1440/s reverse | 5760 / 0 | none | 26.0 / 49.8 ms |
| 2880/s forward | 5796 / 5724 | 116 | 36.7 / 50.8 ms |
| 2880/s reverse | 11512 / 8 | 208 | 52.1 / 101.8 ms |

Records: `wasm.json`, `measure.ts`, `wasm-measure-attempt1.log`, plus native and browser logs in the validation directory. These are fixed-window local experimental observations, not real-world valve efficiency, indefinite pressure containment or a universal 60 fps guarantee. Reverse buildup is more costly. No native/WASM exact-bit parity claim is made.

## Review and finalization

Separate AI reviewer `/root/tesla_review` acknowledged exact source digest `029d8f1f54dfc142c4fae6df4578a97562aaa55da94d1346c5a5903352c5b759` at 2026-10-02 17:00:19 UTC, with no actionable findings. Full record: `target/tesla-continuing-stems-validation/review.md`. Source commit: `3de2929f095d8b03de7fb63de846398823044d72` (`feat(playground): continue Tesla Valve loop returns into the main pipe`). The final chat reports the actual push result. GSD quick workflow, local AGENTS standing authority, Bright Builds sidecar/overrides and loaded architecture/code-shape/testing/verification/language guidance apply. The new active lesson records the owner's correction that return walls define the trunk. Normal main publication is authorized; no package release or optional certification is claimed.
