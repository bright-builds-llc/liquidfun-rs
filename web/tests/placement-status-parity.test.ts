import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { PLACEMENT_STATUS } from "../src/puzzle/placement-status";

const VALIDATE_RS = join(
  dirname(fileURLToPath(import.meta.url)),
  "../../crates/liquidfun-wasm/src/puzzle/validate.rs",
);

const EXPECTED_STATUS_COUNT = 13;

/** `[name, code]` pairs from the body of Rust `enum PlacementStatus`, in source order. */
function rustStatusPairs(): [string, number][] {
  const text = readFileSync(VALIDATE_RS, "utf8");
  const start = text.indexOf("enum PlacementStatus {");
  if (start < 0) {
    throw new Error("enum PlacementStatus not found in validate.rs");
  }

  const end = text.indexOf("\n}", start);
  if (end < 0) {
    throw new Error("enum PlacementStatus is not closed in validate.rs");
  }

  const body = text.slice(start, end);
  return Array.from(body.matchAll(/(\w+) = (\d+),/g), (match) => [
    match[1] ?? "",
    Number(match[2]),
  ]);
}

function toPascalCase(name: string): string {
  return name.charAt(0).toUpperCase() + name.slice(1);
}

describe("placement status parity with Rust (D-01)", () => {
  it("Rust declares thirteen placement statuses", () => {
    // Arrange
    const pairs = rustStatusPairs();

    // Act
    const count = pairs.length;

    // Assert
    expect(count).toBe(EXPECTED_STATUS_COUNT);
  });

  it("every Rust name and code matches PLACEMENT_STATUS", () => {
    // Arrange
    const expected = Object.entries(PLACEMENT_STATUS).map(
      ([name, code]): [string, number] => [toPascalCase(name), code],
    );

    // Act
    const actual = rustStatusPairs();

    // Assert
    expect(actual).toEqual(expected);
  });
});
