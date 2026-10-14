# Feature Research

**Domain:** Six "physics curiosity" scenes for the SolidJS + WASM LiquidFun playground (v1.6)
**Researched:** 2026-10-14
**Confidence:** MEDIUM overall. Real-world reference behavior is HIGH (multiple agreeing sources). How well each phenomenon emerges from LiquidFun particles is MEDIUM to LOW and is flagged per scene; three scenes need an early feasibility spike.

## Scope and Ground Rules

This file covers only the six new scenes. Existing shared features are assumed and listed as dependencies:
catalog record (`web/src/catalog/scene-records.ts` + `SCENE_IDS`), portrait frame, README SVG/WebP plan, scale legend, HUD safe frame, gravity slider (`withGravitySlider`, 2 to 80 m/s², rebuilds the scene), the global phone-tilt gravity toggle, pointer down/move/up, zoom/pan, and render modes including GPU Water.

Codebase facts that shape every scene below (verified in this repo, HIGH confidence):

| Fact | Where | Consequence for v1.6 |
|------|-------|----------------------|
| Particle pressure is clamped at zero (`max(0.0, …)`); the liquid can never pull (no tension) | `crates/liquidfun/src/particle/solver/pressure.rs` lines 63 and 86 | A naive siphon column breaks at the crest. Ferrofluid spikes cannot rely on cohesion unless tensile particles are used |
| Density and gravity scale are per particle **system**, and particles in different systems do not interact | `ParticleSystemDef::with_density` / `with_gravity_scale`; upstream Programmer's Guide ch. 11 ("two particle systems influencing a rigid body while not interacting with each other") | Rayleigh–Taylor cannot use two systems (they would pass through each other). Use one system and give the heavy group extra weight from the bridge |
| Per-particle force over a contiguous range exists | `apply_particle_force_range` in `world/particle_object/system.rs` | Extra weight (R–T) and a magnet pull (ferrofluid) can be bridge-side per-step forces with no engine change |
| Fixture density can change at runtime | `set_fixture_density` in `world/object/fixture_object.rs` | The Cartesian diver can sink by raising its density, using buoyancy that already works in Float or Sink |
| The v1.5 powder spike showed `b2_powderParticle` deposits spread as flat as water (p5–p95 width about 7.1 m vs 7.1 m, height 0.30 vs 0.28 m at 8 s) | `.planning/phases/40-timed-valve-and-teaching-levels/40-08-SUMMARY.md` | **The powder flag alone will not form a sand cone with an angle of repose.** The hourglass needs an extra rule (see below) |
| Canvas pointer forwards `move` only while a pointer is held down; there is no hover | `web/src/input/pointer.ts` `reducePointerEvent` | "Spikes toward the pointer" means press and drag (this also works on phones). A hover-only design would fail on touch and in tests |
| README cues are only `action` and `pointer-up`, sampled at 20 Hz (201 samples per 10 s clip). Clips are circle wireframe with a 4,000-particle render cap | `web/scripts/readme-svg/plans.ts`, `web/src/export/duration.ts`, `web/src/render/particle-limit.ts` | A held squeeze or a held magnet cannot be cued yet. Prefer watch-first defaults or toggle actions that existing `action` cues can drive |
| Canvas background is `#071018` (near black) | `web/src/render/canvas.ts` | A "black" ferrofluid would be invisible. It needs a dark-but-lit color (graphite or blue-violet with a light rim) |
| Phone tilt sets the world gravity vector for any scene | `web/src/input/tilt-binding.ts` → `setGravity(x, y)` | Any bridge force that fakes weight (R–T extra weight, hourglass stick rule, diver) must use the current gravity vector, not a hard-coded `-y` |

## Per-Scene Feature Landscape

### 1. Siphon

**Real-world reference (HIGH):** An inverted-U tube, primed full of liquid, carries water from a higher tank over a crest into a lower tank with no pump. Flow is driven by the height difference between the two free surfaces. Flow slows as the levels approach each other, roughly with the square root of the head difference, and **stops when the levels equalize**. If the source level drops below the inlet, air enters and the siphon **breaks**: the column falls back on both sides and does not restart by itself. The tube must be **primed** first. The crest height is limited by atmospheric pressure (about 10 m for water). If the outlet tank is raised above the source level, the flow **reverses**.

**Defining visual moment (≤10 s):** A full tube with water visibly moving up and over the crest. The upper tank level falls and the lower tank level rises, and the two levels clearly converge within the clip.

**Expected behavior and timing:** It starts primed and flows at t = 0. Flow is strongest at first and visibly slows. At default settings the levels equalize in about 15–25 s, and the 10 s clip shows at least half of the convergence. After equalizing the scene stays calm.

**Feasibility warning (MEDIUM-LOW):** Liquid in the rising leg above the source surface needs negative pressure. LiquidFun clamps pressure at zero, so the column will separate at the crest unless something supplies "atmosphere". Spike these options in order:

