struct PanickingFixtureParticleHook;

impl CollisionDecisionHook for PanickingFixtureParticleHook {
    fn should_collide_fixture_particle(
        &mut self,
        _contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        panic!("intentional fixture-particle hook panic");
    }
}

#[test]
fn step_fixture_particle_hook_panic_poisoning_is_sticky() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let _particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("particle fits")
        .created_particle();

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.step(
            step_configuration(),
            &mut PanickingFixtureParticleHook,
            StepLimits::default(),
        );
    }));

    // Assert
    assert!(panic.is_err());
    assert_eq!(
        world.step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        ),
        Err(StepError::Poisoned)
    );
}

#[derive(Default)]
struct ParticlePrefixOrderHook {
    fixture_particles: Vec<liquidfun::ParticleId>,
    pair_calls: Vec<[liquidfun::ParticleId; 2]>,
}

impl CollisionDecisionHook for ParticlePrefixOrderHook {
    fn should_collide_fixture_particle(
        &mut self,
        contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        self.fixture_particles.push(contact.particle());
        CollisionDirective::Collide
    }

    fn should_collide_particle_pair(
        &mut self,
        contact: ParticlePairContactView<'_>,
    ) -> CollisionDirective {
        self.pair_calls.push(contact.particles());
        CollisionDirective::Collide
    }
}

#[test]
fn step_runs_all_subiterations_newest_system_first_and_skips_paused_systems() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let old_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("old system fits");
    let old_particle = world
        .create_particle_with_def(
            old_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("old particle fits")
        .created_particle();
    let paused_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("paused system fits");
    let paused_particle = world
        .create_particle_with_def(
            paused_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("paused particle fits")
        .created_particle();
    let new_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("new system fits");
    let new_particle = world
        .create_particle_with_def(
            new_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("new particle fits")
        .created_particle();
    let step = step_configuration()
        .with_particle_iterations(3)
        .expect("particle iterations are valid");
    let mut hook = ParticlePrefixOrderHook::default();

    // Act
    world
        .step(step, &mut hook, StepLimits::default())
        .expect("multi-system prefix succeeds");

    // Assert
    assert_eq!(
        hook.fixture_particles,
        vec![
            new_particle,
            new_particle,
            new_particle,
            old_particle,
            old_particle,
            old_particle,
        ]
    );
    assert!(!hook.fixture_particles.contains(&paused_particle));
    assert_eq!(
        world
            .particle_system_view(paused_system)
            .expect("paused system remains live")
            .body_contacts()
            .count(),
        0
    );
}

#[test]
fn step_particle_pair_filter_and_listener_share_the_source_timed_journal() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let flags = ParticleFlags::PARTICLE_CONTACT_FILTER | ParticleFlags::PARTICLE_CONTACT_LISTENER;
    let first = world
        .create_particle_with_def(system, None, &ParticleDef::default().with_flags(flags))
        .expect("first particle fits")
        .created_particle();
    let second = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.0, 0.0))
                .expect("particle position is valid"),
        )
        .expect("second particle fits")
        .created_particle();
    let mut hook = ParticlePrefixOrderHook::default();

    // Act
    let report = world
        .step(step_configuration(), &mut hook, StepLimits::default())
        .expect("particle pair prefix succeeds");

    // Assert
    assert_eq!(hook.pair_calls, vec![[first, second]]);
    assert!(report.lifecycle().iter().any(|event| matches!(
        event,
        StepLifecycleEvent::ParticleContact(liquidfun::ParticleContactEffect::Begin(contact))
            if contact.particles() == [first, second]
    )));
}

#[test]
fn step_off_center_body_impulse_updates_angular_velocity() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity is valid");
    let body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("body is valid"),
        )
        .expect("dynamic body fits");
    let mass_shape =
        Shape::from(CircleShape::new(Vec2::new(0.0, -5.0), 1.0).expect("mass fixture is valid"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(mass_shape, 1.0, 0.0, 0.0, false, FilterData::default())
                .expect("mass fixture is valid"),
        )
        .expect("mass fixture fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let _particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits")
        .created_particle();

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("off-center coupling succeeds");

    // Assert
    assert_ne!(
        world
            .body_snapshot(body)
            .expect("body remains live")
            .angular_velocity()
            .to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn step_listener_limit_rolls_back_particle_contacts_and_rigid_reaction() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_LISTENER),
        )
        .expect("particle fits")
        .created_particle();
    let zero_events = StepLimits::new(0, StepLimits::default().max_commands())
        .expect("zero event limit is valid");

    // Act
    let result = world.step(step_configuration(), &mut NoDecisionHook, zero_events);

    // Assert
    assert_eq!(
        result,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 0,
        })
    );
    assert_eq!(
        world
            .particle_snapshot(particle)
            .expect("particle remains live")
            .velocity(),
        Vec2::ZERO
    );
    assert_eq!(
        world
            .particle_system_view(system)
            .expect("system remains live")
            .body_contacts()
            .count(),
        0
    );
}
