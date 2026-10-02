import type { GeneratedProofSession } from "./session";

/** Direct sessions deliberately reject an invalid gravity sample without poisoning the world. */
export function applyWorkerGravitySample(
  session: Pick<GeneratedProofSession, "setGravity">,
  x: number,
  y: number,
): boolean {
  try {
    session.setGravity(x, y);
    return true;
  } catch {
    return false;
  }
}
