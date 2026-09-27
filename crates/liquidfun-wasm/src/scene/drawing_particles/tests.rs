use liquidfun::particle::{ParticleFlags, ParticleGroupFlags};
use liquidfun::{ParticleGroupId, ParticleSystemId, World};

use super::super::{PointerKind, SceneHooks, SceneId, parse_scene_id};
use crate::ProofFrame;
use crate::session::{SessionCore, SessionError};

#[test]
fn parse_accepts_exact_drawing_particles_token() {
    // Arrange
    let raw = "drawing-particles";

    // Act
    let parsed = parse_scene_id(raw);

    // Assert
    assert_eq!(parsed, Ok(SceneId::DrawingParticles));
}

#[test]
fn parse_rejects_mixed_case_drawing_particles_token() {
    // Arrange
    let raw = "Drawing-Particles";

    // Act
    let parsed = parse_scene_id(raw);

    // Assert
    assert_eq!(parsed, Err(SessionError::UnknownScene));
}

#[test]
fn construction_starts_empty_inside_the_vessel_walls() {
    // Arrange
    let session = open_session();
    let built = open_built();

    // Act
    let frame = ProofFrame::from(
        session
            .capture_frame()
            .expect("empty vessel frame should include the walls"),
    );
    let groups = live_groups(&built.world, built.particle_system);

    // Assert
    assert_eq!(session.particle_count(), 0);
    assert_eq!(live_count(&built.world, built.particle_system), 0);
    assert!(
        groups.is_empty(),
        "Drawing Particles starts with no particle groups"
    );
    assert_vessel_edges(&frame.rigid_segments());
}

#[test]
fn particle_system_uses_the_pinned_radius_cap_and_gravity() {
    // Arrange / Act
    let built = open_built();
    let system = built
        .world
        .particle_system_snapshot(built.particle_system)
        .expect("Drawing Particles system should stay live");
    let definition = system.definition();

    // Assert
    assert_eq!(definition.radius().to_bits(), 0.05_f32.to_bits());
    assert_eq!(definition.maybe_maximum_count(), Some(10240));
    assert!(!definition.destroys_by_age());
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn water_move_joins_the_open_stroke() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = open_built();
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.0,
        2.0,
    );
    let before = live_groups(&world, particle_system);
    let before_count = live_count(&world, particle_system);

    // Act
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Move,
        0.35,
        2.0,
    );
    let after = live_groups(&world, particle_system);

    // Assert
    assert_eq!(before.len(), 1, "the first stamp starts one water group");
    assert_eq!(after, before, "a matching move appends to that group");
    assert!(
        live_count(&world, particle_system) > before_count,
        "the joined move must add particles outside the destroyed pocket"
    );
    assert!(
        !group_flags(&world, after[0]).contains(ParticleGroupFlags::SOLID),
        "water group flags stay empty"
    );
}

#[test]
fn elastic_material_stays_live_and_starts_a_solid_group() {
    // Arrange
    let mut session = open_session();
    session
        .apply_pointer("down", 0.0, 2.0)
        .expect("water stamp should create particles");
    let before = session
        .live_particle_count()
        .expect("water stamp should stay countable");

    // Act
    let recreated = session
        .apply_control("material", "elastic")
        .expect("elastic material should be allowlisted");
    let after_switch = session
        .live_particle_count()
        .expect("material switch should keep the existing particles");

    // Assert
    assert!(!recreated);
    assert_eq!(after_switch, before);

    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = open_built();
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.0,
        2.0,
    );
    hooks
        .apply_control(&mut world, particle_system, "material", "elastic")
        .expect("elastic material should stay live on the hooks");
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.0,
        3.2,
    );
    let groups = live_groups(&world, particle_system);
    let solid_groups = groups
        .iter()
        .copied()
        .filter(|group| group_flags(&world, *group).contains(ParticleGroupFlags::SOLID))
        .collect::<Vec<_>>();
    let water_groups = groups
        .iter()
        .copied()
        .filter(|group| !group_flags(&world, *group).contains(ParticleGroupFlags::SOLID))
        .count();

    assert_eq!(
        groups.len(),
        2,
        "elastic flags must not join the water stroke"
    );
    assert_eq!(
        solid_groups.len(),
        1,
        "elastic paint should start one SOLID group"
    );
    assert_eq!(water_groups, 1, "the earlier water group stays non-solid");
    assert!(
        group_has_particle_flag(
            &world,
            particle_system,
            solid_groups[0],
            ParticleFlags::ELASTIC
        ),
        "elastic particles must carry ParticleFlags::ELASTIC"
    );
}

