//! Private repository orchestration for `liquidfun-rs`.

mod differential;
mod docs;
mod inventory;
mod package;
mod performance;
mod phase10_evidence;
mod phase11_evidence;
mod phase13_acceptance;
mod phase13_evidence;
#[path = "phase13_evidence/promotion.rs"]
mod phase13_promotion;
mod phase9_evidence;
mod provenance;
mod release;
mod safety_evidence;
mod upstream;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = r"Usage: cargo xtask <command> [arguments]

Commands:
  catalog     Run renderer-free reviewed catalog scenarios
  differential Manage semantic Rust/C++ comparison workflows
  docs        Validate documentation contracts
  upstream    Manage the pinned upstream oracle
  inventory   Manage the compatibility inventory
  provenance  Validate provenance records
  phase9-evidence Validate local or exact-ref Phase 9 evidence
  phase10-evidence Validate local or exact-ref Phase 10 evidence
  phase11-evidence Validate local or exact-ref Phase 11 evidence
  safety-evidence Validate typed regression, safety, and coverage evidence
  package     Validate the publishable package
  performance Run sealed paired performance and analysis workflows
  phase13     Produce or validate immutable Phase 13 staged evidence
  release     Audit and attest one complete commit-bound release evidence set
  check       Run the aggregate repository checks";

#[derive(Debug, PartialEq, Eq)]
enum XtaskError {
    Check { message: String },
    Usage { message: String },
    Differential(differential::DifferentialError),
    Docs(docs::DocsError),
    Inventory(inventory::InventoryError),
    Package(package::PackageError),
    Performance(performance::PerformanceCommandError),
    Phase13Acceptance(phase13_acceptance::AcceptanceError),
    Phase13Evidence(phase13_evidence::Phase13EvidenceError),
    Phase13Promotion(phase13_promotion::PromotionError),
    Phase9Evidence(phase9_evidence::Phase9EvidenceError),
    Phase10Evidence(phase10_evidence::Phase10EvidenceError),
    Phase11Evidence(phase11_evidence::Phase11EvidenceError),
    Provenance(provenance::ProvenanceError),
    Release(release::ReleaseError),
    SafetyEvidence(safety_evidence::SafetyEvidenceError),
    Upstream(upstream::UpstreamError),
}

impl XtaskError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage {
            message: message.into(),
        }
    }

    fn check(message: impl Into<String>) -> Self {
        Self::Check {
            message: message.into(),
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Differential(error) => error.exit_code(),
            _ => 1,
        }
    }
}

impl Display for XtaskError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check { message } => write!(formatter, "check: {message}"),
            Self::Usage { message } => write!(formatter, "{message}\n\n{USAGE}"),
            Self::Differential(error) => Display::fmt(error, formatter),
            Self::Docs(error) => Display::fmt(error, formatter),
            Self::Inventory(error) => Display::fmt(error, formatter),
            Self::Package(error) => Display::fmt(error, formatter),
            Self::Performance(error) => Display::fmt(error, formatter),
            Self::Phase13Acceptance(error) => Display::fmt(error, formatter),
            Self::Phase13Evidence(error) => Display::fmt(error, formatter),
            Self::Phase13Promotion(error) => Display::fmt(error, formatter),
            Self::Phase9Evidence(error) => Display::fmt(error, formatter),
            Self::Phase10Evidence(error) => Display::fmt(error, formatter),
            Self::Phase11Evidence(error) => Display::fmt(error, formatter),
            Self::Provenance(error) => Display::fmt(error, formatter),
            Self::Release(error) => Display::fmt(error, formatter),
            Self::SafetyEvidence(error) => Display::fmt(error, formatter),
            Self::Upstream(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for XtaskError {}

fn dispatch(args: &[String]) -> Result<(), XtaskError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(XtaskError::usage("missing command"));
    };

    match command.as_str() {
        "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        "upstream" => upstream::run(command_args).map_err(XtaskError::Upstream),
        "catalog" => differential::run_catalog(command_args).map_err(XtaskError::Differential),
        "differential" => differential::run(command_args).map_err(XtaskError::Differential),
        "docs" => docs::run(command_args).map_err(XtaskError::Docs),
        "inventory" => inventory::run(command_args).map_err(XtaskError::Inventory),
        "provenance" => provenance::run(command_args).map_err(XtaskError::Provenance),
        "package" => package::run(command_args).map_err(XtaskError::Package),
        "performance" => performance::run(command_args).map_err(XtaskError::Performance),
        "phase13" if matches!(command_args, [command, ..] if command == "acceptance") => {
            phase13_acceptance::run(command_args).map_err(XtaskError::Phase13Acceptance)
        }
        "phase13" if phase13_promotion::handles(command_args) => {
            phase13_promotion::run(command_args).map_err(XtaskError::Phase13Promotion)
        }
        "phase13" => phase13_evidence::run(command_args).map_err(XtaskError::Phase13Evidence),
        "release" => release::run(command_args).map_err(XtaskError::Release),
        "safety-evidence" => safety_evidence::run(command_args).map_err(XtaskError::SafetyEvidence),
        "phase9-evidence" => phase9_evidence::run(command_args).map_err(XtaskError::Phase9Evidence),
        "phase10-evidence" => {
            phase10_evidence::run(command_args).map_err(XtaskError::Phase10Evidence)
        }
        "phase11-evidence" => {
            phase11_evidence::run(command_args).map_err(XtaskError::Phase11Evidence)
        }
        "check" => {
            if !command_args.is_empty() {
                return Err(XtaskError::usage("check does not accept arguments"));
            }
            check()
        }
        unknown => Err(XtaskError::usage(format!("unknown command `{unknown}`"))),
    }
}

