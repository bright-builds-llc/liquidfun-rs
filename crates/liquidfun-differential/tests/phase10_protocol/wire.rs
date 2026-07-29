#[test]
fn wire_phase10_request_normalizes_to_byte_identical_canonical_json() {
    // Arrange
    let value = phase10_request_value();
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect("complete Phase 10 request should decode");

    // Act
    let canonical = encode_jsonl(&request, &limits, RecordLimit::Input)
        .expect("validated request should encode");
    let replay = decode_rigid_world_request_jsonl(&canonical, &limits)
        .expect("canonical request should decode");
    let replayed =
        encode_jsonl(&replay, &limits, RecordLimit::Input).expect("replay should encode");

    // Assert
    assert_eq!(replayed, canonical);
}

#[test]
fn tracked_schemas_accept_complete_phase10_request_and_result() {
    // Arrange
    let request_value = phase10_request_value();
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&encode_value(&request_value), &limits)
        .expect("complete Phase 10 request should decode");
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("complete Phase 10 request should run");
    let scenario_schema: Value =
        serde_json::from_slice(SCENARIO_SCHEMA).expect("scenario schema should be valid JSON");
    let trace_schema: Value =
        serde_json::from_slice(TRACE_SCHEMA).expect("trace schema should be valid JSON");
    let scenario_validator =
        jsonschema::validator_for(&scenario_schema).expect("scenario schema should compile");
    let trace_validator =
        jsonschema::validator_for(&trace_schema).expect("trace schema should compile");
    let result_value = serde_json::to_value(result).expect("Phase 10 result should encode");

    // Act
    let scenario_validation = scenario_validator.validate(&request_value["scenario"]);
    let trace_validation = trace_validator.validate(&result_value);

    // Assert
    assert!(
        scenario_validation.is_ok(),
        "complete Phase 10 request should match the tracked scenario schema: {scenario_validation:?}"
    );
    assert!(
        trace_validation.is_ok(),
        "complete Phase 10 result should match the tracked trace schema: {trace_validation:?}"
    );
}

#[test]
fn wire_rejects_unknown_duplicate_private_and_unknown_tag_members() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut private = phase10_request_value();
    phase10_create_definition_mut(&mut private)["pass_id"] = json!("s19");
    let mut unknown_tag = phase10_request_value();
    unknown_tag["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-inspect")
        .expect("inspect action should exist")["action"]["operation"]["kind"] = json!("pass_trace");
    let canonical = encode_value(&phase10_request_value());
    let canonical = String::from_utf8(canonical).expect("fixture should be UTF-8");
    let duplicate = canonical.replacen(
        "\"extension_version\":1",
        "\"extension_version\":1,\"extension_version\":1",
        1,
    );

    // Act
    let private_error = decode_rigid_world_request_jsonl(&encode_value(&private), &limits)
        .expect_err("private pass identity must be rejected");
    let tag_error = decode_rigid_world_request_jsonl(&encode_value(&unknown_tag), &limits)
        .expect_err("unknown operation tag must be rejected");
    let duplicate_error = decode_rigid_world_request_jsonl(duplicate.as_bytes(), &limits)
        .expect_err("duplicate member must be rejected");

    // Assert
    assert_eq!(
        codec_kind(&private_error),
        Some(CodecErrorKind::UnknownField)
    );
    assert_eq!(
        codec_kind(&tag_error),
        Some(CodecErrorKind::UnknownRecordKind)
    );
    assert_eq!(
        codec_kind(&duplicate_error),
        Some(CodecErrorKind::DuplicateMember)
    );
}

