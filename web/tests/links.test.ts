import { describe, expect, it } from "vitest";

import { noticesBlobUrl, sceneBlobUrl } from "../src/catalog/links";

const VALID_SHA = "0123456789abcdef0123456789abcdef01234567";
const DAM_BREAK_PATH = "crates/liquidfun-wasm/src/scene/dam_break.rs";
const REPO_BLOB = "https://github.com/bright-builds-llc/liquidfun-rs/blob";
const ALLOWLIST_MESSAGE = "Scene source path is not allowlisted";

describe("sceneBlobUrl", () => {
  it("builds a host-locked blob URL for a valid 40-hex SHA", () => {
    // Arrange
    const path = DAM_BREAK_PATH;

    // Act
    const href = sceneBlobUrl(path, VALID_SHA);

    // Assert
    expect(href).toBe(`${REPO_BLOB}/${VALID_SHA}/${DAM_BREAK_PATH}`);
  });

  it("falls back to main when the SHA is missing, empty, short, or non-hex", () => {
    // Arrange
    const invalidShas = [undefined, "", "01234567", "not-a-sha", "GHIJ"] as const;

    // Act
    const hrefs = invalidShas.map((maybeSha) =>
      sceneBlobUrl(DAM_BREAK_PATH, maybeSha),
    );

    // Assert
    for (const href of hrefs) {
      expect(href).toBe(`${REPO_BLOB}/main/${DAM_BREAK_PATH}`);
    }
  });

  it("rejects traversal and upstream C++ paths", () => {
    // Arrange
    const rejectedPaths = [
      "../LICENSE",
      "liquidfun/Box2D/Testbed/Tests/Faucet.h",
      "/crates/liquidfun-wasm/src/scene/dam_break.rs",
      "crates/liquidfun-wasm/src/scene/../session.rs",
    ];

    // Act
    const errors = rejectedPaths.map((path) => {
      try {
        sceneBlobUrl(path, VALID_SHA);
        return undefined;
      } catch (error) {
        return error;
      }
    });

    // Assert
    for (const error of errors) {
      expect(error).toBeInstanceOf(Error);
      expect((error as Error).message).toBe(ALLOWLIST_MESSAGE);
    }
  });

  it("never returns javascript, off-host, or google/liquidfun implementation hrefs", () => {
    // Arrange
    const hostileInputs = [
      {
        path: "javascript:alert(1)",
        maybeSha: VALID_SHA,
      },
      {
        path: "https://evil.example/crates/liquidfun-wasm/src/scene/dam_break.rs",
        maybeSha: VALID_SHA,
      },
      {
        path: "https://github.com/google/liquidfun/blob/main/Faucet.h",
        maybeSha: VALID_SHA,
      },
      {
        path: DAM_BREAK_PATH,
        maybeSha: "https://github.com/google/liquidfun/blob/main/Faucet.h",
      },
      {
        path: DAM_BREAK_PATH,
        maybeSha: "javascript:alert(1)",
      },
    ];

    // Act
    const hrefs = hostileInputs.map((input) => {
      try {
        return sceneBlobUrl(input.path, input.maybeSha);
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
        expect((error as Error).message).toBe(ALLOWLIST_MESSAGE);
        return undefined;
      }
    });

    // Assert
    for (const maybeHref of hrefs) {
      if (maybeHref === undefined) {
        continue;
      }

      expect(maybeHref.startsWith(`${REPO_BLOB}/`)).toBe(true);
      expect(maybeHref.includes("javascript:")).toBe(false);
      expect(maybeHref.includes("evil.example")).toBe(false);
      expect(maybeHref.includes("google/liquidfun")).toBe(false);
    }
  });
});

describe("noticesBlobUrl", () => {
  it("builds a host-locked notices blob URL for a valid SHA", () => {
    // Arrange
    const path = "THIRD_PARTY_NOTICES.md";

    // Act
    const href = noticesBlobUrl(VALID_SHA);

    // Assert
    expect(href).toBe(`${REPO_BLOB}/${VALID_SHA}/${path}`);
  });

  it("falls back to main when the SHA is missing", () => {
    // Arrange
    const path = "THIRD_PARTY_NOTICES.md";

    // Act
    const href = noticesBlobUrl(undefined);

    // Assert
    expect(href).toBe(`${REPO_BLOB}/main/${path}`);
  });
});
