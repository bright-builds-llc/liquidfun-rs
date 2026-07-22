//! Security and lifecycle tests for staged differential evidence.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    ArtifactKind, FixtureError, ReviewMetadata, StageRequest, promote_candidate, review_candidate,
    stage_candidate,
};
use liquidfun_test_protocol::{
    CheckpointRecord, EngineKind, FloatBits, HarnessLimits, RecordLimit, TraceBegin, TraceEnd,
    TraceRecord, WorldCounts, decode_handshake_jsonl, decode_scenario_request_jsonl, encode_jsonl,
    trace_payload_sha256,
};

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const GENERATOR_REVISION: &str = "69b9ac469b9ac469b9ac469b9ac469b9ac469b9a";
const REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-request.jsonl");
const TRACE_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-trace.jsonl");
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct FixtureRepository {
    root: PathBuf,
}

impl FixtureRepository {
    fn new() -> Self {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "liquidfun-fixture-workflow-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("reference/artifacts"))
            .expect("fixture artifact directory should be created");
        fs::create_dir_all(root.join("scenarios/regressions"))
            .expect("fixture regression directory should be created");
        fs::write(root.join("THIRD_PARTY_NOTICES.md"), "fixture notices\n")
            .expect("fixture notice should be written");
        fs::write(
            root.join("reference/artifacts/manifest.toml"),
            format!(
                "schema_version = 2\nrecord_schema_version = 2\noracle_revision = \"{ORACLE_REVISION}\"\nrecord_fields = [\n  \"artifact_kind\",\n  \"path\",\n  \"sha256\",\n  \"generator_revision\",\n  \"request_sha256\",\n  \"scenario_content_sha256\",\n  \"scenario_sha256\",\n  \"protocol_version\",\n  \"scenario_schema_version\",\n  \"trace_schema_version\",\n  \"tolerance_profile_version\",\n  \"tolerance_profile_sha256\",\n  \"oracle_revision\",\n  \"adapter_revision\",\n  \"adapter_content_sha256\",\n  \"build_identity_sha256\",\n  \"preset\",\n  \"compiler\",\n  \"target\",\n  \"flags\",\n  \"source\",\n  \"trace_payload_sha256\",\n  \"failure_signature\",\n  \"notice_refs\",\n  \"reviewer\",\n  \"reviewed_at\",\n  \"review_status\",\n]\nartifacts = []\n\n[artifact_schemas.phase11_evidence]\nschema_version = 1\nmanifest_file = \"phase11-v1.json\"\nidentity_file = \"identity.json\"\nprotocol_version = \"catalog-phase11-v1\"\ngenerator_version = \"phase11-evidence-v1\"\npromotion = \"exact-ref-same-run-only\"\n"
            ),
        )
        .expect("fixture manifest should be written");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn stage_trace(&self, artifact_id: &str) -> liquidfun_differential::ArtifactCandidate {
        stage_candidate(
            self.root(),
            StageRequest {
                artifact_id,
                artifact_kind: ArtifactKind::ReviewedTrace,
                scenario_id: "empty-world",
                preset: "oracle-debug",
                session_profile: "one-shot",
                generator_revision: GENERATOR_REVISION,
                request_bytes: REQUEST_BYTES,
                trace_bytes: TRACE_BYTES,
                stderr_bytes: b"bounded fixture stderr\n",
                maybe_failure_signature: None,
            },
        )
        .expect("valid reviewed trace should stage")
    }
}

impl Drop for FixtureRepository {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}

fn approved_review() -> ReviewMetadata<'static> {
    ReviewMetadata::approved("fixture-reviewer", "2026-07-10T10:15:00Z")
}

fn tracked_bytes(repository: &FixtureRepository) -> Vec<(PathBuf, Vec<u8>)> {
    ["reference", "scenarios"]
        .into_iter()
        .flat_map(|relative| collect_regular_files(&repository.root().join(relative)))
        .map(|path| {
            let bytes = fs::read(&path).expect("tracked fixture should remain readable");
            (path, bytes)
        })
        .collect()
}

