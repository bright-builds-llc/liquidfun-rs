import { describe, expect, it } from "vitest";

import {
  IMPLEMENTATION_LINK_LABEL,
  NOTICES_LINK_LABEL,
  SCENE_SOURCE_HEADING,
  implementationHref,
} from "../src/components/SceneCredits";

const VALID_SHA = "0123456789abcdef0123456789abcdef01234567";
const DAM_BREAK_PATH = "crates/liquidfun-wasm/src/scene/dam_break.rs";
const REPO_BLOB = "https://github.com/bright-builds-llc/liquidfun-rs/blob";

describe("implementationHref", () => {
  it("never contains google/liquidfun", () => {
    // Arrange
    const path = DAM_BREAK_PATH;

    // Act
    const href = implementationHref(path, VALID_SHA);
    const fallbackHref = implementationHref(path, "https://github.com/google/liquidfun");

    // Assert
    expect(href).toBe(`${REPO_BLOB}/${VALID_SHA}/${DAM_BREAK_PATH}`);
    expect(href.includes("google/liquidfun")).toBe(false);
    expect(fallbackHref.includes("google/liquidfun")).toBe(false);
    expect(fallbackHref.startsWith(`${REPO_BLOB}/`)).toBe(true);
  });

  it("rejects upstream C++ paths instead of using them as implementation", () => {
    // Arrange
    const upstreamPath =
      "https://github.com/google/liquidfun/blob/main/Faucet.h";

    // Act
    const buildHref = () => implementationHref(upstreamPath, VALID_SHA);

    // Assert
    expect(buildHref).toThrow("Scene source path is not allowlisted");
  });
});

describe("SceneCredits copy", () => {
  it("locks Scene source, View scene source, and Third-party notices", () => {
    // Arrange
    const heading = SCENE_SOURCE_HEADING;
    const implementation = IMPLEMENTATION_LINK_LABEL;
    const notices = NOTICES_LINK_LABEL;

    // Act
    const locked = [heading, implementation, notices];

    // Assert
    expect(locked).toEqual([
      "Scene source",
      "View scene source",
      "Third-party notices",
    ]);
  });
});
