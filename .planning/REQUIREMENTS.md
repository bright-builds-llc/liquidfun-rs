# Requirements: liquidfun-rs v1.3 Reference Testbed Scenes

**Defined:** 2026-09-21
**Core Value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## v1 Requirements

A visitor can open every JavaScript LiquidFun testbed scene the playground does not already have. Dam Break stays. Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel stay as original scenes. Recognizable behavior is enough. This is not a sealed C++ parity claim.

### Shared player

- [ ] **PLAY-01**: Visitor can open Drawing Particles, Elastic Particles, Impulse, Liquid Timer, Particles, Rigid Particles, Soup, Soup Stirrer, Sparky, Surface Tension, Theo Jansen, and Wave Machine from the catalog, and the existing six scenes remain available.
- [ ] **PLAY-02**: Visitor can play, pause, and reset each new scene, and reset restores that scene's initial layout.
- [ ] **PLAY-03**: Each new scene credits the pinned LiquidFun test it ports.

### Basin

- [ ] **BASIN-01**: Visitor can watch Particles: water falls in an open basin and a ball drops into it.
- [ ] **BASIN-02**: Visitor can watch Liquid Timer: tensile, viscous liquid drains through shelves into bottom columns.

### Materials

- [ ] **MAT-01**: Visitor can watch Surface Tension: three colored tensile groups bead and bleed color when a ball hits them.
- [ ] **MAT-02**: Visitor can watch Elastic Particles: three soft clumps deform when a ball falls on them.
- [ ] **MAT-03**: Visitor can watch Rigid Particles: three colored clumps stay solid and do not stretch like jelly when a ball hits them.

### Interaction

- [ ] **ACT-01**: Visitor can watch Soup: a basin of liquid holds floating solid bits.
- [ ] **ACT-02**: Visitor can watch Soup Stirrer: a paddle keeps stirring that soup, and the visitor can free the paddle from its rail or put it back.
- [ ] **ACT-03**: Visitor can click or tap Impulse and shove the whole particle blob.
- [ ] **ACT-04**: Visitor can watch Wave Machine rock on its own and slosh the water inside.
- [ ] **ACT-05**: Visitor can watch Theo Jansen walk under a particle load and can reverse its motor.

### Effects

- [ ] **FX-01**: Visitor can watch Sparky: colliding circles throw fading particle sparks.
- [ ] **FX-02**: Visitor can paint Drawing Particles into an empty vessel, and at least one non-water material looks different from plain water.

## Future Requirements

### Later fidelity

- **DRAW-02**: Visitor can pick every LiquidFun particle material from Drawing Particles, including the full testbed keyboard matrix.
- **PRESET-01**: Visitor can switch Impulse and Liquid Timer among extra particle-type presets.

### Later evidence

- **PARITY-01**: Developer can produce sealed per-scene differential evidence against the pinned C++ tests.
- **BOX2D-01**: Visitor can run the Box2D-only tests that the JavaScript testbed leaves commented out.

## Out of Scope

| Feature | Reason |
| --- | --- |
| Replacing Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, or Water Wheel | Those scenes already ship. This milestone adds the twelve missing testbed entries beside them. |
| Copying the old JavaScript testbed chrome | The existing SolidJS player, catalog, and controls stay. |
| Sealed bit-exact C++ parity for each scene | Hobby playground goal is recognizable behavior and honest limits. |
| Cutting particle counts or raising the 4-step catch-up cap to look smoother | Frame budget and Dam Break behavior stay as they are. |
| A second physics engine, glam, Rayon, or default SIMD | Scenes use the existing native `liquidfun` crate. |
| Crate publication or a git release tag | Still separately authorized. |
| A public “Rust is N×” claim or a filled performance manifest | v1.2 left `reviewed_reports` empty on purpose. |

## Traceability

| Requirement | Phase | Status |
| --- | --- | --- |
| PLAY-01 | — | Pending |
| PLAY-02 | — | Pending |
| PLAY-03 | — | Pending |
| BASIN-01 | — | Pending |
| BASIN-02 | — | Pending |
| MAT-01 | — | Pending |
| MAT-02 | — | Pending |
| MAT-03 | — | Pending |
| ACT-01 | — | Pending |
| ACT-02 | — | Pending |
| ACT-03 | — | Pending |
| ACT-04 | — | Pending |
| ACT-05 | — | Pending |
| FX-01 | — | Pending |
| FX-02 | — | Pending |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 0
- Unmapped: 15

---

*Requirements defined: 2026-09-21*
*Last updated: 2026-09-21 after scope confirmation of all five groups*
