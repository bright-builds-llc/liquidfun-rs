/**
 * Level fingerprint pins (Phase 41 D-17). The same file is checked by the
 * native Rust `puzzle::level_hash` test and by the wasm replay, which proves
 * that both builds produce identical level hashes.
 */

/** Pin file shared by the Rust `level_hash` test and the wasm replay (Phase 41 D-17). */
export const LEVEL_HASH_FILE = "crates/liquidfun-wasm/src/puzzle/level-hashes.txt";

const HEX16 = /^[0-9a-f]{16}$/;

/** Parses `<level-id> <16 hex>` lines; blank and `#` lines are skipped. Throws on any malformed line. */
export function parseLevelHashes(text: string): ReadonlyMap<string, string> {
  const pins = new Map<string, string>();
  const lines = text.split("\n");
  for (const [index, rawLine] of lines.entries()) {
    const line = rawLine.trim();
    if (line.length === 0 || line.startsWith("#")) {
      continue;
    }
    const fields = line.split(" ");
    const [id, hex] = fields;
    if (
      fields.length !== 2 ||
      id === undefined ||
      id.length === 0 ||
      hex === undefined ||
      !HEX16.test(hex)
    ) {
      throw new Error(`Invalid level hash line ${index + 1}`);
    }
    if (pins.has(id)) {
      throw new Error(`Duplicate level hash: ${id}`);
    }
    pins.set(id, hex);
  }
  return pins;
}
