import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { PUZZLE_IDS } from "../src/catalog/puzzles";
import { LEVEL_HASH_FILE, parseLevelHashes } from "../scripts/puzzle-replay/level-hashes";

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const HASH_A = "0123456789abcdef";
const HASH_B = "fedcba9876543210";

describe("parseLevelHashes", () => {
  it("returns pins in file order", () => {
    // Arrange
    const text = `thick-syrup ${HASH_B}\nfirst-pour ${HASH_A}\n`;

    // Act
    const pins = parseLevelHashes(text);

    // Assert
    expect([...pins.entries()]).toEqual([
      ["thick-syrup", HASH_B],
      ["first-pour", HASH_A],
    ]);
  });

  it("skips blank and comment lines", () => {
    // Arrange
    const text = `# header\n\n  \nfirst-pour ${HASH_A}\n# trailer\n`;

    // Act
    const pins = parseLevelHashes(text);

    // Assert
    expect([...pins.keys()]).toEqual(["first-pour"]);
  });

  const badLines: ReadonlyArray<[string, string]> = [
    ["one field", "first-pour"],
    ["three fields", `first-pour ${HASH_A} extra`],
    ["two spaces between fields", `first-pour  ${HASH_A}`],
    ["an uppercase hash", "first-pour 0123456789ABCDEF"],
    ["a 15-digit hash", "first-pour 0123456789abcde"],
    ["a non-hex digit", "first-pour 0123456789abcdeg"],
  ];

  it.each(badLines)("rejects %s with its 1-based line number", (_name, line) => {
    // Arrange
    const text = `# header\n${line}\n`;

    // Act
    const act = () => parseLevelHashes(text);

    // Assert
    expect(act).toThrow("Invalid level hash line 2");
  });

  it("rejects a repeated level id", () => {
    // Arrange
    const text = `first-pour ${HASH_A}\nfirst-pour ${HASH_B}\n`;

    // Act
    const act = () => parseLevelHashes(text);

    // Assert
    expect(act).toThrow("Duplicate level hash: first-pour");
  });

  it("lists every puzzle id in catalog order in the committed pin file", () => {
    // Arrange
    const text = readFileSync(join(REPO_ROOT, LEVEL_HASH_FILE), "utf8");

    // Act
    const pins = parseLevelHashes(text);

    // Assert
    expect([...pins.keys()]).toEqual([...PUZZLE_IDS]);
  });
});
