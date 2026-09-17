import { createHash } from "node:crypto";
import { readFile, realpath } from "node:fs/promises";
import {
  basename,
  isAbsolute,
  relative,
  resolve,
  sep,
} from "node:path";

import {
  parseSourceIdentity,
  type SourceIdentity,
} from "./source-identity";

type Dimensions = {
  readonly width: number;
  readonly height: number;
};

type PngArtifact = {
  readonly path: string;
  readonly sha256: string;
  readonly byteLength: number;
  readonly dimensions: Dimensions;
};

type Observation = {
  readonly stepIndex: number;
  readonly movedFrameCount: number;
  readonly canvasPixelSha256: string;
};

export type BrowserProof = {
  readonly schemaVersion: 1;
  readonly attemptIdentity: string;
  readonly source: SourceIdentity;
  readonly browser: {
    readonly package: string;
    readonly packageVersion: string;
    readonly chromiumRevision: string;
    readonly expectedChromiumVersion: string;
    readonly runtimeChromiumVersion: string;
  };
  readonly pageUrl: string;
  readonly canvas: {
    readonly css: Dimensions;
    readonly backing: Dimensions;
  };
  readonly observations: {
    readonly initial: Observation;
    readonly moving: Observation;
    readonly disposed: Observation;
  };
  readonly counts: {
    readonly particles: number;
    readonly rigidShapes: number;
  };
  readonly assertions: Readonly<Record<AssertionName, true>>;
  readonly artifacts: Readonly<Record<ArtifactName, PngArtifact>>;
};

type AssertionName = (typeof assertionNames)[number];
type ArtifactName = (typeof artifactNames)[number];
type AttachmentName = (typeof attachmentNames)[number];

type Attachment = {
  readonly name: AttachmentName;
  readonly contentType: string;
  readonly path: string;
};

const assertionNames = [
  "loadingObserved",
  "wasmInitialized",
  "rustFrameAdvanced",
  "canvasPixelsChanged",
  "resizeRedrewLastFrame",
  "disposalStoppedFrames",
  "disposalPreservedCanvas",
] as const;
const artifactNames = ["initial", "moving", "disposed"] as const;
const attachmentNames = [
  "canvas-initial.png",
  "canvas-moving.png",
  "canvas-disposed.png",
  "browser-proof.json",
] as const;
const artifactByAttachment = {
  "canvas-initial.png": "initial",
  "canvas-moving.png": "moving",
  "canvas-disposed.png": "disposed",
} as const satisfies Record<
  Exclude<AttachmentName, "browser-proof.json">,
  ArtifactName
>;
const PNG_SIGNATURE = "89504e470d0a1a0a";
const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const MAX_PROOF_BYTES = 1_000_000;
const MAX_REPORT_BYTES = 10_000_000;
const MAX_REPORT_NODES = 100_000;
const MAX_STRING_LENGTH = 10_000;
const MAX_COUNT = 1_000_000;
const MAX_DIMENSION = 16_384;

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function parseExactRecord(
  value: unknown,
  expectedKeys: readonly string[],
  label: string,
): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actualKeys = Object.keys(value).sort();
  const sortedExpectedKeys = [...expectedKeys].sort();
  if (
    actualKeys.length !== sortedExpectedKeys.length ||
    actualKeys.some((key, index) => key !== sortedExpectedKeys[index])
  ) {
    throw new Error(`${label} has unknown or missing entries`);
  }
  return value;
}

