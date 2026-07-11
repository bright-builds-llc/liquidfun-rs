use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use sha2::{Digest, Sha256};

const USAGE: &str = r"Usage: cargo xtask upstream <command> [arguments]

Commands:
  verify
  configure --preset <oracle-debug|oracle-release|oracle-asan-ubsan>
  build --preset <oracle-debug|oracle-release|oracle-asan-ubsan>";

const ALLOWED_PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];
const ADAPTER_INPUT_MANIFEST: &str = "tools/reference/adapter-inputs.txt";
const CMAKE_CANONICAL: Version = Version::new(4, 3, 3);
const CMAKE_FLOOR: Version = Version::new(3, 25, 0);
const NINJA_CANONICAL: Version = Version::new(1, 13, 2);
const NINJA_FLOOR: Version = Version::new(1, 11, 0);
const CLANG_CANONICAL: Version = Version::new(22, 1, 8);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UpstreamError {
    category: &'static str,
    message: String,
}

impl UpstreamError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }

    fn identity(field: &str, expected: &str, actual: &str) -> Self {
        Self::new(
            "identity",
            format!("{field} mismatch: expected `{expected}`, actual `{actual}`"),
        )
    }
}

impl Display for UpstreamError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "upstream/{}: {}", self.category, self.message)
    }
}

impl Error for UpstreamError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl Display for Version {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
struct UpstreamLock {
    repository: String,
    revision: String,
    submodule_path: PathBuf,
}

#[derive(Debug)]
struct ProcessText {
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
struct ToolIdentity {
    name: &'static str,
    first_line: String,
    version: Version,
}

pub(crate) fn run(args: &[String]) -> Result<(), UpstreamError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(UpstreamError::usage("missing upstream command"));
    };

    let repository_root = repository_root()?;

    match command.as_str() {
        "verify" => {
            require_no_arguments(command_args, "verify")?;
            verify(&repository_root).map(|_| ())
        }
        "configure" => {
            let preset = parse_preset(command_args)?;
            let upstream_lock = verify(&repository_root)?;
            let adapter_digest = adapter_source_digest(&repository_root)?;
            let expected_revision = format!(
                "-DLIQUIDFUN_EXPECTED_ORACLE_REVISION={}",
                upstream_lock.revision
            );
            let expected_adapter_digest =
                format!("-DLIQUIDFUN_EXPECTED_ADAPTER_SHA256={adapter_digest}");
            run_cmake(
                &repository_root,
                &[
                    OsString::from("--preset"),
                    OsString::from(preset),
                    OsString::from(expected_revision),
                    OsString::from(expected_adapter_digest),
                ],
                "configure",
            )
        }
        "build" => {
            let preset = parse_preset(command_args)?;
            verify(&repository_root)?;
            run_cmake(
                &repository_root,
                &[
                    OsString::from("--build"),
                    OsString::from("--preset"),
                    OsString::from(preset),
                    OsString::from("--target"),
                    OsString::from("liquidfun-reference"),
                ],
                "build",
            )
        }
        unknown => Err(UpstreamError::usage(format!(
            "unknown upstream command `{unknown}`"
        ))),
    }
}

fn require_no_arguments(args: &[String], command: &str) -> Result<(), UpstreamError> {
    if args.is_empty() {
        return Ok(());
    }

    Err(UpstreamError::usage(format!(
        "upstream {command} does not accept arguments"
    )))
}

fn parse_preset(args: &[String]) -> Result<&str, UpstreamError> {
    let [flag, preset] = args else {
        return Err(UpstreamError::usage(
            "expected `--preset <name>` after the upstream command",
        ));
    };

    if flag != "--preset" {
        return Err(UpstreamError::usage(format!(
            "unknown upstream option `{flag}`; expected `--preset`"
        )));
    }

    if ALLOWED_PRESETS.contains(&preset.as_str()) {
        return Ok(preset);
    }

    Err(UpstreamError::new(
        "preset",
        format!(
            "unknown preset `{preset}`; allowed presets: {}",
            ALLOWED_PRESETS.join(", ")
        ),
    ))
}

fn repository_root() -> Result<PathBuf, UpstreamError> {
    let current_dir = env::current_dir().map_err(|error| {
        UpstreamError::new(
            "filesystem",
            format!("failed to read the current directory: {error}"),
        )
    })?;
    let maybe_root = current_dir.ancestors().find(|candidate| {
        candidate.join("reference/upstream-lock.toml").is_file()
            && candidate.join(".gitmodules").is_file()
    });
    let Some(root) = maybe_root else {
        return Err(UpstreamError::new(
            "repository",
            "could not find a repository root containing reference/upstream-lock.toml and .gitmodules",
        ));
    };

    Ok(root.to_path_buf())
}

