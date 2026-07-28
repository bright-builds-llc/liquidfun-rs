//! Pinned C++ Phase 9 particle-oracle integration tests.

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE9_REQUIRED_POLICY_PATHS,
    Phase9ComparisonOutcome, Phase9DifferentialError, compare_phase9_rigid_world_results,
    execute_rigid_world_process, run_phase9_differential,
};
use liquidfun_test_protocol::{HarnessLimits, decode_rigid_world_request_jsonl};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const WITNESS: &[u8] =
    include_bytes!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.json");
const WITNESS_PROVENANCE: &str =
    include_str!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json");
include!("particle_oracle/setup.rs");
include!("particle_oracle/decoder.rs");
include!("particle_oracle/coupling.rs");
include!("particle_oracle/differential.rs");
