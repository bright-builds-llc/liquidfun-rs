//! Injectable separately compiled child used only by supervisor integration tests.

use std::{
    fs,
    io::{self, BufRead, Write},
    thread,
    time::Duration,
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, adapter_source_digest, effective_compile_command_sha256,
};
use liquidfun_test_protocol::{
    BuildIdentity, BuildIdentityFields, CheckpointRecord, EngineKind, FloatBits, HarnessLimits,
    Phase4BuildIdentityFields, RecordLimit, ScenarioRequestRecord, TraceBegin, TraceEnd,
    TraceRecord, WorldCounts, decode_handshake_jsonl, decode_rigid_world_request_jsonl,
    decode_scenario_request_jsonl, encode_jsonl, trace_payload_sha256,
};

const TRACE_BYTES: &[u8] =
    include_bytes!("../../../../protocol/fixtures/accepted/empty-world-trace.jsonl");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let behavior = behavior()?;
    if behavior == "startup_timeout" {
        thread::sleep(Duration::from_secs(7));
        return Ok(());
    }
    if behavior == "handshake_malformed" {
        io::stdout().write_all(b"{}\n")?;
        return Ok(());
    }

    write_handshake(&behavior)?;
    let stdin = io::stdin();
    let mut reset_epoch = 0_u64;
    for line in stdin.lock().lines() {
        let line = line?;
        reset_epoch = reset_epoch.saturating_add(1);
        handle_request(&behavior, line.as_bytes(), reset_epoch)?;
    }
    Ok(())
}

fn behavior() -> Result<String, Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or("fake executable must have a parent directory")?;
    Ok(fs::read_to_string(directory.join("behavior.txt"))?
        .trim()
        .to_owned())
}

fn handshake_bytes() -> &'static [u8] {
    TRACE_BYTES
        .split_inclusive(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default()
}

fn write_handshake(behavior: &str) -> io::Result<()> {
    let mut bytes = if behavior.starts_with("rigid_d1") || behavior.starts_with("rigid_d2") {
        rigid_handshake(behavior).map_err(io::Error::other)?.0
    } else {
        handshake_bytes().to_vec()
    };
    if behavior == "unsupported_version" {
        bytes = String::from_utf8_lossy(&bytes)
            .replacen("\"protocol_version\":1", "\"protocol_version\":2", 1)
            .into_bytes();
    } else if behavior == "wrong_provenance" {
        bytes = String::from_utf8_lossy(&bytes)
            .replacen(
                "7f20402173fd143a3988c921bc384459c6a858f2",
                "0000000000000000000000000000000000000000",
                1,
            )
            .into_bytes();
    }
    let mut stdout = io::stdout().lock();
    stdout.write_all(&bytes)?;
    stdout.flush()
}

fn handle_request(
    behavior: &str,
    request_bytes: &[u8],
    reset_epoch: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    match behavior {
        "request_timeout" => thread::sleep(Duration::from_secs(12)),
        "nonzero" => {
            eprintln!("child failed");
            std::process::exit(7);
        }
        "signal" => std::process::abort(),
        "eof" => std::process::exit(0),
        "partial" => {
            write_then_exit(b"{")?;
            std::process::exit(0);
        }
        "malformed" => write_then_exit(b"{}\n")?,
        "unknown_kind" => {
            write_then_exit(b"{\"protocol_version\":1,\"record_kind\":\"mystery\"}\n")?;
        }
        "oversized" => {
            let mut bytes =
                vec![b'x'; HarnessLimits::phase2_default_v1().output_record_bytes() + 1];
            bytes.push(b'\n');
            write_then_sleep(&bytes)?;
        }
        "total_overflow" => {
            write_stderr(65 * 1024 * 1024)?;
            thread::sleep(Duration::from_secs(12));
        }
        "scenario_rejected" => {
            eprintln!("scenario rejected: injected failure");
            std::process::exit(8);
        }
        "cpp_adapter_failure" => {
            eprintln!("cpp adapter failure: injected failure");
            std::process::exit(9);
        }
        "math_large_stderr_malformed" => {
            write_stderr(1024 * 1024)?;
            write_then_sleep(b"{}\n")?;
        }
        "second_malformed" if reset_epoch == 2 => write_then_exit(b"{}\n")?,
        _ => emit_trace_behavior(behavior, request_bytes, reset_epoch)?,
    }
    Ok(())
}

