//! Calibration, validation, and optimization-record evidence operations.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::analysis::{
    OptimizationCandidate, OptimizationDecision, WorkloadInterval, evaluate_optimization,
    student_95_interval,
};
use super::paths::{read_bounded_regular_file, write_json_atomically};
use super::{
    PerformanceCommandError, PerformanceContract, PerformanceEnvironment, PreparedCase,
    prepare_cases,
};

#[derive(Debug, Serialize)]
struct CalibrationEntry {
    case_id: String,
    run_deltas_basis_points: [i32; 5],
    interval: WorkloadInterval,
}

pub(super) fn calibrate(
    environment: &PerformanceEnvironment,
    contract: &PerformanceContract,
) -> Result<(), PerformanceCommandError> {
    let expected = prepare_cases(&contract.matrix)?;
    let raw_paths = exact_raw_paths(&environment.paths.raw_directory(), &expected)?;
    let mut entries = Vec::with_capacity(raw_paths.len());
    for (case, path) in expected.iter().zip(raw_paths) {
        let value: Value = serde_json::from_slice(
            &read_bounded_regular_file(&path)
                .map_err(|message| PerformanceCommandError::new("calibration", message))?,
        )
        .map_err(|error| PerformanceCommandError::new("calibration", error.to_string()))?;
        validate_report_identity(&value, case, contract)?;
        let run_deltas = paired_run_deltas(&value)?;
        let interval = student_95_interval(&run_deltas)
            .map_err(|message| PerformanceCommandError::new("calibration", message))?;
        entries.push(CalibrationEntry {
            case_id: case.case_id.clone(),
            run_deltas_basis_points: run_deltas,
            interval,
        });
    }
    let calibration = json!({
        "schema_version": 1,
        "claim_status": "non_claiming_host_calibration",
        "interval_method": "student_t_95",
        "policy_sha256": contract.policy_sha256,
        "matrix_sha256": contract.matrix_sha256,
        "cases": entries,
    });
    write_json_atomically(&environment.paths.calibration(), &calibration)
        .map_err(|message| PerformanceCommandError::new("calibration", message))?;
    println!("performance calibrate: retained five independent runs for every sealed case");
    Ok(())
}

fn exact_raw_paths(
    raw_directory: &Path,
    cases: &[PreparedCase],
) -> Result<Vec<PathBuf>, PerformanceCommandError> {
    let expected = cases
        .iter()
        .map(|case| format!("{}.json", case.case_id))
        .collect::<BTreeSet<_>>();
    let actual = fs::read_dir(raw_directory)
        .map_err(|error| PerformanceCommandError::new("raw_reports", error.to_string()))?
        .map(|entry| {
            entry
                .map_err(|error| PerformanceCommandError::new("raw_reports", error.to_string()))
                .and_then(|entry| {
                    entry.file_name().into_string().map_err(|_| {
                        PerformanceCommandError::new("raw_reports", "non-UTF-8 report name")
                    })
                })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(PerformanceCommandError::new(
            "raw_reports",
            "raw report set differs from the sealed matrix",
        ));
    }
    Ok(cases
        .iter()
        .map(|case| raw_directory.join(format!("{}.json", case.case_id)))
        .collect())
}

fn paired_run_deltas(value: &Value) -> Result<[i32; 5], PerformanceCommandError> {
    let samples = value
        .get("raw_samples")
        .and_then(Value::as_array)
        .ok_or_else(|| PerformanceCommandError::new("calibration", "missing raw_samples"))?;
    if samples.len() != 150 {
        return Err(PerformanceCommandError::new(
            "calibration",
            "each report must retain 150 raw sample pairs",
        ));
    }
    let mut native = [0_u128; 5];
    let mut oracle = [0_u128; 5];
    let mut counts = [0_u16; 5];
    for sample in samples {
        let run = required_u64(sample, "baseline_run")?;
        let index = usize::try_from(run.saturating_sub(1))
            .map_err(|_| PerformanceCommandError::new("calibration", "invalid baseline run"))?;
        if index >= 5 {
            return Err(PerformanceCommandError::new(
                "calibration",
                "baseline run is outside 1..=5",
            ));
        }
        native[index] += u128::from(required_u64(sample, "native_nanoseconds")?);
        oracle[index] += u128::from(required_u64(sample, "oracle_nanoseconds")?);
        counts[index] = counts[index].saturating_add(1);
    }
    if counts != [30; 5] || oracle.contains(&0) {
        return Err(PerformanceCommandError::new(
            "calibration",
            "each independent run must retain 30 nonzero pairs",
        ));
    }
    let mut deltas = [0_i32; 5];
    for index in 0..5 {
        let oracle_sum = i128::try_from(oracle[index])
            .map_err(|_| PerformanceCommandError::new("calibration", "oracle sum overflow"))?;
        let native_sum = i128::try_from(native[index])
            .map_err(|_| PerformanceCommandError::new("calibration", "native sum overflow"))?;
        let numerator = (oracle_sum - native_sum) * 10_000;
        deltas[index] = i32::try_from(numerator / oracle_sum)
            .map_err(|_| PerformanceCommandError::new("calibration", "delta overflow"))?;
    }
    Ok(deltas)
}

fn required_u64(value: &Value, field: &str) -> Result<u64, PerformanceCommandError> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| {
        PerformanceCommandError::new("calibration", format!("missing numeric {field}"))
    })
}

