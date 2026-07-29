#[test]
fn phase9_policy_registry_fails_closed_without_wildcards() {
    // Arrange
    let reviewed = [
        ("particle.storage.identity", Phase9PolicyKind::ExactDiscrete),
        ("particle.configuration.bits", Phase9PolicyKind::ExactBits),
        ("particle.position", Phase9PolicyKind::Ulps),
        (
            "particle.contact.weight",
            Phase9PolicyKind::AbsoluteRelative,
        ),
        (
            "particle.ray.fraction",
            Phase9PolicyKind::DimensionedAbsolute,
        ),
    ];

    // Act
    let actual = reviewed.map(|(path, _)| phase9_policy_for_path(path));

    // Assert
    assert_eq!(PHASE9_REGISTRY_ID, "phase9-v1");
    assert_eq!(actual, reviewed.map(|(_, policy)| Some(policy)));
    assert!(
        PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .all(|path| phase9_policy_for_path(path).is_some())
    );
    assert_eq!(phase9_policy_for_path("particle.*"), None);
    assert_eq!(phase9_policy_for_path("particle.group.topology"), None);
    assert_eq!(phase9_policy_for_path("particle.pair.generation"), None);
    assert_eq!(phase9_policy_for_path("particle.solver.baseline"), None);
}

#[test]
fn comparator_consumes_the_complete_closed_phase9_policy_registry() {
    // Arrange
    let request = phase9_result_request();
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("native Phase 9 request should execute");

    // Act
    let outcome = compare_phase9_rigid_world_results(&request, &native, &native)
        .expect("the closed policy registry should be valid");

    // Assert
    let Phase9ComparisonOutcome::Match { consumed_paths } = outcome else {
        panic!("identical results must match");
    };
    assert_eq!(consumed_paths.as_ref(), PHASE9_REQUIRED_POLICY_PATHS);
}

#[test]
fn comparator_reports_stable_first_exact_policy_path() {
    // Arrange
    let request = phase9_result_request();
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("native Phase 9 request should execute");
    let mut mutated_value = result_value(&native);
    let observations = phase9_observations_mut(&mut mutated_value);
    let statistics = particle_observation_mut(observations, "statistics");
    statistics["observation"]["statistics"]["particle_contact_count"] = json!(1);
    let mutated = decode_result_value(&mutated_value).expect("mutation should remain bounded");

    // Act
    let first = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("the comparator should classify semantic disagreement");
    let second = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("replay should classify semantic disagreement");

    // Assert
    let (
        Phase9ComparisonOutcome::PhysicsMismatch(first),
        Phase9ComparisonOutcome::PhysicsMismatch(second),
    ) = (first, second)
    else {
        panic!("the identity mutation must be a physics mismatch");
    };
    assert_eq!(first.semantic_path(), "particle.statistics.counts");
    assert_eq!(first.timeline_index(), 0);
    assert_eq!(first.checkpoint_index(), second.checkpoint_index());
    assert_eq!(first.observation_index(), second.observation_index());
    assert_eq!(first.signature_sha256(), second.signature_sha256());
}

#[test]
fn comparator_selects_each_reviewed_numeric_policy_class() {
    // Arrange
    let id = |value: &str| ScenarioId::new(value).expect("test ID should be valid");
    let snapshot = Phase9ParticleSnapshot {
        particle_id: id("particle"),
        system_id: id("system"),
        position: Vec2Bits {
            x_bits: FloatBits::new(1.0_f32.to_bits()),
            y_bits: FloatBits::new(0),
        },
        velocity: Vec2Bits {
            x_bits: FloatBits::new(0),
            y_bits: FloatBits::new(0),
        },
        flags_bits: 0,
        color: [0; 4],
        weight_bits: FloatBits::new(1.0_f32.to_bits()),
        force: Vec2Bits {
            x_bits: FloatBits::new(0),
            y_bits: FloatBits::new(0),
        },
        pending_destruction: false,
    };
    let expected = Phase9ParticleObservation::Particle {
        snapshot: snapshot.clone(),
    };
    let mut ulp_snapshot = snapshot.clone();
    ulp_snapshot.position.x_bits = FloatBits::new(1.0_f32.to_bits() + 5);
    let ulp = Phase9ParticleObservation::Particle {
        snapshot: ulp_snapshot,
    };
    let mut relative_snapshot = snapshot.clone();
    relative_snapshot.weight_bits = FloatBits::new(1.25_f32.to_bits());
    let relative = Phase9ParticleObservation::Particle {
        snapshot: relative_snapshot,
    };
    let expected_ray = Phase9ParticleObservation::RayCast {
        terminated: false,
        particle_ids: vec![id("particle")].into_boxed_slice(),
        fractions_bits: vec![FloatBits::new(0.5_f32.to_bits())].into_boxed_slice(),
    };
    let actual_ray = Phase9ParticleObservation::RayCast {
        terminated: false,
        particle_ids: vec![id("particle")].into_boxed_slice(),
        fractions_bits: vec![FloatBits::new(0.75_f32.to_bits())].into_boxed_slice(),
    };

    // Act
    let ulp = compare_phase9_particle_observations(&expected, &ulp)
        .expect("ULP observation should be structurally valid");
    let relative = compare_phase9_particle_observations(&expected, &relative)
        .expect("relative observation should be structurally valid");
    let dimensioned = compare_phase9_particle_observations(&expected_ray, &actual_ray)
        .expect("ray observation should be structurally valid");

    // Assert
    assert_eq!(ulp.expect_mismatch().semantic_path(), "particle.position");
    assert_eq!(
        relative.expect_mismatch().semantic_path(),
        "particle.contact.weight"
    );
    assert_eq!(
        dimensioned.expect_mismatch().semantic_path(),
        "particle.ray.fraction"
    );
}

#[test]
fn comparator_policy_registry_rejects_missing_duplicate_unknown_and_wildcard_paths() {
    // Arrange
    let missing = &PHASE9_REQUIRED_POLICY_PATHS[..PHASE9_REQUIRED_POLICY_PATHS.len() - 1];
    let mut duplicate = PHASE9_REQUIRED_POLICY_PATHS.to_vec();
    duplicate.push(PHASE9_REQUIRED_POLICY_PATHS[0]);
    let mut unknown = PHASE9_REQUIRED_POLICY_PATHS.to_vec();
    unknown.push("particle.group.topology");
    let mut wildcard = PHASE9_REQUIRED_POLICY_PATHS.to_vec();
    wildcard.push("particle.*");

    // Act / Assert
    assert!(validate_phase9_policy_registry(missing).is_err());
    assert!(validate_phase9_policy_registry(&duplicate).is_err());
    assert!(validate_phase9_policy_registry(&unknown).is_err());
    assert!(validate_phase9_policy_registry(&wildcard).is_err());
}
