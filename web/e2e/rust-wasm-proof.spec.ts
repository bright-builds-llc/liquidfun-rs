import { createHash } from "node:crypto";
import {
  mkdir,
  readFile,
  rename,
  stat,
  writeFile,
} from "node:fs/promises";
import { basename, relative, resolve } from "node:path";

import { expect, test, type Locator } from "@playwright/test";

test.skip(
  process.env.PHASE16_CLOSURE_ATTEMPT_DIR === undefined,
  "Phase 16 forensic smoke is opt-in and requires PHASE16_CLOSURE_ATTEMPT_DIR",
);

const LOADING_STATUS = "Loading Rust/WASM session…";
const RUNNING_STATUS = "Running Rust/WASM session";
const DISPOSED_STATUS = "Rust/WASM session disposed";
const RUNTIME_IDENTITY = "Rust engine · WebAssembly";
const CANVAS_NAME =
  "Live Rust physics scene: particles moving in a basin around a rigid body.";
const PNG_SIGNATURE = "89504e470d0a1a0a";
const PLAYWRIGHT_VERSION = "1.63.0";

type Provenance = {
  readonly schemaVersion: number;
  readonly attemptIdentity: string;
  readonly source: {
    readonly revision: string;
    readonly workingTreeSha256: string;
    readonly status: string;
  };
  readonly tools: {
    readonly playwright: {
      readonly packageVersion: string;
      readonly chromiumRevision: string;
      readonly chromiumVersion: string;
    };
  };
};

type Observation = {
  readonly stepIndex: number;
  readonly movedFrameCount: number;
  readonly canvasPixelSha256: string;
};

type PngArtifact = {
  readonly path: string;
  readonly sha256: string;
  readonly byteLength: number;
  readonly dimensions: {
    readonly width: number;
    readonly height: number;
  };
};

function requireAttemptDirectory(): string {
  const maybeAttemptDirectory =
    process.env.PHASE16_CLOSURE_ATTEMPT_DIR;
  if (maybeAttemptDirectory === undefined) {
    throw new Error("PHASE16_CLOSURE_ATTEMPT_DIR is required");
  }
  return resolve(maybeAttemptDirectory);
}

async function numericAttribute(
  locator: Locator,
  name: string,
): Promise<number> {
  const maybeValue = await locator.getAttribute(name);
  if (maybeValue === null || !/^(0|[1-9]\d*)$/.test(maybeValue)) {
    throw new Error(`invalid numeric ${name}: ${String(maybeValue)}`);
  }
  return Number(maybeValue);
}

async function canvasPixelSha256(canvas: Locator): Promise<string> {
  return canvas.evaluate(async (element) => {
    if (!(element instanceof HTMLCanvasElement)) {
      throw new Error("proof viewport is not a Canvas");
    }
    const context = element.getContext("2d");
    if (context === null) {
      throw new Error("Canvas 2D is unavailable");
    }
    const pixels = context.getImageData(
      0,
      0,
      element.width,
      element.height,
    ).data;
    const digest = await crypto.subtle.digest("SHA-256", pixels);
    return Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
  });
}

async function canvasSizing(canvas: Locator): Promise<{
  readonly css: { readonly width: number; readonly height: number };
  readonly backing: { readonly width: number; readonly height: number };
}> {
  return canvas.evaluate((element) => {
    if (!(element instanceof HTMLCanvasElement)) {
      throw new Error("proof viewport is not a Canvas");
    }
    const bounds = element.getBoundingClientRect();
    return {
      css: { width: bounds.width, height: bounds.height },
      backing: { width: element.width, height: element.height },
    };
  });
}

function pngDimensions(bytes: Buffer): {
  readonly width: number;
  readonly height: number;
} {
  if (
    bytes.length < 24 ||
    bytes.subarray(0, 8).toString("hex") !== PNG_SIGNATURE
  ) {
    throw new Error("invalid PNG artifact");
  }
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20),
  };
}

async function pngArtifact(
  repoRoot: string,
  path: string,
): Promise<PngArtifact> {
  const bytes = await readFile(path);
  const fileStatus = await stat(path);
  if (!fileStatus.isFile() || bytes.length === 0) {
    throw new Error(`empty PNG artifact: ${path}`);
  }
  return {
    path: relative(repoRoot, path),
    sha256: createHash("sha256").update(bytes).digest("hex"),
    byteLength: bytes.length,
    dimensions: pngDimensions(bytes),
  };
}

