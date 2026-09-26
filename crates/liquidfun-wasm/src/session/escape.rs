//! Removes playground particles that have fallen out of the playfield.

use liquidfun::math::Vec2;
use liquidfun::{PROXY_TAG_HALF_EXTENT_DIAMETERS, ParticleId, ParticleSystemId, World};

/// Distance along gravity, in meters, past which a particle is below the playfield.
const BELOW_GROUND_METERS: f32 = 12.0;
/// Distance sideways or against gravity, in meters, past which a particle has left.
const ESCAPE_METERS: f32 = 48.0;
const GRAVITY_EPSILON_SQUARED: f32 = 1.0e-8;
/// Fraction of the tag domain kept. The rest is room for one step of motion.
const PROXY_KEEP_FRACTION: f32 = 0.5;

/// Returns whether `position` is far enough along gravity, far enough away, or
/// too close to the particle tag wall to keep simulating.
pub(crate) fn particle_has_escaped(position: Vec2, gravity: Vec2, diameter: f32) -> bool {
    if !position.is_valid() || !diameter.is_finite() || diameter <= 0.0 {
        return true;
    }
    if outside_proxy_keep(position, diameter) {
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
    particle_radius: f32,
) -> Result<(), String> {
    let gravity = world.gravity();
    let diameter = particle_radius * 2.0;
    let escaped = escaped_particle_ids(world, system, gravity, diameter)?;
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
) -> Result<Vec<ParticleId>, String> {
    let view = world
        .particle_system_view(system)
        .map_err(|error| error.to_string())?;
    Ok(view
        .particle_ids()
        .iter()
        .copied()
        .zip(view.positions().iter().copied())
        .filter(|(_particle, position)| particle_has_escaped(*position, gravity, diameter))
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
