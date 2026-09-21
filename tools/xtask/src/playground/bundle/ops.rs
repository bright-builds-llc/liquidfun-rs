//! Copy-only same-HEAD audit-bundle helpers.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::PlaygroundError;
use super::super::identity::{host_identity, repository_root};
use super::super::stamp;

const PAIR_JSON: &str = "pair.json";
const PAIR_MD: &str = "pair.md";
const PROFILE_BLOB: &str = "rust.json.gz";
const PROFILE_IDENTITY: &str = "profile-identity.json";
const BUNDLE_IDENTITY: &str = "audit-bundle-identity.json";
const DISCLAIMER: &str =
    "Unreviewed local playground Dam Break sample. Profile blobs are not the 3x number.";
const FORBIDDEN_PAIR_KEYS: [&str; 4] =
    ["samply", "profile", "not_timing_authority", "cargo_profile"];

struct BundleFlags {
    maybe_pair_stamp: Option<String>,
    maybe_profile_stamp: Option<String>,
}

/// Runs `dam-break-audit-bundle`, copying same-HEAD pair and profile artifacts.
///
/// # Errors
///
/// Returns a closed error when flags, sources, kinds, or git HEADs fail closed.
pub(crate) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let flags = parse_bundle_args(args)?;
    let repository_root = repository_root()?;
    let identity = host_identity(&repository_root);
    let unix_seconds = stamp_unix_seconds()?;
    let maybe_worktree_dirty = worktree_dirty(&repository_root);
    if maybe_worktree_dirty == Some(true) {
        eprintln!("warning: worktree is dirty");
    }
    let stamp_dir = copy_audit_bundle(
        &repository_root,
        &identity.git_head,
        flags.maybe_pair_stamp.as_deref(),
        flags.maybe_profile_stamp.as_deref(),
        unix_seconds,
        maybe_worktree_dirty,
    )?;
    eprintln!("wrote {}", stamp_dir.display());
    Ok(())
}

/// Copies same-HEAD pair and profile artifacts into a new exclusive stamp.
///
/// # Errors
///
/// Returns a closed error when stamp names, source files, kinds, or git HEADs
/// do not match the copy-only audit-bundle contract.
pub(crate) fn copy_audit_bundle(
    repository_root: &Path,
    expected_git_head: &str,
    maybe_pair_stamp: Option<&str>,
    maybe_profile_stamp: Option<&str>,
    unix_seconds: u64,
    maybe_worktree_dirty: Option<bool>,
) -> Result<PathBuf, PlaygroundError> {
    let evidence = stamp::evidence_dir(repository_root);
    let pair_stamp = resolve_stamp(
        &evidence,
        maybe_pair_stamp,
        expected_git_head,
        PAIR_JSON,
        "unprofiled_pair",
        "pair",
    )?;
    let profile_stamp = resolve_stamp(
        &evidence,
        maybe_profile_stamp,
        expected_git_head,
        PROFILE_IDENTITY,
        "samply_cpu",
        "profile",
    )?;
    let pair_dir = evidence.join(&pair_stamp);
    let profile_dir = evidence.join(&profile_stamp);
    let pair_report = read_json(&pair_dir.join(PAIR_JSON))?;
    validate_pair_report(&pair_report, expected_git_head)?;
    let profile_report = read_json(&profile_dir.join(PROFILE_IDENTITY))?;
    validate_kind_and_head(&profile_report, "samply_cpu", expected_git_head, "profile")?;
    require_pair_files(&pair_dir)?;
    require_nonempty_blob(&profile_dir.join(PROFILE_BLOB))?;
    let dest = stamp::mint_exclusive_stamp(repository_root, unix_seconds)?;
    let mut copied = Vec::new();
    copy_named(&pair_dir, &dest, PAIR_JSON, &mut copied)?;
    copy_named(&pair_dir, &dest, PAIR_MD, &mut copied)?;
    copy_named(&profile_dir, &dest, PROFILE_BLOB, &mut copied)?;
    copy_syms_sidecars(&profile_dir, &dest, &mut copied)?;
    write_bundle_identity(
        &dest,
        expected_git_head,
        &pair_stamp,
        &profile_stamp,
        &copied,
        maybe_worktree_dirty,
    )?;
    Ok(dest)
}

