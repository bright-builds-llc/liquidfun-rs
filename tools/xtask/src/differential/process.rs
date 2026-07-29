#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn run_differential(
    repository_root: &Path,
    arguments: &[String],
) -> Result<(), DifferentialError> {
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

pub(super) fn run_process<S: AsRef<std::ffi::OsStr>>(
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

    let exit_code = status.code().and_then(|code| u8::try_from(code).ok());
    let status = status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    let message = format!(
        "`{}` failed while attempting to {operation} (status {status})",
        program.to_string_lossy()
    );
    match exit_code {
        Some(code) => Err(DifferentialError::process_exit(message, code)),
        None => Err(DifferentialError::process(message)),
    }
}