pub(super) fn validate(
    environment: &PerformanceEnvironment,
    contract: &PerformanceContract,
    emit_identity: bool,
) -> Result<(), PerformanceCommandError> {
    let raw_directory = environment.paths.raw_directory();
    if raw_directory.exists() {
        let cases = prepare_cases(&contract.matrix)?;
        for (case, path) in cases.iter().zip(exact_raw_paths(&raw_directory, &cases)?) {
            let value: Value = serde_json::from_slice(
                &read_bounded_regular_file(&path)
                    .map_err(|message| PerformanceCommandError::new("validate", message))?,
            )
            .map_err(|error| PerformanceCommandError::new("validate", error.to_string()))?;
            validate_report_identity(&value, case, contract)?;
            let _run_deltas = paired_run_deltas(&value)?;
        }
    }
    let validation = json!({
        "schema_version": 1,
        "validation_status": "passed",
        "claim_status": "no_generalized_performance_claim",
        "policy_sha256": contract.policy_sha256,
        "matrix_sha256": contract.matrix_sha256,
    });
    if emit_identity {
        println!(
            "{}",
            serde_json::to_string_pretty(&validation)
                .map_err(|error| PerformanceCommandError::new("validate", error.to_string()))?
        );
    } else {
        println!("performance validate: sealed policy and evidence surfaces passed");
    }
    Ok(())
}

fn validate_report_identity(
    value: &Value,
    case: &PreparedCase,
    contract: &PerformanceContract,
) -> Result<(), PerformanceCommandError> {
    let identity = value
        .get("identity")
        .and_then(Value::as_object)
        .ok_or_else(|| PerformanceCommandError::new("validate", "missing report identity"))?;
    let expected = [
        ("scenario_id", case.resolved.identity().slug().as_str()),
        ("policy_sha256", contract.policy_sha256.as_str()),
        ("matrix_sha256", contract.matrix_sha256.as_str()),
        ("catalog_sha256", case.catalog_sha256.as_str()),
        ("resolved_sha256", case.resolved_sha256.as_str()),
    ];
    let identity_matches = expected.into_iter().all(|(field, expected_value)| {
        identity.get(field).and_then(Value::as_str) == Some(expected_value)
    });
    let required_text = [
        "rust_revision",
        "oracle_revision",
        "rust_compiler",
        "rust_linker",
        "oracle_compiler",
        "oracle_linker",
        "target",
        "rust_compile_flags",
        "rust_link_flags",
        "oracle_compile_flags",
        "oracle_link_flags",
        "identity_sha256",
    ];
    if !identity_matches
        || required_text.iter().any(|field| {
            identity
                .get(*field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
        })
        || value.get("compatibility_status").and_then(Value::as_str) != Some("d2_supported")
        || value.get("profile_schema").and_then(Value::as_str) != Some("phase12_v1")
        || value.get("policy")
            != Some(
                &serde_json::to_value(&contract.policy)
                    .map_err(|error| PerformanceCommandError::new("validate", error.to_string()))?,
            )
    {
        return Err(PerformanceCommandError::new(
            "validate",
            format!("report identity drifted for {}", case.case_id),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptimizationRecord {
    schema_version: u8,
    candidate: OptimizationCandidate,
    claimed_disposition: OptimizationDecision,
}

pub(super) fn optimization_check(
    environment: &PerformanceEnvironment,
) -> Result<(), PerformanceCommandError> {
    let bytes = read_bounded_regular_file(&environment.paths.optimization_record())
        .map_err(|message| PerformanceCommandError::new("optimization", message))?;
    let record: OptimizationRecord = serde_json::from_slice(&bytes)
        .map_err(|error| PerformanceCommandError::new("optimization", error.to_string()))?;
    let actual = evaluate_optimization(&record.candidate);
    if record.schema_version != 1
        || actual != record.claimed_disposition
        || actual != OptimizationDecision::Admit
    {
        return Err(PerformanceCommandError::new(
            "optimization",
            format!("optimization candidate was not admitted: {actual:?}"),
        ));
    }
    println!("performance optimization-check: admitted reviewed scalar candidate");
    Ok(())
}