fn parse_bundle_args(args: &[String]) -> Result<BundleFlags, PlaygroundError> {
    let mut maybe_pair_stamp = None;
    let mut maybe_profile_stamp = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--pair-stamp" => {
                maybe_pair_stamp = Some(required_stamp_flag(args, index, "--pair-stamp")?);
                index += 2;
            }
            "--profile-stamp" => {
                maybe_profile_stamp = Some(required_stamp_flag(args, index, "--profile-stamp")?);
                index += 2;
            }
            unknown => {
                return Err(PlaygroundError::usage(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }
    Ok(BundleFlags {
        maybe_pair_stamp,
        maybe_profile_stamp,
    })
}

fn required_stamp_flag(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<String, PlaygroundError> {
    let Some(raw) = args.get(index + 1) else {
        return Err(bundle_err(format!("{flag} requires a stamp name")));
    };
    stamp::parse_stamp_name(raw)
}

fn resolve_stamp(
    evidence: &Path,
    maybe_stamp: Option<&str>,
    expected_git_head: &str,
    identity_file: &str,
    required_kind: &str,
    label: &str,
) -> Result<String, PlaygroundError> {
    if let Some(raw) = maybe_stamp {
        let stamp_name = stamp::parse_stamp_name(raw)?;
        let dir = evidence.join(&stamp_name);
        if !dir.is_dir() {
            return Err(bundle_err(format!("{label} stamp {stamp_name} is missing")));
        }
        return Ok(stamp_name);
    }
    latest_matching_stamp(
        evidence,
        expected_git_head,
        identity_file,
        required_kind,
        label,
    )
}

fn latest_matching_stamp(
    evidence: &Path,
    expected_git_head: &str,
    identity_file: &str,
    required_kind: &str,
    label: &str,
) -> Result<String, PlaygroundError> {
    let entries = fs::read_dir(evidence)
        .map_err(|error| bundle_err(format!("failed to read {}: {error}", evidence.display())))?;
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
        let Ok(report) = read_json(&path.join(identity_file)) else {
            continue;
        };
        let maybe_kind = report.get("kind").and_then(serde_json::Value::as_str);
        let Some(kind) = maybe_kind else {
            continue;
        };
        if kind != required_kind {
            continue;
        }
        let maybe_head = report.get("git_head").and_then(serde_json::Value::as_str);
        let Some(git_head) = maybe_head else {
            continue;
        };
        if git_head == expected_git_head {
            matches.push(name.to_owned());
        }
    }
    matches.sort();
    let Some(stamp_name) = matches.pop() else {
        return Err(bundle_err(format!(
            "no {label} stamp with kind `{required_kind}` for current HEAD"
        )));
    };
    Ok(stamp_name)
}

fn validate_pair_report(
    report: &serde_json::Value,
    expected_git_head: &str,
) -> Result<(), PlaygroundError> {
    validate_kind_and_head(report, "unprofiled_pair", expected_git_head, "pair")?;
    for key in FORBIDDEN_PAIR_KEYS {
        if json_contains_key(report, key) {
            return Err(bundle_err(format!("pair.json must not contain `{key}`")));
        }
    }
    Ok(())
}

fn validate_kind_and_head(
    report: &serde_json::Value,
    required_kind: &str,
    expected_git_head: &str,
    label: &str,
) -> Result<(), PlaygroundError> {
    let maybe_kind = report.get("kind").and_then(serde_json::Value::as_str);
    let Some(kind) = maybe_kind else {
        return Err(bundle_err(format!("{label} report is missing `kind`")));
    };
    if kind != required_kind {
        return Err(bundle_err(format!(
            "{label} kind must be `{required_kind}`, got `{kind}`"
        )));
    }
    let maybe_head = report.get("git_head").and_then(serde_json::Value::as_str);
    let Some(git_head) = maybe_head else {
        return Err(bundle_err(format!("{label} report is missing `git_head`")));
    };
    if git_head != expected_git_head {
        return Err(bundle_err(format!(
            "{label} git_head `{git_head}` does not match current HEAD"
        )));
    }
    Ok(())
}

fn require_pair_files(pair_dir: &Path) -> Result<(), PlaygroundError> {
    for name in [PAIR_JSON, PAIR_MD] {
        let path = pair_dir.join(name);
        if !path.is_file() {
            return Err(bundle_err(format!("missing {}", path.display())));
        }
    }
    Ok(())
}

fn require_nonempty_blob(path: &Path) -> Result<(), PlaygroundError> {
    let metadata = fs::metadata(path)
        .map_err(|error| bundle_err(format!("missing {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(bundle_err(format!(
            "{} is missing or empty",
            path.display()
        )));
    }
    Ok(())
}

