import { describe, expect, it } from "vitest";

import {
  decideOutputDirectoryAction,
  formatCleanupFailure,
  maybeRethrowWithCleanupFailures,
  runCleanupActions,
  terminatePreviewProcess,
  waitForPreviewReadiness,
} from "../scripts/demo-media";

describe("demo media cli helpers", () => {
  it("fails readiness when the owned preview exits around a ready response", async () => {
    // Arrange
    let exitCodeChecks = 0;

    // Act / Assert
    await expect(
      waitForPreviewReadiness({
        isReady: async () => true,
        getExitCode: () => {
          exitCodeChecks += 1;
          return exitCodeChecks === 1 ? null : 1;
        },
        diagnostics: () => "EADDRINUSE",
        delayMilliseconds: 250,
        delay: async () => undefined,
        now: () => 0,
        timeoutAt: 1_000,
      }),
    ).rejects.toThrow("preview process exited early (1): EADDRINUSE");
  });

  it("chooses no-op for clean outputs and fails check mode on differences", () => {
    // Arrange
    const diagnostics = ["changed: manifest.json"];

    // Act
    const generateAction = decideOutputDirectoryAction("generate", []);
    const checkAction = decideOutputDirectoryAction("check", []);
    const replaceAction = decideOutputDirectoryAction("generate", diagnostics);
    const failAction = decideOutputDirectoryAction("check", diagnostics);

    // Assert
    expect(generateAction).toEqual({ kind: "noop" });
    expect(checkAction).toEqual({ kind: "noop" });
    expect(replaceAction).toEqual({ kind: "replace" });
    expect(failAction).toEqual({
      kind: "fail",
      message: "demo media outputs differ:\nchanged: manifest.json",
    });
  });

  it("aggregates cleanup failures with the primary error first", async () => {
    // Arrange
    const cleanupFailures = await runCleanupActions([
      {
        label: "close browser",
        run: async () => {
          throw new Error("browser refused to close");
        },
      },
      {
        label: "stop preview",
        run: async () => {
          throw new Error("preview refused to stop");
        },
      },
    ]);

    // Act / Assert
    expect(cleanupFailures.map((failure) => formatCleanupFailure(failure))).toEqual([
      "close browser: browser refused to close",
      "stop preview: preview refused to stop",
    ]);
    expect(() =>
      maybeRethrowWithCleanupFailures(
        new Error("primary failure"),
        cleanupFailures,
      ),
    ).toThrowError(AggregateError);
    try {
      maybeRethrowWithCleanupFailures(
        new Error("primary failure"),
        cleanupFailures,
      );
    } catch (error) {
      const aggregate = error as AggregateError;
      expect(aggregate.errors).toHaveLength(3);
      expect((aggregate.errors[0] as Error).message).toBe("primary failure");
      expect((aggregate.errors[1] as Error).message).toBe(
        "close browser: browser refused to close",
      );
      expect((aggregate.errors[2] as Error).message).toBe(
        "stop preview: preview refused to stop",
      );
    }
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
