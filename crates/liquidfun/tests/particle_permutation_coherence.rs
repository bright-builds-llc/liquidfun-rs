//! Black-box particle permutation and derived-weight coherence regressions.

use liquidfun::math::Vec2;
use liquidfun::{NoDecisionHook, ParticleDef, StepConfiguration, StepLimits, World};

#[test]
fn compaction_recomputes_weights_from_retained_stable_contacts() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system fits");
    let positions = [0.0, 0.5, 1.0];
    let particles = positions.map(|x| {
        world
            .create_particle_with_def(
                system,
                None,
                &ParticleDef::default()
                    .with_position(Vec2::new(x, 0.0))
                    .expect("particle position is finite"),
            )
            .expect("particle fits")
            .created_particle()
    });
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("step configuration is valid"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("contact refresh succeeds");
    let removed = particles[1];
    let retained_contacts = world
        .particle_system_view(system)
        .expect("particle system remains live")
        .particle_contacts()
        .filter(|contact| !contact.particles().contains(&removed))
        .map(|contact| (contact.particles(), contact.weight()))
        .collect::<Vec<_>>();
    world
        .mark_particle_for_destruction(removed)
        .expect("middle particle becomes pending");

    // Act
    world
        .compact_pending_particles(system)
        .expect("middle particle compacts");

    // Assert
    let view = world
        .particle_system_view(system)
        .expect("particle system remains live");
    let after_contacts = view
        .particle_contacts()
        .map(|contact| (contact.particles(), contact.weight()))
        .collect::<Vec<_>>();
    assert_eq!(after_contacts, retained_contacts);
    let mut expected_weights = vec![0.0; view.particle_ids().len()];
    for contact in view.particle_contacts() {
        for particle in contact.particles() {
            let index = view
                .particle_ids()
                .iter()
                .position(|candidate| *candidate == particle)
                .expect("contact particle remains in the aggregate view");
            expected_weights[index] += contact.weight();
        }
    }
    for contact in view.body_contacts() {
        let index = view
            .particle_ids()
            .iter()
            .position(|candidate| *candidate == contact.particle())
            .expect("body-contact particle remains in the aggregate view");
        expected_weights[index] += contact.weight();
    }
    assert_eq!(view.weights(), expected_weights);
    assert!(view.weights().iter().any(|weight| *weight > 0.0));
}
