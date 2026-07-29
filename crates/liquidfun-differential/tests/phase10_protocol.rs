//! Strict shared Phase 10 particle-group protocol contracts.

#[path = "phase10_protocol/lifecycle_validation.rs"]
mod lifecycle_validation;

use liquidfun_differential::NativeRigidWorldExecutor;
use liquidfun_test_protocol::{
    CodecErrorKind, FloatBits, HarnessLimits, Phase10BehaviorLeaf, Phase10GroupDefinition,
    Phase10GroupDestination, Phase10GroupSource, Phase10Observation, Phase10Operation,
    Phase10PairSnapshot, Phase10Provenance, Phase10SemanticOutcome, Phase10Shape,
    Phase10StateObservation, Phase10TriadSnapshot, Phase10ValidationKind, Phase10Witness,
    Phase10WitnessObservation, RecordLimit, RigidWorldDecodeError, ScenarioId, TransformBits,
    Vec2Bits, WitnessRole, decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    encode_jsonl, validate_phase10_operation,
};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const PHASE9_REQUEST: &[u8] =
    include_bytes!("fixtures/rigid_world/phase9/cases/storage-systems-and-permutations.jsonl");
const SCENARIO_SCHEMA: &[u8] = include_bytes!("../../../protocol/schemas/scenario-v1.schema.json");
const TRACE_SCHEMA: &[u8] = include_bytes!("../../../protocol/schemas/trace-v1.schema.json");
include!("phase10_protocol/setup.rs");
include!("phase10_protocol/semantic.rs");
include!("phase10_protocol/wire.rs");
include!("phase10_protocol/compatibility.rs");
