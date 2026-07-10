use super::{ScenarioDecodeError, ScenarioErrorKind, decode_scenario_request_jsonl};
use crate::{CodecErrorKind, FloatBits, HarnessLimits, RecordLimit, encode_jsonl};

fn step(command_id: &str, timestep_bits: u32) -> String {
    format!(
        "{{\"kind\":\"step\",\"command_id\":\"{command_id}\",\"timestep_bits\":{timestep_bits},\"velocity_iterations\":8,\"position_iterations\":3,\"particle_iterations\":1}}"
    )
}

fn checkpoint(checkpoint_id: &str, command_id: &str, phase: &str, observables: &str) -> String {
    format!(
        "{{\"checkpoint_id\":\"{checkpoint_id}\",\"after_command_id\":\"{command_id}\",\"phase\":\"{phase}\",\"observables\":[{observables}]}}"
    )
}

fn record_with(
    scenario_id: &str,
    source: &str,
    entities: &str,
    commands: &str,
    checkpoints: &str,
) -> String {
    format!(
        concat!(
            "{{\"protocol_version\":1,\"record_kind\":\"scenario_request\",",
            "\"request_id\":\"request-1\",\"scenario_schema_version\":1,",
            "\"requested_trace_schema_version\":1,\"tolerance_profile_version\":1,",
            "\"tolerance_profile_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
            "\"scenario\":{{\"scenario_id\":\"{}\",\"source\":{},",
            "\"gravity_x_bits\":2147483648,\"gravity_y_bits\":2143289410,",
            "\"entities\":[{}],\"commands\":[{}],\"checkpoints\":[{}]}}}}\n"
        ),
        scenario_id, source, entities, commands, checkpoints
    )
}

fn valid_record() -> String {
    let commands = [step("step-1", 100), step("step-2", 200)].join(",");
    let checkpoints = [
        checkpoint(
            "checkpoint-1",
            "step-1",
            "after-first-step",
            "\"world_counts\",\"simulation_time\"",
        ),
        checkpoint(
            "checkpoint-2",
            "step-2",
            "after-second-step",
            "\"world_counts\",\"simulation_time\"",
        ),
    ]
    .join(",");
    record_with(
        "empty-world",
        "{\"kind\":\"seeded\",\"generator_id\":\"phase2\",\"generator_version\":1,\"seed\":42}",
        "",
        &commands,
        &checkpoints,
    )
}

fn codec_kind(error: &ScenarioDecodeError) -> Option<CodecErrorKind> {
    match error {
        ScenarioDecodeError::Codec(error) => Some(error.kind()),
        ScenarioDecodeError::Validation(_) => None,
    }
}

#[test]
fn scenario_decodes_named_or_seeded_ordered_steps_with_exact_float_bits() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let seeded = valid_record();
    let named = seeded.replace(
        "{\"kind\":\"seeded\",\"generator_id\":\"phase2\",\"generator_version\":1,\"seed\":42}",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
    );

    // Act
    let seeded_request = decode_scenario_request_jsonl(seeded.as_bytes(), &limits)
        .expect("seeded scenario should decode");
    let named_request = decode_scenario_request_jsonl(named.as_bytes(), &limits)
        .expect("named scenario should decode");

    // Assert
    assert_eq!(
        seeded_request.scenario().gravity_x_bits(),
        FloatBits::new(0x8000_0000)
    );
    assert_eq!(
        seeded_request.scenario().gravity_y_bits(),
        FloatBits::new(0x7fc0_0042)
    );
    assert_eq!(seeded_request.scenario().commands().len(), 2);
    assert_eq!(named_request.scenario().checkpoints().len(), 2);
}

#[test]
fn scenario_encode_decode_preserves_signed_zero_and_nan_payload_bits() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_scenario_request_jsonl(valid_record().as_bytes(), &limits)
        .expect("fixture should decode");

    // Act
    let encoded = encode_jsonl(&request, &limits, RecordLimit::Input)
        .expect("validated request should encode");
    let decoded =
        decode_scenario_request_jsonl(&encoded, &limits).expect("encoded request should decode");

    // Assert
    assert_eq!(decoded.scenario().gravity_x_bits().bits(), 0x8000_0000);
    assert_eq!(decoded.scenario().gravity_y_bits().bits(), 0x7fc0_0042);
}

