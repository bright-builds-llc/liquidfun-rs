import { createHash } from "node:crypto";
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  unlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { relative, resolve } from "node:path";

import { afterEach, describe, expect, test } from "bun:test";

import { validateBrowserEvidence } from "./browser-evidence";

type EvidenceFixture = {
  readonly repoRoot: string;
  readonly attemptDirectory: string;
  readonly proofPath: string;
  readonly proofAttachmentPath: string;
  readonly movingArtifactPath: string;
  readonly movingAttachmentPath: string;
};

const temporaryDirectories: string[] = [];
const artifactNames = ["initial", "moving", "disposed"] as const;

function pngBytes(width = 4, height = 3): Buffer {
  const bytes = Buffer.alloc(24);
  Buffer.from("89504e470d0a1a0a", "hex").copy(bytes);
  bytes.writeUInt32BE(width, 16);
  bytes.writeUInt32BE(height, 20);
  return bytes;
}

function sha256(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

async function writeJson(path: string, value: unknown): Promise<void> {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

async function createEvidenceFixture(): Promise<EvidenceFixture> {
  const repoRoot = await realpath(
    await mkdtemp(resolve(tmpdir(), "phase16-evidence-")),
  );
  temporaryDirectories.push(repoRoot);
  const attemptDirectory = resolve(
    repoRoot,
    "target/phase16/closure-attempt-9",
  );
  const browserDirectory = resolve(attemptDirectory, "browser");
  const attachmentDirectory = resolve(
    attemptDirectory,
    "playwright-output/attachments",
  );
  await mkdir(browserDirectory, { recursive: true });
  await mkdir(attachmentDirectory, { recursive: true });

  const artifacts: Record<string, unknown> = {};
  const attachments: Array<Record<string, string>> = [];
  for (const name of artifactNames) {
    const bytes = pngBytes();
    const artifactPath = resolve(browserDirectory, `canvas-${name}.png`);
    const attachmentPath = resolve(
      attachmentDirectory,
      `canvas-${name}.png`,
    );
    await writeFile(artifactPath, bytes);
    await cp(artifactPath, attachmentPath);
    artifacts[name] = {
      path: relative(repoRoot, artifactPath),
      sha256: sha256(bytes),
      byteLength: bytes.length,
      dimensions: { width: 4, height: 3 },
    };
    attachments.push({
      name: `canvas-${name}.png`,
      contentType: "image/png",
      path: attachmentPath,
    });
  }

  const proof = {
    schemaVersion: 1,
    attemptIdentity: "closure-attempt-9",
    source: {
      revision: "0123456789abcdef0123456789abcdef01234567",
      workingTreeSha256: "a".repeat(64),
      status: "",
    },
    browser: {
      package: "@playwright/test",
      packageVersion: "1.63.0",
      chromiumRevision: "1234",
      expectedChromiumVersion: "1.2.3",
      runtimeChromiumVersion: "1.2.3",
    },
    pageUrl: "http://127.0.0.1:4173/",
    canvas: {
      css: { width: 4, height: 3 },
      backing: { width: 4, height: 3 },
    },
    observations: {
      initial: {
        stepIndex: 1,
        movedFrameCount: 1,
        canvasPixelSha256: "b".repeat(64),
      },
      moving: {
        stepIndex: 2,
        movedFrameCount: 2,
        canvasPixelSha256: "c".repeat(64),
      },
      disposed: {
        stepIndex: 2,
        movedFrameCount: 2,
        canvasPixelSha256: "c".repeat(64),
      },
    },
    counts: { particles: 192, rigidShapes: 4 },
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
  const proofPath = resolve(browserDirectory, "browser-proof.json");
  await writeJson(proofPath, proof);
  const proofAttachmentPath = resolve(
    attachmentDirectory,
    "browser-proof.json",
  );
  await cp(proofPath, proofAttachmentPath);
  attachments.push({
    name: "browser-proof.json",
    contentType: "application/json",
    path: proofAttachmentPath,
  });
  await writeJson(resolve(attemptDirectory, "playwright-report.json"), {
    suites: [{ specs: [{ tests: [{ results: [{ attachments }] }] }] }],
  });

  return {
    repoRoot,
    attemptDirectory,
    proofPath,
    proofAttachmentPath,
    movingArtifactPath: resolve(browserDirectory, "canvas-moving.png"),
    movingAttachmentPath: resolve(
      attachmentDirectory,
      "canvas-moving.png",
    ),
  };
}

async function rewriteProof(
  fixture: EvidenceFixture,
  mutate: (proof: Record<string, unknown>) => void,
): Promise<void> {
  const proof = JSON.parse(
    await readFile(fixture.proofPath, "utf8"),
  ) as Record<string, unknown>;
  mutate(proof);
  await writeJson(fixture.proofPath, proof);
  await cp(fixture.proofPath, fixture.proofAttachmentPath, {
    force: true,
  });
}

afterEach(async () => {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) =>
      rm(path, { recursive: true, force: true }),
    ),
  );
});

describe("Phase 16 browser evidence", () => {
  test("rejects an empty browser proof", async () => {
    // Arrange
    const fixture = await createEvidenceFixture();
    await writeJson(fixture.proofPath, {});

    // Act
    const validation = validateBrowserEvidence(
      fixture.repoRoot,
      fixture.attemptDirectory,
    );

    // Assert
    await expect(validation).rejects.toThrow(
      "browser proof has unknown or missing entries",
    );
  });

  test("rejects an artifact path that traverses outside browser evidence", async () => {
    // Arrange
    const fixture = await createEvidenceFixture();
    await rewriteProof(fixture, (proof) => {
      const artifacts = proof.artifacts as Record<
        string,
        Record<string, unknown>
      >;
      artifacts.initial.path =
        "target/phase16/closure-attempt-9/browser/../../escape.png";
    });

    // Act
    const validation = validateBrowserEvidence(
      fixture.repoRoot,
      fixture.attemptDirectory,
    );

    // Assert
    await expect(validation).rejects.toThrow(
      "browser artifact initial escapes its evidence directory",
    );
  });

  test("rejects a missing retained artifact", async () => {
    // Arrange
    const fixture = await createEvidenceFixture();
    await unlink(fixture.movingArtifactPath);

    // Act
    const validation = validateBrowserEvidence(
      fixture.repoRoot,
      fixture.attemptDirectory,
    );

    // Assert
    await expect(validation).rejects.toThrow();
  });

  test("rejects an attachment whose bytes differ from its artifact", async () => {
    // Arrange
    const fixture = await createEvidenceFixture();
    await writeFile(fixture.movingAttachmentPath, pngBytes(5, 3));

    // Act
    const validation = validateBrowserEvidence(
      fixture.repoRoot,
      fixture.attemptDirectory,
    );

    // Assert
    await expect(validation).rejects.toThrow(
      "Playwright attachment bytes mismatch: canvas-moving.png",
    );
  });
});
