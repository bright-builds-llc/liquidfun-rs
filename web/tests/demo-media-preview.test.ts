import { describe, expect, it } from "vitest";

import {
  terminatePreviewProcess,
  waitForOwnedPreviewReadiness,
} from "../scripts/demo-media/preview";

describe("demo media preview lifecycle", () => {
  it("fails when no owned ready signal arrives and the child exits", async () => {
    // Arrange
    let fetchCalls = 0;

    // Act / Assert
    await expect(
      waitForOwnedPreviewReadiness({
        readySignal: new Promise<void>(() => undefined),
        exitSignal: Promise.resolve(1),
        getExitCode: () => 1,
        diagnostics: () => "EADDRINUSE",
        expectedIndexHtml: "<html>current</html>",
        fetchIndexHtml: async () => {
          fetchCalls += 1;
          return "<html>stale</html>";
        },
        readyTimeoutMilliseconds: 1_000,
      }),
    ).rejects.toThrow("preview process exited early (1): EADDRINUSE");
    expect(fetchCalls).toBe(0);
  });

  it("accepts owned readiness only after served index matches current dist", async () => {
    // Arrange
    let fetchCalls = 0;

    // Act / Assert
    await expect(
      waitForOwnedPreviewReadiness({
        readySignal: Promise.resolve(),
        exitSignal: new Promise<number | null>(() => undefined),
        getExitCode: () => null,
        diagnostics: () => "no diagnostics",
        expectedIndexHtml: "<html>current</html>",
        fetchIndexHtml: async () => {
          fetchCalls += 1;
          return "<html>current</html>";
        },
        readyTimeoutMilliseconds: 1_000,
      }),
    ).resolves.toBeUndefined();
    expect(fetchCalls).toBe(1);
  });

  it("waits for actual exit after SIGKILL and fails if the process survives", async () => {
    // Arrange
    const killSignals: string[] = [];
    const waitResults = [false, false];

    // Act / Assert
    await expect(
      terminatePreviewProcess({
        getExitCode: () => null,
        kill: (signal) => {
          killSignals.push(signal);
        },
        waitForExit: async () => waitResults.shift() ?? false,
      }),
    ).rejects.toThrow("Preview process did not exit after SIGKILL");
    expect(killSignals).toEqual(["SIGTERM", "SIGKILL"]);
  });
});