#[test]
fn scenario_rejects_duplicate_ids_and_bad_checkpoint_references() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let duplicate_command =
        valid_record().replace("\"command_id\":\"step-2\"", "\"command_id\":\"step-1\"");
    let duplicate_checkpoint = valid_record().replace(
        "\"checkpoint_id\":\"checkpoint-2\"",
        "\"checkpoint_id\":\"checkpoint-1\"",
    );
    let bad_reference = valid_record().replace(
        "\"after_command_id\":\"step-2\"",
        "\"after_command_id\":\"missing\"",
    );

    // Act
    let command_error = decode_scenario_request_jsonl(duplicate_command.as_bytes(), &limits)
        .expect_err("duplicate command ID should fail");
    let checkpoint_error = decode_scenario_request_jsonl(duplicate_checkpoint.as_bytes(), &limits)
        .expect_err("duplicate checkpoint ID should fail");
    let reference_error = decode_scenario_request_jsonl(bad_reference.as_bytes(), &limits)
        .expect_err("unknown command reference should fail");

    // Assert
    assert_eq!(
        command_error.scenario_kind(),
        Some(ScenarioErrorKind::DuplicateCommandId)
    );
    assert_eq!(
        checkpoint_error.scenario_kind(),
        Some(ScenarioErrorKind::DuplicateCheckpointId)
    );
    assert_eq!(
        reference_error.scenario_kind(),
        Some(ScenarioErrorKind::UnknownCommandReference)
    );
}

#[test]
fn scenario_rejects_unknown_duplicate_and_unsupported_boundary_values() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let unknown = valid_record().replace(
        "\"scenario_id\":\"empty-world\"",
        "\"scenario_id\":\"empty-world\",\"mystery\":1",
    );
    let duplicate = valid_record().replace(
        "\"gravity_x_bits\":2147483648",
        "\"gravity_x_bits\":2147483648,\"gravity_x_bits\":0",
    );
    let unsupported = valid_record().replace("\"protocol_version\":1", "\"protocol_version\":2");
    let unknown_kind = valid_record().replace(
        "\"record_kind\":\"scenario_request\"",
        "\"record_kind\":\"mystery\"",
    );

    // Act
    let errors = [unknown, duplicate, unsupported, unknown_kind].map(|record| {
        decode_scenario_request_jsonl(record.as_bytes(), &limits)
            .expect_err("closed boundary value should fail")
    });

    // Assert
    assert_eq!(codec_kind(&errors[0]), Some(CodecErrorKind::UnknownField));
    assert_eq!(
        codec_kind(&errors[1]),
        Some(CodecErrorKind::DuplicateMember)
    );
    assert_eq!(
        codec_kind(&errors[2]),
        Some(CodecErrorKind::UnsupportedVersion)
    );
    assert_eq!(
        codec_kind(&errors[3]),
        Some(CodecErrorKind::UnknownRecordKind)
    );
}

#[test]
fn scenario_enforces_id_and_general_string_limits_at_n_and_n_plus_one() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let accepted_id = valid_record().replace("empty-world", &"a".repeat(128));
    let rejected_id = valid_record().replace("empty-world", &"a".repeat(129));
    let accepted_phase = valid_record().replace("after-first-step", &"p".repeat(4_096));
    let rejected_phase = valid_record().replace("after-first-step", &"p".repeat(4_097));

    // Act
    let accepted = [accepted_id, accepted_phase]
        .map(|record| decode_scenario_request_jsonl(record.as_bytes(), &limits));
    let rejected = [rejected_id, rejected_phase].map(|record| {
        decode_scenario_request_jsonl(record.as_bytes(), &limits)
            .expect_err("N + 1 string should fail")
    });

    // Assert
    assert!(accepted.into_iter().all(|result| result.is_ok()));
    assert!(
        rejected
            .iter()
            .all(|error| codec_kind(error) == Some(CodecErrorKind::BoundaryLimitExceeded))
    );
}

