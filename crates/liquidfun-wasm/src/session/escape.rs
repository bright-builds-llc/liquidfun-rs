//! Removes playground particles that have fallen out of the playfield.

use liquidfun::math::Vec2;
use liquidfun::{PROXY_TAG_HALF_EXTENT_DIAMETERS, ParticleId, ParticleSystemId, World};

use crate::scene::SceneId;

/// Distance along gravity, in meters, past which a particle is below the playfield.
const BELOW_GROUND_METERS: f32 = 12.0;
/// Distance sideways or against gravity, in meters, past which a particle has left.
const ESCAPE_METERS: f32 = 48.0;
/// The 50 m wave tank must fit even when device tilt rotates gravity.
const WAVE_TANK_ESCAPE_METERS: f32 = 80.0;
const GRAVITY_EPSILON_SQUARED: f32 = 1.0e-8;
/// Fraction of the tag domain kept. The rest is room for one step of motion.
const PROXY_KEEP_FRACTION: f32 = 0.5;

/// Fall and escape distances, in meters, that bound one session's playfield.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EscapeLimits {
    fall: f32,
    escape: f32,
}

impl EscapeLimits {
    /// Shared playfield limits: 12 m along gravity, 48 m sideways or against it.
    pub(crate) const SHARED: Self = Self {
        fall: BELOW_GROUND_METERS,
        escape: ESCAPE_METERS,
    };

    /// Per-scene limits; only the 50 m wave tank widens them.
    pub(crate) const fn for_scene(scene: SceneId) -> Self {
        match scene {
            SceneId::WaveTank => Self {
                fall: WAVE_TANK_ESCAPE_METERS,
                escape: WAVE_TANK_ESCAPE_METERS,
            },
            _ => Self::SHARED,
        }
    }
}

/// Returns whether a particle in `scene` has escaped; see [`particle_escaped_within`].
#[cfg(test)]
pub(crate) fn particle_has_escaped(
    scene: SceneId,
    position: Vec2,
    gravity: Vec2,
    diameter: f32,
) -> bool {
    particle_escaped_within(EscapeLimits::for_scene(scene), position, gravity, diameter)
}

/// Returns whether `position` is far enough along gravity, far enough away, or
/// too close to the particle tag wall to keep simulating.
pub(crate) fn particle_escaped_within(
    limits: EscapeLimits,
    position: Vec2,
    gravity: Vec2,
    diameter: f32,
) -> bool {
    if !position.is_valid() || !diameter.is_finite() || diameter <= 0.0 {
        return true;
    }
    if outside_proxy_keep(position, diameter) {
        return true;
    }

    let (fall_limit, escape_limit) = (limits.fall, limits.escape);
    let down = fall_direction(gravity);
    let below = position.x * down.x + position.y * down.y;
    if below > fall_limit {
        return true;
    }

    let sideways_x = position.x - below * down.x;
    let sideways_y = position.y - below * down.y;
    let sideways_squared = sideways_x * sideways_x + sideways_y * sideways_y;
    below < -escape_limit || sideways_squared > escape_limit * escape_limit
}

/// Destroys particles that have left the playfield before the next solver step.
pub(crate) fn evict_escaped_particles(
    world: &mut World,
    system: ParticleSystemId,
    particle_radius: f32,
    limits: EscapeLimits,
) -> Result<(), String> {
    let gravity = world.gravity();
    let diameter = particle_radius * 2.0;
    let escaped = escaped_particle_ids(world, system, gravity, diameter, limits)?;
    for particle in escaped {
        world
            .destroy_particle(particle)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn escaped_particle_ids(
    world: &World,
    system: ParticleSystemId,
    gravity: Vec2,
    diameter: f32,
    limits: EscapeLimits,
) -> Result<Vec<ParticleId>, String> {
    let view = world
        .particle_system_view(system)
        .map_err(|error| error.to_string())?;
    Ok(view
        .particle_ids()
        .iter()
        .copied()
        .zip(view.positions().iter().copied())
        .filter(|(_particle, position)| {
            particle_escaped_within(limits, *position, gravity, diameter)
        })
        .map(|(particle, _position)| particle)
        .collect())
}

/// Small particles reach the tag wall inside the meter-scale playfield.
fn outside_proxy_keep(position: Vec2, diameter: f32) -> bool {
    let keep = PROXY_TAG_HALF_EXTENT_DIAMETERS * PROXY_KEEP_FRACTION;
    let x = position.x / diameter;
    let y = position.y / diameter;
    !x.is_finite() || !y.is_finite() || x.abs() >= keep || y.abs() >= keep
}

fn fall_direction(gravity: Vec2) -> Vec2 {
    let length_squared = gravity.length_squared();
    if !length_squared.is_finite() || length_squared < GRAVITY_EPSILON_SQUARED {
        return Vec2::new(0.0, -1.0);
    }

    let inverse_length = 1.0 / length_squared.sqrt();
    Vec2::new(gravity.x * inverse_length, gravity.y * inverse_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_limits_are_twelve_down_and_forty_eight_away() {
        // Arrange
        let expected = EscapeLimits {
            fall: 12.0,
            escape: 48.0,
        };

        // Act
        let shared = EscapeLimits::SHARED;

        // Assert
        assert_eq!(shared, expected);
    }

    #[test]
    fn wave_tank_widens_both_limits_to_eighty() {
        // Arrange
        let expected = EscapeLimits {
            fall: 80.0,
            escape: 80.0,
        };

        // Act
        let limits = EscapeLimits::for_scene(SceneId::WaveTank);

        // Assert
        assert_eq!(limits, expected);
    }

    #[test]
    fn other_scenes_use_the_shared_limits() {
        // Arrange
        let scenes = [SceneId::DamBreak, SceneId::TeslaValve];

        // Act
        let limits = scenes.map(EscapeLimits::for_scene);

        // Assert
        assert_eq!(limits, [EscapeLimits::SHARED, EscapeLimits::SHARED]);
    }
}
