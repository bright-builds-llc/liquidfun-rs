//! Deterministic level fingerprint that keys saved placements (Phase 41 D-17).
//!
//! FNV-1a 64 is a change detector, not a security hash. The input is the
//! spec's derived `Debug` text, which covers every field automatically; Rust
//! float formatting is integer-only, so native and wasm agree (verified by
//! `just puzzle-replay-wasm`). Derived `Debug` output may change across Rust
//! versions; the toolchain is pinned and the pin test catches any change.

use super::spec::LevelSpec;
use super::tools::GRID_METERS;
use super::valve::STEPS_PER_OPEN_TICK;

/// Bump when the encoding below changes; every saved placement is then discarded.
const ENCODING: &str = "level-hash-v1";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// FNV-1a 64 over raw bytes.
pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

/// 16 lowercase hex digits over the spec's `Debug` text plus the grid and
/// valve-tick units. Saved poses are grid cells and half-second ticks whose
/// units live outside `LevelSpec`, so both are part of the input.
pub(crate) fn level_hash_hex(spec: &LevelSpec) -> String {
    let text = format!("{ENCODING}|{spec:?}|grid={GRID_METERS:?}|tick={STEPS_PER_OPEN_TICK}");
    format!("{:016x}", fnv1a64(text.as_bytes()))
}

#[cfg(test)]
mod tests;
