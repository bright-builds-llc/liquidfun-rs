# Requirements: liquidfun-rs v1.6 Physics Curiosity Scenes

**Defined:** 2026-10-14
**Core Value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

**Milestone goal:** Add six small, self-contained playground scenes of physics people like to share and wonder about. Each scene should be recognizable at a glance on desktop and phone (SEED-004).

Research: `.planning/research/v1.6-curiosity-scenes/SUMMARY.md`.

## v1.6 Requirements

### Groundwork and scene gates

- [x] **PREP-01**: The 25 existing scenes behave exactly as before. A committed, pinned list of their end-state fingerprints is compared by scene id after every phase, and new scene ids may be added.
- [ ] **PREP-02**: Adding a scene needs no edits to hard-coded scene counts or "twenty-five" copy. Counts and copy derive from the scene catalog.
- [ ] **PREP-03**: Time-boxed siphon and hourglass feasibility spikes record measured results against written pass rules, and each ends with a recorded go, reshape or drop decision before its scene phase.

### Every new scene

- [ ] **SCENE-01**: Each new scene appears in the catalog with honest credits, a preview, a portrait phone frame, a README animated preview, the shared scale legend in meters, the gravity slider and the HUD safe frame.
- [ ] **SCENE-02**: Each new scene starts visibly doing its thing on its default settings, with no input, except where the scene invites a press. It stays at or below about 3,000 particles and runs smoothly on phones.
- [ ] **SCENE-03**: Each scene's caption describes what the simulation actually does. Any faked mechanism is named plainly, for example "a scripted magnetic pull" or "extra weight on the heavy layer".

### Cartesian diver

- [ ] **DIVER-01**: Player sees a diver floating near the top of a closed bottle. Pressing and holding on the bottle (or toggling Squeeze from the keyboard) makes it sink, and releasing makes it rise again.
- [ ] **DIVER-02**: Player sees the squeeze: the bottle visibly dents and an air bubble inside the diver shrinks while squeezed.
- [ ] **DIVER-03**: Diver sinking and rising stay reliable across the gravity slider range. A phone long press does not open a system callout.

### Archimedes screw

- [ ] **SCREW-01**: Player sees moving slanted vanes in a tilted trough carry separate pockets of water from a low reservoir up to a higher basin, with the water returning so the scene keeps running.
- [ ] **SCREW-02**: Player can change the screw's speed live and run it in reverse so it carries water back down.

### Fake ferrofluid

- [ ] **FERRO-01**: Player sees a dark liquid puddle (visible on the dark canvas) rise into several separate spikes toward a magnet. The magnet rests under the dish by default.
- [ ] **FERRO-02**: Player can press and drag to move the magnet. The spikes follow, and they collapse when the magnet is released or switched off.
- [ ] **FERRO-03**: Player can adjust magnet strength within a range that never flings the liquid apart, and the scene stays stable while the pointer is held.

### Rayleigh–Taylor instability

- [ ] **RT-01**: Player sees a heavy liquid layer above a light one develop into fingers that sink through the light liquid. The layers overturn within about 10 seconds.
- [ ] **RT-02**: Player can choose the heavy-to-light weight ratio and restart the run. The same settings always produce the same pattern.

### Siphon

- [ ] **SIPHON-01**: Player sees water flow from a higher tank over a crest and down into a lower tank through a primed tube, with the levels converging. Flow stops when the levels are equal. If the spike chose a stand-in mechanism, the caption credits it.
- [ ] **SIPHON-02**: Player can raise or lower the outlet tank, and the flow slows, stops or reverses to match.

### Sand hourglass (conditional on the PREP-03 spike)

- [ ] **SAND-01**: Player sees sand pour through a narrow neck and build a peaked pile, with a measured slope of at least 20°, in the lower bulb.
- [ ] **SAND-02**: Player can flip the hourglass. The glass rotates visibly, or flips instantly as a fallback, and the sand runs again.

If the PREP-03 hourglass spike cannot reach a pile slope of at least 20°, SAND-01 and SAND-02 return to a seed instead of shipping (owner decision, 2026-10-14).

## Future Requirements

Deferred. Tracked but not in the current roadmap.

- **SCREW-03**: A sand or powder material option and a tilt-angle control for the screw.
- **SAND-03**: A neck-width control showing jamming, and a sand-versus-water flow-rate comparison.
- **DIVER-04**: A "no-return depth" demonstration, and multiple divers.
- **FERRO-04**: Hysteresis or memory in the ferrofluid response.
- **SIPHON-03**: Lift-inlet and Prime actions, and two-tone tracer water.
- **RT-03**: Perturbation presets and a flip-tank action.

## Out of Scope

| Feature | Reason |
| --- | --- |
| Engine changes for per-particle or per-group density, or a second particle system per scene | Not an upstream concept, and systems never interact. RT uses bridge-applied weight on one system. |
| Engine air, gas or magnetic-field models | Outside LiquidFun's scope. Fakes live in the bridge with honest captions. |
| Hover-only pointer interaction | Touch has no hover. Interactions are press, drag and toggle. |
| New README or survey cue kinds | Default motion and action toggles cover every scene. |
| Real-world scale scenes | Tiny particles cost about 10× per step. Toy meter scale is used, and the scale legend states it honestly. |
| Claims of suction, compressed air, simulated density or an angle of repose that is not measured | Transparency constraint and the Phase 40 D-25 precedent. |

## Traceability

SCENE-01..03 apply to every scene phase (46–51) and the PREP-01 gate reruns after every phase; each is mapped to one phase for traceability (see ROADMAP.md Shared acceptance rules). SIPHON and SAND follow the Phase 45 spike decisions.

| Requirement | Phase | Status |
| --- | --- | --- |
| PREP-01 | Phase 45 | Complete |
| PREP-02 | Phase 45 | Pending |
| PREP-03 | Phase 45 | Pending |
| SCENE-01 | Phase 46 | Pending |
| SCENE-02 | Phase 46 | Pending |
| SCENE-03 | Phase 46 | Pending |
| DIVER-01 | Phase 46 | Pending |
| DIVER-02 | Phase 46 | Pending |
| DIVER-03 | Phase 46 | Pending |
| SCREW-01 | Phase 47 | Pending |
| SCREW-02 | Phase 47 | Pending |
| FERRO-01 | Phase 48 | Pending |
| FERRO-02 | Phase 48 | Pending |
| FERRO-03 | Phase 48 | Pending |
| RT-01 | Phase 49 | Pending |
| RT-02 | Phase 49 | Pending |
| SIPHON-01 | Phase 50 | Pending |
| SIPHON-02 | Phase 50 | Pending |
| SAND-01 | Phase 51 | Pending |
| SAND-02 | Phase 51 | Pending |

**Coverage:**

- v1.6 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-10-14*
*Last updated: 2026-10-14 after v1.6 roadmap creation (Phases 45–51)*