fn copy_named(
    src_dir: &Path,
    dest: &Path,
    name: &str,
    copied: &mut Vec<String>,
) -> Result<(), PlaygroundError> {
    let from = src_dir.join(name);
    let to = dest.join(name);
    super::copy_file(&from, &to)?;
    copied.push(name.to_owned());
    Ok(())
}

fn copy_syms_sidecars(
    profile_dir: &Path,
    dest: &Path,
    copied: &mut Vec<String>,
) -> Result<(), PlaygroundError> {
    let entries = fs::read_dir(profile_dir).map_err(|error| {
        bundle_err(format!("failed to read {}: {error}", profile_dir.display()))
    })?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            bundle_err(format!("failed to read {}: {error}", profile_dir.display()))
        })?;
        if !entry.path().is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.contains("syms") {
            names.push(name.to_owned());
        }
    }
    names.sort();
    for name in names {
        copy_named(profile_dir, dest, &name, copied)?;
    }
    Ok(())
}

fn write_bundle_identity(
    dest: &Path,
    git_head: &str,
    source_pair_stamp: &str,
    source_profile_stamp: &str,
    copied: &[String],
    maybe_worktree_dirty: Option<bool>,
) -> Result<(), PlaygroundError> {
    let mut report = serde_json::json!({
        "kind": "audit_bundle",
        "timing_authority": "unprofiled_wall_clock",
        "not_timing_authority": true,
        "git_head": git_head,
        "source_pair_stamp": source_pair_stamp,
        "source_profile_stamp": source_profile_stamp,
        "copied": copied,
        "profile_blob": PROFILE_BLOB,
        "disclaimer": DISCLAIMER,
    });
    if let Some(dirty) = maybe_worktree_dirty {
        let Some(object) = report.as_object_mut() else {
            return Err(bundle_err("audit-bundle-identity.json object missing"));
        };
        object.insert("worktree_dirty".to_owned(), serde_json::Value::Bool(dirty));
    }
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| bundle_err(format!("failed to serialize {BUNDLE_IDENTITY}: {error}")))?;
    let path = dest.join(BUNDLE_IDENTITY);
    fs::write(&path, json.as_bytes())
        .map_err(|error| bundle_err(format!("failed to write {}: {error}", path.display())))
}

fn read_json(path: &Path) -> Result<serde_json::Value, PlaygroundError> {
    let bytes = fs::read(path)
        .map_err(|error| bundle_err(format!("failed to read {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| bundle_err(format!("{} is not JSON: {error}", path.display())))
}

fn json_contains_key(value: &serde_json::Value, key: &str) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            map.contains_key(key) || map.values().any(|nested| json_contains_key(nested, key))
        }
        serde_json::Value::Array(items) => {
            items.iter().any(|nested| json_contains_key(nested, key))
        }
        _ => false,
    }
}

fn worktree_dirty(root: &Path) -> Option<bool> {
    let git_program = env::var_os("LIQUIDFUN_XTASK_GIT").unwrap_or_else(|| OsString::from("git"));
    let output = Command::new(git_program)
        .current_dir(root)
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
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

fn bundle_err(message: impl Into<String>) -> PlaygroundError {
    PlaygroundError::new("bundle", message)
}