fn check() -> Result<(), XtaskError> {
    let repository_root = repository_root()?;
    let upstream_path = repository_root.join("third_party/liquidfun");
    let upstream_initialized = directory_has_entries(&upstream_path)?;
    let check_argument = ["check".to_owned()];

    if upstream_initialized {
        println!("check: inventory");
        inventory::run(&check_argument).map_err(XtaskError::Inventory)?;
    } else {
        println!(
            "check: Cargo-only mode - third_party/liquidfun is not initialized; \
             skipping inventory, upstream identity, and full provenance checks"
        );
    }

    println!("check: package isolation");
    let package_argument = ["verify".to_owned()];
    package::run(&package_argument).map_err(XtaskError::Package)?;

    println!("check: protocol schema presentations and fixtures");
    differential::check_protocol(&repository_root).map_err(XtaskError::Differential)?;

    println!("check: documentation contracts");
    docs::run(&check_argument).map_err(XtaskError::Docs)?;

    if upstream_initialized {
        println!("check: upstream identity");
        let upstream_argument = ["verify".to_owned()];
        upstream::run(&upstream_argument).map_err(XtaskError::Upstream)?;

        println!("check: provenance");
        provenance::run(&check_argument).map_err(XtaskError::Provenance)?;
    } else {
        println!("check: artifact provenance (Cargo-only)");
        provenance::check_artifacts(&repository_root).map_err(XtaskError::Provenance)?;
    }

    println!("check: all applicable repository checks passed");
    Ok(())
}

fn repository_root() -> Result<PathBuf, XtaskError> {
    let current_dir = std::env::current_dir()
        .map_err(|error| XtaskError::check(format!("failed to read current directory: {error}")))?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/liquidfun/Cargo.toml").is_file()
    }) else {
        return Err(XtaskError::check(
            "could not find the liquidfun Cargo workspace",
        ));
    };
    Ok(root.to_path_buf())
}

fn directory_has_entries(path: &Path) -> Result<bool, XtaskError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(XtaskError::check(format!(
                "failed to inspect {}: {error}",
                path.display()
            )));
        }
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(XtaskError::check(format!(
            "{} must be an ordinary directory when present",
            path.display()
        )));
    }

    let mut entries = fs::read_dir(path).map_err(|error| {
        XtaskError::check(format!("failed to read {}: {error}", path.display()))
    })?;
    entries
        .next()
        .transpose()
        .map(|entry| entry.is_some())
        .map_err(|error| XtaskError::check(format!("failed to read {}: {error}", path.display())))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let exit_code = error.exit_code();
            eprintln!("error: {error}");
            ExitCode::from(exit_code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{XtaskError, dispatch};

    #[test]
    fn missing_command_returns_usage_error() {
        // Arrange
        let args = Vec::new();

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Err(XtaskError::usage("missing command")));
    }

    #[test]
    fn unknown_command_returns_usage_error() {
        // Arrange
        let args = vec!["unknown".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Err(XtaskError::usage("unknown command `unknown`")));
    }

    #[test]
    fn help_returns_success() {
        // Arrange
        let args = vec!["--help".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_rejects_arguments() {
        // Arrange
        let args = vec!["check".to_owned(), "unexpected".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(
            result,
            Err(XtaskError::usage("check does not accept arguments"))
        );
    }
}
