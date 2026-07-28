use std::time::Duration;

use liquidfun_differential::{
    MinimizationBudget, NativeRigidWorldExecutor, RigidComparisonOutcome, RigidEvaluation,
    RigidMismatchKind, compare_phase7_rigid_world_results, minimize_rigid_world_request,
};
use liquidfun_test_protocol::{
    FieldComparison, FloatBits, HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile,
    RigidWorldRequestRecord, RigidWorldResultRecord, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl,
};
use serde_json::{Value, json};

const PHASE6_POLICY: &str = include_str!("../../../../protocol/tolerances/phase6-v1.toml");
const PHASE7_POLICY: &str = include_str!("../../../../protocol/tolerances/phase7-v1.toml");
include!("phase7_comparator/setup.rs");
include!("phase7_comparator/query_comparison.rs");
include!("phase7_comparator/ray_boundaries.rs");
include!("phase7_comparator/exhaustive_rays.rs");
include!("phase7_comparator/termination_and_minimization.rs");