1. **Weighted lids as atmosphere.** Free floating pistons on both tank surfaces in vertical guides, heavy enough that crest pressure stays positive. This is physically honest, but the lids must not leak and must read as "air pressure".
1. **Modeled tube pull.** A bridge-side force along the tube axis, scaled by the head difference and switched off when the inlet is uncovered. It is robust, but the credits must say so.
1. **Tensile cohesion with a shallow crest.** Weakest. The crest barely rises above the source.

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Two tanks at different heights joined by an inverted-U tube that rises above the source surface | It isn't a siphon unless water climbs over a crest higher than its source surface | MEDIUM | The crest must clearly sit above the source water line in both frames |
| Starts primed, flowing at t = 0 | Priming by suction can't be simulated, and the clip must move immediately | LOW | The tube is pre-filled with particles at build |
| Flow stops when levels equalize | This is the famous behavior and the teaching point | MEDIUM-HIGH | Emerges from the head difference in options 1 and 3; explicit in option 2 |
| Source and outlet levels visibly change | Level change is the only motion a viewer sees in wireframe | LOW | Narrow tanks with a wide tube give a fast level change |
| No particle leaks through tube walls | Leaks read as a bug | MEDIUM | Use thick polygon walls or a chain with margin; tube width ≥ 4 particle diameters |
| Runs within the per-scene particle budget (≤ about 3,000) | Phone frame rate | LOW | |

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Outlet tank height** range control (runtime, not recreate), animating the tank up or down over about 1 s | "Raise the lower tank and the flow reverses" is the best siphon moment | MEDIUM | The tube leg dips in from above so a moving tank slides around a fixed leg. Clamp the range so the leg stays submerged |
| **Lift inlet** action that breaks the siphon, plus a **Prime** action that refills the tube | Shows the second stop condition and why priming matters | MEDIUM | After a break, Prime spawns tube particles from the source. Reset already re-primes |
| Two-tone tracer: source water tinted differently from outlet water (no color mixing) | Makes flow through the tube visible in still frames and the README | LOW | Do not set the color-mixing flag; the tint must stay distinct |
| Level marks on both tanks with a "levels equal" hint | Makes the stop condition legible | LOW-MEDIUM | Static lines; the hint may be HUD text |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Simulating air, bubbles, or suction priming | "Realism" | No gas phase exists, and it is a deep engine change | Start primed, and offer a Prime action |
| An endless emitter feeding the source tank | Keeps it moving forever | Hides "stops when levels equalize", the whole point | A finite source; Reset restarts |
| A siphon that keeps flowing after the inlet leaves the water | Easier to tune option 2 | Physically false, and it contradicts the teaching point | Gate the pull on inlet submersion (a particle count near the inlet) |
| A tall crest marketed as "real 10 m" | Spectacle | Not modeled; dishonest | Keep a modest crest; credits explain any modeled pull |

**README preview:** Watch-first, no cue. The frame must show both tanks, the full tube, and both level changes. Use tinted water so the tube flow reads in wireframe.

**Acceptable 2D simplifications:** Rigid tube shape (no flexible hose), primed start, tanks as open boxes, and "atmosphere" supplied by lids or a modeled pull if credited honestly.

### 2. Archimedes Screw

**Real-world reference (HIGH/MEDIUM):** A helical blade rotates inside a tube or trough inclined at about **30°** (studies range from 20–48°). Each turn scoops a pocket of water from the lower reservoir and carries it up. Speeds are slow, typically **below 100 rpm**. Pockets work best about half full, and overfilling spills back over the flight. Above about 40° the pockets shrink and discharge drops. Reversing the rotation lowers material. Screws also move granular material such as grain or sand, and that is how screw conveyors work.

**Defining visual moment (≤10 s):** Discrete pockets of water riding up a tilted tube between moving slanted blades. A pocket pours out the top into an upper basin, and the upper basin visibly gains liquid.

**Expected behavior and timing:** The first pocket reaches the top within about 3–5 s at the default crank speed, so the README shows at least one or two spills. Lifting continues while the lower reservoir covers the inlet. Optionally a return chute closes the loop so the scene can run forever.

**Representation (MEDIUM-LOW feasibility, spike required):** A true helix is 3D. In a side view a rotating helix looks like slanted flight lines traveling along the axis. The 2D model is kinematic flight plates inside a tilted trough, translating up the axis at `pitch × rpm`. The hard part is the **wrap**: when a flight leaves the top, a new one must appear at the bottom. Teleporting a body into water can fling particles. Spike candidates are phase-wrapped flights that wrap at the submerged inlet (accept a "scoop" splash), flights that emerge through the trough end outside the water, or a periodic flight set reset by exactly one pitch.

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Tube or trough tilted about 25–35°, lower end submerged in a reservoir, upper end over a higher basin | Recognizable silhouette | LOW | Defaults should match real practice (about 30°) |
| Moving slanted flights that carry separate pockets up | Pockets are the mechanism; a steady stream would look like a pump | HIGH | Wrap handling is the main risk. No particles may tunnel through flights |
| Water visibly gained in the upper basin | Proof that it lifts | LOW | |
| **Crank speed** range control (runtime), for example 0–60 rpm with a sensible default | Users expect to drive the machine | LOW-MEDIUM | A `range` control like Washing Machine's Drum speed |
| Liquid returns or is replenished so the scene doesn't idle within 30 s | A watch-first scene should keep working | MEDIUM | A return chute from the upper basin back to the reservoir (closed loop) is better than an emitter |

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Reverse direction (negative crank speed, or a direction preset) | Shows pockets riding down, which is counterintuitive and fun | LOW once flights work | Like Theo Jansen's Motor direction |
| **Material** preset: water or sand-like grains | Screws famously move grain; ties into the hourglass | MEDIUM | Only if the hourglass spike yields a credible granular look |
| **Tilt angle** control showing spill-back above about 40° | Teaches why real screws sit near 30° | HIGH | Re-tilting a kinematic assembly with particles inside is risky. Use `recreates: true` |
| Helix overlay drawn along the shaft, rotating with the phase | Reads instantly as a screw, not a conveyor | MEDIUM | Needs a scene-specific overlay render path. A visual only; physics stays the flight plates |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| A truly rotating 3D helix or spiral-pump geometry | Accuracy | It doesn't map to 2D physics, and a rotating 2D spiral is a different machine (a coil pump) | Translating flights plus an optional helix overlay |
| A direct "lift force" on particles inside the tube | Easy | Pockets vanish into an elevator beam and it reads as fake | Physical flights that push the water |
| High default rpm (over 100) | "More action" | Splash and tunneling, and unrealistic | Default around 20–30 rpm, capped at a stable maximum |
| Pointer-dragging the screw body | Generic playground habit | Breaks the wrap and the kinematics | Pointer pokes water (existing behavior) or does nothing; crank via controls |

