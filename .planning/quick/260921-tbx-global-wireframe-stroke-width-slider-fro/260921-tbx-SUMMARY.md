# Quick 260921-tbx Summary

The Wireframe stroke slider lives next to Rendering. It applies one CSS-pixel width to particle circles, basin walls, and rigid circles while Rendering is Wireframe. Default is 0.30. Solid mode still strokes rigid geometry at 2.

Verified with `bun run typecheck`, Vitest canvas and stroke-width tests, and a local Vite session: the slider moved from 0.30 to 0.10 and 1.50 on Particles, and stored `liquidfun.wireframe-stroke-width.v2`.
