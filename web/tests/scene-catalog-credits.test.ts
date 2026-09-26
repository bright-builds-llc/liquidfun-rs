import { describe, expect, it } from "vitest";

import {
  SCENE_IDS,
  SCENES,
  maybeSceneById,
  type SceneId,
} from "../src/catalog/scenes";

const SHOWCASE_HREF = "https://google.github.io/liquidfun/";
const PARTICLE_GUIDE_HREF =
  "https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html";
const FAUCET_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Faucet.h";
const PARTICLES_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testParticles.js";
const PARTICLES_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Particles.h";
const LIQUID_TIMER_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testLiquidTimer.js";
const LIQUID_TIMER_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/LiquidTimer.h";
const SURFACE_TENSION_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSurfaceTension.js";
const SURFACE_TENSION_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/ParticlesSurfaceTension.h";
const ELASTIC_PARTICLES_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testElasticParticles.js";
const ELASTIC_PARTICLES_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/ElasticParticles.h";
const RIGID_PARTICLES_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testRigidParticles.js";
const RIGID_PARTICLES_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/RigidParticles.h";
const SOUP_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSoup.js";
const SOUP_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Soup.h";
const SOUP_STIRRER_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSoupStirrer.js";
const SOUP_STIRRER_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/SoupStirrer.h";
const IMPULSE_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testImpulse.js";
const IMPULSE_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Impulse.h";
const WAVE_MACHINE_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testWaveMachine.js";
const WAVE_MACHINE_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/WaveMachine.h";
const THEO_JANSEN_JS_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testTheoJansen.js";
const THEO_JANSEN_H_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/TheoJansen.h";
const PINNED_COMMIT = "7f20402173fd143a3988c921bc384459c6a858f2";