fn verify(repository_root: &Path) -> Result<UpstreamLock, UpstreamError> {
    let upstream_lock = read_upstream_lock(repository_root)?;
    verify_gitmodules(repository_root, &upstream_lock)?;
    verify_submodule(repository_root, &upstream_lock)?;
    print_tool_identities()?;

    println!(
        "upstream verified: {} at {}",
        upstream_lock.submodule_path.display(),
        upstream_lock.revision
    );
    Ok(upstream_lock)
}

pub(crate) fn adapter_source_digest(repository_root: &Path) -> Result<String, UpstreamError> {
    let mut digest_input = Sha256::new();
    for relative_path in adapter_input_paths(repository_root)? {
        let path = repository_root.join(&relative_path);
        let bytes = fs::read(&path).map_err(|error| {
            UpstreamError::new(
                "adapter-digest",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        let source_digest = Sha256::digest(bytes);
        digest_input.update(relative_path.as_bytes());
        digest_input.update(b"=");
        digest_input.update(format!("{source_digest:x}").as_bytes());
        digest_input.update(b"\n");
    }

    Ok(format!("{:x}", digest_input.finalize()))
}

fn adapter_input_paths(repository_root: &Path) -> Result<Vec<String>, UpstreamError> {
    let manifest_path = repository_root.join(ADAPTER_INPUT_MANIFEST);
    let contents = fs::read_to_string(&manifest_path).map_err(|error| {
        UpstreamError::new(
            "adapter-digest",
            format!("failed to read {}: {error}", manifest_path.display()),
        )
    })?;
    let mut paths = Vec::new();
    for line in contents.lines() {
        let relative_path = line.trim();
        if relative_path.is_empty() || relative_path.starts_with('#') {
            continue;
        }
        let path = Path::new(relative_path);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || paths.iter().any(|existing| existing == relative_path)
        {
            return Err(UpstreamError::new(
                "adapter-digest",
                format!("invalid adapter input `{relative_path}`"),
            ));
        }
        paths.push(relative_path.to_owned());
    }
    if paths.is_empty() {
        return Err(UpstreamError::new(
            "adapter-digest",
            "adapter input manifest is empty",
        ));
    }
    Ok(paths)
}

fn read_upstream_lock(repository_root: &Path) -> Result<UpstreamLock, UpstreamError> {
    let lock_path = repository_root.join("reference/upstream-lock.toml");
    let contents = fs::read_to_string(&lock_path).map_err(|error| {
        UpstreamError::new(
            "lock",
            format!("failed to read {}: {error}", lock_path.display()),
        )
    })?;

    parse_upstream_lock(&contents)
}

fn parse_upstream_lock(contents: &str) -> Result<UpstreamLock, UpstreamError> {
    let mut maybe_repository = None;
    let mut maybe_revision = None;
    let mut maybe_submodule_path = None;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, raw_value)) = line.split_once('=') else {
            return Err(UpstreamError::new(
                "lock",
                format!("invalid lock entry `{line}`"),
            ));
        };
        let key = key.trim();

        match key {
            "repository" => set_lock_field(&mut maybe_repository, key, raw_value)?,
            "revision" => set_lock_field(&mut maybe_revision, key, raw_value)?,
            "submodule_path" => set_lock_field(&mut maybe_submodule_path, key, raw_value)?,
            _ => {}
        }
    }

    let repository = required_lock_field(maybe_repository, "repository")?;
    let revision = required_lock_field(maybe_revision, "revision")?;
    let submodule_path =
        PathBuf::from(required_lock_field(maybe_submodule_path, "submodule_path")?);

    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(UpstreamError::new(
            "lock",
            format!("revision must be a full lowercase 40-hex commit, actual `{revision}`"),
        ));
    }

    if submodule_path.as_os_str().is_empty()
        || submodule_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(UpstreamError::new(
            "lock",
            format!(
                "submodule_path must be a non-empty relative path without traversal, actual `{}`",
                submodule_path.display()
            ),
        ));
    }

    Ok(UpstreamLock {
        repository,
        revision,
        submodule_path,
    })
}

fn set_lock_field(
    field: &mut Option<String>,
    key: &str,
    raw_value: &str,
) -> Result<(), UpstreamError> {
    if field.is_some() {
        return Err(UpstreamError::new(
            "lock",
            format!("duplicate `{key}` entry"),
        ));
    }

    let value = raw_value.trim();
    let Some(value) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(UpstreamError::new(
            "lock",
            format!("`{key}` must use a quoted string value"),
        ));
    };
    *field = Some(value.to_owned());
    Ok(())
}

