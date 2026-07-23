//! Repository policy for the shared Phase 12 unstable-tool toolchain.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const PHASE12_NIGHTLY: &str = "nightly-2026-07-15";
const REQUIRED_COMPONENTS: [&str; 3] = ["llvm-tools-preview", "miri", "rust-src"];

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn shared_nightly_toolchain_is_exact_and_complete() -> TestResult {
    // Arrange
    let path = workspace_root().join("rust-toolchain-nightly.toml");

    // Act
    let manifest: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let toolchain = manifest
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .ok_or("nightly toolchain table is missing")?;
    let components = toolchain
        .get("components")
        .and_then(toml::Value::as_array)
        .ok_or("nightly toolchain components are missing")?
        .iter()
        .map(|value| value.as_str().ok_or("component must be a string"))
        .collect::<Result<BTreeSet<_>, _>>()?;

    // Assert
    assert_eq!(
        toolchain.get("channel").and_then(toml::Value::as_str),
        Some(PHASE12_NIGHTLY)
    );
    assert_eq!(
        toolchain.get("profile").and_then(toml::Value::as_str),
        Some("minimal")
    );
    assert_eq!(components, REQUIRED_COMPONENTS.into_iter().collect());
    Ok(())
}

#[test]
fn floating_nightly_detector_rejects_every_alias_shape() {
    // Arrange
    let floating = [
        "cargo +nightly fuzz run protocol",
        "channel = \"nightly\"",
        "toolchain: nightly",
        "toolchain: \"nightly\"",
        "rustup toolchain install nightly --profile minimal",
        "rustup run nightly rustc --version",
        "NIGHTLY=nightly",
    ];

    // Act, Assert
    for line in floating {
        assert!(
            floating_nightly_reference(line),
            "floating nightly reference was not rejected: {line}"
        );
    }
    assert!(!floating_nightly_reference(
        "cargo +nightly-2026-07-15 fuzz build"
    ));
    assert!(!floating_nightly_reference(
        "channel = \"nightly-2026-07-15\""
    ));
    assert!(!floating_nightly_reference("toolchain: nightly-2026-07-15"));
    assert!(!floating_nightly_reference(
        "rustup toolchain install nightly-2026-07-15 --profile minimal"
    ));
    assert!(!floating_nightly_reference(
        "rustup run nightly-2026-07-15 rustc --version"
    ));
    assert!(!floating_nightly_reference("NIGHTLY=nightly-2026-07-15"));
}

#[test]
fn phase12_consumers_have_no_floating_nightly_aliases() -> TestResult {
    // Arrange
    let root = workspace_root();
    let mut files = vec![
        root.join("rust-toolchain.toml"),
        root.join("rust-toolchain-nightly.toml"),
        root.join("justfile"),
    ];
    for directory in [".github/workflows", "scripts", "tools"] {
        collect_policy_files(&root.join(directory), &mut files)?;
    }

    // Act
    let mut violations = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        for (index, line) in contents.lines().enumerate() {
            if floating_nightly_reference(line) {
                violations.push(format!("{}:{}: {line}", path.display(), index + 1));
            }
        }
    }

    // Assert
    assert!(
        violations.is_empty(),
        "floating nightly references:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn fuzz_and_safety_contract_plans_depend_on_the_shared_pin() -> TestResult {
    // Arrange
    let phase =
        workspace_root().join(".planning/phases/12-performance-portability-and-release-hardening");

    // Act, Assert
    for plan in ["12-09-PLAN.md", "12-10-PLAN.md"] {
        let contents = fs::read_to_string(phase.join(plan))?;
        assert!(
            contents.contains("depends_on: [\"12-17\"]"),
            "{plan} must depend on the shared nightly predecessor"
        );
    }
    Ok(())
}

fn collect_policy_files(directory: &Path, files: &mut Vec<PathBuf>) -> TestResult {
    if !directory.is_dir() {
        return Ok(());
    }
    for maybe_entry in fs::read_dir(directory)? {
        let entry = maybe_entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_policy_files(&path, files)?;
            continue;
        }
        if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("sh" | "toml" | "yaml" | "yml")
        ) {
            files.push(path);
        }
    }
    Ok(())
}

fn floating_nightly_reference(line: &str) -> bool {
    let compact = line.trim().replace('\'', "\"");
    if compact
        .match_indices("+nightly")
        .any(|(index, _)| !compact[index + "+nightly".len()..].starts_with("-20"))
    {
        return true;
    }
    if compact == "channel = \"nightly\"" {
        return true;
    }
    if compact
        .split_once("toolchain:")
        .is_some_and(|(_, value)| value.trim().trim_matches('"') == "nightly")
    {
        return true;
    }
    compact
        .split_whitespace()
        .any(|token| token == "NIGHTLY=nightly")
        || (compact.contains("rustup")
            && compact.split_whitespace().any(|token| {
                token.trim_matches(|character: char| {
                    character == '"' || character == ',' || character == ';'
                }) == "nightly"
            }))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