#[test]
fn wire_rejects_duplicate_ids_wrong_ownership_flags_versions_and_nonfinite_values() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut duplicate = phase10_request_value();
    phase10_create_definition_mut(&mut duplicate)["member_ids"] =
        json!(["particle-a", "particle-a"]);
    let mut wrong_owner = phase10_request_value();
    let append = wrong_owner["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-append-a")
        .expect("append action should exist");
    append["action"]["operation"]["definition"]["group_id"] = json!("group-other");
    let mut private_flags = phase10_request_value();
    phase10_create_definition_mut(&mut private_flags)["group_flags_bits"] = json!(0x0018);
    let mut wrong_version = phase10_request_value();
    phase10_create_definition_mut(&mut wrong_version)["provenance"]["extension_version"] = json!(2);
    let mut nonfinite = phase10_request_value();
    phase10_create_definition_mut(&mut nonfinite)["transform"]["angle_bits"] =
        json!(f32::NAN.to_bits());

    // Act
    let results = [
        duplicate,
        wrong_owner,
        private_flags,
        wrong_version,
        nonfinite,
    ]
    .map(|value| decode_rigid_world_request_jsonl(&encode_value(&value), &limits));

    // Assert
    assert!(results.into_iter().all(|result| {
        result
            .expect_err("malformed Phase 10 semantic input must fail")
            .rigid_world_kind()
            == Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    }));
}

#[test]
fn wire_particle_boundary_accepts_limit_and_rejects_one_over() {
    // Arrange
    let mut at_limit = definition();
    at_limit.member_ids = (0..liquidfun_test_protocol::PHASE10_MAXIMUM_PARTICLES)
        .map(|index| id(&format!("particle-{index}")))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    at_limit.source = Phase10GroupSource::Filled {
        shapes: vec![Phase10Shape::Circle {
            center: vector(0.0, 0.0),
            radius_bits: bits(1.0),
        }]
        .into_boxed_slice(),
    };
    let at_limit = Phase10Operation::CreateGroup {
        definition: at_limit.clone(),
    };
    let mut over_definition = at_limit_definition(&at_limit);
    over_definition.member_ids = (0..=liquidfun_test_protocol::PHASE10_MAXIMUM_PARTICLES)
        .map(|index| id(&format!("over-particle-{index}")))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let one_over = Phase10Operation::CreateGroup {
        definition: over_definition,
    };

    // Act
    let accepted = validate_phase10_operation(&at_limit);
    let rejected = validate_phase10_operation(&one_over);

    // Assert
    assert_eq!(accepted, Ok(()));
    assert_eq!(
        rejected.map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::BoundaryLimitExceeded)
    );
}

#[test]
fn wire_group_identity_boundary_counts_destroyed_groups_cumulatively() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut at_limit = phase10_request_value();
    add_split_group_identities(&mut at_limit, 59);
    add_transient_created_groups(&mut at_limit, 1);
    let mut one_over = phase10_request_value();
    add_split_group_identities(&mut one_over, 60);
    add_transient_created_groups(&mut one_over, 1);

    // Act
    let accepted = decode_rigid_world_request_jsonl(&encode_value(&at_limit), &limits);
    let rejected = decode_rigid_world_request_jsonl(&encode_value(&one_over), &limits);

    // Assert
    assert!(accepted.is_ok());
    assert_eq!(
        rejected
            .expect_err("the sixty-fifth declared group must be rejected")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    );
}

#[test]
fn wire_group_identity_boundary_counts_multiple_splits_cumulatively() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut at_limit = phase10_request_value();
    add_split_group_identities(&mut at_limit, 60);
    let mut one_over = phase10_request_value();
    add_split_group_identities(&mut one_over, 61);

    // Act
    let accepted = decode_rigid_world_request_jsonl(&encode_value(&at_limit), &limits);
    let rejected = decode_rigid_world_request_jsonl(&encode_value(&one_over), &limits);

    // Assert
    assert!(accepted.is_ok());
    assert_eq!(
        rejected
            .expect_err("the sixty-fifth split identity must be rejected")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    );
}

#[test]
fn wire_inspection_requires_established_phase10_provenance() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut inspection_only = phase10_request_value();
    inspection_only["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .retain(|action| {
            action["action"]["kind"] != "particle_group" || action["action_id"] == "p10-inspect"
        });

    // Act
    let rejected = decode_rigid_world_request_jsonl(&encode_value(&inspection_only), &limits);
    let phase8_control = decode_rigid_world_request_jsonl(PHASE8_REQUEST, &limits);

    // Assert
    assert_eq!(
        rejected
            .expect_err("inspection without Phase 10 provenance must fail during decoding")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    );
    assert!(phase8_control.is_ok());
}