#[test]
fn pointer_up_clears_the_join_without_another_stamp() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = open_built();
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.0,
        2.0,
    );
    let stamped = live_count(&world, particle_system);

    // Act
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Up,
        6.0,
        1.0,
    );
    let after_up = live_count(&world, particle_system);
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.8,
        2.0,
    );

    // Assert
    assert_eq!(
        after_up, stamped,
        "pointer up that ends a stroke only clears"
    );
    assert_eq!(
        live_groups(&world, particle_system).len(),
        2,
        "the next down starts a new group after pointer up"
    );
}

#[test]
fn unpaired_pointer_up_stamps_once_then_clears() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = open_built();

    // Act
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Up,
        0.0,
        2.0,
    );
    let stamped = live_groups(&world, particle_system);
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.8,
        2.0,
    );

    // Assert
    assert_eq!(
        stamped.len(),
        1,
        "an up with no open stroke leaves one stamp"
    );
    assert!(live_count(&world, particle_system) > 0);
    assert_eq!(
        live_groups(&world, particle_system).len(),
        2,
        "the unpaired stamp must clear last before the next down"
    );
}

#[test]
fn stamp_outside_the_opening_still_creates_particles() {
    // Arrange
    let mut session = open_session();

    // Act
    let painted = session.apply_pointer("down", 6.0, 1.0);

    // Assert
    assert_eq!(painted, Ok(()));
    assert!(
        session
            .live_particle_count()
            .expect("outside stamp should stay countable")
            > 0,
        "a point outside the opening still destroys then creates"
    );
}

#[test]
fn erasing_the_joined_group_does_not_leave_a_stale_id() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = open_built();
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Down,
        0.0,
        2.0,
    );
    let first = live_groups(&world, particle_system)[0];

    // Act
    paint(
        &mut *hooks,
        &mut world,
        particle_system,
        PointerKind::Move,
        0.0,
        2.0,
    );
    let next = hooks.apply_pointer(&mut world, particle_system, PointerKind::Move, 0.15, 2.0);

    // Assert
    assert_eq!(next, Ok(()));
    assert!(
        live_count(&world, particle_system) > 0,
        "the replacement stamp must still create particles"
    );
    let maybe_first = world.particle_group_view(first);
    let cleared = maybe_first.is_err() || maybe_first.is_ok_and(|view| view.member_count() == 0);
    assert!(
        cleared,
        "a brush that covers the stroke must drop the emptied group"
    );
}

#[test]
fn unknown_material_and_controls_are_rejected() {
    // Arrange
    let mut session = open_session();
    let rejected = [
        ("material", "powder"),
        ("material", "barrier"),
        ("material", ""),
        ("nope", "water"),
    ];

    // Act / Assert
    for (name, value) in rejected {
        assert_eq!(
            session.apply_control(name, value),
            Err(SessionError::UnknownControl),
            "{name}={value}"
        );
    }
    assert_eq!(
        session.apply_action("clear"),
        Err(SessionError::UnknownControl)
    );
    assert!(matches!(
        super::build(&[("material".to_owned(), "elastic".to_owned())]),
        Err(SessionError::UnknownControl)
    ));
}

