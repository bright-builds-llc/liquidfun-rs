# Phase 23: Baseline pair and named audit - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-21
**Phase:** 23-baseline-pair-and-named-audit
**Mode:** Yolo
**Areas discussed:** SHA-bound live evidence, Audit-bundle stamp layout, Named-function ranking and cause taxonomy, Heap dump gating and dhat placement, Timing-doc refresh honesty

---

## SHA-bound live evidence

| Option | Description | Selected |
|--------|-------------|----------|
| Live pair + live samply at the cited HEAD are DoD | Names must come from real stacks; hunt list stays unranked until then | ✓ |
| Fake cmake/samply DoD like Phase 22 | Cannot satisfy PERF-AUDIT named functions | |
| Rank hunt-list suspects without a profile | Would pre-select causes and skip SIMD-not-first evidence | |

**User's choice:** [auto] Live unprofiled pair and samply `rust.json.gz` at the audit HEAD. Only `pair.json` wall times are the Dam Break ratio.
**Notes:** Fake-tool tests may still cover new xtask plumbing. Do not quote samply duration as the 3× number.

---

## Audit-bundle stamp layout

| Option | Description | Selected |
|--------|-------------|----------|
| New exclusive stamp that copies pair + `rust.json.gz` from same-HEAD siblings | Matches “one dated directory” without overwriting Phase 22 stamps | ✓ |
| Cite two sibling stamps in the audit | Weaker match to success criterion 2 | |
| Merge into the existing pair stamp | Violates Phase 22 never-overwrite / never-mix rule | |

**User's choice:** [auto] Mint a new exclusive audit-bundle stamp; copy, never move or overwrite.
**Notes:** Thin `just` alias; xtask fails closed unless source stamps share HEAD and contain expected files.

---

## Named-function ranking and cause taxonomy

| Option | Description | Selected |
|--------|-------------|----------|
| Committed `docs/native-performance-audit.md` with named stacks, four cause buckets, ratio at HEAD, and “Not found” | PERF-AUDIT; keeps SIMD from being the first move | ✓ |
| Paste flamegraphs and raw `.json.gz` into git | Blobs expire poorly and are gitignored by policy | |
| Require C++ samply to name counterparts | Extra host tooling; source comparison is enough for shape mismatch | |
| Edit particle/rigid kernels in this phase | Belongs to Phase 24 after names exist | |

**User's choice:** [auto] Named Rust functions + classified causes + explicit not-found; C++ names only for shape mismatch via source comparison.
**Notes:** Banner as unreviewed local sample. Leave `manifest.toml` empty.

---

## Heap dump gating and dhat placement

| Option | Description | Selected |
|--------|-------------|----------|
| Private `dhat-heap` on `dam-break-bench` only, after samply shows allocator/`Vec` time | PERF-HEAP; dhat stays off the gate binary and off `liquidfun` | ✓ |
| Always run dhat | Wastes a global-allocator run when CPU is not allocator-bound | |
| Put dhat on `liquidfun` | Breaks package isolation | |
| Make `xctrace` Allocations the scripted default | Host-heavier; keep as fallback if dhat is blocked | |

**User's choice:** [auto] Conditional private dhat; skip and record if samply shows no allocator/`Vec` time.
**Notes:** Thin `just playground-dam-break-heap` alias. Dump stays gitignored with `not_timing_authority`.

---

## Timing-doc refresh honesty

| Option | Description | Selected |
|--------|-------------|----------|
| Refresh `docs/playground-dam-break-timing.md` as SHA-bound unreviewed sample | Success criterion 4; keep honesty banner | ✓ |
| Promote numbers into `reference/performance/manifest.toml` | Would become a false Phase 12 public claim | |
| Leave the `1e5cbcc…` table as if current | Audit HEAD would not match the published sample | |

**User's choice:** [auto] Replace the current recorded sample with the D-01 unprofiled pair; keep unreviewed banner and locked recipe.
**Notes:** Cite the pair or audit-bundle stamp path. Exact historical-pointer wording is discretion.

## Claude's Discretion

- Exact xtask subcommand and identity JSON field names
- Allocator/`Vec` symbol matcher details
- Heap dump in audit-bundle vs sibling stamp
- How many named stacks beyond dominating shares
- Optional one-line pointer to the first exploratory sample

## Deferred Ideas

- Shared hot-path physics edits and ≤ 3× gate — Phase 24
- Other-scene spot-checks — Phase 24
- WASM sanity — Phase 25
- SIMD / Rayon / `unsafe` — later opt-in
- Required C++ samply wrap
- Criterion micros near 3×
- Manifest/README speed claims
