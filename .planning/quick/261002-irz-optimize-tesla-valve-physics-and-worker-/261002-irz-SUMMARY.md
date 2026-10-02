---
phase: quick-261002-irz
status: complete
---

# Tesla performance execution

## Foundation

Created an isolated production-WASM/browser benchmark without changing physics or playground presentation. Original run `20261002-original-01` uses physics source `8c419c4c42aaf99cdfdf8d3d297c184c25d5ae8f` plus the source-bound harness manifest. Three sequential replicates each at forward360/reverse384 warmup and40 fixed steps have identical semantic checkpoints. Mean physics31.374ms forward and110.849ms reverse; actual production-loop paints31.3/s and9.1/s on M4 Max/Chromium153/Metal.

Verification: ordered Cargo fmt/clippy/build/test,1032 native tests including doctests;299 release WASM tests including four rate/direction flow cases;415 web unit tests, typecheck and production build; managed all and Markdown checks pass. Raw reports and methodology are committed before any optimization.

## Remaining stages

Spatial contacts, immutable collision sharing plus scratch reuse, query-only drain, bounded live worker, and rendering cache/batch changes are serial gates. Each accepted predecessor is the next stage's before record; a fresh after record measures the isolated source diff using unchanged fixed workloads. Quality and exact fixed semantic fingerprints remain required.

## Stage 1

Spatial wall contacts use one validated current-index check per pass, collect candidates in stable row order and retain exact fallback/narrowphase behavior. Five legacy-equivalence tests added. Accepted original→stage1 report:6.268ms forward/32.533ms reverse versus31.374/110.849ms. Exact checkpoints match; full native1037/WASM299 and managed/Markdown checks pass. Independent AI review bound to digest6f997a2034a76831600b939079f8926a789a0f0514dbadb58b14ad434ee60844 at2026-10-02 20:17:35UTC. No residual physical behavior change observed.

## Stage 2

Private Arc slices share immutable geometry; World-owned workspace reuses candidate and collider/source/hit buffers while refreshing current metadata. Seven new regressions cover sharing, reuse, freshness, validated swaps and error recovery. Full native1044/WASM299 checks pass. Two unchanged-source after runs have exact original checkpoints. Later repeat20261002-stage2-after-02 is accepted:6.125ms forward/32.282ms reverse; small forward gain, reverse overlaps preceding variation, no substantial reverse gain claimed. Reviewer/root/tesla_plan acknowledges digestbb1600d0298831c6031308eb2351f02c50e9e8a091a8c7e0a7e905e1453bd747 at2026-10-02 20:51:09UTC. Consuming failed kernels may drop capacity as before; usable workspace always restored and legacy rollback preserved.

## Stage 3

AABBqueries use the same checked stable index and skip unused pair enumeration; public neighbourhood/raycast paths retain pairs. Six new regressions and full native1050/WASM299 checks pass. Report20261002-stage3-after-01 remains exactly original at all fixed checkpoints:6.042ms forward/31.957ms reverse, about1% change from Stage2 repeat. Independent reviewer/root/tesla_plan acknowledges digest956f6bfe8742c951206e4664d050066d2bdc91fa778731bff07e3b922cec15fb at2026-10-02 21:05:46UTC. Worker preparation remains isolated pending main integration.

## Stage 4

Tesla live simulation runs in a bounded FIFO worker; non-Tesla/export paths retain direct ownership. Actual final-batch capture and owned transferred buffers are measured. Six source-review findings closed (protocol gravity/buffer/abort and transport/visibility races), with deferred regression tests. Test-only quality fixture corrected after missing-entry browser failure; failed attempts remain separate. All1050 native/299 WASM/456 web unit/61 browser tests pass, one optional forensic skip retained. Corrected20261002-stage4-after-02 matches original physics and Stage3 WASM bytes:5.981/31.574ms direct fixed solver;59.53/29.91 new snapshots/s. Reverse main-thread16ms timer lateness45.806→0.371ms; no worker solver-acceleration claim. Candidate01 retained/rejected for lifecycle quality, not chained. Independent reviewer/root/tesla_plan acknowledges digestd1f88b66a557a75bc78207a021ca69db828721c019b424381eb61e77e8a8b926 at2026-10-02 22:30:06UTC.

## Stage 5

Exact native Path2D projection reuse preserves separate stroke/blending order; GPU locations and stable-count exact attributes are cached. Changing counts use original full straight-line uploads without cache copies/scans. All positions, dynamic circles and labels remain live. Raster experiment replaced after exact pixel failure. Native1050/WASM299/web472/browser61 checks pass, one optional skip;14 corrected Main Chromium comparisons have zero differing bytes. Immutable01/02 candidates exposed overhead/variance; finalfresh adjacent committed-renderer before01/correctedafter03 records remain exactly original physically. Render CPU.1175→.1408ms forward/.1992→.2783ms reverse, rangesoverlap andFPS~unchanged; no rendering speedup claimed. Scope requests fulfilled with honest mixed evidence. Independent reviewer/root/tesla_plan acknowledges digestf7ab6f004c85cbbd9d4f7bce699be3cfa9d522b475c88b0ff166be5e75b99d22 at2026-10-02 23:31:32UTC. No actionable code finding.

## Completion

All five selected changes and immutable serial before/after evidence are finalized. Code/evidence commits:f6398f2 foundation,01728fd spatial,0b9ced7 geometry/scratch,8ac4b5d query,3992aee worker,aa48f26 rendering. Normal main push completed tobright-builds-llc/liquidfun-rs ataa48f26. Final live IAB preview refreshed and observedworker/Playing/Forward1440 and60fps; screenshot target/tesla-performance/final-live-preview.png. Managed worktree archived recoverably after needed diagnostic logs were preserved. Aggregate history and source commit index recorded; archived milestone remains unchanged. Residual limits: reverse physics delivers below60steps/s at this density, and rendering caches did not establish a speedup. No approximation, release, tolerance waiver or dropped particles.
