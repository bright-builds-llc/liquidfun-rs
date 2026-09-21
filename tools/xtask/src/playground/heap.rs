//! Gated private dhat heap dump for Dam Break, after allocator/`Vec` needles.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaygroundError;
use super::counts::{BenchCounts, DEFAULT_MEASURED_STEPS, DEFAULT_WARMUP_STEPS};
use super::identity::{host_identity, repository_root};
use super::stamp;
use super::symbols::{HeapGate, classify_profile_symbols};

const DUMP_NAME: &str = "dhat-heap.json";
const IDENTITY_NAME: &str = "heap-identity.json";
const PROFILE_BLOB: &str = "rust.json.gz";
const BUNDLE_IDENTITY: &str = "audit-bundle-identity.json";
const PROFILE_IDENTITY: &str = "profile-identity.json";
const DISCLAIMER: &str =
    "dhat dump is not the 3x number; unprofiled pair.json remains timing authority.";

struct HeapFlags {
    maybe_stamp: Option<String>,
    counts: BenchCounts,
}

/// Runs `dam-break-heap` only after the sidecar-first allocator/`Vec` heuristic matches.
///
/// # Errors
///
/// Returns a closed error when flags, symbols, cargo, or the dump fail closed.
pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let flags = parse_heap_args(args)?;
    let repository_root = repository_root()?;
    run_from(&repository_root, &flags)
}

fn run_from(repository_root: &Path, flags: &HeapFlags) -> Result<(), PlaygroundError> {
    let identity = host_identity(repository_root);
    let source_dir = resolve_source_dir(
        repository_root,
        &identity.git_head,
        flags.maybe_stamp.as_deref(),
    )?;
    let source_stamp = source_stamp_name(&source_dir)?;
    match classify_profile_symbols(&source_dir)? {
        HeapGate::SkipNoAllocator => Err(heap_err(
            "Heap skipped: samply showed no allocator/Vec time",
        )),
        HeapGate::SkipNoSymbols => Err(heap_err("Heap skipped: could not read symbols")),
        HeapGate::Run { matched_needles } => spawn_dhat(
            repository_root,
            &identity.git_head,
            source_stamp,
            &matched_needles,
            &flags.counts,
        ),
    }
}

