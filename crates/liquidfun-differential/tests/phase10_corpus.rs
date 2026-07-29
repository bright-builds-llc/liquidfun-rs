//! Closed corpus and corruption coverage for Phase 10 evidence.

#[path = "support/coverage_observation.rs"]
mod coverage_observation;
#[path = "phase10_corpus/evidence_output.rs"]
mod evidence_output;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Component, Path, PathBuf},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE10_REQUIRED_POLICY_PATHS,
    Phase10ComparisonMode, Phase10ComparisonOutcome, Phase10EvidenceBinding, Phase10EvidenceLeaf,
    Phase10EvidencePayloads, Phase10EvidenceTestRefs, Phase10EvidenceWitnessRef,
    compare_phase10_observations, execute_rigid_world_process, phase10_policy_calibrations,
    required_phase10_evidence_leaves, validate_phase10_evidence_contract,
};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase10Observation, RecordLimit, RigidWorldObservation, ScenarioId,
    WitnessRole, decode_rigid_world_request_jsonl, encode_jsonl,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const BASE_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const MANIFEST: &str = include_str!("fixtures/rigid_world/phase10/phase10-v1.json");
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
include!("phase10_corpus/evidence_contract.rs");
include!("phase10_corpus/scenarios.rs");
include!("phase10_corpus/manifest.rs");
include!("phase10_corpus/execution.rs");
