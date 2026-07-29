//! Closed Phase 9 particle protocol contracts.

use liquidfun_differential::{
    NativeRigidWorldExecutor, PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS,
    Phase9ComparisonOutcome, Phase9PolicyKind, compare_phase9_particle_observations,
    compare_phase9_rigid_world_results, phase9_policy_for_path, validate_phase9_policy_registry,
};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase9ParticleObservation, Phase9ParticleSnapshot,
    RigidWorldErrorKind, RigidWorldRequestRecord, RigidWorldResultRecord, ScenarioId, Vec2Bits,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_rigid_world_result_against_request,
};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
include!("particle_protocol/fixtures.rs");
include!("particle_protocol/result_validation.rs");
include!("particle_protocol/request_validation.rs");
include!("particle_protocol/codec.rs");
include!("particle_protocol/comparator.rs");