function parseString(
  value: unknown,
  label: string,
  pattern?: RegExp,
): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > MAX_STRING_LENGTH ||
    (pattern !== undefined && !pattern.test(value))
  ) {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function parseInteger(
  value: unknown,
  label: string,
  minimum: number,
  maximum: number,
): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < minimum ||
    value > maximum
  ) {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function parseDimensions(value: unknown, label: string): Dimensions {
  const record = parseExactRecord(value, ["width", "height"], label);
  return {
    width: parseInteger(
      record.width,
      `${label}.width`,
      1,
      MAX_DIMENSION,
    ),
    height: parseInteger(
      record.height,
      `${label}.height`,
      1,
      MAX_DIMENSION,
    ),
  };
}

function parseObservation(value: unknown, label: string): Observation {
  const record = parseExactRecord(
    value,
    ["stepIndex", "movedFrameCount", "canvasPixelSha256"],
    label,
  );
  return {
    stepIndex: parseInteger(
      record.stepIndex,
      `${label}.stepIndex`,
      0,
      MAX_COUNT,
    ),
    movedFrameCount: parseInteger(
      record.movedFrameCount,
      `${label}.movedFrameCount`,
      0,
      MAX_COUNT,
    ),
    canvasPixelSha256: parseString(
      record.canvasPixelSha256,
      `${label}.canvasPixelSha256`,
      SHA256_PATTERN,
    ),
  };
}

function parseArtifact(value: unknown, label: string): PngArtifact {
  const record = parseExactRecord(
    value,
    ["path", "sha256", "byteLength", "dimensions"],
    label,
  );
  return {
    path: parseString(record.path, `${label}.path`),
    sha256: parseString(record.sha256, `${label}.sha256`, SHA256_PATTERN),
    byteLength: parseInteger(
      record.byteLength,
      `${label}.byteLength`,
      1,
      MAX_PROOF_BYTES,
    ),
    dimensions: parseDimensions(record.dimensions, `${label}.dimensions`),
  };
}

function parseBrowserProof(value: unknown): BrowserProof {
  const proof = parseExactRecord(
    value,
    [
      "schemaVersion",
      "attemptIdentity",
      "source",
      "browser",
      "pageUrl",
      "canvas",
      "observations",
      "counts",
      "assertions",
      "artifacts",
    ],
    "browser proof",
  );
  if (proof.schemaVersion !== 1) {
    throw new Error("browser proof schemaVersion must equal 1");
  }

  const browser = parseExactRecord(
    proof.browser,
    [
      "package",
      "packageVersion",
      "chromiumRevision",
      "expectedChromiumVersion",
      "runtimeChromiumVersion",
    ],
    "browser proof browser",
  );
  const canvas = parseExactRecord(
    proof.canvas,
    ["css", "backing"],
    "browser proof canvas",
  );
  const observations = parseExactRecord(
    proof.observations,
    ["initial", "moving", "disposed"],
    "browser proof observations",
  );
  const counts = parseExactRecord(
    proof.counts,
    ["particles", "rigidShapes"],
    "browser proof counts",
  );
  const assertions = parseExactRecord(
    proof.assertions,
    assertionNames,
    "browser proof assertions",
  );
  for (const name of assertionNames) {
    if (assertions[name] !== true) {
      throw new Error(`browser proof assertion failed: ${name}`);
    }
  }
  const artifacts = parseExactRecord(
    proof.artifacts,
    artifactNames,
    "browser proof artifacts",
  );

  return {
    schemaVersion: 1,
    attemptIdentity: parseString(
      proof.attemptIdentity,
      "browser proof attemptIdentity",
      /^closure-attempt-\d+$/,
    ),
    source: parseSourceIdentity(proof.source, "browser proof source"),
    browser: {
      package: parseString(browser.package, "browser proof browser.package"),
      packageVersion: parseString(
        browser.packageVersion,
        "browser proof browser.packageVersion",
      ),
      chromiumRevision: parseString(
        browser.chromiumRevision,
        "browser proof browser.chromiumRevision",
      ),
      expectedChromiumVersion: parseString(
        browser.expectedChromiumVersion,
        "browser proof browser.expectedChromiumVersion",
      ),
      runtimeChromiumVersion: parseString(
        browser.runtimeChromiumVersion,
        "browser proof browser.runtimeChromiumVersion",
      ),
    },
    pageUrl: parseString(proof.pageUrl, "browser proof pageUrl"),
    canvas: {
      css: parseDimensions(canvas.css, "browser proof canvas.css"),
      backing: parseDimensions(
        canvas.backing,
        "browser proof canvas.backing",
      ),
    },
    observations: {
      initial: parseObservation(
        observations.initial,
        "browser proof observations.initial",
      ),
      moving: parseObservation(
        observations.moving,
        "browser proof observations.moving",
      ),
      disposed: parseObservation(
        observations.disposed,
        "browser proof observations.disposed",
      ),
    },
    counts: {
      particles: parseInteger(
        counts.particles,
        "browser proof counts.particles",
        1,
        MAX_COUNT,
      ),
      rigidShapes: parseInteger(
        counts.rigidShapes,
        "browser proof counts.rigidShapes",
        1,
        MAX_COUNT,
      ),
    },
    assertions: assertions as Record<AssertionName, true>,
    artifacts: {
      initial: parseArtifact(
        artifacts.initial,
        "browser proof artifacts.initial",
      ),
      moving: parseArtifact(
        artifacts.moving,
        "browser proof artifacts.moving",
      ),
      disposed: parseArtifact(
        artifacts.disposed,
        "browser proof artifacts.disposed",
      ),
    },
  };
}

async function readBoundedJson(
  path: string,
  maximumBytes: number,
  label: string,
): Promise<{ readonly bytes: Buffer; readonly value: unknown }> {
  const bytes = await readFile(path);
  if (bytes.length === 0 || bytes.length > maximumBytes) {
    throw new Error(`${label} size is invalid`);
  }
  try {
    return {
      bytes,
      value: JSON.parse(bytes.toString("utf8")) as unknown,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : "unknown error";
    throw new Error(`${label} is not valid JSON: ${message}`, {
      cause: error,
    });
  }
}

function requireConfinedPath(
  parentDirectory: string,
  path: string,
  label: string,
): void {
  const relativePath = relative(parentDirectory, path);
  if (
    relativePath === "" ||
    relativePath === ".." ||
    relativePath.startsWith(`..${sep}`) ||
    isAbsolute(relativePath)
  ) {
    throw new Error(`${label} escapes its evidence directory`);
  }
}

async function confinedExistingPath(
  parentDirectory: string,
  path: string,
  label: string,
): Promise<string> {
  requireConfinedPath(parentDirectory, path, label);
  const canonicalPath = await realpath(path);
  requireConfinedPath(parentDirectory, canonicalPath, label);
  return canonicalPath;
}

function pngDimensions(bytes: Buffer, label: string): Dimensions {
  if (
    bytes.length < 24 ||
    bytes.subarray(0, 8).toString("hex") !== PNG_SIGNATURE
  ) {
    throw new Error(`${label} is not a PNG`);
  }
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20),
  };
}

