# Quick 260921-tbx Summary

The Wireframe stroke slider lives next to Rendering. It applies one CSS-pixel width to particle circles, basin walls, and rigid circles while Rendering is Wireframe. Default is 1.00. Solid mode still strokes rigid geometry at 2.

Verified with `bun run typecheck`, Vitest canvas and stroke-width tests, and a local Vite session: the slider moved from 1.00 to 0.25 and 1.50, stayed at 0.25 across Dam Break and Particles, and stored `liquidfun.wireframe-stroke-width.v1`.