async function atomicWriteJson(
  path: string,
  value: unknown,
): Promise<void> {
  const temporaryPath = `${path}.tmp-${process.pid}`;
  await writeFile(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, {
    flag: "wx",
  });
  await rename(temporaryPath, path);
}

async function waitForTwoAnimationFrames(): Promise<void> {
  await new Promise<void>((resolvePromise) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => resolvePromise());
    });
  });
}

test("runs Rust WASM, visibly moves, and freezes after disposal", async ({
  browser,
  page,
}, testInfo) => {
  const repoRoot = resolve(import.meta.dirname, "../..");
  const attemptDirectory = requireAttemptDirectory();
  const browserDirectory = resolve(attemptDirectory, "browser");
  await mkdir(browserDirectory);

  const provenance = JSON.parse(
    await readFile(
      resolve(attemptDirectory, "provenance.json"),
      "utf8",
    ),
  ) as Provenance;
  expect(provenance.schemaVersion).toBe(1);
  expect(provenance.attemptIdentity).toBe(basename(attemptDirectory));
  expect(provenance.tools.playwright.packageVersion).toBe(
    PLAYWRIGHT_VERSION,
  );

  let releaseWasmRequest = (): void => {
    throw new Error("WASM request was not observed");
  };
  const wasmRequestGate = new Promise<void>((resolveRequest) => {
    releaseWasmRequest = resolveRequest;
  });
  await page.route("**/*.wasm", async (route) => {
    await wasmRequestGate;
    await route.continue();
  });

  await page.goto("/", { waitUntil: "domcontentloaded" });
  const status = page.getByRole("status");
  await expect(status).toHaveText(LOADING_STATUS);
  releaseWasmRequest();

  await expect(status).toHaveText(RUNNING_STATUS);
  const main = page.locator("main");
  await expect(main).toHaveAttribute("data-wasm-initialized", "true");
  await expect(page.getByText("Particles: 1920", { exact: true })).toBeVisible();
  await expect(page.getByText("Rigid shapes: 4", { exact: true })).toBeVisible();
  await expect(page.getByText(RUNTIME_IDENTITY, { exact: true })).toBeVisible();

  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);
  await expect
    .poll(() => numericAttribute(main, "data-moved-frame-count"))
    .toBeGreaterThan(0);

  const canvas = page.getByRole("img", { name: CANVAS_NAME });
  await expect(canvas).toBeVisible();
  const beforeResize = await canvasSizing(canvas);
  await page.setViewportSize({ width: 900, height: 700 });
  await expect
    .poll(async () => (await canvasSizing(canvas)).backing.width)
    .not.toBe(beforeResize.backing.width);
  const resizedCanvas = await canvasSizing(canvas);
  const devicePixelRatio = await page.evaluate(
    () => window.devicePixelRatio,
  );
  expect(resizedCanvas.backing.width).toBe(
    Math.round(
      resizedCanvas.css.width * Math.min(devicePixelRatio, 2),
    ),
  );
  expect(resizedCanvas.backing.height).toBe(
    Math.round(
      resizedCanvas.css.height * Math.min(devicePixelRatio, 2),
    ),
  );

  const initialObservation: Observation = {
    stepIndex: await numericAttribute(main, "data-step-index"),
    movedFrameCount: await numericAttribute(
      main,
      "data-moved-frame-count",
    ),
    canvasPixelSha256: await canvasPixelSha256(canvas),
  };
  const initialPath = resolve(browserDirectory, "canvas-initial.png");
  const initialBytes = await canvas.screenshot({ path: initialPath });
  await testInfo.attach("canvas-initial.png", {
    path: initialPath,
    contentType: "image/png",
  });

  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(initialObservation.stepIndex);
  await expect
    .poll(() => numericAttribute(main, "data-moved-frame-count"))
    .toBeGreaterThan(initialObservation.movedFrameCount);

  const movingObservation: Observation = {
    stepIndex: await numericAttribute(main, "data-step-index"),
    movedFrameCount: await numericAttribute(
      main,
      "data-moved-frame-count",
    ),
    canvasPixelSha256: await canvasPixelSha256(canvas),
  };
  const movingPath = resolve(browserDirectory, "canvas-moving.png");
  const movingBytes = await canvas.screenshot({ path: movingPath });
  await testInfo.attach("canvas-moving.png", {
    path: movingPath,
    contentType: "image/png",
  });
  expect(movingBytes.equals(initialBytes)).toBe(false);
  expect(movingObservation.canvasPixelSha256).not.toBe(
    initialObservation.canvasPixelSha256,
  );

  const disposeButton = page.getByRole("button", {
    name: "Dispose session",
  });
  await disposeButton.click();
  await expect(status).toHaveText(DISPOSED_STATUS);
  await expect(disposeButton).toBeDisabled();

  const disposedObservation: Observation = {
    stepIndex: await numericAttribute(main, "data-step-index"),
    movedFrameCount: await numericAttribute(
      main,
      "data-moved-frame-count",
    ),
    canvasPixelSha256: await canvasPixelSha256(canvas),
  };
  const disposedPath = resolve(browserDirectory, "canvas-disposed.png");
  const disposedBytes = await canvas.screenshot({ path: disposedPath });
  await testInfo.attach("canvas-disposed.png", {
    path: disposedPath,
    contentType: "image/png",
  });

  await page.evaluate(waitForTwoAnimationFrames);
  expect(await numericAttribute(main, "data-step-index")).toBe(
    disposedObservation.stepIndex,
  );
  expect(await numericAttribute(main, "data-moved-frame-count")).toBe(
    disposedObservation.movedFrameCount,
  );
  expect(await canvasPixelSha256(canvas)).toBe(
    disposedObservation.canvasPixelSha256,
  );
  const recapturedDisposedBytes = await canvas.screenshot();
  expect(recapturedDisposedBytes.equals(disposedBytes)).toBe(true);

  const finalCanvasSizing = await canvasSizing(canvas);
  const artifacts = {
    initial: await pngArtifact(repoRoot, initialPath),
    moving: await pngArtifact(repoRoot, movingPath),
    disposed: await pngArtifact(repoRoot, disposedPath),
  };
  expect(artifacts.initial.dimensions).toEqual(
    artifacts.moving.dimensions,
  );
  expect(artifacts.moving.dimensions).toEqual(
    artifacts.disposed.dimensions,
  );

  const proofPath = resolve(browserDirectory, "browser-proof.json");
  const proof = {
    schemaVersion: 1,
    attemptIdentity: basename(attemptDirectory),
    source: provenance.source,
    browser: {
      package: "@playwright/test",
      packageVersion: PLAYWRIGHT_VERSION,
      chromiumRevision:
        provenance.tools.playwright.chromiumRevision,
      expectedChromiumVersion:
        provenance.tools.playwright.chromiumVersion,
      runtimeChromiumVersion: browser.version(),
    },
    pageUrl: page.url(),
    canvas: finalCanvasSizing,
    observations: {
      initial: initialObservation,
      moving: movingObservation,
      disposed: disposedObservation,
    },
    counts: {
      particles: 1920,
      rigidShapes: 4,
    },
    assertions: {
      loadingObserved: true,
      wasmInitialized: true,
      rustFrameAdvanced: true,
      canvasPixelsChanged: true,
      resizeRedrewLastFrame: true,
      disposalStoppedFrames: true,
      disposalPreservedCanvas: true,
    },
    artifacts,
  };
  await atomicWriteJson(proofPath, proof);

  const writtenProof = JSON.parse(
    await readFile(proofPath, "utf8"),
  ) as typeof proof;
  expect(writtenProof).toEqual(proof);
  for (const artifact of Object.values(writtenProof.artifacts)) {
    const diskArtifact = await pngArtifact(
      repoRoot,
      resolve(repoRoot, artifact.path),
    );
    expect(diskArtifact).toEqual(artifact);
    expect(artifact.sha256).toMatch(/^[0-9a-f]{64}$/);
  }
  await testInfo.attach("browser-proof.json", {
    path: proofPath,
    contentType: "application/json",
  });
});
