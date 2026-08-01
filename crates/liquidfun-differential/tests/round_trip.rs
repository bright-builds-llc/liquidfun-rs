//! End-to-end native/C++ comparison and CLI outcome tests.

#[path = "support/coverage_observation.rs"]
mod coverage_observation;

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use liquidfun_differential::{
    DifferentialRunOutcome, EmptyWorldAdapter, OracleExecutable, OraclePreset, SessionProfile,
    replay_exact, run_named,
};
use liquidfun_test_protocol::{
    HarnessLimits, MathProbeResult, RecordLimit, decode_math_probe_request_jsonl,
    decode_scenario_request_jsonl, encode_jsonl,
};
use sha2::{Digest, Sha256};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);
static PROCESS_SLOT: Mutex<()> = Mutex::new(());
include!("round_trip/oracle.rs");
include!("round_trip/real_oracle.rs");
include!("round_trip/cli_reports.rs");
include!("round_trip/evidence.rs");
include!("round_trip/minimization.rs");
