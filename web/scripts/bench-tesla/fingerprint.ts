import type { RenderFrame } from "../../src/physics/frame";
import type { SemanticCheckpoint } from "./contracts";
import { canonicalJson, semanticState } from "./model";

export async function semanticSha256(value: unknown): Promise<string> {
  const input = new TextEncoder().encode(canonicalJson(value));
  const digest = await globalThis.crypto.subtle.digest("SHA-256", input);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** Hashes semantic fields while the frame owns copied lanes, outside timed spans. */
export async function fingerprint(frame: RenderFrame): Promise<SemanticCheckpoint> {
  const state = semanticState(frame);
  const [sha256, positionsSha256, colorsSha256, radiiSha256, geometrySha256] = await Promise.all([
    semanticSha256(state),
    semanticSha256({ particlePositions: state.particlePositions }),
    semanticSha256({ particleColors: state.particleColors }),
    semanticSha256({ particleRadii: state.particleRadii }),
    semanticSha256({ rigidSegments: state.rigidSegments, rigidCircles: state.rigidCircles, circleLabels: state.circleLabels }),
  ]);
  return { stepIndex: frame.stepIndex, particleCount: frame.particleCount,
    bodyContactCount: frame.bodyContactCount, stuckCandidateCount: frame.stuckCandidateCount, maxSpeed: frame.maxSpeed,
    rigidShapeCount: frame.rigidShapeCount, sha256, positionsSha256, colorsSha256,
    radiiSha256, geometrySha256 };
}