fn emit_trace_behavior(
    behavior: &str,
    request_bytes: &[u8],
    reset_epoch: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let limits = HarnessLimits::phase2_default_v1();
    let mut framed_request = request_bytes.to_vec();
    framed_request.push(b'\n');
    if behavior.starts_with("rigid_d1") || behavior.starts_with("rigid_d2") {
        return emit_rigid_behavior(behavior, &framed_request, reset_epoch);
    }
    let request = decode_scenario_request_jsonl(&framed_request, &limits)?;
    let identity = fixture_identity()?;
    let records = trace_records(&request, &identity, reset_epoch, behavior)?;
    let mut encoded = records
        .iter()
        .map(|record| encode_jsonl(record, &limits, RecordLimit::Output))
        .collect::<Result<Vec<_>, _>>()?;

    match behavior {
        "large_stderr_valid" => write_stderr(1024 * 1024)?,
        "concurrent_total_overflow" => {
            let stderr = thread::spawn(|| write_stderr(65 * 1024 * 1024));
            let mut stdout = io::stdout().lock();
            for record in encoded {
                stdout.write_all(&record)?;
            }
            stdout.flush()?;
            stderr
                .join()
                .map_err(|_| io::Error::other("stderr writer panicked"))??;
            return Ok(());
        }
        "large_stderr_malformed" => {
            write_stderr(1024 * 1024)?;
            write_then_sleep(b"{}\n")?;
            return Ok(());
        }
        "sanitizer" => {
            io::stderr().write_all(b"ERROR: AddressSanitizer: injected\n")?;
            io::stderr().flush()?;
        }
        "request_mismatch" => {
            replace_all(&mut encoded, request.request_id().as_str(), "wrong-request");
        }
        "identity_mismatch" => replace_all(
            &mut encoded,
            identity.identity_sha256().as_str(),
            &"0".repeat(64),
        ),
        "sequence" => encoded.swap(0, 1),
        "reset" => replace_all(
            &mut encoded,
            "\"reset_verified\":true",
            "\"reset_verified\":false",
        ),
        "trace_too_large" => {
            let begin = encoded.first().ok_or("trace begin should exist")?.clone();
            let checkpoint = encoded.get(1).ok_or("checkpoint should exist")?.clone();
            let mut stdout = io::stdout().lock();
            stdout.write_all(&begin)?;
            for _ in 0..100_000 {
                stdout.write_all(&checkpoint)?;
            }
            stdout.flush()?;
            thread::sleep(Duration::from_secs(12));
            return Ok(());
        }
        _ => {}
    }

    let mut stdout = io::stdout().lock();
    for record in encoded {
        stdout.write_all(&record)?;
    }
    stdout.flush()?;
    if behavior == "sanitizer" {
        thread::sleep(Duration::from_secs(12));
    }
    Ok(())
}

fn emit_rigid_behavior(
    behavior: &str,
    request_bytes: &[u8],
    reset_epoch: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(request_bytes, &limits)?;
    let result = NativeRigidWorldExecutor::execute(&request)?;
    let result_bytes = if behavior == "rigid_d1_mismatch" {
        let mut value = serde_json::to_value(&result)?;
        value["timelines"][0]["checkpoints"][0]["bodies"][0]["active"] =
            serde_json::Value::Bool(false);
        let mut bytes = serde_json::to_vec(&value)?;
        bytes.push(b'\n');
        bytes
    } else {
        encode_jsonl(&result, &limits, RecordLimit::Output)?
    };
    let end = serde_json::json!({
        "protocol_version": 1,
        "record_kind": "rigid_world_end",
        "request_id": request.request_id().as_str(),
        "result_count": 1,
        "reset_epoch": reset_epoch,
        "reset_verified": true,
    });
    let mut end_bytes = serde_json::to_vec(&end)?;
    end_bytes.push(b'\n');
    if behavior.ends_with("_nonzero") {
        eprintln!("rigid child failed after decode");
        std::process::exit(7);
    }
    let mut stdout = io::stdout().lock();
    stdout.write_all(&result_bytes)?;
    stdout.write_all(&end_bytes)?;
    stdout.flush()?;
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "the fake oracle handshake mirrors one complete protocol identity record"
)]
fn rigid_handshake(behavior: &str) -> Result<(Vec<u8>, BuildIdentity), String> {
    let canonical = behavior.starts_with("rigid_d1");
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let repository_root = executable
        .ancestors()
        .nth(4)
        .ok_or_else(|| "fake rigid oracle path does not contain a repository root".to_owned())?;
    let mut adapter_digest =
        adapter_source_digest(repository_root).map_err(|error| error.to_string())?;
    let mut compile_digest = effective_compile_command_sha256(repository_root, "oracle-debug")
        .map_err(|error| error.to_string())?;
    if behavior == "rigid_d1_stale_adapter" {
        adapter_digest = stale_digest(&adapter_digest);
    }
    if behavior == "rigid_d1_stale_compile" {
        compile_digest = stale_digest(&compile_digest);
    }
    let (compiler_id, compiler_version, target, os, target_features) = if canonical {
        (
            "Clang",
            "22.1.8",
            "x86_64-unknown-linux-gnu",
            "linux",
            "<none>",
        )
    } else {
        (
            "AppleClang",
            "21.0.0",
            "x86_64-apple-darwin",
            "macos",
            "<none>",
        )
    };
    let phase4 = Phase4BuildIdentityFields::new(
        &compile_digest,
        compiler_id,
        compiler_version,
        target,
        "baseline",
        target_features,
        "<none>",
        "O0",
        "precise",
        "off",
        "ieee",
        "scalar baseline",
        os,
        if canonical { "glibc" } else { "libSystem" },
        if canonical { "libm" } else { "libSystem" },
        "nearest_ties_even",
        true,
    );
    let identity = BuildIdentity::new(
        BuildIdentityFields::new(
            "7f20402173fd143a3988c921bc384459c6a858f2",
            "fixture-rigid-adapter-v1",
            &adapter_digest,
            "oracle-debug",
            compiler_id,
            compiler_version,
            target,
            "Debug",
            "-O0 -g",
            "-lc++",
            "none",
        )
        .with_phase4(phase4),
    )
    .map_err(|error| error.to_string())?;
    let build_identity = serde_json::json!({
        "oracle_revision": identity.oracle_revision(),
        "adapter_revision": identity.adapter_revision(),
        "adapter_content_sha256": identity.adapter_content_sha256().as_str(),
        "cmake_preset": identity.cmake_preset(),
        "compiler_id": compiler_id,
        "compiler_version": compiler_version,
        "target": target,
        "build_type": "Debug",
        "effective_compile_flags": "-O0 -g",
        "effective_link_flags": "-lc++",
        "sanitizer_mode": "none",
        "compile_command_sha256": compile_digest,
        "target_triple": target,
        "target_cpu": "baseline",
        "target_features": target_features,
        "sdk_or_sysroot": "<none>",
        "optimization": "O0",
        "fp_model": "precise",
        "fp_contract": "off",
        "denormal_mode": "ieee",
        "feature_set": "scalar baseline",
        "os": os,
        "libc": if canonical { "glibc" } else { "libSystem" },
        "libm": if canonical { "libm" } else { "libSystem" },
        "rounding_mode": "nearest_ties_even",
        "gradual_underflow": true,
    });
    let mut bytes = serde_json::to_vec(&serde_json::json!({
        "protocol_version": 1,
        "record_kind": "handshake",
        "supported_scenario_versions": [1],
        "supported_trace_versions": [1],
        "supported_tolerance_versions": [1],
        "build_identity": build_identity,
        "identity_sha256": identity.identity_sha256().as_str(),
    }))
    .map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok((bytes, identity))
}

