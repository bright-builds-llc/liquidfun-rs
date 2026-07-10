use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::upstream;

const USAGE: &str = r"Usage: cargo xtask differential <command> [arguments]

Commands:
  compare  --scenario empty-world --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  replay   --scenario empty-world --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  minimize --scenario empty-world --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  fixture stage   --scenario empty-world --preset <preset> --session-profile <profile> --artifact-kind <reviewed-trace|minimized-regression> --artifact-id <id>
  fixture review  --artifact-id <id> --reviewer <identity> --reviewed-at <UTC timestamp> --review-status <approved|rejected>
  fixture promote --artifact-id <id>";

const ALLOWED_SCENARIOS: [&str; 1] = ["empty-world"];
const ALLOWED_PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];
const ALLOWED_PROFILES: [&str; 3] = ["one-shot", "reuse", "sanitizer"];
const ALLOWED_ARTIFACT_KINDS: [&str; 2] = ["reviewed-trace", "minimized-regression"];
const ALLOWED_REVIEW_STATUSES: [&str; 2] = ["approved", "rejected"];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DifferentialError {
    category: &'static str,
    message: String,
}

impl DifferentialError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }

    fn process(message: impl Into<String>) -> Self {
        Self::new("process", message)
    }
}

impl Display for DifferentialError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "differential/{}: {}",
            self.category, self.message
        )
    }
}

impl Error for DifferentialError {}

#[derive(Debug, PartialEq, Eq)]
struct RunnerInvocation {
    arguments: Vec<String>,
    oracle_dependent: bool,
}

pub(crate) fn run(args: &[String]) -> Result<(), DifferentialError> {
    let invocation = parse_invocation(args)?;
    let repository_root = repository_root()?;

    if invocation.oracle_dependent {
        upstream::run(&["verify".to_owned()]).map_err(|error| {
            DifferentialError::new("upstream", format!("oracle verification failed: {error}"))
        })?;
    }

    run_differential(&repository_root, &invocation.arguments)
}

pub(crate) fn check_protocol(repository_root: &Path) -> Result<(), DifferentialError> {
    for (label, arguments) in [
        (
            "protocol schema presentations",
            [
                "test",
                "--package",
                "liquidfun-test-protocol",
                "--all-features",
                "--lib",
                "schema::tests",
            ]
            .as_slice(),
        ),
        (
            "protocol fixtures",
            [
                "test",
                "--package",
                "liquidfun-test-protocol",
                "--all-features",
                "--test",
                "fixtures",
            ]
            .as_slice(),
        ),
    ] {
        let cargo = env::var_os("LIQUIDFUN_XTASK_CARGO").unwrap_or_else(|| OsString::from("cargo"));
        run_process(&cargo, arguments, repository_root, label)?;
    }
    Ok(())
}

fn parse_invocation(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing differential command"));
    };

    match command.as_str() {
        "compare" | "replay" | "minimize" => parse_scenario_command(command, command_args),
        "fixture" => parse_fixture_command(command_args),
        unknown => Err(DifferentialError::usage(format!(
            "unknown differential command `{unknown}`"
        ))),
    }
}

fn parse_scenario_command(
    command: &str,
    args: &[String],
) -> Result<RunnerInvocation, DifferentialError> {
    let options = parse_options(args)?;
    require_exact_options(&options, &["--scenario", "--preset", "--session-profile"])?;
    let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
    let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
    let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;

    Ok(RunnerInvocation {
        arguments: option_arguments(
            &[command],
            &[
                ("--scenario", scenario),
                ("--preset", preset),
                ("--session-profile", profile),
            ],
        ),
        oracle_dependent: true,
    })
}

