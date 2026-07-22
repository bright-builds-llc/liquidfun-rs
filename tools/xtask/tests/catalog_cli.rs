//! Command-level coverage for the bounded headless scenario catalog surface.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_RUNNER: OnceLock<Result<PathBuf, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

struct CatalogFixture {
    marker: PathBuf,
}

impl CatalogFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let marker = env::temp_dir().join(format!(
            "liquidfun-catalog-cli-{}-{id}.txt",
            std::process::id()
        ));
        match fs::remove_file(&marker) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        Ok(Self { marker })
    }

    fn command(&self) -> io::Result<Command> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .current_dir(workspace_root())
            .env("LIQUIDFUN_XTASK_DIFFERENTIAL", fake_runner()?)
            .env("LIQUIDFUN_TEST_DIFFERENTIAL_MARKER", &self.marker);
        Ok(command)
    }

    fn arguments(&self) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(&self.marker)?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    fn cleanup(self) -> io::Result<()> {
        match fs::remove_file(self.marker) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

#[test]
fn list_and_inspect_forward_only_closed_arguments() -> TestResult {
    // Arrange
    let cases = [
        vec!["list"],
        vec![
            "inspect",
            "--scenario",
            "joint-distance-behavior",
            "--output",
            "json",
        ],
    ];

    for arguments in cases {
        let fixture = CatalogFixture::new()?;
        let mut command = fixture.command()?;
        command.arg("catalog").args(&arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        let mut expected = vec!["catalog".to_owned()];
        expected.extend(arguments.into_iter().map(str::to_owned));
        assert_eq!(fixture.arguments()?, expected);
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn run_replay_and_compare_forward_canonical_structured_arguments() -> TestResult {
    // Arrange
    for action in ["run", "replay", "compare"] {
        let fixture = CatalogFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = catalog_execution_arguments(action);
        command.arg("catalog").args(&arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        let mut expected = vec!["catalog".to_owned()];
        expected.extend(arguments);
        assert_eq!(fixture.arguments()?, expected);
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn run_forwards_a_bounded_typed_controller_script_as_one_argument() -> TestResult {
    // Arrange
    let fixture = CatalogFixture::new()?;
    let mut command = fixture.command()?;
    let script = "pause,resume,step,restart,scenario-action:action-0001,capture:checkpoint-0001";
    let mut arguments = catalog_execution_arguments("run");
    replace_option(&mut arguments, "--commands", script);
    command.arg("catalog").args(&arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    let forwarded = fixture.arguments()?;
    assert_eq!(forwarded[0], "catalog");
    assert_eq!(forwarded[1], "run");
    assert_eq!(forwarded.last().map(String::as_str), Some(script));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn invalid_settings_are_rejected_before_runner_start() -> TestResult {
    // Arrange / Act / Assert
    for (option, value) in [
        ("--timestep", "0"),
        ("--timestep", "NaN"),
        ("--velocity-iterations", "0"),
        ("--position-iterations", "1025"),
        ("--particle-iterations", "-1"),
    ] {
        assert_rejected_option(option, value, "differential/catalog-settings")?;
    }
    Ok(())
}

#[test]
fn unknown_values_and_options_are_rejected_before_runner_start() -> TestResult {
    // Arrange / Act / Assert
    for (option, value, category) in [
        (
            "--scenario",
            "unknown-scenario",
            "differential/catalog-scenario",
        ),
        ("--oracle-preset", "custom", "differential/catalog-usage"),
        ("--session-profile", "forever", "differential/catalog-usage"),
        ("--output", "path", "differential/catalog-usage"),
    ] {
        assert_rejected_option(option, value, category)?;
    }

    let fixture = CatalogFixture::new()?;
    let mut command = fixture.command()?;
    let mut arguments = catalog_execution_arguments("run");
    arguments.extend(["--write-to".to_owned(), "outside".to_owned()]);
    command.arg("catalog").args(arguments);
    let output = command.output()?;
    assert_failure_category(&output, "differential/catalog-usage");
    assert!(!fixture.marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn oversized_or_injectable_scripts_are_rejected_without_shell_effects() -> TestResult {
    // Arrange
    let oversized = std::iter::repeat_n("step", 129)
        .collect::<Vec<_>>()
        .join(",");
    for script in [
        oversized,
        "pause;touch-owned".to_owned(),
        "scenario-action:$(touch-owned)".to_owned(),
        "capture:../../outside".to_owned(),
        "unknown".to_owned(),
    ] {
        let fixture = CatalogFixture::new()?;
        let mut command = fixture.command()?;
        let mut arguments = catalog_execution_arguments("run");
        replace_option(&mut arguments, "--commands", &script);
        command.arg("catalog").args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert_failure_category(&output, "differential/catalog-script");
        assert!(!fixture.marker.exists());
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn catalog_categories_and_runner_statuses_have_stable_exit_codes() -> TestResult {
    // Arrange / Act / Assert
    for (option, value, expected) in [
        ("--scenario", "missing-scenario", 65),
        ("--timestep", "0", 66),
        ("--commands", "unknown", 67),
    ] {
        let fixture = CatalogFixture::new()?;
        let mut command = fixture.command()?;
        let mut arguments = catalog_execution_arguments("run");
        replace_option(&mut arguments, option, value);
        let output = command.arg("catalog").args(arguments).output()?;
        assert_eq!(output.status.code(), Some(expected), "{}", stderr(&output));
        assert!(!fixture.marker.exists());
        fixture.cleanup()?;
    }

    let fixture = CatalogFixture::new()?;
    let mut command = fixture.command()?;
    let output = command
        .env("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL", "1")
        .arg("catalog")
        .args(catalog_execution_arguments("run"))
        .output()?;
    assert_eq!(output.status.code(), Some(42), "{}", stderr(&output));
    fixture.cleanup()?;
    Ok(())
}

fn catalog_execution_arguments(action: &str) -> Vec<String> {
    [
        action,
        "--scenario",
        "joint-distance-behavior",
        "--seed",
        "none",
        "--timestep",
        "0.016666668",
        "--velocity-iterations",
        "8",
        "--position-iterations",
        "3",
        "--particle-iterations",
        "1",
        "--oracle-preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--output",
        "human",
        "--commands",
        "step,capture:checkpoint-0001",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn replace_option(arguments: &mut [String], option: &str, value: &str) {
    let index = arguments
        .iter()
        .position(|argument| argument == option)
        .expect("test option should exist");
    value.clone_into(&mut arguments[index + 1]);
}

fn assert_rejected_option(option: &str, value: &str, category: &str) -> TestResult {
    let fixture = CatalogFixture::new()?;
    let mut command = fixture.command()?;
    let mut arguments = catalog_execution_arguments("run");
    replace_option(&mut arguments, option, value);
    command.arg("catalog").args(arguments);

    let output = command.output()?;

    assert_failure_category(&output, category);
    assert!(!fixture.marker.exists());
    fixture.cleanup()?;
    Ok(())
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected `{category}` in {}",
        stderr(output)
    );
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask is nested under tools")
        .to_path_buf()
}

fn fake_runner() -> io::Result<&'static PathBuf> {
    match FAKE_RUNNER.get_or_init(compile_fake_runner) {
        Ok(path) => Ok(path),
        Err(message) => Err(io::Error::other(message.clone())),
    }
}

fn compile_fake_runner() -> Result<PathBuf, String> {
    let output_directory = env::temp_dir().join(format!(
        "liquidfun-catalog-cli-tools-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_directory).map_err(|error| error.to_string())?;
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_differential_tool.rs");
    let destination = output_directory.join(if cfg!(windows) {
        "fake-catalog.exe"
    } else {
        "fake-catalog"
    });
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(rustc)
        .arg(source)
        .arg("--edition=2024")
        .arg("-o")
        .arg(&destination)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(destination)
}