fn spawn_dhat(
    repository_root: &Path,
    git_head: &str,
    source_stamp: &str,
    matched_needles: &[String],
    counts: &BenchCounts,
) -> Result<(), PlaygroundError> {
    let unix_seconds = stamp_unix_seconds()?;
    let stamp_dir = stamp::mint_exclusive_stamp(repository_root, unix_seconds)?;
    let dump_path = stamp_dir.join(DUMP_NAME);
    let cargo = cargo_program();
    let warmup = counts.warmup_steps.to_string();
    let steps = counts.measured_steps.to_string();
    let output = Command::new(&cargo)
        .current_dir(repository_root)
        .env("LIQUIDFUN_DHAT_HEAP_FILE", &dump_path)
        .args([
            "run",
            "-p",
            "liquidfun-wasm",
            "--profile",
            "profiling",
            "--bin",
            "dam-break-bench",
            "--features",
            "dhat-heap",
            "--",
            "--warmup",
            &warmup,
            "--steps",
            &steps,
        ])
        .output()
        .map_err(|error| heap_err(format!("{}: {error}", PathBuf::from(&cargo).display())))?;
    if !output.status.success() {
        return Err(heap_err(format!(
            "dhat-heap cargo run failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    require_nonempty_dump(&dump_path)?;
    write_heap_identity(&stamp_dir, git_head, source_stamp, matched_needles, counts)?;
    eprintln!("wrote {}", dump_path.display());
    Ok(())
}

fn parse_heap_args(args: &[String]) -> Result<HeapFlags, PlaygroundError> {
    let mut maybe_stamp = None;
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--stamp" => {
                maybe_stamp = Some(required_stamp_flag(args, index)?);
                index += 2;
            }
            "--warmup" => {
                warmup_steps = parse_u32_flag(args, index, "--warmup")?;
                index += 2;
            }
            "--steps" => {
                measured_steps = parse_u32_flag(args, index, "--steps")?;
                index += 2;
            }
            unknown => {
                return Err(PlaygroundError::usage(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }
    if measured_steps == 0 {
        return Err(PlaygroundError::usage("--steps must be greater than 0"));
    }
    Ok(HeapFlags {
        maybe_stamp,
        counts: BenchCounts {
            warmup_steps,
            measured_steps,
        },
    })
}

fn required_stamp_flag(args: &[String], index: usize) -> Result<String, PlaygroundError> {
    let Some(raw) = args.get(index + 1) else {
        return Err(PlaygroundError::usage("--stamp requires a stamp name"));
    };
    stamp::parse_stamp_name(raw)
}

fn parse_u32_flag(args: &[String], index: usize, flag: &str) -> Result<u32, PlaygroundError> {
    let Some(raw) = args.get(index + 1) else {
        return Err(PlaygroundError::usage(format!(
            "{flag} requires a non-negative integer"
        )));
    };
    raw.parse::<u32>()
        .map_err(|_error| PlaygroundError::usage(format!("{flag} requires a non-negative integer")))
}

fn resolve_source_dir(
    repository_root: &Path,
    expected_git_head: &str,
    maybe_stamp: Option<&str>,
) -> Result<PathBuf, PlaygroundError> {
    let evidence = stamp::evidence_dir(repository_root);
    if let Some(raw) = maybe_stamp {
        let stamp_name = stamp::parse_stamp_name(raw)?;
        let dir = evidence.join(&stamp_name);
        if !dir.is_dir() {
            return Err(heap_err(format!("stamp {stamp_name} is missing")));
        }
        return Ok(dir);
    }
    latest_profile_or_bundle(&evidence, expected_git_head)
}

fn latest_profile_or_bundle(
    evidence: &Path,
    expected_git_head: &str,
) -> Result<PathBuf, PlaygroundError> {
    let entries = fs::read_dir(evidence)
        .map_err(|error| heap_err(format!("failed to read {}: {error}", evidence.display())))?;
    let mut matches = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if stamp::parse_stamp_name(name).is_err() {
            continue;
        }
        if !path.join(PROFILE_BLOB).is_file() {
            continue;
        }
        if stamp_matches_head(&path, expected_git_head) {
            matches.push(name.to_owned());
        }
    }
    matches.sort();
    let Some(stamp_name) = matches.pop() else {
        return Err(heap_err(
            "no profile or audit-bundle stamp with rust.json.gz for current HEAD",
        ));
    };
    Ok(evidence.join(stamp_name))
}

fn stamp_matches_head(dir: &Path, expected_git_head: &str) -> bool {
    identity_matches(
        &dir.join(BUNDLE_IDENTITY),
        "audit_bundle",
        expected_git_head,
    ) || identity_matches(&dir.join(PROFILE_IDENTITY), "samply_cpu", expected_git_head)
}

fn identity_matches(path: &Path, required_kind: &str, expected_git_head: &str) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    value.get("kind").and_then(serde_json::Value::as_str) == Some(required_kind)
        && value.get("git_head").and_then(serde_json::Value::as_str) == Some(expected_git_head)
}

fn source_stamp_name(source_dir: &Path) -> Result<&str, PlaygroundError> {
    let Some(name) = source_dir.file_name().and_then(|name| name.to_str()) else {
        return Err(heap_err("source stamp name is not UTF-8"));
    };
    Ok(name)
}

fn require_nonempty_dump(path: &Path) -> Result<(), PlaygroundError> {
    let metadata = fs::metadata(path)
        .map_err(|error| heap_err(format!("missing {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(heap_err(format!("{} is missing or empty", path.display())));
    }
    Ok(())
}

fn write_heap_identity(
    stamp_dir: &Path,
    git_head: &str,
    source_stamp: &str,
    matched_needles: &[String],
    counts: &BenchCounts,
) -> Result<(), PlaygroundError> {
    let report = serde_json::json!({
        "kind": "dhat_heap",
        "not_timing_authority": true,
        "cargo_profile": "profiling",
        "features": ["dhat-heap"],
        "git_head": git_head,
        "source_stamp": source_stamp,
        "matched_needles": matched_needles,
        "dump": DUMP_NAME,
        "warmup_steps": counts.warmup_steps,
        "measured_steps": counts.measured_steps,
        "disclaimer": DISCLAIMER,
    });
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| heap_err(format!("failed to serialize {IDENTITY_NAME}: {error}")))?;
    let path = stamp_dir.join(IDENTITY_NAME);
    fs::write(&path, json.as_bytes())
        .map_err(|error| heap_err(format!("failed to write {}: {error}", path.display())))
}

fn cargo_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_CARGO")
        .or_else(|| env::var_os("CARGO"))
        .unwrap_or_else(|| OsString::from("cargo"))
}

fn stamp_unix_seconds() -> Result<u64, PlaygroundError> {
    match env::var("LIQUIDFUN_XTASK_STAMP_UNIX") {
        Ok(raw) => parse_stamp_unix(&raw),
        Err(env::VarError::NotPresent) => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|error| {
                PlaygroundError::new(
                    "stamp",
                    format!("system clock is before Unix epoch: {error}"),
                )
            }),
        Err(env::VarError::NotUnicode(_)) => Err(PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer",
        )),
    }
}

fn parse_stamp_unix(raw: &str) -> Result<u64, PlaygroundError> {
    if raw.contains('/') || raw.contains('\\') || raw.contains("..") {
        return Err(PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer, not a path",
        ));
    }
    raw.parse::<u64>().map_err(|_| {
        PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer",
        )
    })
}

