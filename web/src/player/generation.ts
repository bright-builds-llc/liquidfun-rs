/** Increment the player generation so later async loads can be discarded. */
export function nextGeneration(current: number): number {
  return current + 1;
}

/** True when a later start, reset, retry, or route change outran this load. */
export function isStaleGeneration(started: number, current: number): boolean {
  return started !== current;
}