#[test]
fn non_finite_pointer_is_rejected_before_the_hook() {
    // Arrange
    let mut session = open_session();

    // Act
    let rejected = session.apply_pointer("move", f32::NAN, 1.0);
    let painted = session.apply_pointer("down", 0.0, 2.0);

    // Assert
    assert_eq!(rejected, Err(SessionError::InvalidPointer));
    assert_eq!(painted, Ok(()));
    assert!(
        session
            .live_particle_count()
            .expect("finite stamp should stay countable")
            > 0
    );
}

#[test]
fn advance_several_steps_does_not_panic() {
    // Arrange
    let mut session = open_session();
    session
        .apply_pointer("down", 0.0, 2.0)
        .expect("a stamp should exist before stepping");

    // Act
    let stepped = session.advance(4);

    // Assert
    assert_eq!(stepped, Ok(()));
    assert_eq!(session.step_index(), 4);
}

fn open_session() -> SessionCore {
    SessionCore::create(SceneId::DrawingParticles)
        .expect("Drawing Particles should construct an empty vessel")
}

fn open_built() -> super::super::BuiltScene {
    super::build(&[]).expect("Drawing Particles should construct an empty vessel")
}

fn live_count(world: &World, system: ParticleSystemId) -> usize {
    world
        .particle_system_snapshot(system)
        .expect("Drawing Particles system should stay live")
        .particle_count()
}

fn live_groups(world: &World, system: ParticleSystemId) -> Vec<ParticleGroupId> {
    let view = world
        .particle_system_view(system)
        .expect("Drawing Particles system should stay live");
    let mut seen = Vec::new();
    for maybe_group in view.group_ids() {
        let Some(group) = maybe_group else {
            continue;
        };
        if !seen.contains(group) {
            seen.push(*group);
        }
    }
    seen
}

fn group_flags(world: &World, group: ParticleGroupId) -> ParticleGroupFlags {
    world
        .particle_group_view(group)
        .expect("paint group should stay live")
        .flags()
}

fn group_has_particle_flag(
    world: &World,
    system: ParticleSystemId,
    group: ParticleGroupId,
    flag: ParticleFlags,
) -> bool {
    let view = world
        .particle_system_view(system)
        .expect("Drawing Particles system should stay live");
    view.flags()
        .iter()
        .zip(view.group_ids())
        .any(|(flags, maybe_group)| *maybe_group == Some(group) && flags.contains(flag))
}

fn paint(
    hooks: &mut dyn SceneHooks,
    world: &mut World,
    system: ParticleSystemId,
    kind: PointerKind,
    world_x: f32,
    world_y: f32,
) {
    hooks
        .apply_pointer(world, system, kind, world_x, world_y)
        .unwrap_or_else(|error| panic!("finite pointer sample should paint: {error:?}"));
}

fn assert_vessel_edges(segments: &[f32]) {
    let edges = [
        ((-4.0, 0.0), (4.0, 0.0)),
        ((-2.0, -2.0), (-2.0, 6.0)),
        ((2.0, -2.0), (2.0, 6.0)),
        ((-4.0, 4.0), (4.0, 4.0)),
    ];
    for (start, end) in edges {
        assert!(
            covers_edge(segments, start, end),
            "vessel frame should include edge {start:?} -> {end:?}"
        );
    }
}

fn covers_edge(segments: &[f32], start: (f32, f32), end: (f32, f32)) -> bool {
    segments.chunks_exact(4).any(|chunk| {
        let forward = chunk[0].to_bits() == start.0.to_bits()
            && chunk[1].to_bits() == start.1.to_bits()
            && chunk[2].to_bits() == end.0.to_bits()
            && chunk[3].to_bits() == end.1.to_bits();
        let backward = chunk[0].to_bits() == end.0.to_bits()
            && chunk[1].to_bits() == end.1.to_bits()
            && chunk[2].to_bits() == start.0.to_bits()
            && chunk[3].to_bits() == start.1.to_bits();
        forward || backward
    })
}