fn collect_regular_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut entries = fs::read_dir(root)
        .expect("fixture directory should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("fixture entries should be readable");
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_regular_files(&path));
        } else {
            files.push(path);
        }
    }
    files
}

#[test]
fn reviewed_trace_stages_reviews_diffs_and_promotes() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("empty-world-trace-v1");
    assert!(
        candidate.directory().starts_with(
            fs::canonicalize(repository.root().join("target"))
                .expect("target staging root should canonicalize")
        )
    );

    // Act
    let review = review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect("valid candidate should replay and review");
    let promotion = promote_candidate(repository.root(), candidate.artifact_id())
        .expect("approved candidate should promote");

    // Assert
    assert_eq!(review.review_status(), "approved");
    assert_eq!(review.diff(), "accepted artifact is absent\n");
    assert_eq!(
        promotion.artifact_path(),
        fs::canonicalize(repository.root())
            .expect("fixture repository should canonicalize")
            .join("reference/artifacts/traces/empty-world-v1.jsonl")
    );
    assert_eq!(
        fs::read(promotion.artifact_path()).expect("promoted trace should be readable"),
        TRACE_BYTES
    );
    let manifest = fs::read_to_string(repository.root().join("reference/artifacts/manifest.toml"))
        .expect("manifest should remain readable");
    assert!(manifest.contains("review_status = \"reviewed\""));
    assert!(manifest.contains("reference/artifacts/traces/empty-world-v1.jsonl"));
    assert!(manifest.contains("artifact_kind = \"trace\""));
    assert!(manifest.contains("adapter_revision = \"fixture-adapter-v1\""));
    assert!(manifest.contains("build_identity_sha256"));
    assert!(manifest.contains("trace_payload_sha256"));
    assert!(manifest.contains("reviewer = \"fixture-reviewer\""));
    assert!(manifest.contains("reviewed_at = \"2026-07-10T10:15:00Z\""));
}

#[test]
fn review_is_read_only_for_accepted_evidence() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("read-only-review");
    let before = tracked_bytes(&repository);

    // Act
    review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect("review should succeed");

    // Assert
    assert_eq!(tracked_bytes(&repository), before);
}

#[test]
fn artifact_identifier_rejects_traversal() {
    // Arrange
    let repository = FixtureRepository::new();

    // Act
    let error = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "../escape",
            artifact_kind: ArtifactKind::ReviewedTrace,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: TRACE_BYTES,
            stderr_bytes: b"",
            maybe_failure_signature: None,
        },
    )
    .expect_err("traversal should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::InvalidIdentifier { .. }));
}

#[cfg(unix)]
#[test]
fn staging_rejects_a_symlinked_boundary() {
    use std::os::unix::fs::symlink;

    // Arrange
    let repository = FixtureRepository::new();
    let outside = repository.root().join("outside");
    fs::create_dir(&outside).expect("outside fixture directory should be created");
    fs::create_dir(repository.root().join("target"))
        .expect("target fixture directory should be created");
    symlink(&outside, repository.root().join("target/differential"))
        .expect("boundary symlink should be created");

    // Act
    let error = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "symlink-boundary",
            artifact_kind: ArtifactKind::ReviewedTrace,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: TRACE_BYTES,
            stderr_bytes: b"",
            maybe_failure_signature: None,
        },
    )
    .expect_err("symlinked staging boundary should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::Symlink { .. }));
}

#[test]
fn promotion_rejects_an_unreviewed_candidate() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("unreviewed");

    // Act
    let error = promote_candidate(repository.root(), candidate.artifact_id())
        .expect_err("unreviewed candidate should not promote");

    // Assert
    assert!(matches!(error, FixtureError::ReviewRequired));
}

