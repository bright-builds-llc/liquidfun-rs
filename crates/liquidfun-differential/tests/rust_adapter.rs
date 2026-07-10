//! Native empty-world adapter contract tests.

use liquidfun_differential::EmptyWorldAdapter;
use liquidfun_test_protocol::{
    EngineKind, HarnessLimits, ScenarioRequestRecord, WorldCounts, decode_scenario_request_jsonl,
};

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-request.jsonl");

fn fixture_request() -> ScenarioRequestRecord {
    decode_scenario_request_jsonl(REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("checked-in request should validate")
}

#[test]
fn adapter_emits_ordered_exact_empty_world_trace() {
    // Arrange
    let request = fixture_request();
    let mut adapter =
        EmptyWorldAdapter::new(ORACLE_REVISION).expect("native identity should validate");

    // Act
    let trace = adapter
        .execute(&request)
        .expect("validated empty-world request should execute");

    // Assert
    assert_eq!(trace.engine_kind(), EngineKind::NativeRust);
    assert_eq!(trace.scenario_id().as_str(), "empty-world");
    assert_eq!(trace.request_id(), request.request_id());
    assert_eq!(trace.checkpoints().len(), 2);
    assert_eq!(
        trace
            .checkpoints()
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id().as_str())
            .collect::<Vec<_>>(),
        ["after-step-1", "after-step-2"]
    );
    assert_eq!(
        trace
            .checkpoints()
            .iter()
            .map(|checkpoint| checkpoint.simulation_time_bits().bits())
            .collect::<Vec<_>>(),
        [0.5_f32.to_bits(), 1.0_f32.to_bits()]
    );
    assert!(
        trace
            .checkpoints()
            .iter()
            .all(|checkpoint| checkpoint.world_counts() == WorldCounts::zero())
    );
}

#[test]
fn adapter_binds_native_identity_and_request_hashes() {
    // Arrange
    let request = fixture_request();
    let mut adapter =
        EmptyWorldAdapter::new(ORACLE_REVISION).expect("native identity should validate");

    // Act
    let trace = adapter
        .execute(&request)
        .expect("validated empty-world request should execute");

    // Assert
    assert_eq!(adapter.build_identity().oracle_revision(), ORACLE_REVISION);
    assert_eq!(adapter.build_identity().cmake_preset(), "native-rust");
    assert_eq!(
        adapter.build_identity().adapter_revision(),
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(
        trace.identity_sha256(),
        adapter.build_identity().identity_sha256()
    );
    assert_eq!(
        trace.tolerance_profile_sha256(),
        request.tolerance_profile_sha256()
    );
    assert_eq!(
        trace.scenario_sha256().as_str(),
        "49642b2ea489384be7850f595269e6366003f7bfab260ab1f9270a9cfcb0fd9e"
    );
}

#[test]
fn adapter_destroys_per_request_state_and_advances_reset_epoch() {
    // Arrange
    let request = fixture_request();
    let mut adapter =
        EmptyWorldAdapter::new(ORACLE_REVISION).expect("native identity should validate");

    // Act
    let first = adapter
        .execute(&request)
        .expect("first request should execute");
    let second = adapter
        .execute(&request)
        .expect("second request should execute independently");

    // Assert
    assert_eq!(first.reset_epoch(), 1);
    assert_eq!(second.reset_epoch(), 2);
    assert_eq!(first.checkpoints(), second.checkpoints());
}

#[test]
fn invalid_nonempty_scenario_never_reaches_adapter() {
    // Arrange
    let bytes = String::from_utf8(REQUEST_BYTES.to_vec())
        .expect("fixture is UTF-8")
        .replace("\"entities\":[]", "\"entities\":[{}]");

    // Act
    let result =
        decode_scenario_request_jsonl(bytes.as_bytes(), &HarnessLimits::phase2_default_v1());

    // Assert
    assert!(result.is_err());
}
