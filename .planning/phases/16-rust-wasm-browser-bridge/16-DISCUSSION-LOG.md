# Phase 16: Rust WASM Browser Bridge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-17
**Phase:** 16-rust-wasm-browser-bridge
**Mode:** Yolo
**Areas discussed:** Browser proof, Bridge and frame contract, Build and dependency isolation, Verification and phase boundaries

---

## Browser proof

### What the proof demonstrates

| Option | Description | Selected |
| --- | --- | --- |
| Minimal persistent particle-and-rigid-body basin | Real Rust physics visibly changes bulk frame state across browser steps without building the complete player. | ✓ |
| Compile-only WASM proof | Establishes target compatibility but does not prove browser instantiation, rendering, or real motion. | |
| Complete Dam Break player | Delivers more product surface but crosses into Phase 17 and Phase 18 responsibilities. | |

**Choice:** Minimal persistent particle-and-rigid-body basin.
**Notes:** The real-browser outcome must be traceable to changing Rust frame arrays rather than canned JavaScript animation.

### Frontend extent

| Option | Description | Selected |
| --- | --- | --- |
| Minimal SolidJS canvas proof | Uses the milestone stack and exposes loading, running, failure, and Rust/WASM identity status. | ✓ |
| Full shared player shell | Adds playback, reset, navigation, and robust lifecycle behavior owned by Phase 17. | |
| Raw HTML harness | Reduces initial code but fails to prove the approved SolidJS package integration. | |

**Choice:** Minimal SolidJS Canvas 2D proof with a dark default.
**Notes:** Gallery navigation, polished controls, and stable scene URLs remain later work.

---

## Bridge and frame contract

### Live ownership

| Option | Description | Selected |
| --- | --- | --- |
| Opaque owned session with explicit disposal | Keeps world and handles private, models one owner, and gives the web shell a deterministic cleanup operation. | ✓ |
| Stateless calls rebuilding a world | Avoids lifecycle state but cannot support persistent stepping efficiently or naturally. | |
| Expose engine handles to JavaScript | Leaks internal identity and invalidation concerns across an experimental boundary. | |

**Choice:** One opaque session owns one world and exposes checked methods plus explicit disposal.
**Notes:** A trap invalidates the session and requires recreation.

### Frame transport

| Option | Description | Selected |
| --- | --- | --- |
| Owned copied typed arrays | Preserves safety across memory growth and sends bulk particle/shape data with a constant number of calls. | ✓ |
| Zero-copy WASM memory views | May reduce copies but needs a measured benefit and explicit invalidation contract. | |
| Per-particle getters | Simple to expose but violates the bulk-boundary requirement and creates excessive crossings. | |

**Choice:** Owned copied typed arrays with documented element types and strides.
**Notes:** Keep particle positions, colors, radii, and bounded rigid geometry separate from coarse status values.

### API breadth

| Option | Description | Selected |
| --- | --- | --- |
| Proof-sized typed API | Exposes construction, ordinary stepping, frame capture, coarse counts/status, and disposal only. | ✓ |
| Full engine bindings | Creates a broad unstable browser API before concrete consumers establish its needs. | |
| Evidence protocol compatibility API | Imports repository-development semantics into a user-facing runtime. | |

**Choice:** Proof-sized typed API.
**Notes:** Use public engine views directly and avoid speculative changes to `liquidfun`.

---

## Build and dependency isolation

### Rust package placement

| Option | Description | Selected |
| --- | --- | --- |
| Private wrapper workspace crate | Keeps browser dependencies outward from the public engine and preserves native default members. | ✓ |
| Add `wasm-bindgen` directly to `liquidfun` | Leaks browser concerns into ordinary Cargo consumption. | |
| Wrap differential or testbed crates | Pulls private protocol, renderer, and native assumptions into the browser runtime. | |

**Choice:** Unpublished `liquidfun-wasm` crate with `cdylib` plus `rlib`.
**Notes:** It depends only on the public `liquidfun` crate.

### Frontend and generated package

