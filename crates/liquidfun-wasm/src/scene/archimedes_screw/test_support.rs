//! Test-only helpers shared by the screw's hooks, spike and flow tests.

use liquidfun::StepConfiguration;
use liquidfun::math::{Transform, Vec2};

use super::geometry::{
    FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, INLET_LENGTH, OUTLET_LENGTH,
    PARTICLE_RADIUS, TUBE_LENGTH, VANE_COUNT, VANE_HALF_SPAN, VANE_OVERHANG, distance_to_boundary,
    in_basin, in_tube, quad_contains, vane_center, vane_q, vane_quads, wall_quads,
};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore};

/// How far past the frame a particle may fly before it counts as escaped (P2).
const ESCAPE_MARGIN: f32 = 0.5;

/// The session's fixed step: 1/60 s, 8 velocity and 3 position iterations,
/// and the default two particle iterations (no `particle_iterations` arm).
pub(super) fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(2)
        .expect("two particle iterations")
}

/// Particles deeper than half a radius inside a static wall or a vane (P1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct LeakCounts {
    pub(super) inside_walls: usize,
    pub(super) inside_vanes: usize,
}

fn deeper_than_half_radius(quad: &[Vec2; 4], point: Vec2) -> bool {
    quad_contains(quad, point) && distance_to_boundary(quad, point) > PARTICLE_RADIUS / 2.0
}

/// Counts particle centers more than `PARTICLE_RADIUS / 2` inside any wall
/// quad, and inside any vane quad with the train at `train`.
pub(super) fn leak_counts(positions: &[Vec2], train: Transform) -> LeakCounts {
    let walls = wall_quads();
    let vanes = vane_quads().map(|quad| quad.map(|corner| train.apply(corner)));
    let inside = |quads: &[[Vec2; 4]], point: Vec2| {
        quads
            .iter()
            .any(|quad| deeper_than_half_radius(quad, point))
    };
    LeakCounts {
        inside_walls: positions
            .iter()
            .filter(|point| inside(&walls, **point))
            .count(),
        inside_vanes: positions
            .iter()
            .filter(|point| inside(&vanes, **point))
            .count(),
    }
}

/// Particles in the basin region.
pub(super) fn basin_count(positions: &[Vec2]) -> usize {
    positions.iter().filter(|point| in_basin(**point)).count()
}

/// Particle counts of every fully sealed pocket at lattice `phase`: pockets
/// whose whole `q` range lies in `[INLET_LENGTH + VANE_OVERHANG, TUBE_LENGTH - OUTLET_LENGTH]` (P4).
pub(super) fn sealed_pocket_counts(positions: &[Vec2], phase: f32) -> Vec<usize> {
    (0..VANE_COUNT - 1)
        .filter_map(|index| {
            let low = vane_center(index, phase) + VANE_HALF_SPAN;
            let high = vane_center(index + 1, phase) - VANE_HALF_SPAN;
            let sealed = low >= INLET_LENGTH + VANE_OVERHANG && high <= TUBE_LENGTH - OUTLET_LENGTH;
            sealed.then(|| {
                positions
                    .iter()
                    .filter(|point| in_tube(**point) && (low..=high).contains(&vane_q(**point)))
                    .count()
            })
        })
        .collect()
}

/// Largest particle speed.
pub(super) fn max_speed(velocities: &[Vec2]) -> f32 {
    velocities
        .iter()
        .map(|velocity| velocity.length())
        .fold(0.0, f32::max)
}

/// Particles that are non-finite or outside the frame grown by `ESCAPE_MARGIN` (P2).
pub(super) fn escaped(positions: &[Vec2]) -> usize {
    positions
        .iter()
        .filter(|point| {
            !point.is_valid()
                || point.x < FRAME_MIN_X - ESCAPE_MARGIN
                || point.x > FRAME_MAX_X + ESCAPE_MARGIN
                || point.y < FRAME_MIN_Y - ESCAPE_MARGIN
                || point.y > FRAME_MAX_Y + ESCAPE_MARGIN
        })
        .count()
}

/// Advances the session `steps` fixed steps in chunks the session accepts.
pub(super) fn advance(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(MAX_ADVANCE_STEPS);
        session.advance(chunk).expect("the screw scene steps");
        remaining -= chunk;
    }
}
