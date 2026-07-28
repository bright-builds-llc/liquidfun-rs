#[cfg(feature = "differential-internals")]
#[test]
fn implicit_aggregate_mass_type_change_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("static body should fit");
    let density = f32::MAX / 4.0;
    let definition = high_density_sensor_circle_fixture(density);
    let first = world
        .create_fixture(body, &definition)
        .expect("first static fixture should fit");
    let second = world
        .create_fixture(body, &definition)
        .expect("second static fixture should fit");
    let contact_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(1.5, 0.0), 0.0, true)
                .expect("contact body definition should be valid"),
        )
        .expect("contact body should fit");
    world
        .create_fixture(contact_body, &circle_fixture())
        .expect("contact fixture should fit");
    let report = world
        .step(
            phase6_step_configuration(),
            &mut NoopHook,
            StepLimits::default(),
        )
        .expect("sensor contacts should be discovered");
    assert!(world.contact_count() > 0);
    assert!(!report.contact_transitions().is_empty());
    let body_before = world.body_snapshot(body).expect("body should remain live");
    let fixtures_before = [second, first].map(|fixture| {
        world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live")
    });
    let contacts_before = world.rigid_contact_diagnostics();
    let proxy_count_before = world.broad_phase_entry_count();

    // Act
    let result = world.set_body_type(body, BodyType::Dynamic);

    // Assert
    assert_eq!(
        result,
        Err(BodyTypeChangeError::InvalidAggregateMass(
            AggregateMassError::NonFiniteMass
        ))
    );
    let body_after = world.body_snapshot(body).expect("body should remain live");
    assert_eq!(body_after.body_type(), BodyType::Static);
    assert_mass_bits_equal(body_after, body_before);
    assert_eq!(world.rigid_contact_diagnostics(), contacts_before);
    assert!(world.rigid_drain_contact_transitions().is_empty());
    assert_eq!(world.broad_phase_entry_count(), proxy_count_before);
    assert_eq!(
        [second, first].map(|fixture| world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live")),
        fixtures_before
    );
    let records = world
        .destroy_body(body)
        .expect("body cascade should succeed");
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body { fixtures, .. }) if fixtures == &[second, first]
    ));
}

#[cfg(feature = "differential-internals")]
#[test]
fn implicit_aggregate_mass_fixture_destruction_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("dynamic body should fit");
    let zero_density = high_density_sensor_circle_fixture(0.0);
    let first = world
        .create_fixture(body, &zero_density)
        .expect("first fixture should fit");
    let second = world
        .create_fixture(body, &zero_density)
        .expect("second fixture should fit");
    let target = world
        .create_fixture(body, &zero_density)
        .expect("target fixture should fit");
    let density = f32::MAX / 4.0;
    for fixture in [first, second, target] {
        world
            .set_fixture_density(fixture, density)
            .expect("individual fixture mass should remain finite");
    }
    let contact_body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::new(1.5, 0.0), 0.0, true)
                .expect("contact body definition should be valid"),
        )
        .expect("contact body should fit");
    world
        .create_fixture(contact_body, &circle_fixture())
        .expect("contact fixture should fit");
    let report = world
        .step(
            phase6_step_configuration(),
            &mut NoopHook,
            StepLimits::default(),
        )
        .expect("sensor contacts should be discovered");
    assert!(world.contact_count() > 0);
    assert!(!report.contact_transitions().is_empty());
    let body_before = world.body_snapshot(body).expect("body should remain live");
    let fixtures_before = [target, second, first].map(|fixture| {
        world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live")
    });
    let contacts_before = world.rigid_contact_diagnostics();
    let proxy_count_before = world.broad_phase_entry_count();

    // Act
    let result = world.destroy_fixture(target);

    // Assert
    assert_eq!(
        result,
        Err(FixtureDestructionError::InvalidAggregateMass(
            AggregateMassError::NonFiniteMass
        ))
    );
    assert!(world.contains_fixture(target));
    assert_mass_bits_equal(
        world.body_snapshot(body).expect("body should remain live"),
        body_before,
    );
    assert_eq!(world.rigid_contact_diagnostics(), contacts_before);
    assert!(world.rigid_drain_contact_transitions().is_empty());
    assert_eq!(world.broad_phase_entry_count(), proxy_count_before);
    assert_eq!(
        [target, second, first].map(|fixture| world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live")),
        fixtures_before
    );
    let records = world
        .destroy_body(body)
        .expect("body cascade should succeed");
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body { fixtures, .. }) if fixtures == &[target, second, first]
    ));
}

#[test]
fn implicit_aggregate_mass_body_cascade_skips_reset() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("dynamic body should fit");
    let zero_density = high_density_circle_fixture(0.0);
    let first = world
        .create_fixture(body, &zero_density)
        .expect("first fixture should fit");
    let second = world
        .create_fixture(body, &zero_density)
        .expect("second fixture should fit");
    let density = f32::MAX / 4.0;
    for fixture in [first, second] {
        world
            .set_fixture_density(fixture, density)
            .expect("individual fixture mass should remain finite");
    }

    // Act
    let records = world
        .destroy_body(body)
        .expect("body cascade should succeed");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            liquidfun::DestroyedId::Fixture(second),
            liquidfun::DestroyedId::Fixture(first),
            liquidfun::DestroyedId::Body(body),
        ]
    );
}

#[test]
fn mutation_material_edits_preserve_exact_accepted_bits() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let friction = f32::from_bits(0x3e4c_cccd);
    let restitution = -0.0_f32;

    // Act
    world
        .set_fixture_friction(fixture, friction)
        .expect("friction should be accepted");
    world
        .set_fixture_restitution(fixture, restitution)
        .expect("restitution should be accepted");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.friction().to_bits(), friction.to_bits());
    assert_eq!(snapshot.restitution().to_bits(), restitution.to_bits());
}

#[test]
fn mutation_sensor_and_filter_update_fixture_state_without_entry_churn() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let filter = FilterData::new(0x0004, 0x00f0, -3);

    // Act
    world
        .set_fixture_sensor(fixture, true)
        .expect("fixture should remain live");
    world
        .set_fixture_filter(fixture, filter)
        .expect("fixture should remain live");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert!(snapshot.is_sensor());
    assert_eq!(snapshot.filter_data(), filter);
    assert_eq!(snapshot.broad_phase_entry_count(), 1);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn mutation_invalid_material_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let before = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Act
    let result = world.set_fixture_friction(fixture, f32::NAN);
    let after = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(
        result,
        Err(FixtureMutationError::InvalidValue(
            FixtureDefError::NonFiniteFriction
        ))
    );
    assert_eq!(after, before);
}
