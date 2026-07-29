#[test]
fn request_accepts_every_finite_infinite_lifetime_bit_pattern() {
    // Arrange
    let finite_infinite_bits = [
        (-1.0_f32).to_bits(),
        (-0.0_f32).to_bits(),
        0.0_f32.to_bits(),
    ];

    // Act
    let decoded = finite_infinite_bits.map(|lifetime_bits| {
        let mut value = phase9_lifecycle_value();
        value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] = json!(lifetime_bits);
        decode_value(&value).map(|request| {
            request.scenario().timelines()[0].particles()[0]
                .lifetime_bits
                .bits()
        })
    });

    // Assert
    assert_eq!(decoded, finite_infinite_bits.map(Ok));
}

#[test]
fn request_rejects_nonfinite_particle_lifetimes() {
    // Arrange
    let nonfinite_bits = [
        f32::NAN.to_bits(),
        f32::INFINITY.to_bits(),
        f32::NEG_INFINITY.to_bits(),
    ];

    // Act
    let results = nonfinite_bits.map(|lifetime_bits| {
        let mut value = phase9_lifecycle_value();
        value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] = json!(lifetime_bits);
        decode_value(&value)
    });

    // Assert
    assert!(results.iter().all(Result::is_err));
}

#[test]
fn request_rejects_duplicate_particle_system_creation() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-create-system-a",
        "phase9-create-system-a-again",
        json!({ "kind": "create_system", "system_id": "phase9-system-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_particle_creation_before_owner_system() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let system_index = actions
        .iter()
        .position(|record| record["action_id"] == "phase9-create-system-a")
        .expect("system creation should exist");
    let particle_index = actions
        .iter()
        .position(|record| record["action_id"] == "phase9-create-particle-a")
        .expect("particle creation should exist");
    actions.swap(system_index, particle_index);

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_duplicate_particle_creation() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-create-particle-a",
        "phase9-create-particle-a-again",
        json!({ "kind": "create_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_unknown_particle_use() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    phase9_action_mut(&mut value, "phase9-inspect-particle-a")["action"]["action"]["particle_id"] =
        json!("unknown-particle");

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_pending_particle_use_and_repeated_mark() {
    // Arrange
    let mut pending_use = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut pending_use,
        "phase9-mark-a",
        "phase9-inspect-pending-a",
        json!({ "kind": "inspect_particle", "particle_id": "phase9-particle-a" }),
    );
    let mut repeated_mark = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut repeated_mark,
        "phase9-mark-a",
        "phase9-mark-a-again",
        json!({ "kind": "mark_for_destruction", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&pending_use);
    assert_invalid_particle_action(&repeated_mark);
}

#[test]
fn request_rejects_particle_recreation_after_compaction() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-compact-a",
        "phase9-recreate-particle-a",
        json!({ "kind": "create_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_particle_use_after_owner_system_destruction() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-destroy-system-a",
        "phase9-inspect-destroyed-a",
        json!({ "kind": "inspect_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_cross_system_particle_range() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    phase9_action_mut(&mut value, "phase9-force-a")["action"]["action"]["particle_ids"] =
        json!(["phase9-particle-a", "phase9-particle-b"]);

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_destroyed_or_unknown_query_owner() {
    // Arrange
    let mut destroyed = phase9_lifecycle_value();
    phase9_action_mut(&mut destroyed, "phase9-query-b")["action"]["action"]["system_id"] =
        json!("phase9-system-a");
    let mut unknown = phase9_lifecycle_value();
    phase9_action_mut(&mut unknown, "phase9-query-b")["action"]["action"]["system_id"] =
        json!("unknown-system");

    // Act / Assert
    assert_invalid_particle_action(&destroyed);
    assert_invalid_particle_action(&unknown);
}
