import { describe, expect, it } from "vitest";

import { readBuildInfo } from "../src/build-info";

describe("readBuildInfo", () => {
  it("shows Unavailable and no URLs when env is empty", () => {
    // Arrange
    const env = {};

    // Act
    const info = readBuildInfo(env);

    // Assert
    expect(info.version).toBe("Unavailable");
    expect(info.commitLabel).toBe("Unavailable");
    expect(info.buildLabel).toBe("Unavailable");
    expect(info.maybeCommitUrl).toBeUndefined();
    expect(info.maybeBuildUrl).toBeUndefined();
  });

  it("uses VITE_APP_VERSION when present", () => {
    // Arrange
    const env = { VITE_APP_VERSION: "0.0.0" };

    // Act
    const info = readBuildInfo(env);

    // Assert
    expect(info.version).toBe("0.0.0");
  });

  it("shortens a 40-char hex SHA and builds the commit URL", () => {
    // Arrange
    const sha = "0123456789abcdef0123456789abcdef01234567";

    // Act
    const info = readBuildInfo({ VITE_GIT_SHA: sha });

    // Assert
    expect(info.commitLabel.length).toBeGreaterThanOrEqual(7);
    expect(info.commitLabel.length).toBeLessThanOrEqual(12);
    expect(sha.startsWith(info.commitLabel)).toBe(true);
    expect(info.maybeCommitUrl).toBe(
      `https://github.com/bright-builds-llc/liquidfun-rs/commit/${sha}`,
    );
  });

  it("rejects a non-hex or short SHA", () => {
    // Arrange
    const invalidShas = [
      "not-a-sha",
      "01234567",
      "0123456789ABCDEF0123456789abcdef01234567",
    ];

    // Act
    const results = invalidShas.map((sha) => readBuildInfo({ VITE_GIT_SHA: sha }));

    // Assert
    for (const info of results) {
      expect(info.commitLabel).toBe("Unavailable");
      expect(info.maybeCommitUrl).toBeUndefined();
    }
  });

  it("accepts only this repo Actions run URLs", () => {
    // Arrange
    const accepted =
      "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/123456789";
    const rejected = [
      "javascript:alert(1)",
      "https://evil.example/actions/runs/1",
      "https://github.com/other/repo/actions/runs/1",
      "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/abc",
    ];

    // Act
    const acceptedInfo = readBuildInfo({ VITE_BUILD_URL: accepted });
    const rejectedInfos = rejected.map((url) =>
      readBuildInfo({ VITE_BUILD_URL: url }),
    );

    // Assert
    expect(acceptedInfo.maybeBuildUrl).toBe(accepted);
    for (const info of rejectedInfos) {
      expect(info.maybeBuildUrl).toBeUndefined();
    }
  });
});