**README preview:** Watch-first at default speed. Show at least one pocket climbing the full length and spilling within 10 s, and frame the whole incline plus both basins.

**Acceptable 2D simplifications:** Side-view flights instead of a helix, an open trough instead of a closed tube, kinematic motion (no motor torque or load), and a closed-loop return chute.

### 3. Sand Hourglass

**Real-world reference (HIGH):** Dry sand piles at an **angle of repose of about 30–35°** (34° is the commonly quoted value; rounded grains pile lower). A thin steady stream falls through the neck. The upper bulb forms a **funnel-shaped crater** that drains from the center. The lower bulb grows a **cone**. Unlike a water clock, the **flow rate stays roughly constant regardless of fill height**, scaling with neck width to the 5/2 power (Beverloo law). The usual explanation is Janssen-style wall support, though that explanation is contested. Necks narrower than a few grain diameters jam by arching. Flipping restarts it.

**Defining visual moment (≤10 s):** A thin stream through the neck, a dimple or crater in the top surface, and a **peaked pile** growing in the bottom bulb. The peak is what separates sand from water. A flat puddle reads as "water in an hourglass".

**Expected behavior and timing:** It pours from t = 0 with all sand in the top bulb. The pile is visible by about 2–3 s. Full drain at default takes about 20–40 s, long enough that Flip matters but short enough to watch. Flip restarts the pour.

**Feasibility warning (LOW for a cone with the powder flag alone):** The v1.5 spike measured powder deposits as flat as water. The pile needs a granular rule, which should be spiked early:

1. **Bridge-side stick-slip friction:** a supported particle below a speed threshold has its velocity zeroed (or is temporarily flagged as wall) only when the local slope is below tan(about 32°). Grains on a steeper slope keep sliding. This is deterministic, cheap, and honest if credited ("a simple sand rule"). It must release on Flip and on any gravity or tilt change.
1. **Rigid-particle grains:** tiny `rigidParticleGroup` clumps of 2–4 particles that interlock. Cost and stability are unknown.
1. **Rigid circle or polygon bodies:** Box2D friction gives repose, but hundreds of bodies are costly, and circles roll. Last resort.

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Two-bulb glass with a narrow neck, sand in the top bulb at start | Silhouette | LOW | The neck must be ≥ about 4–5 particle diameters or it may jam permanently |
| Thin continuous stream through the neck | Signature look | LOW-MEDIUM | |
| **Bottom pile with a visible peak and slopes near 30–35°** | It's what makes it sand | HIGH | Needs the granular rule spike. Accept a 25–40° range |
| Top surface drains from the center (crater) | Second signature | MEDIUM | Comes free if the stick rule applies in the top bulb too |
| **Flip** action that restarts the pour | Named in the milestone goal | MEDIUM | See the flip options below. A keyboard-reachable `action` control |
| Sand-colored grains, visibly grainy (powder-like scatter, no water sheen) | Material read | LOW | Shaded blob or GPU Water may make it look wet; Wireframe or Particles modes read as grains |

**Flip options (pick in planning):** (a) **Animated rotation**: a kinematic glass rotates 180° over about 0.8–1.2 s and the sand tumbles with it. This is the most satisfying, but it risks tunneling through thin walls, so use thick polygon walls and a modest angular speed. (b) **Instant mirror**: mirror every particle about the center and keep the walls fixed. This is robust but less delightful. (c) Phone tilt already flips gravity for free (see differentiators). Option (a) is the differentiator; (b) is an acceptable table-stakes fallback.

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Animated 180° flip | The "turn it over" moment people share | MEDIUM-HIGH | Thick walls, a limited angular speed, and the stick rule released during rotation |
| **Phone tilt**: turning the phone upside down makes the sand run the other way | A free delight on phones, using existing global tilt | LOW if the stick rule follows the gravity vector | The stick rule and slope test must use the current gravity direction |
| Flow counter (grains through the neck per second) or a run timer | Lets curious users check the "constant flow" claim | MEDIUM | Report what the simulation does. Do not claim Beverloo constancy unless measured |
| **Neck width** control (recreate) showing jamming at narrow widths | Real granular phenomenon (arching) | MEDIUM | Particles without friction may not arch. Spike before promising |
| Material preset (Sand or Water) for a side-by-side lesson | Water slows as head drops; sand ideally doesn't | MEDIUM | Valuable only if the sand rule really changes the flow curve |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Shipping plain powder-flag particles as "sand" | Zero effort | v1.5 evidence shows it piles flat like water, so it isn't recognizable | The granular rule spike, or keep the hourglass scene out of the milestone |
| Freezing grains permanently (wall particles that never release) | Simple pile | Flip and tilt break; the pile becomes a sculpture | Reversible stick rule, released on Flip and gravity changes |
| Claiming "constant flow like a real hourglass" | Popular fact | The simulation may not reproduce it | Show a measured flow counter, or state the real-world fact as background only |
| A neck that is too narrow by default | Realism | Permanent jam in the default view | Default neck about 5 particle diameters |

**README preview:** Watch-first. Show the stream plus a visibly peaked pile by about 5 s. A Flip `action` cue near sample 140–160 (7–8 s) is optional and only worth adding if the animated flip looks clean. Otherwise skip the flip in the clip.

**Acceptable 2D simplifications:** A 2D cross-section (wedges instead of cones), a rule-based friction proxy, and a grain size far larger than real sand.

### 4. Cartesian Diver

