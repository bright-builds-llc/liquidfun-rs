//! Closed Phase 9 witness-corpus and evidence-boundary tests.

#[path = "support/coverage_observation.rs"]
mod coverage_observation;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE9_REGISTRY_ID,
    PHASE9_REQUIRED_POLICY_PATHS, Phase9ComparisonOutcome, Phase9CrossRunProof,
    Phase9CrossRunProofRecord, Phase9EvidenceMismatch, Phase9EvidencePayloadRef,
    RigidComparisonOutcome, RigidMismatchReport, compare_complete_phase9_rigid_world_results,
    compare_phase8_rigid_world_results, effective_compile_command_sha256,
    execute_rigid_world_process, phase9_policy_for_path, run_phase9_differential,
    validate_phase9_cross_run_proofs, validate_phase9_evidence_bindings,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    Phase9ObservationKind, Phase9SemanticAssertion, Phase9WitnessBinding,
    Phase9WitnessBindingErrorKind, RigidWorldResultRecord, RigidWorldWitnessFamily, ScenarioId,
    Sha256Hex, decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_phase9_witness_bindings,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RETAINED_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const MANIFEST: &str = include_str!("fixtures/rigid_world/phase9/phase9-v1.json");
// Phase 9's reviewed corpus predates the Phase 13 exact-ref witness promotion. Retain the exact
// pre-promotion bytes from 60325e3^ (SHA-256 08d41d25...) so the immutable Phase 9 -> 10 -> 11
// authority chain remains independently verifiable. Its historical provenance is commit a23b5f0
// and provenance SHA-256 fe2f934a...; the current promoted witness remains a separate authority.
const PINNED_WITNESS: &[u8] =
    include_bytes!("fixtures/rigid_world/phase9/lifecycle-contact-witnesses.phase9-retained.json");
const PHASE6_POLICY: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const PHASE7_POLICY: &str = include_str!("../../../protocol/tolerances/phase7-v1.toml");
const PHASE8_POLICY: &str = include_str!("../../../protocol/tolerances/phase8-v1.toml");
const PHASE6_POLICY_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_POLICY_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const PHASE8_POLICY_SHA256: &str =
    "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf";
const FAKE_PHASE9_RESULT_UNITS: [&str; 4] = [
    "collision_probe.cpp",
    "math_probe.cpp",
    "protocol_bits.cpp",
    "rigid_world.cpp",
];
const COMMON_ACTIONS: &[&str] = &[
    "create-growable",
    "create-fixed",
    "create-phase9-a",
    "create-phase9-b",
    "create-phase9-coupling",
    "create-phase9-capacity",
    "create-phase9-c",
    "create-phase9-d",
    "inspect-system",
    "inspect-particle",
    "resume",
    "statistics",
    "statistics-fixed",
    "phase9-step",
    "destroy-fixed",
    "destroy-growable",
];
const FORCE_ACTIONS: &[&str] = &[
    "position",
    "velocity",
    "force",
    "inspect-after-force",
    "impulse",
    "inspect-after-impulse",
];

const REQUIRED_BRANCHES: &[&str] = &[
    "multiple_systems",
    "newest_first",
    "paused_system",
    "stable_ids_sort",
    "stable_ids_compact",
    "optional_lanes",
    "fixed_buffer",
    "growable_buffer",
    "fixed_full",
    "teardown",
    "finite_lifetime",
    "infinite_lifetime",
    "equal_lifetime",
    "oldest_lifetime",
    "maximum_lifetime",
    "requested_destruction_callback",
    "unrequested_destruction_callback",
    "zombie_pending",
    "capacity_eviction",
    "particle_contact",
    "body_contact",
    "strict_contact_enabled",
    "strict_contact_disabled",
    "listener_flag_enabled",
    "listener_flag_disabled",
    "filter_flag_enabled",
    "filter_flag_disabled",
    "contact_order",
    "contact_multiplicity",
    "coupling_fields",
    "dynamic_body_reaction",
    "static_body_no_reaction",
    "force_range",
    "impulse_range",
    "statistics_counts",
    "collision_energy",
    "stuck_candidates",
    "system_aabb",
    "world_aabb",
    "system_culling",
    "query_continue",
    "query_terminate",
    "system_ray",
    "world_ray",
    "ray_culling",
    "ray_start_inside_exclusion",
    "ray_ignore",
    "ray_continue",
    "ray_clip",
    "ray_terminate",
    "retained_phase6_through_phase8",
    "phase10_rejection",
    "closed_policy_registry",
    "replay_identity",
    "minimization_identity",
    "first_divergence_stability",
    "d0_byte_identity",
    "debug_release_agreement",
];

include!("phase9_corpus/model.rs");
include!("phase9_corpus/declarations.rs");
include!("phase9_corpus/scenarios.rs");
include!("phase9_corpus/witness_system.rs");
include!("phase9_corpus/witness_assertions.rs");
include!("phase9_corpus/execution.rs");
include!("phase9_corpus/witness_validation.rs");
include!("phase9_corpus/corpus_binding.rs");
include!("phase9_corpus/retained_support.rs");
include!("phase9_corpus/retained_tests.rs");
include!("phase9_corpus/fake_oracle.rs");
include!("phase9_corpus/workflow.rs");
