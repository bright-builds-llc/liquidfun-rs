//! Washing Machine construction, live drum speed, and a short tumble.

use std::f32::consts::TAU;

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleFlags, ParticleGroupFlags};
use liquidfun::{JointDef, ParticleSystemId, World, WorldObservationLimits};

use super::super::SceneId;
use super::geometry::INNER_RADIUS;
use super::{DEFAULT_DRUM_RPM, build};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore, SessionError};

#[test]
fn create_builds_a_partially_filled_drum() {
    // Arrange / Act
    let session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    let (water, cloth, solid_groups) = session.read_particles(load_census);

    // Assert
    session
        .capture_frame()
        .expect("the drum drawing must fit the frame lane");

    // Assert
    assert_eq!(
        water, 3000,
        "the lower drum must start with 3000 water particles (got {water})"
    );
    assert!(
        cloth > 30,
        "the drum must start with a few elastic socks (got {cloth})"
    );
    assert_eq!(
        solid_groups, 3,
        "the three socks must be solid elastic groups"
    );
    assert_eq!(session.particle_count(), water + cloth);
    assert_eq!(
        session.rigid_shape_count(),
        64,
        "inner and outer wall chords plus four ribs use the frame lane"
    );
}

#[test]
fn particle_radius_is_sixty_percent_of_the_previous_size() {
    // Arrange / Act
    let super::super::BuiltScene {
        particle_radius, ..
    } = build(&[]).expect("Washing Machine should construct");

    // Assert
    let expected = 0.0106_f32 * 2.0 * 0.6;
    assert_eq!(
        particle_radius.to_bits(),
        expected.to_bits(),
        "the shared radius must be 0.6 times the previous size (got {particle_radius})"
    );
}

#[test]
fn clothes_start_as_socks() {
    // Arrange / Act
    let session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    let socks = session.read_particles(solid_group_positions);

    // Assert — cuff and toe are the far ends, and the foot sits on one side of that line.
    assert_eq!(socks.len(), 3, "the drum starts with three socks");
    for (index, positions) in socks.iter().enumerate() {
        let (length, heavier_side, lighter_side) = sock_silhouette(positions);
        assert!(
            length > 0.32,
            "sock {index} must reach from cuff to toe (got {length})"
        );
        assert!(
            heavier_side > lighter_side.saturating_mul(2),
            "sock {index} must keep its foot on one side of the cuff-to-toe line (heavier {heavier_side}, lighter {lighter_side})"
        );
    }
}

#[test]
fn default_drum_speed_matches_twenty_rpm() {
    // Arrange
    let super::super::BuiltScene { world, .. } =
        build(&[]).expect("Washing Machine should construct");

    // Act
    let speed = revolute_motor_speed(&world);

    // Assert
    let expected = f32::from(DEFAULT_DRUM_RPM) * TAU / 60.0;
    assert!(
        (speed - expected).abs() < 1.0e-5,
        "the drum must start at {DEFAULT_DRUM_RPM} rpm (got {speed}, expected {expected})"
    );
}

#[test]
fn drum_speed_control_is_live() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    let before = session.particle_count();

    // Act
    let applied = session.apply_control("drum-speed", "36");

    // Assert
    assert_eq!(applied, Ok(false));
    assert_eq!(session.particle_count(), before);
}

#[test]
fn drum_speed_rejects_off_scale_tokens() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");

    // Act
    let rejected = ["01", "49", "-1", "20.0", "fast", ""]
        .map(|value| session.apply_control("drum-speed", value));

    // Assert
    assert!(
        rejected
            .iter()
            .all(|result| *result == Err(SessionError::UnknownControl))
    );
}

#[test]
fn stopped_drum_stays_level() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    session
        .apply_control("drum-speed", "0")
        .expect("0 rpm is on the spinner");

    // Act
    advance_steps(&mut session, 88);
    let angle = session
        .motor_angle()
        .expect("the drum should report its angle");

    // Assert
    assert!(angle.abs() < 0.08, "0 rpm must hold the drum (got {angle})");
}

#[test]
fn fast_spin_turns_the_drum_and_lifts_water() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    session
        .apply_control("drum-speed", "48")
        .expect("48 rpm is the spinner maximum");
    let starting_top = session.read_particles(highest_water);

    // Act
    advance_steps(&mut session, 120);
    let angle = session
        .motor_angle()
        .expect("the drum should report its angle");
    let (highest, farthest) = session.read_particles(|world, system| {
        (
            highest_water(world, system),
            farthest_particle(world, system),
        )
    });

    // Assert — 48 rpm for two seconds is 1.6 turns.
    assert!(
        angle > TAU,
        "48 rpm must turn the drum through at least one revolution (got {angle})"
    );
    assert!(
        highest > starting_top + 0.35,
        "ribs must lift water above the resting fill (start {starting_top}, later {highest})"
    );
    assert!(
        farthest < INNER_RADIUS + 0.05,
        "water must stay inside the drum (farthest {farthest}, inner {INNER_RADIUS})"
    );
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let batch = remaining.min(MAX_ADVANCE_STEPS);
        session
            .advance(batch)
            .expect("the drum should keep stepping");
        remaining -= batch;
    }
}

