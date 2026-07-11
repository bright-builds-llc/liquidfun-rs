use serde_json::Value;

use super::{
    check_tolerance_profile_presentation, render_protocol_schema, render_scenario_schema,
    render_tolerance_profile_presentation, render_trace_schema,
};

const TRACKED_TOLERANCE_PROFILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/tolerances/phase2-v1.toml"
));
const TRACKED_PROTOCOL_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/schemas/protocol-v1.schema.json"
));
const TRACKED_SCENARIO_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/schemas/scenario-v1.schema.json"
));
const TRACKED_TRACE_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/schemas/trace-v1.schema.json"
));

#[test]
fn schema_presentations_are_byte_stable_and_newline_terminated() {
    // Arrange and Act
    let presentations = [
        (render_protocol_schema(), TRACKED_PROTOCOL_SCHEMA),
        (render_scenario_schema(), TRACKED_SCENARIO_SCHEMA),
        (render_trace_schema(), TRACKED_TRACE_SCHEMA),
    ];

    // Assert
    for (rendered, tracked) in presentations {
        assert_eq!(rendered, tracked);
        assert!(rendered.ends_with('\n'));
    }
}

#[test]
fn schema_presentations_keep_records_closed_and_versions_explicit() {
    // Arrange
    let schemas = [
        TRACKED_PROTOCOL_SCHEMA,
        TRACKED_SCENARIO_SCHEMA,
        TRACKED_TRACE_SCHEMA,
    ];

    // Act
    let parsed = schemas.map(|schema| {
        serde_json::from_str::<Value>(schema).expect("tracked schema must be valid JSON")
    });

    // Assert
    for schema in &parsed {
        assert_closed_records(schema);
    }
    assert!(TRACKED_PROTOCOL_SCHEMA.contains("\"protocol_version\": 1"));
    assert!(TRACKED_PROTOCOL_SCHEMA.contains("\"scenario_schema_version\": 1"));
    assert!(TRACKED_PROTOCOL_SCHEMA.contains("\"trace_schema_version\": 1"));
    assert!(TRACKED_PROTOCOL_SCHEMA.contains("\"tolerance_profile_version\": 1"));
    assert!(TRACKED_PROTOCOL_SCHEMA.contains("\"collision_probe_request\""));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"scenario_schema_version\": 1"));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"math.rotation\""));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"fma_witness\""));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"collision.distance.result\""));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"time_of_impact\""));
    assert!(TRACKED_SCENARIO_SCHEMA.contains("\"collision_probe_result\""));
    assert!(TRACKED_TRACE_SCHEMA.contains("\"trace_schema_version\": 1"));
    assert!(TRACKED_TRACE_SCHEMA.contains("\"math_probe_end\""));
    assert!(TRACKED_TRACE_SCHEMA.contains("\"initial_fraction\""));
    assert!(schemas.iter().all(|schema| schema.contains(
        "Typed Rust and C++ validation remains authoritative for cross-field references"
    )));
}

#[test]
fn tolerance_profile_presentation_is_strict_and_byte_stable() {
    // Arrange
    let unsupported_version = TRACKED_TOLERANCE_PROFILE.replacen("version = 1", "version = 2", 1);
    let mismatched_hash = TRACKED_TOLERANCE_PROFILE.replacen(
        "177db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3",
        "077db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3",
        1,
    );
    let unknown_field = format!("unknown = true\n{TRACKED_TOLERANCE_PROFILE}");
    let decimal_threshold =
        TRACKED_TOLERANCE_PROFILE.replacen("max_bits = 1065353216", "max_bits = 1.0", 1);
    let duplicate_policy = format!(
        "{TRACKED_TOLERANCE_PROFILE}\n[[float_policies]]\nfield = \"simulation_time\"\nscope = \"phase2_trace\"\npolicy = {{ kind = \"exact_bits\" }}\n"
    );

    // Act
    let rendered = render_tolerance_profile_presentation();
    let strict_results = [
        check_tolerance_profile_presentation(&unsupported_version),
        check_tolerance_profile_presentation(&mismatched_hash),
        check_tolerance_profile_presentation(&unknown_field),
        check_tolerance_profile_presentation(&decimal_threshold),
        check_tolerance_profile_presentation(&duplicate_policy),
    ];

    // Assert
    assert_eq!(rendered, TRACKED_TOLERANCE_PROFILE);
    assert!(rendered.ends_with('\n'));
    assert!(rendered.contains("field = \"simulation_time\""));
    assert!(rendered.contains("kind = \"exact_bits\""));
    assert!(rendered.contains("max_bits = 1065353216"));
    assert!(rendered.contains("absolute_bits = 1065353216"));
    assert!(rendered.contains("relative_bits = 1048576000"));
    assert!(strict_results.into_iter().all(|result| result.is_err()));
}

#[test]
fn tolerance_profile_presentation_matches_typed_authority() {
    // Arrange and Act
    let result = check_tolerance_profile_presentation(TRACKED_TOLERANCE_PROFILE);

    // Assert
    assert!(result.is_ok());
}

fn assert_closed_records(value: &Value) {
    if value.get("type").and_then(Value::as_str) == Some("object") {
        assert_eq!(
            value.get("additionalProperties").and_then(Value::as_bool),
            Some(false)
        );
    }
    match value {
        Value::Array(items) => {
            for item in items {
                assert_closed_records(item);
            }
        }
        Value::Object(fields) => {
            for field in fields.values() {
                assert_closed_records(field);
            }
        }
        _ => {}
    }
}