async function validateArtifacts(
  repoRoot: string,
  browserDirectory: string,
  proof: BrowserProof,
): Promise<Readonly<Record<ArtifactName, Buffer>>> {
  const bytesByArtifact = {} as Record<ArtifactName, Buffer>;
  for (const name of artifactNames) {
    const artifact = proof.artifacts[name];
    const resolvedPath = resolve(repoRoot, artifact.path);
    const confinedPath = await confinedExistingPath(
      browserDirectory,
      resolvedPath,
      `browser artifact ${name}`,
    );
    const bytes = await readFile(confinedPath);
    if (
      bytes.length !== artifact.byteLength ||
      createHash("sha256").update(bytes).digest("hex") !== artifact.sha256
    ) {
      throw new Error(`browser artifact hash mismatch: ${name}`);
    }
    const dimensions = pngDimensions(bytes, `browser artifact ${name}`);
    if (
      dimensions.width !== artifact.dimensions.width ||
      dimensions.height !== artifact.dimensions.height
    ) {
      throw new Error(`browser artifact dimensions mismatch: ${name}`);
    }
    bytesByArtifact[name] = bytes;
  }
  return bytesByArtifact;
}

function collectAttachments(value: unknown): readonly Attachment[] {
  const attachments: Attachment[] = [];
  const pending: unknown[] = [value];
  let visitedNodes = 0;
  while (pending.length > 0) {
    const current = pending.pop();
    visitedNodes += 1;
    if (visitedNodes > MAX_REPORT_NODES) {
      throw new Error("Playwright report is too complex");
    }
    if (Array.isArray(current)) {
      pending.push(...current);
      continue;
    }
    if (!isRecord(current)) {
      continue;
    }
    if (
      typeof current.name === "string" &&
      typeof current.contentType === "string" &&
      typeof current.path === "string"
    ) {
      if (!attachmentNames.includes(current.name as AttachmentName)) {
        throw new Error(`unknown Playwright attachment: ${current.name}`);
      }
      attachments.push({
        name: current.name as AttachmentName,
        contentType: current.contentType,
        path: current.path,
      });
    }
    pending.push(...Object.values(current));
  }
  return attachments;
}

async function validateAttachments(
  attemptDirectory: string,
  proofBytes: Buffer,
  artifactBytes: Readonly<Record<ArtifactName, Buffer>>,
  reportValue: unknown,
): Promise<void> {
  const attachments = collectAttachments(reportValue);
  const attachmentsByName = new Map<AttachmentName, Attachment>();
  for (const attachment of attachments) {
    if (attachmentsByName.has(attachment.name)) {
      throw new Error(`duplicate Playwright attachment: ${attachment.name}`);
    }
    attachmentsByName.set(attachment.name, attachment);
  }
  for (const name of attachmentNames) {
    const maybeAttachment = attachmentsByName.get(name);
    if (maybeAttachment === undefined) {
      throw new Error(`missing Playwright attachment: ${name}`);
    }
    const expectedContentType =
      name === "browser-proof.json" ? "application/json" : "image/png";
    if (maybeAttachment.contentType !== expectedContentType) {
      throw new Error(`Playwright attachment content type mismatch: ${name}`);
    }
    const resolvedPath = resolve(maybeAttachment.path);
    const confinedPath = await confinedExistingPath(
      attemptDirectory,
      resolvedPath,
      `Playwright attachment ${name}`,
    );
    const attachmentBytes = await readFile(confinedPath);
    const expectedBytes =
      name === "browser-proof.json"
        ? proofBytes
        : artifactBytes[artifactByAttachment[name]];
    if (!attachmentBytes.equals(expectedBytes)) {
      throw new Error(`Playwright attachment bytes mismatch: ${name}`);
    }
  }
}

export async function validateBrowserEvidence(
  repoRoot: string,
  attemptDirectory: string,
): Promise<BrowserProof> {
  const browserDirectory = resolve(attemptDirectory, "browser");
  const proofPath = resolve(browserDirectory, "browser-proof.json");
  const proofJson = await readBoundedJson(
    proofPath,
    MAX_PROOF_BYTES,
    "browser proof",
  );
  const proof = parseBrowserProof(proofJson.value);
  if (proof.attemptIdentity !== basename(attemptDirectory)) {
    throw new Error("browser proof attempt identity mismatch");
  }
  const artifactBytes = await validateArtifacts(
    repoRoot,
    browserDirectory,
    proof,
  );
  const reportJson = await readBoundedJson(
    resolve(attemptDirectory, "playwright-report.json"),
    MAX_REPORT_BYTES,
    "Playwright report",
  );
  await validateAttachments(
    attemptDirectory,
    proofJson.bytes,
    artifactBytes,
    reportJson.value,
  );
  return proof;
}
