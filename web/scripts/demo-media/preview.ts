import { spawn } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";

const READY_TIMEOUT_MILLISECONDS = 30_000;

export type PreviewProcess = ReturnType<typeof spawn>;

export type WaitForOwnedPreviewReadinessDependencies = {
  readonly readySignal: Promise<void>;
  readonly exitSignal: Promise<number | null>;
  readonly getExitCode: () => number | null;
  readonly diagnostics: () => string;
  readonly expectedIndexHtml: string;
  readonly fetchIndexHtml: () => Promise<string>;
  readonly readyTimeoutMilliseconds: number;
};

export type TerminatePreviewProcessDependencies = {
  readonly getExitCode: () => number | null;
  readonly kill: (signal: NodeJS.Signals) => void;
  readonly waitForExit: (timeoutMilliseconds: number) => Promise<boolean>;
};

export function startPreviewProcess(webDirectory: string): PreviewProcess {
  const previewProcess = spawn(
    "bun",
    ["run", "preview", "--", "--strictPort"],
    {
      cwd: webDirectory,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  previewProcess.stdout.setEncoding("utf8");
  previewProcess.stderr.setEncoding("utf8");
  return previewProcess;
}

export async function waitForPreview(
  previewProcess: PreviewProcess,
  url: string,
  expectedIndexHtml: string,
): Promise<void> {
  const monitor = createPreviewProcessMonitor(previewProcess);
  await waitForOwnedPreviewReadiness({
    readySignal: monitor.readySignal,
    exitSignal: monitor.exitSignal,
    getExitCode: () => previewProcess.exitCode,
    diagnostics: monitor.diagnostics,
    expectedIndexHtml,
    fetchIndexHtml: async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`preview responded with HTTP ${response.status}`);
      }
      return await response.text();
    },
    readyTimeoutMilliseconds: READY_TIMEOUT_MILLISECONDS,
  });
}

export async function stopPreviewProcess(
  previewProcess: PreviewProcess,
): Promise<void> {
  await terminatePreviewProcess({
    getExitCode: () => previewProcess.exitCode,
    kill: (signal) => {
      previewProcess.kill(signal);
    },
    waitForExit: async (timeoutMilliseconds) =>
      await waitForExit(previewProcess, timeoutMilliseconds),
  });
}

export function hasOwnedPreviewReadySignal(output: string): boolean {
  return /Local:\s+http:\/\/127\.0\.0\.1:4173(?:\/|\/liquidfun-rs\/)/.test(output);
}

export async function waitForOwnedPreviewReadiness(
  dependencies: WaitForOwnedPreviewReadinessDependencies,
): Promise<void> {
  await Promise.race([
    dependencies.readySignal,
    dependencies.exitSignal.then((exitCode) => {
      throw new Error(
        `preview process exited early (${exitCode ?? "unknown"}): ${dependencies.diagnostics()}`,
      );
    }),
    delay(dependencies.readyTimeoutMilliseconds).then(() => {
      throw new Error(`preview did not become ready: ${dependencies.diagnostics()}`);
    }),
  ]);

  const exitCodeBeforeFetch = dependencies.getExitCode();
  if (exitCodeBeforeFetch !== null) {
    throw new Error(
      `preview process exited early (${exitCodeBeforeFetch}): ${dependencies.diagnostics()}`,
    );
  }

  const servedIndexHtml = await dependencies.fetchIndexHtml();
  if (servedIndexHtml !== dependencies.expectedIndexHtml) {
    throw new Error("preview index html does not match current web/dist/index.html");
  }

  const exitCodeAfterFetch = dependencies.getExitCode();
  if (exitCodeAfterFetch !== null) {
    throw new Error(
      `preview process exited early (${exitCodeAfterFetch}): ${dependencies.diagnostics()}`,
    );
  }
}

export async function terminatePreviewProcess(
  dependencies: TerminatePreviewProcessDependencies,
): Promise<void> {
  if (dependencies.getExitCode() !== null) {
    return;
  }

  dependencies.kill("SIGTERM");
  const exitedAfterTerminate = await dependencies.waitForExit(5_000);
  if (exitedAfterTerminate) {
    return;
  }

  dependencies.kill("SIGKILL");
  const exitedAfterKill = await dependencies.waitForExit(5_000);
  if (!exitedAfterKill) {
    throw new Error("Preview process did not exit after SIGKILL");
  }
}

function createPreviewProcessMonitor(
  processToWatch: PreviewProcess,
): {
  readonly readySignal: Promise<void>;
  readonly exitSignal: Promise<number | null>;
  readonly diagnostics: () => string;
} {
  let stdout = "";
  let stderr = "";
  let resolveReadySignal!: () => void;
  let readyResolved = false;
  const readySignal = new Promise<void>((resolvePromise) => {
    resolveReadySignal = resolvePromise;
  });
  const exitSignal = new Promise<number | null>((resolvePromise) => {
    if (processToWatch.exitCode !== null) {
      resolvePromise(processToWatch.exitCode);
      return;
    }

    processToWatch.once("exit", (exitCode) => {
      resolvePromise(exitCode);
    });
  });

  const { stdout: stdoutStream, stderr: stderrStream } = processToWatch;
  if (stdoutStream === null || stderrStream === null) {
    throw new Error("Preview process streams are unavailable");
  }

  const maybeResolveReady = (): void => {
    if (readyResolved) {
      return;
    }
    const output = `${stdout}\n${stderr}`;
    if (!hasOwnedPreviewReadySignal(output)) {
      return;
    }

    readyResolved = true;
    resolveReadySignal();
  };
  stdoutStream.on("data", (chunk: string) => {
    stdout += chunk;
    maybeResolveReady();
  });
  stderrStream.on("data", (chunk: string) => {
    stderr += chunk;
    maybeResolveReady();
  });

  return {
    readySignal,
    exitSignal,
    diagnostics: () => {
      const combined = `${stdout}\n${stderr}`.trim();
      return combined.length === 0 ? "no preview output" : combined;
    },
  };
}

async function waitForExit(
  processToWatch: PreviewProcess,
  timeoutMilliseconds: number,
): Promise<boolean> {
  const maybeExitCode = processToWatch.exitCode;
  if (maybeExitCode !== null) {
    return true;
  }

  return await new Promise<boolean>((resolvePromise) => {
    const timeout = setTimeout(() => {
      cleanup();
      resolvePromise(false);
    }, timeoutMilliseconds);

    const handleExit = (): void => {
      cleanup();
      resolvePromise(true);
    };

    const cleanup = (): void => {
      clearTimeout(timeout);
      processToWatch.off("exit", handleExit);
    };

    processToWatch.once("exit", handleExit);
  });
}