fn heap_err(message: impl Into<String>) -> PlaygroundError {
    PlaygroundError::new("heap", message)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::{HeapFlags, parse_heap_args, run_from};
    use crate::playground::counts::{DEFAULT_MEASURED_STEPS, DEFAULT_WARMUP_STEPS};

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
    const SOURCE_STAMP: &str = "2001-09-09T01-46-40Z";

    fn fixture_root(label: &str) -> PathBuf {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target/xtask-test-fixtures")
            .join(format!(
                "playground-heap-{}-{id}-{label}",
                std::process::id()
            ));
        fs::create_dir_all(root.join("target/dam-break-perf"))
            .expect("fixture evidence dir should be created");
        root
    }

    fn source_dir(root: &Path) -> PathBuf {
        let dir = root.join("target/dam-break-perf").join(SOURCE_STAMP);
        fs::create_dir_all(&dir).expect("source stamp should be created");
        dir
    }

    fn write_gzip(path: &Path, body: &[u8]) {
        let file = fs::File::create(path).expect("gzip file should be created");
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(body).expect("gzip body should write");
        encoder.finish().expect("gzip should finish");
    }

    fn stamp_flags() -> HeapFlags {
        parse_heap_args(&[String::from("--stamp"), String::from(SOURCE_STAMP)])
            .expect("stamp flags should parse")
    }

    fn evidence_stamp_names(root: &Path) -> Vec<String> {
        let evidence = root.join("target/dam-break-perf");
        let mut names = Vec::new();
        let entries = fs::read_dir(&evidence).expect("evidence dir should exist");
        for entry in entries {
            let entry = entry.expect("evidence entry");
            if entry.path().is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        names
    }

    fn dump_exists(root: &Path) -> bool {
        let evidence = root.join("target/dam-break-perf");
        let Ok(entries) = fs::read_dir(&evidence) else {
            return false;
        };
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            if entry.path().join("dhat-heap.json").is_file() {
                return true;
            }
        }
        false
    }

    #[test]
    fn parse_heap_args_defaults_and_rejects_bundle_flags() {
        // Arrange / Act
        let defaults = parse_heap_args(&[]).expect("defaults should parse");
        let pair = parse_heap_args(&[String::from("--pair-stamp"), String::from(SOURCE_STAMP)]);
        let profile =
            parse_heap_args(&[String::from("--profile-stamp"), String::from(SOURCE_STAMP)]);

        // Assert
        assert_eq!(defaults.counts.warmup_steps, DEFAULT_WARMUP_STEPS);
        assert_eq!(defaults.counts.measured_steps, DEFAULT_MEASURED_STEPS);
        assert!(pair.is_err());
        assert!(profile.is_err());
        let Err(pair_error) = pair else {
            panic!("pair-stamp should be unknown");
        };
        let Err(profile_error) = profile else {
            panic!("profile-stamp should be unknown");
        };
        let pair_message = pair_error.to_string();
        let profile_message = profile_error.to_string();
        assert!(pair_message.contains("unknown argument `--pair-stamp`"));
        assert!(profile_message.contains("unknown argument `--profile-stamp`"));
    }

    #[test]
    fn skip_no_symbols_does_not_mint_or_write_dump() {
        // Arrange
        let root = fixture_root("no-symbols");
        let dir = source_dir(&root);
        fs::write(dir.join("rust.json.gz"), b"fake-samply-json-gz")
            .expect("fake gzip bytes should write");

        // Act
        let result = run_from(&root, &stamp_flags());

        // Assert
        let error = result.expect_err("inconclusive symbols should skip");
        let message = error.to_string();
        assert!(message.contains("could not read symbols"));
        assert!(!message.contains("no allocator"));
        assert_eq!(evidence_stamp_names(&root), vec![SOURCE_STAMP.to_owned()]);
        assert!(!dump_exists(&root));
        assert!(!dir.join("heap-identity.json").is_file());
    }

    #[test]
    fn skip_no_allocator_does_not_mint_or_write_dump() {
        // Arrange
        let root = fixture_root("no-allocator");
        let dir = source_dir(&root);
        write_gzip(
            &dir.join("rust.json.gz"),
            br#"{"threads":[{"stringArray":["liquidfun::particle::proxy::from_view"]}]}"#,
        );

        // Act
        let result = run_from(&root, &stamp_flags());

        // Assert
        let error = result.expect_err("readable symbols without needles should skip");
        let message = error.to_string();
        assert!(message.contains("no allocator") || message.contains("no allocator/Vec"));
        assert!(!message.contains("could not read symbols"));
        assert_eq!(evidence_stamp_names(&root), vec![SOURCE_STAMP.to_owned()]);
        assert!(!dump_exists(&root));
    }
}
