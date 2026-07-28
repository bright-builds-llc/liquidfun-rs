use super::{
    AcceptanceError, AcceptanceErrorKind, AcceptanceStep, Command, HeadSnapshot, Output, Path, env,
    validate_head_snapshot, validate_relative_path,
};

pub(super) struct CommandSpec {
    program: &'static str,
    args: &'static [&'static str],
}

impl CommandSpec {
    fn display(&self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(super) fn effect_steps() -> Vec<(AcceptanceStep, Vec<CommandSpec>)> {
    vec![
        (
            AcceptanceStep::Provenance,
            vec![
                xtask(&[
                    "phase13",
                    "evidence",
                    "check",
                    "--tracked",
                    "--require-reviewed",
                ]),
                xtask(&["provenance", "check"]),
                xtask(&["inventory", "check"]),
            ],
        ),
        (
            AcceptanceStep::ReviewedReplay,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--all-features",
                "--test",
                "catalog_regressions",
                "tracked_catalog_regressions_replay_byte_identically_without_writes",
            ])],
        ),
        (
            AcceptanceStep::Diagnosis,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--all-features",
                "--test",
                "catalog_regressions",
                "diagnosis",
            ])],
        ),
        (
            AcceptanceStep::Regression,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--all-features",
                "--test",
                "catalog_regressions",
                "tracked_catalog_regressions_replay_byte_identically_without_writes",
            ])],
        ),
        (
            AcceptanceStep::OracleBuild,
            vec![
                xtask(&["upstream", "verify"]),
                xtask(&["upstream", "configure", "--preset", "oracle-debug"]),
                xtask(&["upstream", "build", "--preset", "oracle-debug"]),
            ],
        ),
        (
            AcceptanceStep::LiveReplay,
            vec![xtask(&[
                "phase13",
                "evidence",
                "live-check",
                "--tracked",
                "--require-reviewed",
            ])],
        ),
    ]
}

pub(super) fn command_evidence(commands: &[CommandSpec]) -> String {
    commands
        .iter()
        .map(CommandSpec::display)
        .collect::<Vec<_>>()
        .join(" && ")
}

pub(super) fn required_command_evidence() -> Vec<(AcceptanceStep, String)> {
    effect_steps()
        .into_iter()
        .map(|(step, commands)| (step, command_evidence(&commands)))
        .collect()
}

const fn xtask(args: &'static [&'static str]) -> CommandSpec {
    CommandSpec {
        program: "xtask",
        args,
    }
}

const fn cargo(args: &'static [&'static str]) -> CommandSpec {
    CommandSpec {
        program: "cargo",
        args,
    }
}

pub(super) fn run_command(
    repository_root: &Path,
    spec: &CommandSpec,
) -> Result<(), AcceptanceError> {
    let mut command = if spec.program == "xtask" {
        let executable = env::current_exe().map_err(AcceptanceError::from)?;
        Command::new(executable)
    } else {
        Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
    };
    successful_output(
        command.current_dir(repository_root).args(spec.args),
        &spec.display(),
    )
    .map(|_output| ())
}

pub(super) fn assert_head(
    repository_root: &Path,
    expected_sha: &str,
) -> Result<(), AcceptanceError> {
    let observed_sha = git_text(repository_root, &["rev-parse", "HEAD"])?;
    let status = git_text(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )?;
    validate_head_snapshot(&HeadSnapshot {
        expected_sha: expected_sha.to_owned(),
        observed_sha,
        clean: status.is_empty(),
    })
}

pub(super) fn is_ancestor(
    repository_root: &Path,
    older: &str,
    newer: &str,
) -> Result<bool, AcceptanceError> {
    let output = run_process(
        Command::new("git").arg("-C").arg(repository_root).args([
            "merge-base",
            "--is-ancestor",
            older,
            newer,
        ]),
        "check Git ancestry",
    )?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(process_failure("check Git ancestry", &output)),
    }
}

pub(super) fn git_file_exists(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<bool, AcceptanceError> {
    validate_relative_path(path)?;
    let object = format!("{revision}:{path}");
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["cat-file", "-e", &object]),
        "inspect closure input",
    )?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1 | 128) => Ok(false),
        _ => Err(process_failure("inspect closure input", &output)),
    }
}

pub(super) fn git_file(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Vec<u8>, AcceptanceError> {
    validate_relative_path(path)?;
    let object = format!("{revision}:{path}");
    successful_output(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["show", &object]),
        "read closure input",
    )
    .map(|output| output.stdout)
}

pub(super) fn git_text(repository_root: &Path, args: &[&str]) -> Result<String, AcceptanceError> {
    let output = successful_output(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(args),
        "query Git",
    )?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| {
            AcceptanceError::new(
                AcceptanceErrorKind::Identity,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

pub(super) fn run_process(command: &mut Command, action: &str) -> Result<Output, AcceptanceError> {
    command.output().map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Process,
            format!("failed to {action}: {error}"),
        )
    })
}

pub(super) fn successful_output(
    command: &mut Command,
    action: &str,
) -> Result<Output, AcceptanceError> {
    let output = run_process(command, action)?;
    if output.status.success() {
        return Ok(output);
    }
    Err(process_failure(action, &output))
}

fn process_failure(action: &str, output: &Output) -> AcceptanceError {
    AcceptanceError::new(
        AcceptanceErrorKind::Process,
        format!(
            "`{action}` failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    )
}
