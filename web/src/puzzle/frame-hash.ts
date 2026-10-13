/**
 * FNV-1a 64 frame hash shared with the native `scene_spot::frame_hash` test
 * module (Phase 37 D-01): each lane's raw little-endian f32 bytes, in order.
 */

/** FNV-1a 64 offset basis. */
export const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const MASK = (1n << 64n) - 1n;

/** Folds `bytes` into an FNV-1a 64 hash that starts at `seed`. */
export function fnv1a64Bytes(
  bytes: Uint8Array,
  seed: bigint = FNV_OFFSET,
): bigint {
  let hash = seed;
  for (const byte of bytes) {
    hash ^= BigInt(byte);
    hash = (hash * FNV_PRIME) & MASK;
  }
  return hash;
}

function assertLittleEndian(): void {
  if (new Uint8Array(Uint16Array.of(1).buffer)[0] !== 1) {
    throw new Error("puzzle-replay requires a little-endian host");
  }
}

/**
 * Hashes every lane's raw bytes in order. Reading the bytes directly keeps
 * `-0.0` and every other exact bit pattern, matching the Rust `to_bits` mix.
 */
export function fnv1a64Lanes(lanes: readonly Float32Array[]): bigint {
  assertLittleEndian();
  let hash = FNV_OFFSET;
  for (const lane of lanes) {
    hash = fnv1a64Bytes(
      new Uint8Array(lane.buffer, lane.byteOffset, lane.byteLength),
      hash,
    );
  }
  return hash;
}

/** Formats a hash as 16 lowercase hex digits. */
export function hex16(hash: bigint): string {
  return hash.toString(16).padStart(16, "0");
}
