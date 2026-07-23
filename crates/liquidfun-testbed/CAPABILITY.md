# Replacement renderer capability decision

The private `liquidfun-testbed` package uses the exact
`eframe-egui-0.35.0+tiny-skia-0.12.0` adapter. The complete replacement
capability matrix passed on 2026-07-23 against the immutable Phase 11 fixture
after the eframe desktop shell migration.

The selected dependency stack is exactly
`eframe-0.35.0+egui-0.35.0+tiny-skia-0.12.0`. Macroquad and its retired
`ttf-parser` 0.21.1 branch are no longer part of the workspace graph, and the
temporary advisory allowlist is empty.

This decision covers the private passive adapter boundary. Pixel output,
screenshots, and frame timing remain diagnostic only: they are not LiquidFun
compatibility evidence. The matrix does not claim solver parity, renderer
performance, production performance, or broader platform sign-off.

## Reproduction

The capability command renders through `TinySkiaImageRenderer`, the real
headless-safe CPU image backend behind the replacement adapter. It creates
actual `tiny-skia` pixel maps, composites the complete capability scene, and
encodes diagnostic PNG files. It does not substitute a renderer-neutral fake or
require a window or display server.

```console
CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w CARGO_BUILD_JOBS=1 cargo run -p liquidfun-testbed -- --capability-check --fixture crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json --output target/testbed-capability-phase12-18
```

The recorded run used Rust 1.97.0 on `aarch64-apple-darwin`. The immutable
`phase11-v1` fixture hash was
`ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860`.
The loader verified all 10 catalog, mapping, payload, and inherited-proof
artifacts before rendering three reviewed cases across eight families.

## Required capability matrix

| Capability | Objective result |
| -------------------------------- | ------------------------------------------------------------------------ |
| Rigid contacts | PASS — 3 visible contact points |
| Contact normals | PASS — 3 directed normal arrows |
| Particle contacts and colors | PASS — 12 particles, 6 contacts, and 4 colors |
| Broad-phase AABBs | PASS — 4 distinct outlined bounds |
| Profile names | PASS — 5 structural names with no duration values |
| Synchronized overlay | PASS — 3 aligned Rust/oracle primitive pairs |
| Side-by-side difference | PASS — 2 labeled synchronized panels |
| Focus halo and label | PASS — persistent 2px halo and semantic label |
| Semantic capture acknowledgement | PASS — deterministic checkpoint acknowledgement shown |
| Diagnostic screenshot | PASS — PNG saved with diagnostic-only warning |
| Keyboard controls | PASS — 6 typed shortcuts map to presentation intents |
| Keyboard focus | PASS — persistent 2px focus ring with contrast above 3:1 |
| Dense text | PASS — 16 inspector rows at 640x480 |
| DPI scaling | PASS — complete 1x, 1.25x, and 2x images |
| Resize | PASS — centered 800x600 resized frame |
| Minimum desktop size | PASS — complete 640x480 frame |
| Passive session controller | PASS — 0 logical steps and 0 captures before and after rendering |
| Immutable comparison model | PASS — the same 10 exact comparison entries before and after rendering |
| Bounded finite inputs | PASS — strict fixture hashes, finite fixed geometry, and reviewed limits |
| Confined regular output | PASS — every output is a non-link regular file below `target/` |

## Measurements

| Measurement | Result |
| ----------------------------------- | -------------: |
| Minimum complete viewport | 640x480 |
| Resized viewport | 800x600 |
| Maximum exercised DPI scale | 2x |
| Minimum non-background pixels | 275,394 |
| Minimum text contrast ratio | 12.261033:1 |
| Minimum control target | 44px |
| Persistent focus ring | 2px |
| Dense inspector rows | 16 |
| Typed keyboard bindings | 6 |
| Distinct particle colors | 4 |
| Contact points / normal arrows | 3 / 3 |
| Particle contacts | 6 |
| Broad-phase AABBs | 4 |
| Structural profile names | 5 |
| Overlay pairs / side-by-side panels | 3 / 2 |

The adapter received shared `&SessionController` and `&ComparisonModel`
references. Rendering all three frames, taking screenshots, resizing, changing
DPI, and evaluating keyboard and focus presentation left controller state at
`NoSelection`, logical steps at 0, captures at 0, comparison state at
`ExactMatch`, and comparison entries at 10. The visual package therefore owns
no simulation tick, checkpoint, or comparison authority.

## Artifacts

Artifacts are generated under `target/testbed-capability-phase12-18/` and
remain untracked diagnostics.

| Artifact | Bytes | SHA-256 |
| --------------------------------------- | -----: | ------------------------------------------------------------------ |
| `capability-report.json` | 5,172 | `dc69993192fca7ec3384ccd1f9552ed3f64bd1c6acb2b59d25bc08def9f454d7` |
| `replacement-capability-640x480.png` | 10,986 | `5c3b804f7f74900f95cc04f36180f068397048402018a82ac9de3479051852c5` |
| `replacement-capability-800x600.png` | 13,430 | `4459a7ba7924738823e7ea7471b47794cd617ae9363e7b7bb7d66037391da20c` |
| `replacement-capability-1280x960.png` | 19,205 | `f78323496b8255d3f45288c8a607b9944a48809d8f43c701ee3f78396e27cefa` |

## Scope and fallback disposition

The replacement closes the inherited dependency-security obligation without
expanding the adapter's authority. A different renderer stack is permitted only
after a reproducible failure in required UI density, capture fidelity,
accessibility, GPU inspection, render-target control, or supported platform
behavior. Preference, aesthetics, speculative performance, and screenshots are
not fallback or compatibility triggers. Bevy and renderer-owned simulation
schedules remain prohibited.