describe("scene catalog credits", () => {
  it("credits this repo's scene modules and host-locked inspiration", () => {
    // Arrange
    const expectedPaths: Readonly<Record<SceneId, string>> = {
      "dam-break": "crates/liquidfun-wasm/src/scene/dam_break.rs",
      fountain: "crates/liquidfun-wasm/src/scene/fountain.rs",
      "float-or-sink": "crates/liquidfun-wasm/src/scene/float_or_sink.rs",
      "color-mixer": "crates/liquidfun-wasm/src/scene/color_mixer.rs",
      "jelly-drop": "crates/liquidfun-wasm/src/scene/jelly_drop.rs",
      "water-wheel": "crates/liquidfun-wasm/src/scene/water_wheel.rs",
      particles: "crates/liquidfun-wasm/src/scene/particles.rs",
      "liquid-timer": "crates/liquidfun-wasm/src/scene/liquid_timer.rs",
      "surface-tension": "crates/liquidfun-wasm/src/scene/surface_tension.rs",
      "elastic-particles":
        "crates/liquidfun-wasm/src/scene/elastic_particles.rs",
      "rigid-particles": "crates/liquidfun-wasm/src/scene/rigid_particles.rs",
      soup: "crates/liquidfun-wasm/src/scene/soup.rs",
      "soup-stirrer": "crates/liquidfun-wasm/src/scene/soup_stirrer.rs",
      impulse: "crates/liquidfun-wasm/src/scene/impulse.rs",
      "wave-machine": "crates/liquidfun-wasm/src/scene/wave_machine.rs",
      "theo-jansen": "crates/liquidfun-wasm/src/scene/theo_jansen.rs",
      "liquid-tumbler": "crates/liquidfun-wasm/src/scene/liquid_tumbler.rs",
    };
    const pinnedBasinInspiration = {
      particles: [
        { label: "Pinned Particles.js", href: PARTICLES_JS_HREF },
        { label: "Pinned Particles.h", href: PARTICLES_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "liquid-timer": [
        { label: "Pinned LiquidTimer.js", href: LIQUID_TIMER_JS_HREF },
        { label: "Pinned LiquidTimer.h", href: LIQUID_TIMER_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "surface-tension": [
        { label: "Pinned SurfaceTension.js", href: SURFACE_TENSION_JS_HREF },
        {
          label: "Pinned ParticlesSurfaceTension.h",
          href: SURFACE_TENSION_H_HREF,
        },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "elastic-particles": [
        { label: "Pinned ElasticParticles.js", href: ELASTIC_PARTICLES_JS_HREF },
        { label: "Pinned ElasticParticles.h", href: ELASTIC_PARTICLES_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "rigid-particles": [
        { label: "Pinned RigidParticles.js", href: RIGID_PARTICLES_JS_HREF },
        { label: "Pinned RigidParticles.h", href: RIGID_PARTICLES_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      soup: [
        { label: "Pinned Soup.js", href: SOUP_JS_HREF },
        { label: "Pinned Soup.h", href: SOUP_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "soup-stirrer": [
        { label: "Pinned SoupStirrer.js", href: SOUP_STIRRER_JS_HREF },
        { label: "Pinned SoupStirrer.h", href: SOUP_STIRRER_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      impulse: [
        { label: "Pinned Impulse.js", href: IMPULSE_JS_HREF },
        { label: "Pinned Impulse.h", href: IMPULSE_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "wave-machine": [
        { label: "Pinned WaveMachine.js", href: WAVE_MACHINE_JS_HREF },
        { label: "Pinned WaveMachine.h", href: WAVE_MACHINE_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
      "theo-jansen": [
        { label: "Pinned TheoJansen.js", href: THEO_JANSEN_JS_HREF },
        { label: "Pinned TheoJansen.h", href: THEO_JANSEN_H_HREF },
        { label: "LiquidFun showcase", href: SHOWCASE_HREF },
      ],
    } as const;

    // Act
    const implementationPaths = SCENES.map(
      (scene) => scene.credits.implementationPath,
    );
    const inspirationById = Object.fromEntries(
      SCENES.map((scene) => [
        scene.id,
        scene.credits.inspiration.map((item) => item.href),
      ]),
    );
    const pinnedInspirationHrefs = SCENES.flatMap((scene) =>
      scene.credits.inspiration.map((item) => item.href),
    ).filter((href) => href.includes("github.com/google/liquidfun/blob/"));
    const particlesInspiration = maybeSceneById("particles")?.credits.inspiration;
    const liquidTimerInspiration =
      maybeSceneById("liquid-timer")?.credits.inspiration;
    const surfaceTensionInspiration =
      maybeSceneById("surface-tension")?.credits.inspiration;
    const elasticParticlesInspiration =
      maybeSceneById("elastic-particles")?.credits.inspiration;
    const rigidParticlesInspiration =
      maybeSceneById("rigid-particles")?.credits.inspiration;
    const soupInspiration = maybeSceneById("soup")?.credits.inspiration;
    const soupStirrerInspiration =
      maybeSceneById("soup-stirrer")?.credits.inspiration;
    const impulseInspiration = maybeSceneById("impulse")?.credits.inspiration;
    const waveMachineInspiration =
      maybeSceneById("wave-machine")?.credits.inspiration;
    const theoJansenInspiration =
      maybeSceneById("theo-jansen")?.credits.inspiration;

    // Assert
    expect(implementationPaths).toEqual(
      SCENE_IDS.map((id) => expectedPaths[id]),
    );
    expect(
      implementationPaths.some((path) => path.includes("google/liquidfun")),
    ).toBe(false);
    expect(inspirationById["dam-break"]).toEqual([SHOWCASE_HREF]);
    expect(inspirationById.fountain).toEqual([FAUCET_HREF, SHOWCASE_HREF]);
    expect(inspirationById["float-or-sink"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["color-mixer"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["jelly-drop"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["water-wheel"]).toEqual([SHOWCASE_HREF]);
    expect(inspirationById["liquid-tumbler"]).toEqual([SHOWCASE_HREF]);
    expect(particlesInspiration).toEqual([...pinnedBasinInspiration.particles]);
    expect(liquidTimerInspiration).toEqual([
      ...pinnedBasinInspiration["liquid-timer"],
    ]);
    expect(surfaceTensionInspiration).toEqual([
      ...pinnedBasinInspiration["surface-tension"],
    ]);
    expect(elasticParticlesInspiration).toEqual([
      ...pinnedBasinInspiration["elastic-particles"],
    ]);
    expect(rigidParticlesInspiration).toEqual([
      ...pinnedBasinInspiration["rigid-particles"],
    ]);
    expect(soupInspiration).toEqual([...pinnedBasinInspiration.soup]);
    expect(soupStirrerInspiration).toEqual([
      ...pinnedBasinInspiration["soup-stirrer"],
    ]);
    expect(impulseInspiration).toEqual([...pinnedBasinInspiration.impulse]);
    expect(waveMachineInspiration).toEqual([
      ...pinnedBasinInspiration["wave-machine"],
    ]);
    expect(theoJansenInspiration).toEqual([
      ...pinnedBasinInspiration["theo-jansen"],
    ]);
    expect(
      pinnedInspirationHrefs.every((href) => href.includes(PINNED_COMMIT)),
    ).toBe(true);
    expect(maybeSceneById("particles")?.controls).toHaveLength(1);
    expect(maybeSceneById("liquid-timer")?.controls).toHaveLength(1);
    expect(maybeSceneById("surface-tension")?.controls).toHaveLength(1);
    expect(maybeSceneById("elastic-particles")?.controls).toHaveLength(1);
    expect(maybeSceneById("rigid-particles")?.controls).toHaveLength(1);
    expect(maybeSceneById("soup")?.controls).toHaveLength(1);
    expect(maybeSceneById("wave-machine")?.controls).toHaveLength(3);
    expect(maybeSceneById("liquid-tumbler")?.controls).toHaveLength(1);
  });
});
