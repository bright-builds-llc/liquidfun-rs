//! Removes playground particles that have fallen out of the playfield.

use liquidfun::math::Vec2;
use liquidfun::{ParticleId, ParticleSystemId, World};

/// Distance along gravity, in meters, past which a particle is below the playfield.
const BELOW_GROUND_METERS: f32 = 12.0;
/// Distance sideways or against gravity, in meters, past which a particle has left.
const ESCAPE_METERS: f32 = 48.0;
const GRAVITY_EPSILON_SQUARED: f32 = 1.0e-8;

/// Returns whether `position` is far enough along gravity, or far enough away, to drop.
pub(crate) fn particle_has_escaped(position: Vec2, gravity: Vec2) -> bool {
    if !position.is_valid() {
        return true;
    }

    let down = fall_direction(gravity);
    let below = position.x * down.x + position.y * down.y;
    if below > BELOW_GROUND_METERS {
        return true;
    }

    let sideways_x = position.x - below * down.x;
    let sideways_y = position.y - below * down.y;
    let sideways_squared = sideways_x * sideways_x + sideways_y * sideways_y;
    below < -ESCAPE_METERS || sideways_squared > ESCAPE_METERS * ESCAPE_METERS
}

/// Destroys particles that have left the playfield before the next solver step.
pub(crate) fn evict_escaped_particles(
    world: &mut World,
    system: ParticleSystemId,
) -> Result<(), String> {
    let gravity = world.gravity();
    let escaped = escaped_particle_ids(world, system, gravity)?;
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
) -> Result<Vec<ParticleId>, String> {
    let view = world
        .particle_system_view(system)
        .map_err(|error| error.to_string())?;
    Ok(view
        .particle_ids()
        .iter()
        .copied()
        .zip(view.positions().iter().copied())
        .filter(|(_particle, position)| particle_has_escaped(*position, gravity))
        .map(|(particle, _position)| particle)
        .collect())
}

fn fall_direction(gravity: Vec2) -> Vec2 {
    let length_squared = gravity.length_squared();
    if !length_squared.is_finite() || length_squared < GRAVITY_EPSILON_SQUARED {
        return Vec2::new(0.0, -1.0);
    }

    let inverse_length = 1.0 / length_squared.sqrt();
    Vec2::new(gravity.x * inverse_length, gravity.y * inverse_length)
}