fn required_lock_field(maybe_field: Option<String>, key: &str) -> Result<String, UpstreamError> {
    maybe_field.ok_or_else(|| UpstreamError::new("lock", format!("missing `{key}` entry")))
}

fn verify_gitmodules(
    repository_root: &Path,
    upstream_lock: &UpstreamLock,
) -> Result<(), UpstreamError> {
    let gitmodules_path = repository_root.join(".gitmodules");
    let contents = fs::read_to_string(&gitmodules_path).map_err(|error| {
        UpstreamError::new(
            "gitmodules",
            format!("failed to read {}: {error}", gitmodules_path.display()),
        )
    })?;
    let expected_path = upstream_lock.submodule_path.to_string_lossy();
    let mut maybe_path = None;
    let mut maybe_url = None;
    let mut in_submodule_section = false;

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if maybe_path.as_deref() == Some(expected_path.as_ref()) {
                break;
            }
            maybe_path = None;
            maybe_url = None;
            in_submodule_section = line.starts_with("[submodule ");
            continue;
        }
        if !in_submodule_section {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "path" => maybe_path = Some(value.trim().to_owned()),
            "url" => maybe_url = Some(value.trim().to_owned()),
            _ => {}
        }
    }

    if maybe_path.as_deref() != Some(expected_path.as_ref()) {
        return Err(UpstreamError::new(
            "gitmodules",
            format!("missing submodule path `{expected_path}`"),
        ));
    }
    let actual_url = maybe_url.as_deref().unwrap_or("<missing>");
    if actual_url != upstream_lock.repository {
        return Err(UpstreamError::identity(
            ".gitmodules repository URL",
            &upstream_lock.repository,
            actual_url,
        ));
    }

    Ok(())
}

fn verify_submodule(
    repository_root: &Path,
    upstream_lock: &UpstreamLock,
) -> Result<(), UpstreamError> {
    let submodule_path = repository_root.join(&upstream_lock.submodule_path);
    if !submodule_path.is_dir() {
        return Err(UpstreamError::new(
            "missing-submodule",
            format!(
                "{} is absent; initialize it with `git submodule update --init --recursive {}`",
                upstream_lock.submodule_path.display(),
                upstream_lock.submodule_path.display()
            ),
        ));
    }

    let git_program = tool_program("LIQUIDFUN_XTASK_GIT", "git");
    let gitlink = run_text_command(
        &git_program,
        &[
            OsString::from("-C"),
            repository_root.as_os_str().to_owned(),
            OsString::from("ls-tree"),
            OsString::from("HEAD"),
            OsString::from("--"),
            upstream_lock.submodule_path.as_os_str().to_owned(),
        ],
        None,
        "read parent gitlink",
    )?;
    let gitlink_revision = parse_gitlink_revision(&gitlink.stdout)?;
    verify_identity(
        "gitlink revision",
        &upstream_lock.revision,
        gitlink_revision,
    )?;

    let checkout = run_git_in_submodule(&git_program, &submodule_path, &["rev-parse", "HEAD"])?;
    verify_identity(
        "checked-out revision",
        &upstream_lock.revision,
        checkout.stdout.trim(),
    )?;

    let status = run_git_in_submodule(&git_program, &submodule_path, &["status", "--short"])?;
    if !status.stdout.trim().is_empty() {
        return Err(UpstreamError::new(
            "dirty",
            format!("upstream worktree is dirty:\n{}", status.stdout.trim_end()),
        ));
    }

    let remote = run_git_in_submodule(
        &git_program,
        &submodule_path,
        &["remote", "get-url", "origin"],
    )?;
    verify_identity(
        "upstream origin URL",
        &upstream_lock.repository,
        remote.stdout.trim(),
    )
}

fn run_git_in_submodule(
    git_program: &OsStr,
    submodule_path: &Path,
    args: &[&str],
) -> Result<ProcessText, UpstreamError> {
    let mut command_args = vec![OsString::from("-C"), submodule_path.as_os_str().to_owned()];
    command_args.extend(args.iter().map(OsString::from));
    run_text_command(
        git_program,
        &command_args,
        None,
        "inspect upstream repository",
    )
}

fn parse_gitlink_revision(output: &str) -> Result<&str, UpstreamError> {
    let mut fields = output.split_whitespace();
    let maybe_mode = fields.next();
    let maybe_kind = fields.next();
    let maybe_revision = fields.next();
    let (Some("160000"), Some("commit"), Some(revision)) = (maybe_mode, maybe_kind, maybe_revision)
    else {
        return Err(UpstreamError::new(
            "gitlink",
            format!("expected a 160000 commit entry, actual `{}`", output.trim()),
        ));
    };

    Ok(revision)
}

