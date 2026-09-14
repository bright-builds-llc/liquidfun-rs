use super::transaction_support::{Fixture, recipe};
use liquidfun::math::Vec2;
use liquidfun::particle::ParticleGroupDestination;
use liquidfun::{CreateObjectError, HandleError, ParticleFlags};

#[test]
fn capacity_rejection_preserves_populated_state_then_available_creation_succeeds() {
    // Arrange
    let mut fixture = Fixture::with_capacity(5);
    let before = fixture.snapshot();
    // Act
    let result = fixture.world.create_particle_group(
        fixture.system,
        &recipe(
            vec![Vec2::new(30.0, 0.0), Vec2::new(31.0, 0.0)],
            ParticleGroupDestination::New,
        ),
    );
    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::Arena(
            liquidfun::ArenaInsertError::CapacityExceeded { limit: 5 }
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let created = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(vec![Vec2::new(30.0, 0.0)], ParticleGroupDestination::New),
        )
        .expect("available creation after rejection");
    assert_eq!(
        fixture
            .world
            .particle_group_view(created)
            .expect("new group live")
            .member_count(),
        1
    );
    assert_eq!(
        fixture
            .world
            .particle_system_statistics(fixture.system)
            .expect("system live")
            .particle_count(),
        4
    );
}

fn invalid_topology(
    destination: ParticleGroupDestination,
) -> liquidfun::particle::ParticleGroupRecipe {
    let positions = (0..64)
        .scan(0.0_f32, |x, _| {
            let position = Vec2::new(*x, 0.0);
            *x += 1_000.0;
            Some(position)
        })
        .collect();
    recipe(positions, destination)
        .with_particle_flags(ParticleFlags::ELASTIC)
        .with_lifetime(5.0)
        .expect("finite lifetime")
}

#[test]
fn stale_append_preserves_populated_views_then_live_target_remains_appendable() {
    // Arrange
    let mut fixture = Fixture::new();
    let before = fixture.snapshot();
    // Act
    let result = fixture.world.create_particle_group(
        fixture.system,
        &recipe(
            vec![Vec2::ZERO],
            ParticleGroupDestination::AppendTo(fixture.stale),
        ),
    );
    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let returned = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(
                vec![Vec2::new(0.5, 0.5)],
                ParticleGroupDestination::AppendTo(fixture.target),
            ),
        )
        .expect("live target remains appendable");
    assert_eq!(returned, fixture.target);
    assert_eq!(
        fixture
            .world
            .particle_group_view(returned)
            .expect("live target")
            .member_count(),
        4
    );
}

#[test]
fn late_create_rejection_preserves_populated_views_and_deferred_lifecycle() {
    // Arrange
    let mut fixture = Fixture::new();
    let before = fixture.snapshot();
    // Act
    let result = fixture.world.create_particle_group(
        fixture.system,
        &invalid_topology(ParticleGroupDestination::New),
    );
    // Assert
    assert_eq!(result, Err(CreateObjectError::InvalidParticleGroupTopology));
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let group = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(vec![Vec2::new(30.0, 0.0)], ParticleGroupDestination::New),
        )
        .expect("valid creation after rejection");
    assert!(!fixture.groups.contains(&group));
    let view = fixture
        .world
        .particle_group_view(group)
        .expect("new group live");
    assert_eq!(view.member_count(), 1);
    assert_eq!(
        fixture
            .world
            .particle_snapshot(view.member_ids()[0])
            .expect("new member live")
            .maybe_group(),
        Some(group)
    );
    assert_eq!(
        fixture
            .world
            .particle_system_statistics(fixture.system)
            .expect("live system")
            .group_count(),
        fixture.groups.len()
    );
}

#[test]
fn late_append_rejection_preserves_target_then_valid_append_retains_identity_order() {
    // Arrange
    let mut fixture = Fixture::new();
    let before = fixture.snapshot();
    let members = fixture
        .world
        .particle_group_view(fixture.target)
        .expect("target live")
        .member_ids()
        .to_vec();
    // Act
    let result = fixture.world.create_particle_group(
        fixture.system,
        &invalid_topology(ParticleGroupDestination::AppendTo(fixture.target)),
    );
    // Assert
    assert_eq!(result, Err(CreateObjectError::InvalidParticleGroupTopology));
    assert_eq!(fixture.snapshot(), before);
    fixture.assert_deferred_lifecycle();
    let returned = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(
                vec![Vec2::new(0.5, 0.5)],
                ParticleGroupDestination::AppendTo(fixture.target),
            ),
        )
        .expect("valid append after rejection");
    assert_eq!(returned, fixture.target);
    let view = fixture
        .world
        .particle_group_view(returned)
        .expect("target live");
    assert_eq!(&view.member_ids()[..members.len()], members);
    assert_eq!(view.member_count(), members.len() + 1);
    assert_eq!(
        fixture
            .world
            .particle_system_statistics(fixture.system)
            .expect("live system")
            .group_count(),
        fixture.groups.len() - 1
    );
}

#[test]
fn wrong_system_append_preserves_both_systems_then_remains_usable() {
    // Arrange
    let mut fixture = Fixture::new();
    let other = fixture
        .world
        .create_particle_system()
        .expect("other system fits");
    let before = fixture.snapshot();
    let other_before = super::transaction_support::snapshot(&fixture.world, other, &[]);
    // Act
    let result = fixture.world.create_particle_group(
        other,
        &recipe(
            vec![Vec2::ZERO],
            ParticleGroupDestination::AppendTo(fixture.target),
        ),
    );
    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert_eq!(fixture.snapshot(), before);
    assert_eq!(
        super::transaction_support::snapshot(&fixture.world, other, &[]),
        other_before
    );
    fixture.assert_deferred_lifecycle();
    let returned = fixture
        .world
        .create_particle_group(
            fixture.system,
            &recipe(
                vec![Vec2::new(0.5, 0.5)],
                ParticleGroupDestination::AppendTo(fixture.target),
            ),
        )
        .expect("valid owner append");
    assert_eq!(returned, fixture.target);
    assert_eq!(
        fixture
            .world
            .particle_group_view(returned)
            .expect("live target")
            .member_count(),
        4
    );
}
