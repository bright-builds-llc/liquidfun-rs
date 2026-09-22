import { describe, expect, it } from "vitest";

import { maybeParseSvgExportRequest } from "../src/export/messages";
import { CAPTURE_PROFILE } from "../scripts/demo-media/model";
import {
  DAM_BREAK_README_SECONDS,
  DAM_BREAK_SVG_REPO_PATH,
  damBreakSvgRequest,
} from "../scripts/dam-break-svg/request";
import {
  DAM_BREAK_SVG_BEGIN,
  DAM_BREAK_SVG_END,
  damBreakAnimatedSvgSection,
  upsertDamBreakAnimatedSvgSection,
} from "../scripts/dam-break-svg/section";

const INSTALL_ANCHOR = "The private, unpublished `liquidfun-wasm` wrapper";
const HOBBY_HEADING = "## Hobby development";

function readmeWithoutSection(): string {
  return [
    "[![Water Wheel simulation preview](docs/assets/demos/water-wheel.webp)](docs/assets/demos/water-wheel.mp4)",
    "",
    `${INSTALL_ANCHOR} and SolidJS player rebuild`,
    "from a clean checkout.",
    "",
    HOBBY_HEADING,
    "",
    "The goal is fun.",
    "",
  ].join("\n");
}

describe("damBreakSvgRequest", () => {
  it("selects the default 10 second Dam Break export", () => {
    // Arrange
    const request = damBreakSvgRequest();

    // Act
    const parsed = maybeParseSvgExportRequest(request);

    // Assert
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
    expect(DAM_BREAK_README_SECONDS).toBe(10);
    expect(DAM_BREAK_SVG_REPO_PATH).toBe("docs/assets/demos/dam-break-10s.svg");
  });
});

describe("upsertDamBreakAnimatedSvgSection", () => {
  it("inserts the animated SVG section at the end of the web playground section", () => {
    // Arrange
    const readme = readmeWithoutSection();

    // Act
    const updated = upsertDamBreakAnimatedSvgSection(readme);

    // Assert
    expect(updated).toContain(damBreakAnimatedSvgSection());
    expect(updated.indexOf(INSTALL_ANCHOR)).toBeLessThan(updated.indexOf(DAM_BREAK_SVG_BEGIN));
    expect(updated.indexOf(DAM_BREAK_SVG_END)).toBeLessThan(updated.indexOf(HOBBY_HEADING));
    expect(updated).toContain(
      "[![Dam Break 10 second animated SVG](docs/assets/demos/dam-break-10s.svg)](docs/assets/demos/dam-break-10s.svg)",
    );
    expect(updated.endsWith(`${HOBBY_HEADING}\n\nThe goal is fun.\n`)).toBe(true);
  });

  it("leaves an already current section byte-for-byte unchanged", () => {
    // Arrange
    const once = upsertDamBreakAnimatedSvgSection(readmeWithoutSection());

    // Act
    const twice = upsertDamBreakAnimatedSvgSection(once);

    // Assert
    expect(twice).toBe(once);
  });

  it("replaces a stale marked section and places it above Hobby development", () => {
    // Arrange
    const stale = [
      "Gallery stays.",
      "",
      DAM_BREAK_SVG_BEGIN,
      "stale",
      DAM_BREAK_SVG_END,
      "",
      "Tail stays.",
      "",
      HOBBY_HEADING,
      "",
    ].join("\n");

    // Act
    const updated = upsertDamBreakAnimatedSvgSection(stale);

    // Assert
    expect(updated.startsWith("Gallery stays.\n\nTail stays.\n\n")).toBe(true);
    expect(updated).toContain("### Dam Break animated SVG");
    expect(updated.indexOf(DAM_BREAK_SVG_END)).toBeLessThan(updated.indexOf(HOBBY_HEADING));
    expect(updated).not.toContain("stale");
  });

  it("rejects a README that has only one marker", () => {
    // Arrange
    const readme = `${DAM_BREAK_SVG_BEGIN}\nno end\n`;

    // Act
    const update = () => upsertDamBreakAnimatedSvgSection(readme);

    // Assert
    expect(update).toThrow("Dam Break animated SVG markers are incomplete.");
  });

  it("rejects a README with no install-paragraph anchor", () => {
    // Arrange
    const readme = "No playground install paragraph.\n";

    // Act
    const update = () => upsertDamBreakAnimatedSvgSection(readme);

    // Assert
    expect(update).toThrow("Hobby development heading");
  });
});