fn stale_digest(current: &str) -> String {
    let replacement = if current.starts_with('0') { '1' } else { '0' };
    format!("{replacement}{}", &current[1..])
}

fn fixture_identity() -> Result<BuildIdentity, Box<dyn std::error::Error>> {
    Ok(
        decode_handshake_jsonl(handshake_bytes(), &HarnessLimits::phase2_default_v1())?
            .build_identity()
            .clone(),
    )
}

fn trace_records(
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
    reset_epoch: u64,
    behavior: &str,
) -> Result<Vec<TraceRecord>, Box<dyn std::error::Error>> {
    let begin = TraceBegin::for_request(request, EngineKind::CppOracle, identity)?;
    let mut time = 0.0_f32;
    let mut checkpoints = Vec::new();
    for (ordinal, (command, requested)) in request
        .scenario()
        .commands()
        .iter()
        .zip(request.scenario().checkpoints())
        .enumerate()
    {
        time += command.timestep_bits().to_f32();
        if (behavior == "value_mismatch"
            || (behavior == "second_value_mismatch" && reset_epoch == 2))
            && ordinal == 1
        {
            time += 0.25;
        }
        checkpoints.push(CheckpointRecord::new(
            request.request_id().clone(),
            requested.checkpoint_id().clone(),
            u32::try_from(ordinal)?,
            requested.phase(),
            FloatBits::from_f32(time),
            WorldCounts::zero(),
            identity.identity_sha256().clone(),
        )?);
    }
    let end = TraceEnd::new(
        request.request_id().clone(),
        u32::try_from(checkpoints.len())?,
        trace_payload_sha256(&checkpoints)?,
        reset_epoch,
        true,
        identity.identity_sha256().clone(),
    );
    Ok(std::iter::once(TraceRecord::Begin(begin))
        .chain(checkpoints.into_iter().map(TraceRecord::Checkpoint))
        .chain(std::iter::once(TraceRecord::End(end)))
        .collect())
}

fn replace_all(records: &mut [Vec<u8>], from: &str, to: &str) {
    for record in records {
        *record = String::from_utf8_lossy(record)
            .replace(from, to)
            .into_bytes();
    }
}

fn write_stderr(bytes: usize) -> io::Result<()> {
    let chunk = vec![b'e'; 16 * 1024];
    let mut stderr = io::stderr().lock();
    for _ in 0..bytes / chunk.len() {
        stderr.write_all(&chunk)?;
    }
    stderr.flush()
}

fn write_then_exit(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn write_then_sleep(bytes: &[u8]) -> io::Result<()> {
    write_then_exit(bytes)?;
    thread::sleep(Duration::from_secs(12));
    Ok(())
}
