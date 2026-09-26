import { describe, expect, it } from "vitest";

import { formatLocalBuiltAtLabel, readBuildInfo } from "../src/build-info";

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
    expect(info.builtAtLabel).toBe("Unavailable");
    expect(info.maybeCommitUrl).toBeUndefined();
    expect(info.maybeBuildUrl).toBeUndefined();
    expect(info.maybeBuiltAtIso).toBeUndefined();
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

  it("formats a UTC ISO build timestamp", () => {
    // Arrange
    const withMillis = "2026-09-22T15:27:03.123Z";
    const withoutMillis = "2024-02-29T00:00:00Z";

    // Act
    const withMillisInfo = readBuildInfo({ VITE_BUILT_AT: `  ${withMillis}  ` });
    const withoutMillisInfo = readBuildInfo({ VITE_BUILT_AT: withoutMillis });

    // Assert
    expect(withMillisInfo.builtAtLabel).toBe("2026-09-22 15:27:03 UTC");
    expect(withMillisInfo.maybeBuiltAtIso).toBe(withMillis);
    expect(withoutMillisInfo.builtAtLabel).toBe("2024-02-29 00:00:00 UTC");
    expect(withoutMillisInfo.maybeBuiltAtIso).toBe(withoutMillis);
  });

  it("rejects timestamps that are not real UTC instants", () => {
    // Arrange
    const rejected = [
      "not-a-timestamp",
      "2026-09-22 15:27:03 UTC",
      "2026-09-22T15:27:03+00:00",
      "2025-02-29T00:00:00.000Z",
      "2026-13-01T00:00:00.000Z",
      "2026-09-22T24:00:00.000Z",
    ];

    // Act
    const results = rejected.map((builtAt) =>
      readBuildInfo({ VITE_BUILT_AT: builtAt }),
    );

    // Assert
    for (const info of results) {
      expect(info.builtAtLabel).toBe("Unavailable");
      expect(info.maybeBuiltAtIso).toBeUndefined();
    }
  });

  it("formats a build instant in the requested time zone", () => {
    // Arrange
    const iso = "2026-09-22T15:27:03.123Z";

    // Act
    const newYork = formatLocalBuiltAtLabel(iso, "America/New_York");
    const kolkata = formatLocalBuiltAtLabel(iso, "Asia/Kolkata");
    const utc = formatLocalBuiltAtLabel(iso, "UTC");

    // Assert
    expect(newYork).toBe("2026-09-22 11:27:03 EDT");
    expect(kolkata).toBe("2026-09-22 20:57:03 GMT+5:30");
    expect(utc).toBe("2026-09-22 15:27:03 UTC");
  });

  it("shifts the calendar day when local time is on the previous date", () => {
    // Arrange
    const iso = "2024-02-29T00:00:00Z";

    // Act
    const label = formatLocalBuiltAtLabel(iso, "America/Los_Angeles");

    // Assert
    expect(label).toBe("2024-02-28 16:00:00 PST");
  });

  it("defaults to the runtime local time zone", () => {
    // Arrange
    const iso = "2026-09-22T15:27:03.123Z";
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;

    // Act
    const localLabel = formatLocalBuiltAtLabel(iso);
    const explicitLabel = formatLocalBuiltAtLabel(iso, timeZone);

    // Assert
    expect(localLabel).toBe(explicitLabel);
  });

  it("returns Unavailable when the instant or time zone cannot be formatted", () => {
    // Arrange
    const iso = "2026-09-22T15:27:03.123Z";

    // Act
    const invalidInstant = formatLocalBuiltAtLabel("not-a-timestamp", "UTC");
    const invalidZone = formatLocalBuiltAtLabel(iso, "Not/AZone");

    // Assert
    expect(invalidInstant).toBe("Unavailable");
    expect(invalidZone).toBe("Unavailable");
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
