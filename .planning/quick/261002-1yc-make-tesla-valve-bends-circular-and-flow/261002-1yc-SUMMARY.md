---
phase: quick-261002-1yc
plan: "01"
status: complete
generated_by: gsd-execute-plan
generated_at: "2026-10-02T07:42:56Z"
source_commit: c2b5e364623dba0cb62ac8b1956ed2f8e9db56dd
source_diff_sha256: fee38086a717bf52c1b7380e8a3ac8b7b63c230f19989e8cb0a2c5aa271995fd
---

# Quick task 261002-1yc: Circular bends and uniform pipe width

The requested checkpoint was already committed and published. Main integrated the compatible preview-bot update at `b77b116`, and the initial normal push reported everything up to date. This task implements the subsequent requested curvature and channel-width changes.

## Geometry

The winding trunk and four alternating bypasses share one 0.17 m clear width. Nominal border separation is 0.23 m; both borders have the same 0.03 m physical wall half-thickness. Each bypass centerline is a true 180-degree circle at radius 0.17 m, with concentric nominal borders at 0.055 and 0.285 m. Tangent arms use 60 degrees in a rigid orthonormal stage basis. No unequal axis scaling or independently fitted outer curves remain.

Four trunk legs alternate by 20 degrees at a vertical pitch of 0.85 m. The trunk turns have 0.30 m centerline radii and 40-degree sweeps. Their nominal border radii are 0.185 and 0.415 m. The channel has uniform width along its straight runs and bends; fork/merge junctions necessarily contain wider transition regions. Shared analytic boundaries generate the 420 displayed segments and overlapping checked wall boxes. Splitter islands use a filled circle plus a convex four-point stem and the same thick border fixtures. No physics-engine files or dependencies changed.

The source remains 5 mm particles at 1440/s by default and 2880/s maximum, with speed magnitude 2 m/s. Its 16×3 grid at 10.2 mm spacing now follows the tilted inlet. Port extensions, mirror axis y=2.15 and the existing drain threshold y=0.32 keep both orientations contained. Desktop maxY is 4.15; portrait bounds [-1.2,-1.4,1.2,4.35] leave both ports clear of the controls at 390×844. The shared metric legend remains visible.

Simplification review: one centerline/width and rigid transforms replace unrelated curve fits. Existing checked box fixtures retain pressure containment; no boolean geometry dependency, tolerance relaxation, guard beads or engine change is needed.

## Flow evidence

Exact native and production-WASM checks inspect the conduit boundary and exclude all four solid islands every four simulation steps. All four cases pass for four seconds. Native and WASM counts differ slightly, so these results do not claim exact-bit cross-runtime parity.

| Rate and orientation | Native live / discharged | WASM live / discharged | WASM average / p95 step |
| --- | --- | --- | --- |
| 1440/s forward | 2414 / 3346 | 2416 / 3344 | 15.8 / 20.1 ms |
| 1440/s reverse | 5449 / 311 | 5457 / 303 | 24.1 / 44.9 ms |
| 2880/s forward | 4808 / 6712 | 4816 / 6704 | 30.7 / 39.4 ms |
| 2880/s reverse | 10798 / 722 | 10759 / 761 | 48.2 / 90.6 ms |

Evidence is under `target/tesla-circular-validation`: `native.json`, `wasm.json`, `measure.ts` and associated logs. Production WASM first discharge steps were 100/128/100/120 respectively. Native tests preserve earlier forward arrival and a discharge margin of 300 after three seconds. These are fixed-window local simulation observations under concurrent desktop load, not physical-valve efficiency measurements or controlled performance benchmarks. Longer reverse runs accumulate water and can be slower; the observed phone preview reached roughly 10–12 fps after buildup. The maximum live-particle and render bounds remain unchanged; 60 fps is not promised.

## Verification and retained failures

- Ordered core format, strict Clippy, build and tests passed in `target/main-pull-resolution/native-check`. Core checks total 1032 tests including doctests; logs are `core-*.log` and the final ordered `precommit-*.log`.
- Strict affected-package Clippy and build passed; all 294 affected-package tests passed in 396.77 seconds (`wasm-test.log`).
- Production WASM/web builds, TypeScript checks and all 404 web unit tests passed. Typechecking initially raced a WASM rebuild that temporarily removes generated bindings; it passed after the build completed. This was a verification ordering issue, not a source failure.
- Managed Bright Builds, Markdown and diff checks passed. The final live browser showed forward/reverse geometry and working zero/reset controls; the first broad browser run passed 53 tests but failed its opt-in historical forensic smoke because the output-directory environment variable also enabled that smoke without its required provenance.json. Its artifacts are preserved in `browser-attempt1`; the normal 44-test player suite then passed in 38.0 seconds without that opt-in flag (`browser-player-final.log`).
- Gallery regeneration recorded 600 frames at 60 fps for ten seconds, updating only Tesla Valve SVG/WebP. WebP size is 5,159,254 bytes, using the existing explicit local Arial setting.
- The worker's initial red circular-radius assertion exposed the previous 68 mm island radius. A new physical-width measurement initially used the engine's face-separation distance, which is not Euclidean corner distance; the assertion was corrected to measure actual fixture edges without changing engine or geometry. Eleven final pure geometry/emitter tests pass. Physical bend clearance matches the 85 mm half-width within 1.5 mm, accounting for sampled thick-box corners. Temporary filesystem diagnostic tests were removed.

## Review and finalization

Separate AI reviewer `/root/tesla_review` acknowledged exact source digest `fee38086a717bf52c1b7380e8a3ac8b7b63c230f19989e8cb0a2c5aa271995fd` at 2026-10-02 07:41:37 UTC, with no actionable findings. Its complete inspection and acknowledgment are preserved in `target/tesla-circular-validation/review.md`. Source commit: `c2b5e364623dba0cb62ac8b1956ed2f8e9db56dd` (`feat(playground): make Tesla Valve bends circular and pipes uniform`). The final chat reports the actual push result. Ordinary non-force main publication uses AGENTS.md standing authorization. This task uses the GSD quick workflow, current hobby scope, loaded Bright Builds guidance and the appended lesson about deriving paired boundaries from a shared width. No package release or optional strict qualification is claimed.