fn revolute_motor_speed(world: &World) -> f32 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the drum motor");
    let revolute: Vec<_> = observation
        .joints()
        .iter()
        .filter_map(|joint| match joint.snapshot().definition() {
            JointDef::Revolute(definition) => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(revolute.len(), 1, "exactly one revolute joint");
    assert!(
        revolute[0].is_motor_enabled(),
        "the drum motor must be enabled"
    );
    revolute[0].motor_speed()
}

fn load_census(world: &World, system: ParticleSystemId) -> (usize, usize, usize) {
    let view = world
        .particle_system_view(system)
        .expect("the drum particle system should stay live");
    let water = view
        .flags()
        .iter()
        .filter(|flags| !flags.intersects(ParticleFlags::ELASTIC | ParticleFlags::SPRING))
        .count();
    let cloth = view
        .flags()
        .iter()
        .filter(|flags| flags.contains(ParticleFlags::ELASTIC))
        .count();
    let mut seen = Vec::new();
    let mut solid_groups = 0_usize;
    for maybe_group in view.group_ids() {
        let Some(group) = maybe_group else {
            continue;
        };
        if seen.contains(group) {
            continue;
        }
        seen.push(*group);
        let group_view = world
            .particle_group_view(*group)
            .expect("a clothing group should stay live");
        if group_view.flags().contains(ParticleGroupFlags::SOLID) {
            solid_groups += 1;
        }
    }
    (water, cloth, solid_groups)
}

fn solid_group_positions(world: &World, system: ParticleSystemId) -> Vec<Vec<Vec2>> {
    let view = world
        .particle_system_view(system)
        .expect("the drum particle system should stay live");
    let mut seen = Vec::new();
    let mut socks = Vec::new();
    for maybe_group in view.group_ids() {
        let Some(group) = maybe_group else {
            continue;
        };
        if seen.contains(group) {
            continue;
        }
        seen.push(*group);
        let group_view = world
            .particle_group_view(*group)
            .expect("a clothing group should stay live");
        if !group_view.flags().contains(ParticleGroupFlags::SOLID) {
            continue;
        }
        let positions = view
            .positions()
            .iter()
            .zip(view.group_ids())
            .filter_map(|(position, maybe_member)| {
                maybe_member
                    .filter(|member| member == group)
                    .map(|_| *position)
            })
            .collect();
        socks.push(positions);
    }
    socks
}

/// Cuff-to-toe length, and how many particles sit on each side of that line.
fn sock_silhouette(positions: &[Vec2]) -> (f32, usize, usize) {
    let centroid = positions
        .iter()
        .copied()
        .reduce(|total, position| total + position)
        .map_or(Vec2::ZERO, |total| {
            let count =
                u16::try_from(positions.len()).expect("bounded test sock count fits in u16");
            total * (1.0 / f32::from(count))
        });
    let cuff = farthest_from(positions, centroid);
    let toe = farthest_from(positions, cuff);
    let axis = toe - cuff;
    let mut positive = 0_usize;
    let mut negative = 0_usize;
    for position in positions {
        let side = axis.cross(*position - cuff);
        if side >= 0.0 {
            positive += 1;
        } else {
            negative += 1;
        }
    }
    (
        axis.length(),
        positive.max(negative),
        positive.min(negative),
    )
}

fn farthest_from(positions: &[Vec2], origin: Vec2) -> Vec2 {
    positions
        .iter()
        .copied()
        .fold(origin, |farthest, position| {
            if (position - origin).length_squared() > (farthest - origin).length_squared() {
                position
            } else {
                farthest
            }
        })
}

fn highest_water(world: &World, system: ParticleSystemId) -> f32 {
    let view = world
        .particle_system_view(system)
        .expect("the drum particle system should stay live");
    view.positions()
        .iter()
        .zip(view.flags())
        .filter(|(_position, flags)| {
            !flags.intersects(ParticleFlags::ELASTIC | ParticleFlags::SPRING)
        })
        .map(|(position, _flags)| position.y)
        .fold(f32::MIN, f32::max)
}

fn farthest_particle(world: &World, system: liquidfun::ParticleSystemId) -> f32 {
    particle_positions(world, system)
        .into_iter()
        .map(Vec2::length)
        .fold(0.0, f32::max)
}

fn particle_positions(world: &World, system: liquidfun::ParticleSystemId) -> Vec<Vec2> {
    world
        .particle_system_view(system)
        .expect("the drum particle system should stay live")
        .positions()
        .to_vec()
}