#[test]
fn scenario_enforces_entity_and_command_collection_limits_at_n_and_n_plus_one() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let entities_n = vec!["{}"; 4_096].join(",");
    let entities_n_plus_one = vec!["{}"; 4_097].join(",");
    let commands_n = (0..4_096)
        .map(|index| step(&format!("step-{index}"), index))
        .collect::<Vec<_>>()
        .join(",");
    let commands_n_plus_one = format!("{commands_n},{}", step("step-extra", 4_096));
    let entity_n_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        &entities_n,
        &step("step-1", 1),
        "",
    );
    let entity_n_plus_one_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        &entities_n_plus_one,
        &step("step-1", 1),
        "",
    );
    let command_n_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &commands_n,
        "",
    );
    let command_n_plus_one_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &commands_n_plus_one,
        "",
    );

    // Act
    let entity_n_error = decode_scenario_request_jsonl(entity_n_record.as_bytes(), &limits)
        .expect_err("schema 1 should reject parsed nonempty entities");
    let entity_n_plus_one_error =
        decode_scenario_request_jsonl(entity_n_plus_one_record.as_bytes(), &limits)
            .expect_err("entity N + 1 should fail while decoding");
    let command_n_result = decode_scenario_request_jsonl(command_n_record.as_bytes(), &limits);
    let command_n_plus_one_error =
        decode_scenario_request_jsonl(command_n_plus_one_record.as_bytes(), &limits)
            .expect_err("command N + 1 should fail while decoding");

    // Assert
    assert_eq!(
        entity_n_error.scenario_kind(),
        Some(ScenarioErrorKind::EntityDefinitionsNotSupported)
    );
    assert_eq!(
        codec_kind(&entity_n_plus_one_error),
        Some(CodecErrorKind::BoundaryLimitExceeded)
    );
    assert!(command_n_result.is_ok());
    assert_eq!(
        codec_kind(&command_n_plus_one_error),
        Some(CodecErrorKind::BoundaryLimitExceeded)
    );
}

#[test]
fn scenario_enforces_checkpoint_and_observable_collection_limits_at_n_and_n_plus_one() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let checkpoints_n = (0..4_096)
        .map(|index| checkpoint(&format!("checkpoint-{index}"), "step-1", "phase", ""))
        .collect::<Vec<_>>()
        .join(",");
    let checkpoints_n_plus_one = format!(
        "{checkpoints_n},{}",
        checkpoint("checkpoint-extra", "step-1", "phase", "")
    );
    let observables_n = vec!["\"world_counts\""; 128].join(",");
    let observables_n_plus_one = vec!["\"world_counts\""; 129].join(",");
    let checkpoint_n_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &step("step-1", 1),
        &checkpoints_n,
    );
    let checkpoint_n_plus_one_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &step("step-1", 1),
        &checkpoints_n_plus_one,
    );
    let observable_n_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &step("step-1", 1),
        &checkpoint("checkpoint-1", "step-1", "phase", &observables_n),
    );
    let observable_n_plus_one_record = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        &step("step-1", 1),
        &checkpoint("checkpoint-1", "step-1", "phase", &observables_n_plus_one),
    );

    // Act
    let checkpoint_n_result =
        decode_scenario_request_jsonl(checkpoint_n_record.as_bytes(), &limits);
    let checkpoint_n_plus_one_error =
        decode_scenario_request_jsonl(checkpoint_n_plus_one_record.as_bytes(), &limits)
            .expect_err("checkpoint N + 1 should fail while decoding");
    let observable_n_error = decode_scenario_request_jsonl(observable_n_record.as_bytes(), &limits)
        .expect_err("duplicate observable should fail after N items parse");
    let observable_n_plus_one_error =
        decode_scenario_request_jsonl(observable_n_plus_one_record.as_bytes(), &limits)
            .expect_err("observable N + 1 should fail while decoding");

    // Assert
    assert!(checkpoint_n_result.is_ok());
    assert_eq!(
        codec_kind(&checkpoint_n_plus_one_error),
        Some(CodecErrorKind::BoundaryLimitExceeded)
    );
    assert_eq!(
        observable_n_error.scenario_kind(),
        Some(ScenarioErrorKind::DuplicateObservable)
    );
    assert_eq!(
        codec_kind(&observable_n_plus_one_error),
        Some(CodecErrorKind::BoundaryLimitExceeded)
    );
}

#[test]
fn scenario_rejects_empty_steps_and_invalid_solver_iteration_counts() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let empty = record_with(
        "empty-world",
        "{\"kind\":\"named\",\"name\":\"empty-world\"}",
        "",
        "",
        "",
    );
    let zero = valid_record().replace("\"velocity_iterations\":8", "\"velocity_iterations\":0");
    let excessive =
        valid_record().replace("\"position_iterations\":3", "\"position_iterations\":256");

    // Act
    let errors = [empty, zero, excessive].map(|record| {
        decode_scenario_request_jsonl(record.as_bytes(), &limits)
            .expect_err("invalid step contract should fail")
    });

    // Assert
    assert_eq!(
        errors[0].scenario_kind(),
        Some(ScenarioErrorKind::NoCommands)
    );
    assert_eq!(
        errors[1].scenario_kind(),
        Some(ScenarioErrorKind::ZeroSolverIterations)
    );
    assert_eq!(
        errors[2].scenario_kind(),
        Some(ScenarioErrorKind::SolverIterationsExceeded)
    );
}