#[test]
fn review_rejects_a_dirty_candidate_hash() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("dirty-hash");
    fs::write(candidate.directory().join("request.jsonl"), b"tampered\n")
        .expect("candidate tamper should be written");

    // Act
    let error = review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect_err("dirty candidate should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::HashMismatch { .. }));
}

#[test]
fn stage_rejects_a_wrong_reported_build_identity() {
    // Arrange
    let repository = FixtureRepository::new();
    let trace = String::from_utf8(TRACE_BYTES.to_vec())
        .expect("trace fixture should be UTF-8")
        .replacen(
            "\"identity_sha256\":\"56b1b4d459fef5fc7abcd7072566ac92732284e73f99c79885a80770a9f0fafd\"",
            "\"identity_sha256\":\"0000000000000000000000000000000000000000000000000000000000000000\"",
            1,
        );

    // Act
    let error = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "wrong-identity",
            artifact_kind: ArtifactKind::ReviewedTrace,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: trace.as_bytes(),
            stderr_bytes: b"",
            maybe_failure_signature: None,
        },
    )
    .expect_err("wrong reported identity should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::Replay(_)));
}

#[test]
fn review_rejects_a_partial_candidate() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("partial");
    fs::remove_file(candidate.directory().join("trace.jsonl"))
        .expect("trace should be removed for partial fixture");

    // Act
    let error = review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect_err("partial candidate should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::MissingCandidateFile { .. }));
}

#[test]
fn stage_rejects_oversized_retained_stderr() {
    // Arrange
    let repository = FixtureRepository::new();
    let oversized = vec![b'x'; HarnessLimits::phase2_default_v1().retained_stderr_bytes() + 1];

    // Act
    let error = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "oversized",
            artifact_kind: ArtifactKind::ReviewedTrace,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: TRACE_BYTES,
            stderr_bytes: &oversized,
            maybe_failure_signature: None,
        },
    )
    .expect_err("oversized retained stderr should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::SizeLimit { .. }));
}

#[test]
fn promotion_rejects_an_existing_destination_without_overwrite() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("overwrite");
    review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect("candidate should review");
    let destination = repository
        .root()
        .join("reference/artifacts/traces/empty-world-v1.jsonl");
    fs::create_dir_all(destination.parent().expect("destination has a parent"))
        .expect("destination parent should be created");
    fs::write(&destination, b"existing accepted bytes\n")
        .expect("existing artifact should be written");

    // Act
    let error = promote_candidate(repository.root(), candidate.artifact_id())
        .expect_err("existing destination should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::DestinationExists { .. }));
    assert_eq!(
        fs::read(destination).expect("existing artifact should remain readable"),
        b"existing accepted bytes\n"
    );
}

#[test]
fn promotion_rejects_candidate_changes_after_review() {
    // Arrange
    let repository = FixtureRepository::new();
    let candidate = repository.stage_trace("race-change");
    review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect("candidate should review");
    let mut file = OpenOptions::new()
        .append(true)
        .open(candidate.directory().join("stderr.txt"))
        .expect("candidate stderr should be appendable for race fixture");
    file.write_all(b"changed after review\n")
        .expect("candidate race change should be written");

    // Act
    let error = promote_candidate(repository.root(), candidate.artifact_id())
        .expect_err("post-review changes should fail closed");

    // Assert
    assert!(matches!(error, FixtureError::HashMismatch { .. }));
}

