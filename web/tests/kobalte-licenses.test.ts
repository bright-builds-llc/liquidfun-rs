import { describe, expect, it } from "vitest";

import {
  deriveRuntimeClosure,
  inventoryDifferences,
  type InstalledPackage,
} from "../scripts/verify-kobalte-licenses";

function installedPackage(
  name: string,
  dependencies: Readonly<Record<string, string>> = {},
): InstalledPackage {
  return {
    name,
    version: "1.0.0",
    license: "MIT",
    source: `https://example.com/${name}`,
    directory: `/node_modules/${name}`,
    dependencies,
  };
}

describe("deriveRuntimeClosure", () => {
  it("recursively follows runtime dependencies only", () => {
    // Arrange
    const packages = new Map([
      ["root", installedPackage("root", { child: "^1.0.0" })],
      ["child", installedPackage("child", { leaf: "^1.0.0" })],
      ["development", installedPackage("development")],
      ["leaf", installedPackage("leaf")],
      ["peer", installedPackage("peer")],
    ]);

    // Act
    const closure = deriveRuntimeClosure("root", (name) => {
      const maybePackage = packages.get(name);
      if (maybePackage === undefined) {
        throw new Error(`unexpected package ${name}`);
      }
      return maybePackage;
    });

    // Assert
    expect(closure.map((entry) => entry.name)).toEqual([
      "child",
      "leaf",
      "root",
    ]);
  });
});

describe("inventoryDifferences", () => {
  it("reports both missing and extra tracked packages", () => {
    // Arrange
    const derived = [
      installedPackage("derived-only"),
      installedPackage("shared"),
    ];
    const tracked = [
      {
        name: "shared",
        version: "1.0.0",
        license: "MIT",
        source: "https://example.com/shared",
      },
      {
        name: "tracked-only",
        version: "1.0.0",
        license: "MIT",
        source: "https://example.com/tracked-only",
      },
    ];

    // Act
    const differences = inventoryDifferences(derived, tracked);

    // Assert
    expect(differences).toContain("missing tracked entry: derived-only");
    expect(differences).toContain("extra tracked entry: tracked-only");
  });
});
