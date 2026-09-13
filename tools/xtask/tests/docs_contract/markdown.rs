use std::{collections::BTreeSet, env, fs, os::unix::fs::PermissionsExt, process::Command};

use super::{FIXTURE_ID, Ordering, PathBuf, TestResult, workspace_root};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(formatter: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!("liquidfun-markdown-{}-{id}", std::process::id()));
        fs::create_dir(&root)?;
        let fixture = Self { root };
        fs::create_dir(fixture.root.join("bin"))?;
        fixture.executable("mdformat", formatter)?;
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&fixture.root)
                .status()?
                .success()
        );
        Ok(fixture)
    }

    fn executable(&self, name: &str, contents: &str) -> std::io::Result<()> {
        let file = self.root.join("bin").join(name);
        fs::write(&file, contents)?;
        fs::set_permissions(file, fs::Permissions::from_mode(0o755))
    }

    fn run(&self) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let mut paths = vec![self.root.join("bin")];
        paths.extend(env::split_paths(
            &env::var_os("PATH").ok_or("PATH is required")?,
        ));
        Ok(Command::new("bash")
            .arg(workspace_root().join("scripts/markdown-check.sh"))
            .current_dir(&self.root)
            .env("PATH", env::join_paths(paths)?)
            .output()?)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("owned markdown fixture should clean up");
    }
}

#[test]
fn checks_tracked_and_untracked_markdown_without_ignored_cache_traversal() -> TestResult {
    // Arrange
    let fixture =
        Fixture::new("#!/usr/bin/env bash\nprintf '%s\\0' \"$@\" > formatter-arguments.bin\n")?;
    fs::write(fixture.root.join("tracked.md"), "# Tracked\n")?;
    assert!(
        Command::new("git")
            .args(["add", "tracked.md"])
            .current_dir(&fixture.root)
            .status()?
            .success()
    );
    for file in ["with spaces.md", "-leading.md"] {
        fs::write(fixture.root.join(file), "# New\n")?;
    }
    fs::create_dir(fixture.root.join("target"))?;
    fs::write(fixture.root.join(".gitignore"), "/target/\n")?;
    fs::write(
        fixture.root.join("target/ignored.md"),
        "not repository documentation",
    )?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(fixture.root.join("formatter-arguments.bin"))?;
    let args = bytes
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(&args[..2], [b"--check".as_slice(), b"--".as_slice()]);
    assert_eq!(
        args[2..].iter().copied().collect::<BTreeSet<_>>(),
        [
            b"tracked.md".as_slice(),
            b"with spaces.md".as_slice(),
            b"-leading.md".as_slice()
        ]
        .into()
    );
    Ok(())
}

#[test]
fn formatter_failure_is_not_masked() -> TestResult {
    // Arrange
    let fixture =
        Fixture::new("#!/usr/bin/env bash\nprintf invoked > formatter-invoked\nexit 9\n")?;
    fs::write(fixture.root.join("unformatted.md"), "# Bad\n")?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert!(!output.status.success());
    assert_eq!(
        fs::read(fixture.root.join("formatter-invoked"))?,
        b"invoked"
    );
    Ok(())
}

#[test]
fn git_failure_is_not_masked_by_successful_formatter() -> TestResult {
    // Arrange
    let fixture = Fixture::new("#!/usr/bin/env bash\nexit 0\n")?;
    fixture.executable("git", "#!/usr/bin/env bash\nexit 42\n")?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert_eq!(output.status.code(), Some(42));
    Ok(())
}