#[test]
fn minimized_regression_promotes_exact_scenario_source_and_signature() {
    // Arrange
    let repository = FixtureRepository::new();
    let request = decode_scenario_request_jsonl(REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("request fixture should validate");
    let mismatch_trace = mismatch_trace_bytes();
    let candidate = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "empty-world-regression-v1",
            artifact_kind: ArtifactKind::MinimizedRegression,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: &mismatch_trace,
            stderr_bytes: b"synthetic mismatch fixture\n",
            maybe_failure_signature: None,
        },
    )
    .expect("semantic mismatch should stage as a regression");
    let metadata = fs::read_to_string(candidate.directory().join("candidate.toml"))
        .expect("candidate metadata should be readable");

    // Act
    review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect("same-signature regression should review");
    let promotion = promote_candidate(repository.root(), candidate.artifact_id())
        .expect("reviewed regression should promote");

    // Assert
    assert_eq!(
        fs::read(promotion.artifact_path()).expect("regression should be readable"),
        serde_json::to_vec(request.scenario()).expect("scenario should serialize")
    );
    assert!(metadata.contains("source_json"));
    assert!(metadata.contains("failure_signature_json"));
    assert!(!metadata.contains("failure_signature_json = \"\""));
}

#[test]
fn review_rejects_a_changed_regression_signature() {
    // Arrange
    let repository = FixtureRepository::new();
    let mismatch_trace = mismatch_trace_bytes();
    let candidate = stage_candidate(
        repository.root(),
        StageRequest {
            artifact_id: "wrong-signature",
            artifact_kind: ArtifactKind::MinimizedRegression,
            scenario_id: "empty-world",
            preset: "oracle-debug",
            session_profile: "one-shot",
            generator_revision: GENERATOR_REVISION,
            request_bytes: REQUEST_BYTES,
            trace_bytes: &mismatch_trace,
            stderr_bytes: b"",
            maybe_failure_signature: None,
        },
    )
    .expect("mismatch should stage");
    let metadata_path = candidate.directory().join("candidate.toml");
    let metadata = fs::read_to_string(&metadata_path)
        .expect("candidate metadata should be readable")
        .replace("simulation_time", "world_counts");
    fs::write(&metadata_path, metadata).expect("signature tamper should be written");

    // Act
    let error = review_candidate(
        repository.root(),
        candidate.artifact_id(),
        approved_review(),
    )
    .expect_err("changed signature should fail closed");

    // Assert
    assert!(matches!(
        error,
        FixtureError::HashMismatch { .. } | FixtureError::SignatureMismatch
    ));
}

fn mismatch_trace_bytes() -> Vec<u8> {
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_scenario_request_jsonl(REQUEST_BYTES, &limits)
        .expect("checked-in request should validate");
    let handshake_bytes = TRACE_BYTES
        .split_inclusive(|byte| *byte == b'\n')
        .next()
        .expect("checked-in trace should contain a handshake");
    let identity = decode_handshake_jsonl(handshake_bytes, &limits)
        .expect("checked-in handshake should validate")
        .build_identity()
        .clone();
    let begin = TraceBegin::for_request(&request, EngineKind::CppOracle, &identity)
        .expect("trace begin should build");
    let checkpoints = request
        .scenario()
        .checkpoints()
        .iter()
        .enumerate()
        .map(|(ordinal, checkpoint)| {
            let time = if ordinal == 0 { 0.5 } else { 1.25 };
            CheckpointRecord::new(
                request.request_id().clone(),
                checkpoint.checkpoint_id().clone(),
                u32::try_from(ordinal).expect("two checkpoints fit in u32"),
                checkpoint.phase(),
                FloatBits::from_f32(time),
                WorldCounts::zero(),
                identity.identity_sha256().clone(),
            )
            .expect("checkpoint should build")
        })
        .collect::<Vec<_>>();
    let end = TraceEnd::new(
        request.request_id().clone(),
        2,
        trace_payload_sha256(&checkpoints).expect("checkpoint payload should hash"),
        1,
        true,
        identity.identity_sha256().clone(),
    );
    let mut bytes = handshake_bytes.to_vec();
    for record in std::iter::once(TraceRecord::Begin(begin))
        .chain(checkpoints.into_iter().map(TraceRecord::Checkpoint))
        .chain(std::iter::once(TraceRecord::End(end)))
    {
        bytes.extend(
            encode_jsonl(&record, &limits, RecordLimit::Output)
                .expect("typed trace record should encode"),
        );
    }
    bytes
}
