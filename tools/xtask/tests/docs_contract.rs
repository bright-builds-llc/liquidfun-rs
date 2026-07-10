//! Command-level coverage for the machine-audited testing-layer contract.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const LAYERS: [&str; 12] = [
    "unit",
    "integration/API",
    "doctest",
    "upstream compatibility",
    "differential",
    "property",
    "checked-in regression",
    "fuzz",
    "Miri/UB-aliasing",
    "native sanitizer",
    "benchmark",
    "coverage",
];
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type TestResult = Result<(), Box<dyn Error>>;

struct DocsFixture {
    root: PathBuf,
}

impl DocsFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-docs-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root)?;
        fs::copy(workspace_root().join("TESTING.md"), root.join("TESTING.md"))?;
        Ok(Self { root })
    }

    fn command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["docs", "check"])
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .output()
    }

    fn remove_layer(&self, layer: &str) -> io::Result<()> {
        let prefix = format!("| {layer} |");
        self.rewrite(|line| (!line.starts_with(&prefix)).then(|| line.to_owned()))
    }

    fn duplicate_layer(&self, layer: &str) -> io::Result<()> {
        let prefix = format!("| {layer} |");
        self.rewrite(|line| {
            if line.starts_with(&prefix) {
                Some(format!("{line}\n{line}"))
            } else {
                Some(line.to_owned())
            }
        })
    }

    fn replace_cell(&self, layer: &str, index: usize, value: &str) -> io::Result<()> {
        let prefix = format!("| {layer} |");
        let mut found = false;
        self.rewrite(|line| {
            if !line.starts_with(&prefix) {
                return Some(line.to_owned());
            }
            found = true;
            let mut cells = parse_row(line).ok()?;
            let cell = cells.get_mut(index)?;
            value.clone_into(cell);
            Some(format!("| {} |", cells.join(" | ")))
        })?;
        if !found {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("testing layer `{layer}` is absent from fixture"),
            ));
        }
        Ok(())
    }

    fn rewrite(&self, mut map_line: impl FnMut(&str) -> Option<String>) -> io::Result<()> {
        let path = self.root.join("TESTING.md");
        let contents = fs::read_to_string(&path)?;
        let rewritten = contents
            .lines()
            .filter_map(&mut map_line)
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{rewritten}\n"))
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn check_accepts_repository_testing_contract() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_each_missing_required_layer() -> TestResult {
    // Arrange, Act, Assert
    for layer in LAYERS {
        let fixture = DocsFixture::new()?;
        fixture.remove_layer(layer)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/layer");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_duplicate_layer() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.duplicate_layer("unit")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/layer");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_every_blank_required_column() -> TestResult {
    // Arrange, Act, Assert
    for index in 0..9 {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", index, "")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/schema");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_invalid_status() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_cell("unit", 1, "active")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/status");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_invalid_placement() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_cell("unit", 7, "nightly")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/placement");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_missing_semantic_markers() -> TestResult {
    // Arrange, Act, Assert
    for (index, value) in [
        (3, "Documented command."),
        (4, "Installed toolchain."),
        (5, "Standard diagnostics."),
        (6, "Run it again."),
        (8, "Some evidence."),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", index, value)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/content");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_forbidden_placeholder_terms() -> TestResult {
    // Arrange, Act, Assert
    for placeholder in ["TODO", "TBD", "placeholder", "REPLACE_ME", "replace with"] {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", 2, placeholder)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/placeholder");
        fixture.cleanup()?;
    }
    Ok(())
}

fn parse_row(line: &str) -> io::Result<Vec<String>> {
    let trimmed = line.trim();
    let Some(contents) = trimmed
        .strip_prefix('|')
        .and_then(|contents| contents.strip_suffix('|'))
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid Markdown table row `{trimmed}`"),
        ));
    };
    Ok(contents
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect())
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(category),
        "expected `{category}` in stderr:\n{stderr}"
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask manifest directory must be nested beneath the workspace root")
}
