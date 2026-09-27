use liquidfun::particle::ParticleFlags;
use liquidfun::{ParticleGroupId, ParticleSystemId, World};

use super::super::{SceneId, parse_scene_id};
use crate::ProofFrame;
use crate::session::{SessionCore, SessionError};

#[test]
fn parse_scene_id_accepts_sparky() {
    // Arrange
    let raw = "sparky";

    // Act
    let parsed = parse_scene_id(raw);

    // Assert
    assert_eq!(parsed, Ok(SceneId::Sparky));
}

#[test]
fn construction_has_six_circles_and_no_particles() {
    // Arrange
    let session = open_session();
    let built = super::build(&[]).expect("Sparky should construct the chamber");

    // Act
    let circles = built
        .hooks
        .collect_circles(&built.world)
        .expect("sparkable circles should export");
    let frame = ProofFrame::from(
        session
            .capture_frame()
            .expect("the chamber frame should include six circles"),
    );

    // Assert
    assert_eq!(session.particle_count(), 0);
    assert_eq!(session.live_particle_count().expect("live count"), 0);
    assert_eq!(circles.len(), 6);
    assert_eq!(frame.rigid_circles().len(), 6 * 3);
}

#[test]
fn sparkable_contact_creates_powder_that_expires_after_lifetime() {
    // Arrange
    let mut session = open_session();

    // Act
    let Some(group) = first_powder_group(&mut session, 180) else {
        panic!("a sparkable contact should create a powder group within 180 steps");
    };
    advance_steps(&mut session, 45);
    let still_listed = session.read_particles(|world, system| group_is_live(world, system, group));

    // Assert
    assert!(
        !still_listed,
        "the recorded powder group must be gone after 45 further steps of 1/60 s"
    );
}

#[test]
fn particle_count_stays_bounded_across_three_hundred_steps() {
    // Arrange
    let mut session = open_session();

    // Act / Assert
    for _ in 0..75 {
        advance_steps(&mut session, 4);
        let count = session
            .live_particle_count()
            .expect("Sparky particles should stay countable");
        assert!(
            count < 4000,
            "particle count {count} climbed without a bound"
        );
    }
}

#[test]
fn first_burst_fades_in_the_copied_color_lane() {
    // Arrange
    let mut session = open_session();
    let Some(group) = first_powder_group(&mut session, 180) else {
        panic!("a sparkable contact should create a powder group before fade is visible");
    };
    let spawned = group_colors(&session, group);

    // Act
    advance_steps(&mut session, 30);
    let faded = group_colors(&session, group);

    // Assert
    assert!(
        !spawned.is_empty(),
        "the first burst should copy a color lane"
    );
    assert!(
        spawned
            .iter()
            .any(|color| color[0] > 0 || color[1] > 0 || color[2] > 0),
        "the first copied burst color must not be all zeros"
    );
    assert!(
        spawned.iter().all(|color| color[3] == 255),
        "spawn alpha stays 255"
    );
    assert!(
        faded.iter().any(|color| color[3] == 255
            && spawned.iter().any(|original| {
                color[0] < original[0] || color[1] < original[1] || color[2] < original[2]
            })),
        "a still-live member of the first burst must be darker after thirty steps"
    );
}

#[test]
fn pointer_does_not_change_the_particle_count() {
    // Arrange
    let mut session = open_session();
    let before = session
        .live_particle_count()
        .expect("a fresh chamber should report zero particles");

    // Act
    let down = session.apply_pointer("down", 0.0, 10.0);
    let moved = session.apply_pointer("move", 1.0, 9.0);
    let up = session.apply_pointer("up", 1.0, 9.0);

    // Assert
    assert_eq!(down, Ok(()));
    assert_eq!(moved, Ok(()));
    assert_eq!(up, Ok(()));
    assert_eq!(session.live_particle_count().expect("live count"), before);
}

#[test]
fn unknown_control_and_action_are_rejected() {
    // Arrange
    let mut session = open_session();

    // Act
    let control = session.apply_control("spark", "1");
    let action = session.apply_action("spark");

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
}

#[test]
fn destruction_by_age_stays_off() {
    // Arrange / Act
    let built = super::build(&[]).expect("Sparky should construct");
    let definition = built
        .world
        .particle_system_snapshot(built.particle_system)
        .expect("Sparky system should stay live")
        .definition();

    // Assert
    assert!(!definition.destroys_by_age());
    assert_eq!(definition.radius().to_bits(), 0.25_f32.to_bits());
    assert_eq!(definition.maybe_maximum_count(), Some(10240));
}

fn open_session() -> SessionCore {
    SessionCore::create(SceneId::Sparky).expect("Sparky should construct")
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let batch = remaining.min(4);
        let stepped = session.advance(batch);
        assert_eq!(
            stepped,
            Ok(()),
            "Sparky step failed: {}",
            session.failure_detail()
        );
        remaining -= batch;
    }
}

fn first_powder_group(session: &mut SessionCore, step_limit: u32) -> Option<ParticleGroupId> {
    for _ in 0..step_limit {
        advance_steps(session, 1);
        if let Some(group) = session.read_particles(|world, system| powder_group(world, system)) {
            return Some(group);
        }
    }
    None
}

fn powder_group(world: &World, system: ParticleSystemId) -> Option<ParticleGroupId> {
    let view = world
        .particle_system_view(system)
        .expect("Sparky system should stay live");
    let flags = view.flags();
    let groups = view.group_ids();
    groups.iter().zip(flags).find_map(|(maybe_group, flags)| {
        let group = (*maybe_group)?;
        flags.contains(ParticleFlags::POWDER).then_some(group)
    })
}

fn group_is_live(world: &World, system: ParticleSystemId, group: ParticleGroupId) -> bool {
    let view = world
        .particle_system_view(system)
        .expect("Sparky system should stay live");
    view.group_ids().contains(&Some(group))
}

fn group_colors(session: &SessionCore, group: ParticleGroupId) -> Vec<[u8; 4]> {
    let frame = ProofFrame::from(
        session
            .capture_frame()
            .expect("a live burst should copy its color lane"),
    );
    let copied = frame.particle_colors();
    session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("Sparky system should stay live");
        view.group_ids()
            .iter()
            .enumerate()
            .filter_map(|(index, maybe_group)| (*maybe_group == Some(group)).then_some(index))
            .filter_map(|index| {
                let start = index.checked_mul(4)?;
                Some([
                    *copied.get(start)?,
                    *copied.get(start + 1)?,
                    *copied.get(start + 2)?,
                    *copied.get(start + 3)?,
                ])
            })
            .collect()
    })
}