fn parse_fixture_command(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let Some((action, action_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing fixture action"));
    };
    let options = parse_options(action_args)?;

    match action.as_str() {
        "stage" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--artifact-kind",
                    "--preset",
                    "--scenario",
                    "--session-profile",
                ],
            )?;
            let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
            let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
            let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;
            let artifact_kind =
                require_allowed(&options, "--artifact-kind", &ALLOWED_ARTIFACT_KINDS)?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "stage"],
                    &[
                        ("--scenario", scenario),
                        ("--preset", preset),
                        ("--session-profile", profile),
                        ("--artifact-kind", artifact_kind),
                        ("--artifact-id", artifact_id),
                    ],
                ),
                oracle_dependent: true,
            })
        }
        "review" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--review-status",
                    "--reviewed-at",
                    "--reviewer",
                ],
            )?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            let reviewer = required_option(&options, "--reviewer")?;
            let reviewed_at = required_option(&options, "--reviewed-at")?;
            let review_status =
                require_allowed(&options, "--review-status", &ALLOWED_REVIEW_STATUSES)?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "review"],
                    &[
                        ("--artifact-id", artifact_id),
                        ("--reviewer", reviewer),
                        ("--reviewed-at", reviewed_at),
                        ("--review-status", review_status),
                    ],
                ),
                oracle_dependent: false,
            })
        }
        "promote" => {
            require_exact_options(&options, &["--artifact-id"])?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "promote"],
                    &[("--artifact-id", artifact_id)],
                ),
                oracle_dependent: false,
            })
        }
        unknown => Err(DifferentialError::usage(format!(
            "unknown fixture action `{unknown}`"
        ))),
    }
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, DifferentialError> {
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let option = &pair[0];
        if !option.starts_with("--") {
            return Err(DifferentialError::usage(format!(
                "unexpected positional argument `{option}`"
            )));
        }
        if options.insert(option.clone(), pair[1].clone()).is_some() {
            return Err(DifferentialError::usage(format!(
                "duplicate differential option `{option}`"
            )));
        }
    }
    if !args.chunks_exact(2).remainder().is_empty() {
        return Err(DifferentialError::usage(
            "every differential option requires one value",
        ));
    }
    Ok(options)
}

fn require_exact_options(
    options: &BTreeMap<String, String>,
    expected: &[&str],
) -> Result<(), DifferentialError> {
    if options.len() == expected.len()
        && options
            .keys()
            .all(|option| expected.contains(&option.as_str()))
    {
        return Ok(());
    }
    Err(DifferentialError::usage(
        "differential command options do not match the registered command shape",
    ))
}

fn required_option<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, DifferentialError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| DifferentialError::usage(format!("missing required option `{name}`")))
}

fn require_allowed<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
    allowed: &[&str],
) -> Result<&'a str, DifferentialError> {
    let value = required_option(options, name)?;
    if allowed.contains(&value) {
        return Ok(value);
    }
    Err(DifferentialError::usage(format!(
        "unregistered value `{value}` for `{name}`; allowed values: {}",
        allowed.join(", ")
    )))
}

fn option_arguments(prefix: &[&str], options: &[(&str, &str)]) -> Vec<String> {
    prefix
        .iter()
        .copied()
        .chain(options.iter().flat_map(|(option, value)| [*option, *value]))
        .map(str::to_owned)
        .collect()
}

fn run_differential(repository_root: &Path, arguments: &[String]) -> Result<(), DifferentialError> {
    if let Some(program) = env::var_os("LIQUIDFUN_XTASK_DIFFERENTIAL") {
        return run_process(
            &program,
            arguments,
            repository_root,
            "run differential command",
        );
    }

    let cargo = env::var_os("LIQUIDFUN_XTASK_CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cargo_arguments = [
        "run",
        "--quiet",
        "--package",
        "liquidfun-differential",
        "--bin",
        "liquidfun-differential",
        "--",
    ]
    .iter()
    .map(|argument| (*argument).to_owned())
    .collect::<Vec<_>>();
    cargo_arguments.extend_from_slice(arguments);
    run_process(
        &cargo,
        &cargo_arguments,
        repository_root,
        "run differential command",
    )
}

fn run_process<S: AsRef<std::ffi::OsStr>>(
    program: &std::ffi::OsStr,
    arguments: &[S],
    repository_root: &Path,
    operation: &str,
) -> Result<(), DifferentialError> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(repository_root)
        .status()
        .map_err(|error| {
            DifferentialError::process(format!(
                "failed to start `{}` while attempting to {operation}: {error}",
                program.to_string_lossy()
            ))
        })?;
    if status.success() {
        return Ok(());
    }

    let status = status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    Err(DifferentialError::process(format!(
        "`{}` failed while attempting to {operation} (status {status})",
        program.to_string_lossy()
    )))
}

fn repository_root() -> Result<PathBuf, DifferentialError> {
    let current_dir = env::current_dir().map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("reference/upstream-lock.toml").is_file()
            && candidate.join(".gitmodules").is_file()
    }) else {
        return Err(DifferentialError::new(
            "repository",
            "could not find reference/upstream-lock.toml and .gitmodules",
        ));
    };
    Ok(root.to_path_buf())
}