fn verify_identity(field: &str, expected: &str, actual: &str) -> Result<(), UpstreamError> {
    if actual == expected {
        return Ok(());
    }

    Err(UpstreamError::identity(field, expected, actual))
}

fn print_tool_identities() -> Result<(), UpstreamError> {
    let cmake = read_tool_identity("cmake", "LIQUIDFUN_XTASK_CMAKE", "cmake", &["--version"])?;
    let ninja = read_tool_identity("ninja", "LIQUIDFUN_XTASK_NINJA", "ninja", &["--version"])?;
    let compiler = read_tool_identity("c++", "LIQUIDFUN_XTASK_CXX", "c++", &["--version"])?;

    print_tool_identity(&cmake, CMAKE_CANONICAL, Some(CMAKE_FLOOR))?;
    print_tool_identity(&ninja, NINJA_CANONICAL, Some(NINJA_FLOOR))?;
    print_tool_identity(&compiler, CLANG_CANONICAL, None)
}

fn read_tool_identity(
    name: &'static str,
    env_var: &str,
    default_program: &str,
    args: &[&str],
) -> Result<ToolIdentity, UpstreamError> {
    let program = tool_program(env_var, default_program);
    let command_args: Vec<OsString> = args.iter().map(OsString::from).collect();
    let output = run_text_command(&program, &command_args, None, "read tool identity")?;
    let first_line = output
        .stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_owned();
    let version = parse_version(&first_line).ok_or_else(|| {
        UpstreamError::new(
            "tool",
            format!("could not parse {name} version from `{first_line}`"),
        )
    })?;

    Ok(ToolIdentity {
        name,
        first_line,
        version,
    })
}

fn print_tool_identity(
    identity: &ToolIdentity,
    canonical: Version,
    maybe_floor: Option<Version>,
) -> Result<(), UpstreamError> {
    println!("tool {}: {}", identity.name, identity.first_line);

    if let Some(floor) = maybe_floor
        && identity.version < floor
    {
        return Err(UpstreamError::new(
            "tool",
            format!(
                "{} {} is below the supported local floor {}",
                identity.name, identity.version, floor
            ),
        ));
    }

    if identity.version != canonical {
        eprintln!(
            "warning: {} {} differs from canonical {}",
            identity.name, identity.version, canonical
        );
    }

    Ok(())
}

fn parse_version(text: &str) -> Option<Version> {
    text.split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .find_map(|candidate| {
            let mut numbers = candidate.split('.');
            let major = numbers.next()?.parse().ok()?;
            let minor = numbers.next()?.parse().ok()?;
            let patch = numbers.next().unwrap_or("0").parse().ok()?;
            Some(Version::new(major, minor, patch))
        })
}

fn run_cmake(
    repository_root: &Path,
    args: &[OsString],
    operation: &'static str,
) -> Result<(), UpstreamError> {
    let cmake_program = tool_program("LIQUIDFUN_XTASK_CMAKE", "cmake");
    let reference_dir = repository_root.join("tools/reference");
    let output = run_text_command(&cmake_program, args, Some(&reference_dir), operation)?;

    print!("{}", output.stdout);
    eprint!("{}", output.stderr);
    Ok(())
}

fn tool_program(env_var: &str, default_program: &str) -> OsString {
    env::var_os(env_var).unwrap_or_else(|| OsString::from(default_program))
}

fn run_text_command(
    program: &OsStr,
    args: &[OsString],
    maybe_current_dir: Option<&Path>,
    operation: &'static str,
) -> Result<ProcessText, UpstreamError> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(current_dir) = maybe_current_dir {
        command.current_dir(current_dir);
    }

    let output = command.output().map_err(|error| {
        UpstreamError::new(
            "process",
            format!(
                "failed to start `{}` while attempting to {operation}: {error}",
                program.to_string_lossy()
            ),
        )
    })?;
    process_output(program, operation, &output)
}

fn process_output(
    program: &OsStr,
    operation: &'static str,
    output: &Output,
) -> Result<ProcessText, UpstreamError> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        return Ok(ProcessText { stdout, stderr });
    }

    let status = output.status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    let diagnostic = process_diagnostic(&stdout, &stderr);
    Err(UpstreamError::new(
        "process",
        format!(
            "`{}` failed while attempting to {operation} (status {status}): {diagnostic}",
            program.to_string_lossy()
        ),
    ))
}

fn process_diagnostic(stdout: &str, stderr: &str) -> String {
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "<no stdout or stderr>".to_owned(),
        (false, true) => format!("stdout:\n{stdout}"),
        (true, false) => format!("stderr:\n{stderr}"),
        (false, false) => format!("stdout:\n{stdout}\nstderr:\n{stderr}"),
    }
}
