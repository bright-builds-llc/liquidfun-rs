# Phase 47 Deferred Items

Out-of-scope discoveries logged during execution (not fixed here).

## From 47-01

- `crates/liquidfun/src/particle/solver/boundary/support.rs:98`: `warning: unreachable expression` (an early `return Ok(())` at line 79 makes the trailing `Ok(())` unreachable). It appears only in `--release` builds of the `liquidfun` engine crate, which this phase must not edit. It predates Phase 47 and does not affect the debug clippy gate.

## From 47-04

- README SVG recorder (`web/scripts/readme-svg/`): animated `<line>` elements are keyed by segment index with `calcMode="linear"`. When a scene's drawn segment list is reordered between 20 Hz samples, a line sweeps across the frame for 50 ms in the SVG. The screw has 516 such >40 px jumps (clipped vane outlines entering and leaving the housings, 4 per pitch). The same happens in washing-machine (2,936), hydraulic-fountain (24) and stacked-drip (16), so this predates Phase 47. The 60 fps WebP that the README gallery embeds renders every step and shows no sweep (frames 7-10 checked). A fix would key lines by a stable segment identity or emit discrete keyframes on reassignment.