**Real-world reference (HIGH):** A sealed bottle full of water holds a small diver (dropper or pen cap) with an air bubble that floats near the top. **Squeezing** raises the pressure, the bubble compresses (Boyle's law), and water enters the diver. The diver gets denser and **sinks**. **Releasing** lets the bubble re-expand and the diver **rises**. Visibly, the water level inside the dropper rises when squeezed. There is also a lesser-known subtlety: below a **critical depth** the extra water pressure keeps the bubble compressed, so the diver **stays at the bottom even after release**. This is an unstable equilibrium (a fold catastrophe in the literature) and needs a tall column to observe.

**Defining visual moment (≤10 s):** The diver floats, a squeeze makes it sink, a release brings it back up. The bottle walls visibly bow while squeezed, and the bubble inside the diver shrinks.

**Expected behavior and timing:** At rest the diver floats near the top. While squeezed it sinks to the bottom in about 1.5–3 s. On release it rises in about 2–4 s. It responds immediately, with no long lag.

**Feasibility (MEDIUM-HIGH):** Buoyancy from particle pressure already works (Float or Sink). Changing fixture density at runtime is supported. Near-neutral bodies jitter in particle fluid, so leave a clear density margin (for example about 0.85× water at rest, about 1.2× squeezed) and add linear damping for a calm sink.

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Tall bottle full of water with a small diver floating near the top | Silhouette, and a natural portrait scene | LOW | Close the bottle so water can't slosh out |
| **Press and hold to squeeze, release to let go** (pointer down = squeeze, up = release) | Named in the milestone goal | MEDIUM | Uses the existing pointer down/up. Anywhere on the canvas, or the bottle only; pick one and say it in the hint |
| A keyboard-accessible squeeze (a `Squeeze` toggle action, or a range control "Squeeze" 0–100%) | Repo rule: labeled controls work from the keyboard | LOW-MEDIUM | A toggle also makes the README cue possible with the existing `action` cue kind |
| Diver sinks while squeezed and rises on release, reliably | The whole trick | MEDIUM | Density switch via `set_fixture_density`, with margins and damping |
| Visible squeeze feedback (walls bow inward, or the bottle outline tints) | The user must see that the press registered | LOW-MEDIUM | Draw-only or small kinematic wall motion. Real wall motion raises the water level, which is acceptable |
| Touch press-and-hold works on iPhone without a text-selection or callout menu | The main phone interaction | LOW | `touch-action: none` and suppress `contextmenu` and long-press callouts on the canvas. Verify in e2e |

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Bubble inside the diver shrinks, and the internal water line rises with squeeze | Shows *why*, which is Boyle's law made visible | MEDIUM | An overlay or a child fixture whose drawn size follows the squeeze value |
| **Depth-dependent compression** so a deep diver stays down after release ("no return" depth) | A surprising real behavior few demos show | MEDIUM | Effective volume V = V₀·P₀/(P₀ + squeeze + ρ·g·depth), computed per step in the bridge |
| Squeeze strength slider (light to firm) | Lets users hover the diver mid-bottle | LOW | A range control at the `hud` surface |
| Two or three divers with different bubble sizes that sink at different squeezes | A classic multi-diver toy | MEDIUM | Defer unless cheap |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Simulating air and compressible gas in particles | Realism | No gas phase, and particles are incompressible-ish | A scripted density or volume model driven by the squeeze value |
| Deep press relying on pointer pressure or force touch | "Feels real" | Not portable, and missing on most devices | Binary press plus an optional strength slider |
| Making the squeeze move the water directly (pushing particles down) | Easy | It isn't the mechanism, and it looks like a plunger | Change diver density only |
| Hover to squeeze | Desktop habit | No hover on touch, and hover isn't forwarded | Press and hold |

**README preview:** It needs a cue because the default floats still. Use the Squeeze toggle `action` at about sample 20 (1 s) and again at about sample 110 (5.5 s) to release, so the clip shows a full sink and rise. A press-hold pointer cue would need a new `pointer-down`/`pointer-up` cue pair. Avoid that by giving the scene a toggle action.

**Acceptable 2D simplifications:** A rigid diver with a scripted density, the bubble as a visual, and a bottle that may be slightly open-topped if the squeeze is modeled rather than pressurized.

### 5. Rayleigh–Taylor Instability

**Real-world reference (HIGH):** A heavy fluid resting on a light fluid under gravity is unstable. Any small ripple grows. The linear growth rate is √(A·g·k), where the Atwood number A = (ρ₁−ρ₂)/(ρ₁+ρ₂) and k is the wavenumber, so short wavelengths grow fastest. Surface tension and viscosity suppress the shortest wavelengths. After the linear stage, heavy **spikes (fingers)** fall and light **bubbles** rise. Finger tips roll up into **mushroom caps** (Kelvin–Helmholtz), then neighbors merge, and the interface ends as a mixed or overturned layer. Higher A gives faster, more asymmetric growth with narrow spikes and broad bubbles. Standard SPH can over-damp fine structure, so expect blobbier fingers in a particle sim.

**Defining visual moment (≤10 s):** Two strongly contrasting layers with a flat interface, dimples within about 1 s, clear fingers by about 2–4 s, and mushroom-ish caps and overturn by about 6–10 s. The whole story fits inside the README clip.

**Expected behavior and timing:** It starts in the unstable configuration. A seeded deterministic perturbation (a few sinusoidal modes plus small fixed noise) gives reproducible fingers and stable fingerprints. By the end, heavy fluid sits at the bottom and the scene settles.

**Feasibility (MEDIUM; spike recommended but no engine change needed):** Use **one particle system** with two groups. Give the heavy group extra weight each step (`apply_particle_force_range` with an extra force = (ratio − 1)·m·g along the current gravity vector). Two systems would pass through each other. A per-group density change in the engine would be a deep change that risks the 25 fingerprints. Do not set color mixing. Optionally set the tensile or viscous flag to smooth the interface (realistic short-wave suppression).

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Heavy layer on top of light layer in a closed tank, two high-contrast colors | Silhouette; colors are the only cue | LOW | For example warm orange over deep blue; colors never mix |
| Fingers visibly form and descend within about 3 s | The defining moment | MEDIUM | Tune the extra weight and the initial perturbation. Seeded so it's deterministic |
| Light fluid rises as bubbles and the layers overturn by about 10 s | Completes the story | MEDIUM | |
| **Density ratio** control (recreate), for example 1.2×–3× (Atwood about 0.09–0.5) | Users expect to vary the driving force; named in the brief | LOW | Maps to the bridge's extra-weight factor. Show the ratio, and optionally the Atwood number, in the label or hint |
| **Restart** (Flip tank or Re-layer) so the show can replay without full Reset | Watch-first scenes still want a replay | LOW-MEDIUM | Reset may be enough. A dedicated action is nicer |
| Extra weight follows the gravity vector (slider and phone tilt) | Tilt is global; hard-coding −y breaks the illusion | LOW | Gravity 2–80 m/s² scales the growth speed, which is a free lesson |

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Perturbation** preset: single mode, few modes, or noise | Shows wavelength selection: one big mushroom vs many fingers | LOW | Deterministic seeds only |
| **Flip tank** action that rotates the stable (heavy-below) tank 180° to trigger the instability on demand | The physical way people demo it with a sealed tank | MEDIUM | Can share the hourglass flip machinery. Instant mirror is acceptable |
| Interface tension or viscosity toggle (tensile or viscous flag) | Shows that small fingers get suppressed | MEDIUM | Flags are per particle, set at creation, so it's a recreate control |
| Phone tilt: tip the stable tank sideways to set off sloshing fingers | Free with global tilt | LOW | Comes with the gravity-vector rule |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Two particle systems with different densities | Looks like the obvious "two densities" | Systems don't interact, so the fluids would pass through each other | One system, per-group extra weight |
| Engine-level per-particle mass or per-group density | "Proper" physics | Deep solver change; risks the 25 scene fingerprints | Bridge-side force |
| Color-mixing flag | Pretty blending | Smears the interface, so fingers become invisible | Fixed per-group colors |
| Unseeded random perturbation | Variety | Breaks determinism and fingerprint tests | Seeded perturbation presets |
| Claiming quantitative √(A·g·k) growth | Sounds scientific | Not validated; SPH damps growth | Describe it qualitatively; any growth chart is "observed in this sim" |

**README preview:** Watch-first, no cue. This is the best README candidate: flat start, fingers, overturn, all inside 10 s. Keep it at or under about 3,000 particles so the wireframe circle cap isn't hit, and make the two colors stay distinct in wireframe.

**Acceptable 2D simplifications:** Immiscible particle groups, density simulated as extra weight, a seeded rather than spontaneous perturbation, and coarse mushroom caps.

### 6. Fake Ferrofluid

**Real-world reference (HIGH for phenomenon):** A ferrofluid puddle is flat with no field. Above a critical field it erupts into a regular array of **cone-shaped spikes** (the Rosensweig or normal-field instability). The spacing is set by the capillary-gravity length, roughly centimeter scale and independent of field strength, while the spike *height* grows with field strength. The onset is **hysteretic**. Near a single magnet, spikes crowd toward the strongest field, the whole puddle creeps toward the magnet, and spikes lean along field lines. Removing the magnet collapses the spikes back to a glossy puddle. Physically based SPH ferrofluid needs many particles to show spikes. Production VFX commonly fakes the spikes procedurally.

**Defining visual moment (≤10 s):** A dark glossy puddle rises into a crown of sharp spikes reaching toward the magnet, then collapses when the magnet leaves.

**Expected behavior and timing:** Spikes appear within about 0.5 s of the magnet arriving and collapse within about 1 s of it leaving. Dragging the magnet makes the spikes and the puddle follow.

**Representation (MEDIUM; spike recommended):** A pure point attraction makes a **blob or cone, or tears droplets off** like iron filings, not a spike comb. Fake the instability:

- per-particle force toward the magnet with a capped, softened falloff (no singularity)
- plus a spatial modulation: virtual "pins" spaced at the spike wavelength along the puddle, or a cos(k·x) lateral term, whose strength scales with magnet proximity
- with the tensile flag for cohesion so the liquid stays one body

Field strength must stay below the point where particles detach from the puddle.

#### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Dark, glossy liquid puddle in a shallow dish | Silhouette | LOW | The color must contrast with the `#071018` canvas: graphite or blue-violet with lit edges, not pure black |
| **Multiple separate spikes** (not one blob) rising toward the magnet | Named in the milestone goal; spikes are the whole appeal | HIGH | Needs the modulated-force fake and tuning. Spike count about 5–9 across the dish |
| **Press and drag** moves the magnet; spikes follow and lean toward it | "Spikes toward the pointer" | MEDIUM | Pointer down/move/up already exist. Hover isn't forwarded and doesn't exist on touch |
| Releasing the pointer (or Magnet off) collapses the spikes back to a puddle | Contrast between on and off is the "wow" | LOW-MEDIUM | |
| Default magnet already active (for example under the dish center) so the scene is alive on load | Watch-first; the README needs motion without new cue kinds | LOW | When the pointer releases, the magnet returns to its rest position, or stays where dropped (decide in planning) |
| **Magnet strength** range control (0 = off) | Shows the threshold: no spikes below it, taller spikes above | LOW | A runtime range control |
| A drawn magnet marker at the field source | Users must see what the liquid reacts to | LOW-MEDIUM | Overlay or a sensor body. Mark it as a HUD obstacle only if it is chrome; it is world content here |

#### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Magnet position preset: below the dish (symmetric crown) or follow the pointer (leaning spikes) | Two classic looks | LOW | |
| Puddle creeps toward an off-center magnet | Matches real behavior; feels alive | LOW | Falls out of the attraction term |
| Hysteresis: spikes persist slightly below onset when strength ramps down | Faithful to the literature | MEDIUM | Optional; easy to get wrong |
| A GPU Water look tuned for ferrofluid (dark tint, strong highlight) | Gloss is the signature | MEDIUM | Only if the existing Water mode's tint keeps the dark color; no new render mode |

#### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Pure point attraction toward the cursor | Easiest | Pulls particles into a ball or launches droplets, so it reads as gravity well, not ferrofluid | Capped falloff + spatial spike modulation + cohesion |
| Real magnetostatics (dipole–dipole particle forces, Kelvin force) | "Physically correct" | O(n²) or field solve; known to levitate surfaces; far beyond scope | A fake field, credited as such |
| Hover-driven magnet | Natural on desktop | No hover on touch; pointer moves are only forwarded while pressed | Press and drag |
| Pure black liquid | "Ferrofluid is black" | Invisible on the near-black canvas | A dark tinted color with highlights |
| Unbounded strength slider | Spectacle | Particles detach and fly to the pointer | Clamp the maximum below detachment |

**README preview:** Watch-first via the default magnet under the dish, so spikes are visible from about 1 s. An optional `action` cue (for example "Magnet off" at about 6 s and back on at about 8 s) shows the collapse and rise using the existing cue kind. A moving magnet in the clip would need a new held-pointer cue, so avoid that.

**Acceptable 2D simplifications:** A 2D crown (a row of spikes, not hexagons), a scripted field, a spike spacing not tied to real capillary length, and labeling it "fake ferrofluid" in the title or description.

## Cross-Scene Table Stakes (Every New Scene)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Catalog record: title, one-line description, interaction hint, controls, honest credits ("original experimental scene"; credit fakes such as a modeled pull, stick rule, or fake field) | Repo catalog rules and transparency rule | LOW | `SCENE_IDS` order must match README plans |
| Portrait frame in `portrait-bounds.ts` that fills an iPhone canvas (about 390×844) | Repo rule; `portrait-bounds.test.ts` | LOW-MEDIUM | Hourglass, diver bottle, and R–T are naturally tall. Siphon, screw, and ferrofluid are wide, so frame the action (siphon: the tube plus both water lines) |
| README SVG/WebP plan, with cues where the default is still | Repo rule; `assertReadmeSvgPlanCoverage` | LOW | Cue needs: diver (squeeze toggle), optional hourglass flip, optional ferrofluid toggle |
| Shared scale legend in meters, with plausible real dimensions where cheap | Repo rule | LOW | Suggested: hourglass about 0.2–0.3 m, diver bottle about 0.3 m, ferro dish about 0.1–0.15 m, R–T tank about 0.3–1 m. Siphon and screw can be about 1 m. Particle radius and count decide the real limits |
| Gravity slider (`withGravitySlider`), with scene logic following the gravity vector | Every scene has it; tilt is global | LOW | Bridge forces (R–T, sand stick rule, magnet) must not assume −y |
| HUD safe frame; new on-canvas chrome carries `data-hud-obstacle` on leaf elements | Repo rule; `hud-safe-frame.spec.ts` | LOW | Squeeze, Flip, and magnet controls are panel or HUD controls, not canvas chrome |
| Watch-first or immediate motion on load (within about 1 s) | Recognizable at a glance | LOW-MEDIUM | Diver is the exception: it floats still but must invite a press (hint text) |
| Deterministic build and steps (seeded perturbations, no `HashMap` order, no time-based randomness) | Existing fingerprint culture | LOW | Add new scene fingerprints; the 25 existing ones must stay identical |
| Particle budget ≤ about 3,000 per scene | Phone frame rate; render cap 4,000; v1.5 levels stayed under 3,000 | LOW | |
| Works in every render mode (Wireframe, Shaded blob, GPU Water, …) with readable colors | Existing picker | LOW | Sand and ferrofluid colors need checking in each mode |

## Feature Dependencies

```
Granular "sand rule" spike (stick-slip / rigid grains)
    └──required by──> Hourglass peaked pile (table stakes)
                          └──enables──> Screw "Material: sand" (differentiator)
                          └──enables──> Hourglass Sand vs Water lesson

Bridge per-step force on a contiguous group range (apply_particle_force_range)
    ├──required by──> R–T extra weight (density ratio)
    └──required by──> Ferrofluid magnet field

Gravity-vector-aware bridge forces
    └──required by──> R–T, hourglass stick rule, ferrofluid (slider + phone tilt correctness)

Flip machinery (kinematic 180° rotation or instant mirror)
    ├──required by──> Hourglass Flip (table stakes)
    └──enhances──> R–T Flip tank (differentiator)

Runtime fixture density change (set_fixture_density)
    └──required by──> Cartesian diver sink/rise

Toggle-style `action` control
    └──enables──> README cues for diver squeeze and ferrofluid on/off without new cue kinds

Press-and-hold pointer (existing down/move/up)
    ├──required by──> Diver squeeze
    └──required by──> Ferrofluid magnet drag

"Atmosphere" mechanism (weighted lids OR modeled tube pull) ──required by──> Siphon crest above source

Kinematic flight wrap without particle blow-up ──required by──> Archimedes screw pockets

Scene overlay drawing (helix, bubble, magnet marker, level marks)
    └──enhances──> Screw, Diver, Ferrofluid, Siphon (differentiators; magnet marker is table stakes)

Two particle systems ──conflicts──> R–T (systems don't interact)
Color-mixing flag ──conflicts──> R–T interface and siphon tracer
Hover-based input ──conflicts──> phone support (no hover, not forwarded)
```

### Dependency Notes

- **The hourglass requires the sand-rule spike.** v1.5 measured powder piling flat. Without a granular rule the scene fails its own recognizability criterion. If the spike fails, drop or replace the hourglass, as v1.5 dropped powder.
- **R–T and ferrofluid share the bridge force pattern.** Build it once (with gravity-vector awareness) in whichever lands first. The seed rated R–T riskiest because of per-group density; the one-system extra-weight approach removes the engine change, so R–T risk drops to MEDIUM.
- **Siphon and screw carry the highest new-mechanism risk** (negative pressure and the flight wrap). They belong late in the milestone, consistent with "riskiest scenes last".
- **The flip machinery is shared** between the hourglass (required) and R–T (optional), so build it with the hourglass.
- **The diver and ferrofluid need a toggle action** so README cues work with the existing `action` cue kind. That avoids extending `ReadmeSvgCue` with held-pointer cues.

## MVP Definition

### Launch With (v1.6)

- [ ] **Cartesian diver**: press-hold and keyboard squeeze, sink and rise, visible squeeze feedback, iPhone long-press safe. Lowest risk; reuses buoyancy and density.
- [ ] **Rayleigh–Taylor**: one system, seeded perturbation, density ratio control, gravity-aware extra weight. Best README story; medium risk.
- [ ] **Fake ferrofluid**: default magnet, press-drag magnet, strength control, separate spikes, visible dark color. Medium-high risk on the spike look.
- [ ] **Sand hourglass**: pour, peaked pile (after the spike passes), Flip (instant mirror is acceptable). Gated on the sand-rule spike.
- [ ] **Siphon**: primed start, crest above source, stops at equal levels, outlet-height control. Gated on the atmosphere spike.
- [ ] **Archimedes screw**: tilted trough, moving flights, pockets, crank speed, closed-loop return. Gated on the flight-wrap spike.

### Add After Validation (within v1.6 if cheap)

- [ ] Diver bubble-shrink visual and depth-dependent "no return" behavior, once the basic diver is stable.
- [ ] Animated hourglass flip and phone-tilt flip, once the stick rule releases correctly.
- [ ] Siphon Lift inlet and Prime actions, plus tracer tint, once the atmosphere approach is chosen.
- [ ] R–T perturbation presets and Flip tank, once the flip machinery exists.
- [ ] Screw reverse direction, an easy follow-on.

### Future Consideration

- [ ] Screw material switch (sand), which depends on a credible granular rule.
- [ ] Screw tilt-angle control with spill-back. High risk for a kinematic assembly holding particles.
- [ ] Hourglass neck-width jamming and Sand vs Water flow-rate lesson. Requires evidence that the sim reproduces it.
- [ ] Ferrofluid hysteresis and multiple-diver variants.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Diver press-to-sink plus keyboard toggle | HIGH | LOW-MEDIUM | P1 |
| R–T fingers with density ratio | HIGH | MEDIUM | P1 |
| Ferrofluid separate spikes plus drag magnet | HIGH | MEDIUM-HIGH | P1 |
| Hourglass peaked pile plus Flip | HIGH | HIGH (spike) | P1 if the spike passes |
| Siphon stops at equal levels | HIGH | HIGH (spike) | P1 if the spike passes |
| Screw pockets plus crank speed | HIGH | HIGH (spike) | P1 if the spike passes |
| Siphon outlet-height control (flow reversal) | HIGH | MEDIUM | P2 |
| Diver bubble visual | MEDIUM | MEDIUM | P2 |
| Animated hourglass flip / phone-tilt flip | HIGH | MEDIUM-HIGH | P2 |
| R–T perturbation presets / Flip tank | MEDIUM | LOW-MEDIUM | P2 |
| Siphon Lift inlet / Prime / tracer tint | MEDIUM | LOW-MEDIUM | P2 |
| Screw reverse | MEDIUM | LOW | P2 |
| Diver no-return depth | MEDIUM | MEDIUM | P3 |
| Screw helix overlay | MEDIUM | MEDIUM | P3 |
| Screw tilt control / sand material | MEDIUM | HIGH | P3 |
| Hourglass flow counter / neck jamming | LOW-MEDIUM | MEDIUM | P3 |
| Ferrofluid hysteresis | LOW | MEDIUM | P3 |

## Recognizability Criteria (Concrete, Testable)

| Scene | Must be true by t = 10 s at defaults (portrait and landscape) | Suggested automated proxy |
|-------|----------------------------------------------------------------|---------------------------|
| Siphon | Particles present in the tube above the source water line at all sampled times; source level fell and outlet level rose by ≥ 30% of the initial difference | Particle count in the crest region > 0 for every sample; level heights from particle y-percentiles per tank |
| Archimedes screw | ≥ 1 distinct pocket has crossed the top end; upper basin count increased | Upper basin particle count over time |
| Hourglass | A stream is present in the neck; the bottom deposit's height-to-half-width ratio corresponds to a slope ≥ about 25° | Fit the slope from deposit column heights. Compare against water as a control (as in the v1.5 spike method) |
| Cartesian diver | After the squeeze cue the diver reaches the bottom third within 3 s; after release it returns to the top third within 4 s | Diver body y over time |
| Rayleigh–Taylor | By 4 s ≥ 3 heavy-colored fingers reach below the initial interface by ≥ 20% of tank height; by 10 s heavy group mean y is below light group mean y | Per-group y statistics; count of x-bins where heavy particles penetrate |
| Ferrofluid | With the magnet on, the surface height profile shows ≥ 4 local maxima (spikes) whose peaks rise ≥ 2× the flat puddle depth; with it off, the profile flattens within 1 s | Column-max heights across the dish, then count peaks |

## Competitor and Reference Analysis

| Scene | Typical web demos / references | Our approach |
|-------|-------------------------------|--------------|
| Siphon | Textbook diagrams; Steve Mould's "world's tallest siphon" video; interactive visualizations with a scripted flow | Real particles over a real crest, with honest atmosphere modeling and outlet-height play |
| Archimedes screw | Animated diagrams of a rotating helix; screw-conveyor CAD animations | Physical particle pockets lifted by moving flights in 2D side view |
| Hourglass | Hourglass visual effects in particle toys often just pour water-like dots | Peaked pile via a credited sand rule plus Flip; the peak is the differentiator |
| Cartesian diver | Classroom demo videos; simple 2D applets with scripted density | Press-and-hold on any device, with optional bubble and no-return depth |
| Rayleigh–Taylor | CFD and SPH research renders; fluid toys with dye | Interactive density ratio and perturbation in a real particle sim, with a full story in 10 s |
| Ferrofluid | Procedural VFX spikes; real ferrofluid videos (e.g. art displays) | Particle puddle with a faked field: modulated attraction plus cohesion, press-drag magnet |

## Sources

Codebase (HIGH):
- `crates/liquidfun/src/particle/solver/pressure.rs` (non-negative pressure clamp)
- `crates/liquidfun/src/particle/definition/system_definition.rs` (per-system density and gravity scale)
- `crates/liquidfun/src/world/particle_object/system.rs` (`apply_particle_force_range`)
- `crates/liquidfun/src/world/object/fixture_object.rs` (`set_fixture_density`)
- `third_party/liquidfun/.../Programmers-Guide/html/md__chapter11__particles.html` (multiple systems do not interact; powder, tensile, and viscous flags)
- `.planning/phases/40-timed-valve-and-teaching-levels/40-08-SUMMARY.md` (powder piles flat like water)
- `web/src/input/pointer.ts`, `web/scripts/readme-svg/plans.ts`, `web/src/export/duration.ts`, `web/src/render/canvas.ts`, `web/src/render/particle-limit.ts`, `web/src/input/tilt-binding.ts`, `web/src/catalog/gravity-slider.ts`

Physics references:
- [Angle of repose, Wikipedia](https://en.wikipedia.org/wiki/Angle_of_repose): dry sand about 34°, with a 30–35° range (MEDIUM-HIGH, several agreeing sources)
- [UW angle of repose handout](https://sites.math.washington.edu/~aloveles/ProjectGallery/AngleOfRepose.pdf), [UCSB MRL](https://mrlweb.mrl.ucsb.edu/node/268)
- [Flow rate of granular materials through an orifice (arXiv 0707.4550)](https://arxiv.org/pdf/0707.4550): Beverloo law
- [Flow in an hourglass: particle friction and stiffness matter (arXiv 2104.08186)](https://arxiv.org/pdf/2104.08186)
- [MIT Technology Review: The mystery of sand flow through an hourglass](https://www.technologyreview.com/2010/05/19/121892/the-mystery-of-sand-flow-through-an-hourglass)
- [KSB: Archimedean screw pump](https://www.ksb.com/en-global/centrifugal-pump-lexicon/article/archimedean-screw-pump-1117986): about 30° incline, below 100 rpm
- [IIUM screw pump study](https://irep.iium.edu.my/122871/), [UMP CFD study](https://umpir.ump.edu.my/id/eprint/41228/), [Firgelli Archimedes screw](https://www.firgelliauto.com/blogs/mechanisms/archimedes-screw): about 50% pocket fill (vendor, LOW)
- [Cartesian diver, Wikipedia](https://en.wikipedia.org/wiki/Cartesian_diver): unstable equilibrium
- [The Cartesian diver and the fold catastrophe (Coimbra)](https://estudogeral.uc.pt/bitstream/10316/14165/1/The%20Cartesian%20diver%20and%20the%20fold%20catastrophe.pdf), [Coimbra demonstration apparatus](https://baes.uc.pt/bitstream/10316/12329/1/A%20demonstration%20apparatus%20for%20the%20Cartesian%20diver.pdf)
- [Purdue: The Cartesian Diver demo](https://chemed.chem.purdue.edu/genchem/demosheets/1.2.html), [LibreTexts: The Cartesian Diver](https://chem.libretexts.org/Ancillary_Materials/Demos_Techniques_and_Experiments/Lecture_Demonstrations/Additional_Demos/The_Cartesian_Diver)
- [Rayleigh–Taylor instability, Wikipedia](https://en.wikipedia.org/wiki/Rayleigh%E2%80%93Taylor_instability)
- [Late-time immiscible RTI, lattice Boltzmann (arXiv 2009.14655)](https://arxiv.org/pdf/2009.14655), [3D single-mode RTI (arXiv 2201.04915)](https://arxiv.org/pdf/2201.04915)
- [SPH RTI advantages and limitations (HAL)](https://hal.archives-ouvertes.fr/hal-02127871), [Pressure-entropy SPH and mixing (arXiv 1206.5006)](https://arxiv.org/pdf/1206.5006)
- [Normal field instability under side-wall effects (arXiv 0904.3864)](https://ar5iv.arxiv.org/html/0904.3864), [Gollwitzer et al. 2010 NJP](https://epub.uni-bayreuth.de/3956/1/Gollwitzer_2010_New_J._Phys._12_093037.pdf): Rosensweig spikes, hexagons, hysteresis
- [On the accurate large-scale simulation of ferrofluids (80.lv summary)](https://80.lv/articles/accurate-large-scale-simulation-of-ferrofluids-002sgr/), [Visual simulation of magnetic fluids (SciTePress 2012)](https://www.scitepress.org/Papers/2012/38673/38673.pdf): SPH struggles with spikes; procedural fakes are common
- [Firgelli: Siphon mechanism](https://www.firgelliauto.com/blogs/mechanisms/siphon), [tec-science siphon](https://www.tec-science.com/?p=24687), [Steve Mould world's tallest siphon](https://thekidshouldseethis.com/post/worlds-tallest-siphon-steve-mould-video): priming, stop conditions, about 10 m limit

---
*Feature research for: v1.6 Physics Curiosity Scenes (liquidfun-rs playground)*
*Researched: 2026-10-14*