| Option | Description | Selected |
| --- | --- | --- |
| SolidJS, TypeScript, Vite, Bun, and `wasm-pack` | Matches the approved milestone and managed defaults while producing a static browser proof. | ✓ |
| Raw JavaScript | Avoids package setup but does not establish the intended typed frontend boundary. | |
| SolidStart or another server framework | Adds server features that GitHub Pages and this static proof do not need. | |

**Choice:** SolidJS/TypeScript Vite app under `web/`, using Bun and `wasm-pack --target web`.
**Notes:** Direct dependencies and tool inputs are pinned; generated bindings remain ignored.

### Minimal UI dependency

| Option | Description | Selected |
| --- | --- | --- |
| Semantic HTML and scoped CSS | Keeps the proof small while meeting the dark-default and status-feedback needs. | ✓ |
| Full component library immediately | Adds dependency and styling surface before the shared gallery/player is designed. | |
| No user-visible page | Cannot demonstrate visible real-browser stepping. | |

**Choice:** Semantic HTML and scoped dark CSS without a component library.
**Notes:** Component-library evaluation belongs with later gallery-level UI work.

### Contributor command

| Option | Description | Selected |
| --- | --- | --- |
| One reproducible repository command | Regenerates current-checkout bindings before typechecking and building the page. | ✓ |
| Separate undocumented commands | Makes clean-checkout reproduction easy to misorder. | |
| Commit generated artifacts | Risks stale glue and WASM combinations and creates noisy binary churn. | |

**Choice:** One documented repo-owned command with ignored generated output.
**Notes:** Native Cargo defaults remain unchanged.

---

## Verification and phase boundaries

### Browser evidence

| Option | Description | Selected |
| --- | --- | --- |
| Focused real-browser movement and disposal smoke | Instantiates generated WASM, proves Rust state changes, renders it, and exercises cleanup. | ✓ |
| WASM compilation only | Leaves runtime instantiation and visual data flow unproven. | |
| Broad cross-browser matrix | Exceeds hobby scope and Phase 16's focused uncertainty. | |

**Choice:** One focused Chromium smoke against the built app.
**Notes:** This is a bridge proof, not Phase 19's milestone-wide browser suite.

### Unit verification

| Option | Description | Selected |
| --- | --- | --- |
| Native and TypeScript unit tests plus one browser smoke | Keeps pure validation, packing, and projection cheap while proving the integrated edge once. | ✓ |
| Browser tests only | Makes core failures slower and harder to diagnose. | |
| No focused bridge tests | Leaves the new typed boundary unprotected. | |

**Choice:** Focused native and TypeScript tests around the one integration smoke.
**Notes:** Follow one-concern Arrange/Act/Assert structure.

### Inherited boundaries

| Option | Description | Selected |
| --- | --- | --- |
| Preserve current isolation contracts | Keeps native Cargo, opaque ownership, renderer independence, and optional heavy qualification intact. | ✓ |
| Reuse private evidence/testbed runtime | Couples browser delivery to development-only architecture. | |
| Revive strict native qualification | Conflicts with the accepted hobby scope. | |

**Choice:** Preserve existing architecture and hobby-scope boundaries.
**Notes:** Browser runtime dependencies point only toward `liquidfun`.

### Deferred capability

| Option | Description | Selected |
| --- | --- | --- |
| Keep Phase 16 as the narrow bridge proof | Leaves player lifecycle, deployment, six scenes, and polish in their mapped phases. | ✓ |
| Build the complete web milestone now | Breaks roadmap sequencing and mixes separate acceptance surfaces. | |
| Add a public npm-compatible API | Introduces a stability and publication promise explicitly outside v1.1. | |

**Choice:** Keep Phase 16 narrow.
**Notes:** No scope was added beyond `WASM-01` through `WASM-03`.

## Claude's Discretion

- Exact private type and module names.
- Exact copied frame-array stride, proof-scene tuning, and focused test fixtures.
- Exact compatible dependency pins after implementation-time validation.

## Deferred Ideas

- Shared player lifecycle and Pages deployment — Phase 17.
- Six authored demos and catalog presentation — Phase 18.
- Pointer/touch, responsive/accessibility polish, and milestone browser verification — Phase 19.
- Public package publication, stable JS API, workers, threads, zero-copy views, WebGPU, and broad browser guarantees — outside v1.1.
