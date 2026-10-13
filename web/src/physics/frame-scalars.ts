const INVALID_FRAME_MESSAGE = "Invalid Rust/WASM frame";

/** Fails closed on any malformed or inconsistent generated frame value. */
export function invalidFrame(): never {
  throw new Error(INVALID_FRAME_MESSAGE);
}

/** Accepts a safe non-negative integer. */
export function parseNonNegativeInteger(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0) {
    return invalidFrame();
  }

  return value;
}

/** Accepts a safe integer of at least one. */
export function parsePositiveInteger(value: number): number {
  const parsed = parseNonNegativeInteger(value);
  if (parsed < 1) {
    return invalidFrame();
  }

  return parsed;
}
