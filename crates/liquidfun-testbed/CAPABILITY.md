# Renderer capability decision

Macroquad 0.4.15 is retained for the private `liquidfun-testbed` package. The
required capability matrix passed on 2026-07-22 after Plan 11-18 had already
closed the headless workflow and package-isolation gate. No allowed fallback
failure occurred, so `winit`, `wgpu`, and `egui` were not added.

This decision covers the Phase 11 private adapter boundary. It does not promote
pixel output, frame timing, or screenshots to compatibility evidence, and it
does not claim the broader platform or performance sign-off owned by Phase 12.

## Reproduction

The capability command uses Macroquad's real headless-safe CPU image adapter:
`macroquad::texture::Image` and `macroquad::color::Color`. It creates actual
Macroquad image render targets, composites the complete capability scene, and
uses `Image::export_png` for diagnostic screenshots. It does not substitute a
renderer-neutral fake or require a window/display server.

```console
CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-24 cargo run -p liquidfun-testbed -- --capability-check --fixture crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json --output target/testbed-capability
```

The recorded run used Rust 1.97.0 on `aarch64-apple-darwin`. The immutable
`phase11-v1` fixture hash was
`ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860`.
The loader verified all 10 catalog, mapping, payload, and inherited-proof
artifacts before rendering three reviewed cases across eight families.

## Required capability matrix

| Capability                       | Objective result                                                         |
| -------------------------------- | ------------------------------------------------------------------------ |
| Rigid contacts                   | PASS — 3 visible contact points                                          |
| Contact normals                  | PASS — 3 directed normal arrows                                          |
| Particle contacts and colors     | PASS — 12 particles, 6 contacts, and 4 colors                            |
| Broad-phase AABBs                | PASS — 4 distinct outlined bounds                                        |
| Profile names                    | PASS — 5 structural names with no duration values                        |
| Synchronized overlay             | PASS — 3 aligned Rust/oracle primitive pairs                             |
| Side-by-side difference          | PASS — 2 labeled synchronized panels                                     |
| Focus halo and label             | PASS — persistent 2px halo and semantic label                            |
| Semantic capture acknowledgement | PASS — deterministic checkpoint acknowledgement shown                    |
| Diagnostic screenshot            | PASS — PNG saved with diagnostic-only warning                            |
| Keyboard controls                | PASS — 6 Macroquad `KeyCode` bindings map to typed presentation intents  |
| Keyboard focus                   | PASS — persistent 2px focus ring with contrast above 3:1                 |
| Dense text                       | PASS — 16 inspector rows at 640x480                                      |
| DPI scaling                      | PASS — complete 1x and 2x images                                         |
| Resize                           | PASS — centered 800x600 resized frame                                    |
| Minimum desktop size             | PASS — complete 640x480 frame                                            |
| Passive session controller       | PASS — 0 logical steps and 0 captures before and after rendering         |
| Immutable comparison model       | PASS — the same 10 exact comparison entries before and after rendering   |
| Bounded finite inputs            | PASS — strict fixture hashes, finite fixed geometry, and reviewed limits |
| Confined regular output          | PASS — every output is a non-link regular file below `target/`           |

## Measurements

| Measurement                         |         Result |
| ----------------------------------- | -------------: |
| Minimum complete viewport           |        640x480 |
| Resized viewport                    |        800x600 |
| Maximum exercised DPI scale         | 2x at 1280x960 |
| Minimum non-background pixels       |        275,394 |
| Minimum text contrast ratio         |    12.262949:1 |
| Minimum control target              |           44px |
| Persistent focus ring               |            2px |
| Dense inspector rows                |             16 |
| Macroquad keyboard bindings         |              6 |
| Distinct particle colors            |              4 |
| Contact points / normal arrows      |          3 / 3 |
| Particle contacts                   |              6 |
| Broad-phase AABBs                   |              4 |
| Structural profile names            |              5 |
| Overlay pairs / side-by-side panels |          3 / 2 |

The adapter received shared `&SessionController` and `&ComparisonModel`
references. Rendering all three frames, taking screenshots, resizing, changing
DPI, and evaluating keyboard/focus presentation left controller state at
`NoSelection`, logical steps at 0, captures at 0, comparison state at
`ExactMatch`, and comparison entries at 10. The visual package therefore owns
no simulation tick, checkpoint, or comparison authority.

## Artifacts

Artifacts are generated under `target/testbed-capability/` and remain
untracked diagnostics.

| Artifact                            |  Bytes | SHA-256                                                            |
| ----------------------------------- | -----: | ------------------------------------------------------------------ |
| `capability-report.json`            |  4,997 | `1244140399ef73714e5ccc929b2d20833f41c97ab356dfa5be25fbf2933c1a5b` |
| `macroquad-capability-640x480.png`  | 43,511 | `e5d44b22a06ab2d1a3b6fdbc0b98680cf06ea0d2fd604f8d08fc02ec16acd6ef` |
| `macroquad-capability-800x600.png`  | 47,284 | `85f818d2973ae97a9c1c95d0b786308e970297771bb24935474ffd4826a766df` |
| `macroquad-capability-1280x960.png` | 70,507 | `493477af9ba36d64f98ffe6be16feeaf60c87bcbf0613485b9a3767b63696f43` |

## Fallback disposition

The heavier fallback is permitted only after a reproducible Macroquad failure
in required UI density, capture fidelity, accessibility, GPU inspection,
render-target control, or supported platform behavior. This run produced no
such failure. Preference, aesthetics, or speculative future needs are not
fallback triggers. Bevy and renderer-owned simulation schedules remain
prohibited.
