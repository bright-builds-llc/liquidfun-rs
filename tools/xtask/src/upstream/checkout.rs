use super::{
    Component, OsStr, OsString, Path, PathBuf, ProcessText, UpstreamError, UpstreamLock, fs,
    print_tool_identities, run_text_command, tool_program,
};

pub(super) fn verify(repository_root: &Path) -> Result<UpstreamLock, UpstreamError> {
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

pub(super) fn read_upstream_lock(repository_root: &Path) -> Result<UpstreamLock, UpstreamError> {
    let lock_path = repository_root.join("reference/upstream-lock.toml");
    let contents = fs::read_to_string(&lock_path).map_err(|error| {
        UpstreamError::new(
            "lock",
            format!("failed to read {}: {error}", lock_path.display()),
        )
    })?;

    parse_upstream_lock(&contents)
}

pub(super) fn parse_upstream_lock(contents: &str) -> Result<UpstreamLock, UpstreamError> {
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

pub(super) fn set_lock_field(
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

pub(super) fn required_lock_field(
    maybe_field: Option<String>,
    key: &str,
) -> Result<String, UpstreamError> {
    maybe_field.ok_or_else(|| UpstreamError::new("lock", format!("missing `{key}` entry")))
}

pub(super) fn verify_gitmodules(
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

pub(super) fn verify_submodule(
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

pub(super) fn run_git_in_submodule(
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

pub(super) fn parse_gitlink_revision(output: &str) -> Result<&str, UpstreamError> {
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

pub(super) fn verify_identity(
    field: &str,
    expected: &str,
    actual: &str,
) -> Result<(), UpstreamError> {
    if actual == expected {
        return Ok(());
    }

    Err(UpstreamError::identity(field, expected, actual))
}
