import { describe, expect, it } from "vitest";

import { SCENE_IDS } from "../src/catalog/scenes";
import { maybeParseSvgExportRequest } from "../src/export/messages";
import { CAPTURE_PROFILE } from "../scripts/demo-media/model";
import {
  assertReadmeSvgPlanCoverage,
  README_SVG_PLANS,
  README_SVG_SECONDS,
  readmeSvgRepoPath,
  readmeSvgRequest,
  readmeWebpRepoPath,
} from "../scripts/readme-svg/plans";
import {
  README_SVG_BEGIN,
  README_SVG_END,
  readmeSvgGalleryBody,
  upsertReadmeSvgGallery,
} from "../scripts/readme-svg/section";

const INSTALL_ANCHOR = "The private, unpublished `liquidfun-wasm` wrapper";
const HOBBY_HEADING = "## Hobby development";
const RETIRED_BEGIN = "<!-- dam-break-animated-svg:begin -->";
const RETIRED_END = "<!-- dam-break-animated-svg:end -->";

function legacyReadme(): string {
  return [
    "### Demo gallery",
    "",
    "Each preview is generated deterministically from the Rust/WASM playground.",
    "",
    "[![Dam Break simulation preview](docs/assets/demos/dam-break.webp)](docs/assets/demos/dam-break.mp4)",
    "",
    `${INSTALL_ANCHOR} and SolidJS player rebuild`,
    "from a clean checkout.",
    "",
    RETIRED_BEGIN,
    "",
    "### Dam Break animated SVG",
    "",
    "[![Dam Break 10 second animated SVG](docs/assets/demos/dam-break-10s.svg)](docs/assets/demos/dam-break-10s.svg)",
    "",
    RETIRED_END,
    "",
    HOBBY_HEADING,
    "",
    "The goal is fun.",
    "",
  ].join("\n");
}

describe("readmeSvgRequest", () => {
  it("selects the default 10 second export for every catalog scene", () => {
    // Arrange
    assertReadmeSvgPlanCoverage();
    const damBreak = README_SVG_PLANS[0];
    expect(damBreak).toBeDefined();
    if (damBreak === undefined) {
      return;
    }

    // Act
    const request = readmeSvgRequest(damBreak);
    const parsed = maybeParseSvgExportRequest(request);

    // Assert
    expect(README_SVG_PLANS.map((plan) => plan.id)).toEqual([...SCENE_IDS]);
    expect(parsed).toEqual(request);
    expect(request).toMatchObject({
      sceneId: "dam-break",
      title: "Dam Break",
      durationSeconds: 10,
      controls: [],
      viewportWidth: CAPTURE_PROFILE.viewport.width,
      viewportHeight: CAPTURE_PROFILE.viewport.height,
      zoom: 1,
      panX: 0,
      panY: 0,
      renderMode: "wireframe",
      wireframeStrokeWidth: 0.3,
      maxRenderedParticles: 4000,
    });
    expect(README_SVG_SECONDS).toBe(10);
    expect(readmeSvgRepoPath("dam-break")).toBe("docs/assets/readme/dam-break-10s.svg");
    expect(readmeSvgRepoPath("theo-jansen")).toBe("docs/assets/readme/theo-jansen-10s.svg");
    expect(readmeWebpRepoPath("dam-break")).toBe("docs/assets/readme/dam-break-10s.webp");
    expect(readmeWebpRepoPath("theo-jansen")).toBe("docs/assets/readme/theo-jansen-10s.webp");
  });

  it("nudges scenes that stay still until an action or pointer", () => {
    // Arrange
    const floatOrSink = README_SVG_PLANS.find((plan) => plan.id === "float-or-sink");
    const impulse = README_SVG_PLANS.find((plan) => plan.id === "impulse");
    const damBreak = README_SVG_PLANS.find((plan) => plan.id === "dam-break");

    // Act
    const floatCue = floatOrSink?.cues[0];
    const impulseCue = impulse?.cues[0];

    // Assert
    expect(damBreak?.cues).toEqual([]);
    expect(floatCue).toEqual({ atSample: 0, kind: "action", name: "drop-body" });
    expect(impulseCue).toEqual({
      atSample: 20,
      kind: "pointer-up",
      worldX: 1,
      worldY: 2,
    });
  });
});

describe("upsertReadmeSvgGallery", () => {
  it("replaces the raster gallery and the retired Dam Break section", () => {
    // Arrange
    const readme = legacyReadme();

    // Act
    const updated = upsertReadmeSvgGallery(readme, README_SVG_PLANS);

    // Assert
    expect(updated).toContain(readmeSvgGalleryBody(README_SVG_PLANS));
    expect(updated).toContain("60 fps animated WebP");
    expect(updated).toContain(
      "[![Dam Break simulation preview](docs/assets/readme/dam-break-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break)",
    );
    expect(updated).toContain("[Animated SVG](docs/assets/readme/dam-break-10s.svg)");
    expect(updated).toContain(
      "[![Theo Jansen simulation preview](docs/assets/readme/theo-jansen-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/theo-jansen)",
    );
    expect(updated).not.toContain("](docs/assets/readme/dam-break-10s.svg)]");
    expect(updated).not.toContain(".mp4");
    expect(updated).not.toContain(RETIRED_BEGIN);
    expect(updated).not.toContain("### Dam Break animated SVG");
    expect(updated.indexOf(README_SVG_BEGIN)).toBeLessThan(updated.indexOf(INSTALL_ANCHOR));
    expect(updated.indexOf(README_SVG_END)).toBeLessThan(updated.indexOf(HOBBY_HEADING));
    expect(updated.endsWith(`${HOBBY_HEADING}\n\nThe goal is fun.\n`)).toBe(true);
  });

  it("leaves an already current gallery byte-for-byte unchanged", () => {
    // Arrange
    const once = upsertReadmeSvgGallery(legacyReadme(), README_SVG_PLANS);

    // Act
    const twice = upsertReadmeSvgGallery(once, README_SVG_PLANS);

    // Assert
    expect(twice).toBe(once);
  });

  it("replaces a stale marked gallery without moving the install paragraph", () => {
    // Arrange
    const stale = [
      "### Demo gallery",
      "",
      README_SVG_BEGIN,
      "stale webp",
      README_SVG_END,
      "",
      `${INSTALL_ANCHOR} stays.`,
      "",
      HOBBY_HEADING,
      "",
    ].join("\n");

    // Act
    const updated = upsertReadmeSvgGallery(stale, README_SVG_PLANS);

    // Assert
    expect(updated).toContain("#### [Fountain](");
    expect(updated).not.toContain("stale webp");
    expect(updated.indexOf(README_SVG_END)).toBeLessThan(updated.indexOf(INSTALL_ANCHOR));
  });

  it("rejects a README that has only one gallery marker", () => {
    // Arrange
    const readme = `### Demo gallery\n\n${README_SVG_BEGIN}\nno end\n`;

    // Act
    const update = () => upsertReadmeSvgGallery(readme, README_SVG_PLANS);

    // Assert
    expect(update).toThrow("README SVG gallery markers are incomplete.");
  });

  it("rejects a README with no demo gallery", () => {
    // Arrange
    const readme = "No gallery.\n";

    // Act
    const update = () => upsertReadmeSvgGallery(readme, README_SVG_PLANS);

    // Assert
    expect(update).toThrow("demo gallery");
  });
});
