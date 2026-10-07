//! Bit-exact end-state fingerprint for the scene survey (Phase 35 D-07).
//!
//! FNV-1a 64 is used instead of the standard library default hasher because its
//! algorithm is unspecified and may change between Rust releases, which would
//! make fingerprints from different toolchains incomparable.

use liquidfun::{ParticleSystemId, World, WorldObservationLimits};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Incremental 64-bit FNV-1a hasher over little-endian bytes.
pub(super) struct Fnv1a(u64);

impl Fnv1a {
    /// Starts at the FNV-1a 64 offset basis.
    pub(super) const fn new() -> Self {
        Self(FNV_OFFSET)
    }

    /// Mixes one byte.
    pub(super) fn mix_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(FNV_PRIME);
    }

    /// Mixes the four little-endian bytes of `value`.
    pub(super) fn mix_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.mix_byte(byte);
        }
    }

    /// Mixes the IEEE bit pattern of `value`, so `-0.0` and `0.0` differ.
    pub(super) fn mix_f32(&mut self, value: f32) {
        self.mix_u32(value.to_bits());
    }

    /// Current hash value.
    pub(super) const fn finish(&self) -> u64 {
        self.0
    }
}

/// Hashes the live particle and body state, or `None` when a read fails.
///
/// Field order: particle count; per particle position x/y and velocity x/y
/// bits; a color-lane marker byte and, when present, each color's RGBA bytes;
/// body count; per body position x/y, angle, linear velocity x/y and angular
/// velocity bits in observation order.
pub(super) fn end_state_fingerprint(world: &World, system: ParticleSystemId) -> Option<u64> {
    let mut hash = Fnv1a::new();
    let view = world.particle_system_view(system).ok()?;
    hash.mix_u32(u32::try_from(view.positions().len()).ok()?);
    for (position, velocity) in view.positions().iter().zip(view.velocities()) {
        hash.mix_f32(position.x);
        hash.mix_f32(position.y);
        hash.mix_f32(velocity.x);
        hash.mix_f32(velocity.y);
    }
    match view.maybe_colors() {
        Some(colors) => {
            hash.mix_byte(1);
            for color in colors {
                for component in color.components() {
                    hash.mix_byte(component);
                }
            }
        }
        None => hash.mix_byte(0),
    }

    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .ok()?;
    hash.mix_u32(u32::try_from(observation.bodies().len()).ok()?);
    for body in observation.bodies() {
        let snapshot = body.snapshot();
        hash.mix_f32(snapshot.position().x);
        hash.mix_f32(snapshot.position().y);
        hash.mix_f32(snapshot.angle());
        hash.mix_f32(snapshot.linear_velocity().x);
        hash.mix_f32(snapshot.linear_velocity().y);
        hash.mix_f32(snapshot.angular_velocity());
    }
    Some(hash.finish())
}

/// Formats a fingerprint as 16 lowercase hex digits.
pub(super) fn fingerprint_hex(value: u64) -> String {
    format!("{value:016x}")
}
