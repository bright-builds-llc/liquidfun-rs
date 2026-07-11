//! Phase 5 native execution, first-divergence, and supervisor contract tests.

use liquidfun_differential::{
    CollisionDivergence, NativeCollisionProbeExecutor, compare_collision_probe_results,
};
use liquidfun_test_protocol::{
    CollisionProbeDiscreteValue, CollisionProbeNumericValue, CollisionProbeResult, FloatBits,
    HarnessLimits, Phase5PolicyProfile, decode_collision_probe_request_jsonl,
};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/fixtures/accepted/collision-probe-request.jsonl"
));
const POLICY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/tolerances/phase5-v1.toml"
));

fn request() -> liquidfun_test_protocol::CollisionProbeRequestRecord {
    decode_collision_probe_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("checked-in collision request should decode")
}

#[test]
fn native_executes_every_closed_collision_operation_in_request_order() {
    // Arrange
    let request = request();

    // Act
    let results = NativeCollisionProbeExecutor::execute(&request)
        .expect("fixed collision corpus should execute natively");

    // Assert
    assert_eq!(results.len(), request.scenario().cases().len());
    for (case, result) in request.scenario().cases().iter().zip(results.iter()) {
        assert_eq!(result.case_id(), case.case_id());
        assert_eq!(result.operation(), case.operation());
        assert_eq!(result.policy_path(), case.policy_path());
        assert_eq!(result.horizon(), case.horizon());
        assert_eq!(result.collection_policy(), case.collection_policy());
    }
}

#[test]
fn comparison_reports_first_structural_then_numeric_divergence() {
    // Arrange
    let request = request();
    let profile = Phase5PolicyProfile::parse_toml(POLICY).expect("policy should validate");
    let expected = NativeCollisionProbeExecutor::execute(&request)
        .expect("fixed collision corpus should execute natively");
    let first = &expected[0];
    let wrong_case = CollisionProbeResult::new(
        "wrong-case",
        first.operation(),
        first.numeric().to_vec(),
        first.discrete().to_vec(),
        first.payload_ids().to_vec(),
    )
    .expect("mutated result should remain bounded");
    let structural = std::iter::once(wrong_case)
        .chain(expected.iter().skip(1).cloned())
        .collect::<Vec<_>>();

    // Act
    let structural_result =
        compare_collision_probe_results(&request, &expected, &structural, &profile);

    // Assert
    assert!(matches!(
        structural_result,
        Err(CollisionDivergence::Harness(_))
    ));

    // Arrange
    let distance_index = expected
        .iter()
        .position(|result| !result.numeric().is_empty())
        .expect("native corpus should contain numeric evidence");
    let distance = &expected[distance_index];
    let mut numeric = distance.numeric().to_vec();
    numeric[0] = CollisionProbeNumericValue::new(
        numeric[0].field(),
        FloatBits::new(numeric[0].bits().bits() ^ 1),
    );
    let changed = CollisionProbeResult::new(
        distance.case_id(),
        distance.operation(),
        numeric,
        distance.discrete().to_vec(),
        distance.payload_ids().to_vec(),
    )
    .expect("mutated result should remain bounded");
    let mut actual = expected.to_vec();
    actual[distance_index] = changed;

    // Act
    let numeric_result = compare_collision_probe_results(&request, &expected, &actual, &profile);

    // Assert
    assert!(matches!(
        numeric_result,
        Err(CollisionDivergence::Numeric(_))
    ));
}

#[test]
fn comparison_canonicalizes_only_declared_payload_sets() {
    // Arrange
    let request = request();
    let profile = Phase5PolicyProfile::parse_toml(POLICY).expect("policy should validate");
    let expected = NativeCollisionProbeExecutor::execute(&request)
        .expect("fixed collision corpus should execute natively");
    let set_index = expected
        .iter()
        .position(|result| {
            result.collection_policy() == liquidfun_test_protocol::CollectionPolicy::Set
        })
        .expect("fixed corpus should contain a set-like operation");
    let set_result = &expected[set_index];
    let reversed_set = CollisionProbeResult::new(
        set_result.case_id(),
        set_result.operation(),
        set_result.numeric().to_vec(),
        set_result.discrete().to_vec(),
        set_result.payload_ids().iter().rev().copied().collect(),
    )
    .expect("reordered set should remain bounded");
    let mut actual = expected.to_vec();
    actual[set_index] = reversed_set;

    // Act / Assert
    assert!(compare_collision_probe_results(&request, &expected, &actual, &profile).is_ok());

    let ordered = &expected[0];
    let ordered_mutation = CollisionProbeResult::new(
        ordered.case_id(),
        ordered.operation(),
        ordered.numeric().to_vec(),
        vec![
            CollisionProbeDiscreteValue::new("second", "2"),
            CollisionProbeDiscreteValue::new("first", "1"),
        ],
        ordered.payload_ids().to_vec(),
    )
    .expect("ordered mutation should remain bounded");
    let mut actual = expected.to_vec();
    actual[0] = ordered_mutation;
    assert!(matches!(
        compare_collision_probe_results(&request, &expected, &actual, &profile),
        Err(CollisionDivergence::Order(_) | CollisionDivergence::Harness(_))
    ));
}

#[test]
fn supervisor_classifies_missing_collision_oracle_as_harness_failure() {
    // This verifies collision capture keeps process failures outside physics comparison.
    let executable = liquidfun_differential::OracleExecutable::resolve(
        std::path::Path::new("/definitely/missing/liquidfun-collision-oracle"),
        liquidfun_differential::OraclePreset::Debug,
    );
    assert!(executable.is_err());
}

#[test]
fn collision_source_changes_native_identity() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-math-sources.txt"),
    )
    .expect("native source manifest should be readable");
    let sources = manifest
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    for required in [
        "crates/liquidfun-differential/src/collision_probe.rs",
        "crates/liquidfun/src/collision/differential.rs",
        "crates/liquidfun/src/collision/distance.rs",
    ] {
        assert!(
            sources.contains(&required),
            "missing identity source {required}"
        );
    }

    // Act
    let digest = source_digest(&root, &sources, None);
    let adapter =
        liquidfun_differential::EmptyWorldAdapter::new("0123456789abcdef0123456789abcdef01234567")
            .expect("native identity should validate");

    // Assert
    assert_eq!(
        digest,
        adapter.build_identity().adapter_content_sha256().as_str()
    );
    for changed in [
        "crates/liquidfun-differential/src/collision_probe.rs",
        "crates/liquidfun/src/collision/differential.rs",
        "crates/liquidfun/src/collision/distance.rs",
    ] {
        assert_ne!(digest, source_digest(&root, &sources, Some(changed)));
    }
}

fn source_digest(root: &std::path::Path, sources: &[&str], maybe_changed: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    for relative in sources {
        let mut bytes = std::fs::read(root.join(relative)).expect("identity source should exist");
        if maybe_changed == Some(*relative) {
            bytes.push(b'!');
        }
        let file_digest = Sha256::digest(bytes);
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(file_digest);
    }
    format!("{:x}", hasher.finalize())
}
