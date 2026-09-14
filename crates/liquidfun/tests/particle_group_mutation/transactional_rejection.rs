use super::transaction_support::{Fixture, recipe, snapshot};
use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleGroupDestination, ParticleGroupFlags};
use liquidfun::{HandleError, LifecycleEvent, ParticleGroupMutationError};

#[test]
fn stale_join_preserves_populated_views_and_live_neighbor_remains_joinable() {
    // Arrange
    let mut fixture = Fixture::new();
    let before = fixture.snapshot();
    // Act
    let result = fixture
        .world
        .join_particle_groups(fixture.target, fixture.stale);
    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let second = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(vec![Vec2::new(0.5, 0.5)], ParticleGroupDestination::New),
        )
        .expect("live neighbor fits");
    let report = fixture
        .world
        .join_particle_groups(fixture.target, second)
        .expect("live neighbor joins");
    assert_eq!(*report.value(), fixture.target);
    assert_eq!(
        fixture
            .world
            .particle_group_view(fixture.target)
            .expect("target live")
            .member_count(),
        4
    );
    let [LifecycleEvent::Destruction(record)] = report.lifecycle() else {
        panic!("one joined shell destruction");
    };
    assert_eq!(
        record.destroyed(),
        liquidfun::DestroyedId::ParticleGroup(second)
    );
}

#[test]
fn cross_system_join_preserves_both_sources_then_valid_join_publishes_one_destruction() {
    // Arrange
    let mut fixture = Fixture::new();
    let other = fixture
        .world
        .create_particle_system()
        .expect("other system fits");
    let foreign = fixture
        .world
        .create_particle_group(
            other,
            &recipe(vec![Vec2::ZERO], ParticleGroupDestination::New),
        )
        .expect("foreign group fits");
    let before = fixture.snapshot();
    let other_before = snapshot(&fixture.world, other, &[foreign]);
    // Act
    let result = fixture.world.join_particle_groups(fixture.target, foreign);
    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    assert_eq!(snapshot(&fixture.world, other, &[foreign]), other_before);
    fixture.assert_deferred_lifecycle();
    let second = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(vec![Vec2::new(0.5, 0.5)], ParticleGroupDestination::New),
        )
        .expect("neighbor group fits");
    let first_members = fixture
        .world
        .particle_group_view(fixture.target)
        .expect("target live")
        .member_ids()
        .to_vec();
    let second_members = fixture
        .world
        .particle_group_view(second)
        .expect("neighbor live")
        .member_ids()
        .to_vec();
    let report = fixture
        .world
        .join_particle_groups(fixture.target, second)
        .expect("valid join follows rejection");
    assert_eq!(*report.value(), fixture.target);
    assert_eq!(
        fixture
            .world
            .particle_group_view(fixture.target)
            .expect("survivor live")
            .member_ids(),
        [first_members, second_members].concat()
    );
    let [LifecycleEvent::Destruction(record)] = report.lifecycle() else {
        panic!("exactly one joined shell destruction");
    };
    assert_eq!(
        record.destroyed(),
        liquidfun::DestroyedId::ParticleGroup(second)
    );
}

#[test]
fn stale_split_preserves_observations_then_live_split_preserves_member_order() {
    // Arrange
    let mut fixture = Fixture::new();
    let stale = fixture.stale;
    let before = fixture.snapshot();
    // Act
    let result = fixture.world.split_particle_group(stale);
    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let disconnected = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(
                vec![Vec2::new(30.0, 0.0), Vec2::new(40.0, 0.0)],
                ParticleGroupDestination::New,
            ),
        )
        .expect("disconnected group fits");
    let members = fixture
        .world
        .particle_group_view(disconnected)
        .expect("disconnected group live")
        .member_ids()
        .to_vec();
    let groups = fixture
        .world
        .split_particle_group(disconnected)
        .expect("valid split after rejection");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0], disconnected);
    assert_ne!(groups[1], disconnected);
    assert_eq!(
        fixture
            .world
            .particle_group_view(groups[0])
            .expect("first component retained")
            .member_ids(),
        &members[..1]
    );
    assert_eq!(
        fixture
            .world
            .particle_group_view(groups[1])
            .expect("second component allocated")
            .member_ids(),
        &members[1..]
    );
}

#[test]
fn stale_flags_preserve_observations_then_live_flags_update_only_requested_metadata() {
    // Arrange
    let mut fixture = Fixture::new();
    let stale = fixture.stale;
    let before = fixture.snapshot();
    // Act
    let result = fixture
        .world
        .set_particle_group_flags(stale, ParticleGroupFlags::RIGID);
    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let members = fixture
        .world
        .particle_group_view(fixture.target)
        .expect("target live")
        .member_ids()
        .to_vec();
    let flags =
        ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY;
    fixture
        .world
        .set_particle_group_flags(fixture.target, flags)
        .expect("valid flags after rejection");
    let view = fixture
        .world
        .particle_group_view(fixture.target)
        .expect("target live");
    assert_eq!(view.flags(), flags);
    assert_eq!(view.member_ids(), members);
    assert!(view.maybe_depths().is_some());
}
