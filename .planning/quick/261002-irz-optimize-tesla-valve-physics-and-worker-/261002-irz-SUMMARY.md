---
phase: quick-261002-irz
status: in-progress
---

# Tesla performance execution

## Foundation

Created an isolated production-WASM/browser benchmark without changing physics or playground presentation. Original run `20261002-original-01` uses physics source `8c419c4c42aaf99cdfdf8d3d297c184c25d5ae8f` plus the source-bound harness manifest. Three sequential replicates each at forward360/reverse384 warmup and40 fixed steps have identical semantic checkpoints. Mean physics31.374ms forward and110.849ms reverse; actual production-loop paints31.3/s and9.1/s on M4 Max/Chromium153/Metal.

Verification: ordered Cargo fmt/clippy/build/test,1032 native tests including doctests;299 release WASM tests including four rate/direction flow cases;415 web unit tests, typecheck and production build; managed all and Markdown checks pass. Raw reports and methodology are committed before any optimization.

## Remaining stages

Spatial contacts, immutable collision sharing plus scratch reuse, query-only drain, bounded live worker, and rendering cache/batch changes are serial gates. Each accepted predecessor is the next stage's before record; a fresh after record measures the isolated source diff using unchanged fixed workloads. Quality and exact fixed semantic fingerprints remain required.

## Stage 1

Spatial wall contacts use one validated current-index check per pass, collect candidates in stable row order and retain exact fallback/narrowphase behavior. Five legacy-equivalence tests added. Accepted original→stage1 report:6.268ms forward/32.533ms reverse versus31.374/110.849ms. Exact checkpoints match; full native1037/WASM299 and managed/Markdown checks pass. Independent AI review bound to digest6f997a2034a76831600b939079f8926a789a0f0514dbadb58b14ad434ee60844 at2026-10-02 20:17:35UTC. No residual physical behavior change observed.
