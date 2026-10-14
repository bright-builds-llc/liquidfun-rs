//! Test-only helpers shared by the screw's hooks, spike and flow tests.

use liquidfun::math::{Transform, Vec2};
use liquidfun::{BodyType, StepConfiguration, WorldObservationLimits};

use super::geometry::{
    FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, INLET_LENGTH, OUTLET_LENGTH,
    PARTICLE_RADIUS, TUBE_LENGTH, VANE_COUNT, VANE_HALF_SPAN, VANE_OVERHANG, distance_to_boundary,
    in_basin, in_chute, in_reservoir, in_tube, quad_contains, vane_center, vane_q, vane_quads,
    wall_quads,
};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore};

/// How far past the frame a particle may fly before it counts as escaped (P2).
const ESCAPE_MARGIN: f32 = 0.5;
/// Steps a checked run lets the water settle before leak samples start.
const FLOW_SETTLE_STEPS: u32 = 60;
/// A checked run samples the reservoir surface every this many steps (D-08).
const SURFACE_SAMPLE_EVERY: u32 = 60;
/// Percentile of reservoir particle heights read as its surface (robust to splashes).
const SURFACE_PERCENTILE: usize = 95;

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

/// Particles per region; `lost` is outside every region (research Pitfall 6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct RegionCounts {
    pub(super) reservoir: usize,
    pub(super) tube: usize,
    pub(super) basin: usize,
    pub(super) chute: usize,
    pub(super) lost: usize,
}

/// Counts each particle in the first region that holds it: tube, basin, chute, reservoir.
pub(super) fn region_counts(positions: &[Vec2]) -> RegionCounts {
    let mut counts = RegionCounts::default();
    for point in positions.iter().copied() {
        let slot = if in_tube(point) {
            &mut counts.tube
        } else if in_basin(point) {
            &mut counts.basin
        } else if in_chute(point) {
            &mut counts.chute
        } else if in_reservoir(point) {
            &mut counts.reservoir
        } else {
            &mut counts.lost
        };
        *slot += 1;
    }
    counts
}

/// The 95th-percentile height of reservoir particles, or `None` when it is empty (D-08).
pub(super) fn reservoir_surface_y(positions: &[Vec2]) -> Option<f32> {
    let mut heights: Vec<f32> = positions
        .iter()
        .filter(|point| in_reservoir(**point))
        .map(|point| point.y)
        .collect();
    heights.sort_by(f32::total_cmp);
    let last = heights.len().checked_sub(1)?;
    heights.get(last * SURFACE_PERCENTILE / 100).copied()
}

/// Current particle positions of a session.
pub(super) fn positions(session: &SessionCore) -> Vec<Vec2> {
    session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("water")
            .positions()
            .to_vec()
    })
}

/// Transform of the session's only kinematic body, the vane train.
pub(super) fn train_transform(session: &SessionCore) -> Transform {
    session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation")
            .bodies()
            .iter()
            .map(|body| body.snapshot())
            .find(|snapshot| snapshot.body_type() == BodyType::Kinematic)
            .expect("the vane train")
            .transform()
    })
}

/// Measurements from one checked session run (D-08, D-19, D-20).
#[derive(Debug, Clone, Copy)]
pub(super) struct FlowReport {
    pub(super) max_inside_walls: usize,
    pub(super) max_inside_vanes: usize,
    /// Lowest sampled reservoir surface; infinite when never sampled.
    pub(super) min_reservoir_surface: f32,
    pub(super) start: RegionCounts,
    pub(super) end: RegionCounts,
    pub(super) built: usize,
    pub(super) live: usize,
}

/// Advances `steps` in session-sized chunks. After a 60-step settle it samples
/// leaks every chunk and the reservoir surface every 60th step, then records
/// the final regions and the live particle count.
pub(super) fn run_checked(session: &mut SessionCore, steps: u32) -> FlowReport {
    let mut report = FlowReport {
        max_inside_walls: 0,
        max_inside_vanes: 0,
        min_reservoir_surface: f32::INFINITY,
        start: region_counts(&positions(session)),
        end: RegionCounts::default(),
        built: session.particle_count(),
        live: 0,
    };
    let mut elapsed = 0;
    while elapsed < steps {
        let chunk = (steps - elapsed).min(MAX_ADVANCE_STEPS);
        session.advance(chunk).expect("the screw scene steps");
        let previous = elapsed;
        elapsed += chunk;
        if elapsed <= FLOW_SETTLE_STEPS {
            continue;
        }
        let current = positions(session);
        let leaks = leak_counts(&current, train_transform(session));
        report.max_inside_walls = report.max_inside_walls.max(leaks.inside_walls);
        report.max_inside_vanes = report.max_inside_vanes.max(leaks.inside_vanes);
        if elapsed / SURFACE_SAMPLE_EVERY > previous / SURFACE_SAMPLE_EVERY {
            let surface = reservoir_surface_y(&current).unwrap_or(f32::NEG_INFINITY);
            report.min_reservoir_surface = report.min_reservoir_surface.min(surface);
        }
    }
    report.end = region_counts(&positions(session));
    report.live = session.live_particle_count().expect("live count");
    report
}
