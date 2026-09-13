use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use super::{TestResult, workflow_source, workspace_root};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!(
            "liquidfun-canonical-step-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        let fixture = Self { root };
        fs::create_dir(fixture.root.join("bin"))?;
        fs::create_dir_all(fixture.root.join("failure/logs"))?;
        let tool_source = workspace_root()
            .join("tools/xtask/tests/phase13_1_canonical_native_workflow/fake-tool.sh");
        for tool in [
            "curl",
            "sha256sum",
            "chmod",
            "sudo",
            "tar",
            "unzip",
            "rustup",
            "uname",
            "rustc",
            "cmake",
            "ninja",
            "clang++-22",
        ] {
            let destination = fixture.root.join("bin").join(tool);
            fs::copy(&tool_source, &destination)?;
            fs::set_permissions(destination, fs::Permissions::from_mode(0o755))?;
        }
        Ok(fixture)
    }

    fn run(
        &self,
        step: &str,
        overrides: &[(&str, &str)],
    ) -> Result<Output, Box<dyn std::error::Error>> {
        let source = workflow_source()?;
        let step_source = source
            .split(&format!("      - name: {step}\n"))
            .nth(1)
            .ok_or("workflow step is missing")?
            .split("      - name:")
            .next()
            .ok_or("step body is missing")?;
        let script = step_source
            .split("        run: |\n")
            .nth(1)
            .ok_or("step script is missing")?
            .lines()
            .map(|line| line.strip_prefix("          ").unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n");
        let mut paths = vec![self.root.join("bin")];
        paths.extend(env::split_paths(
            &env::var_os("PATH").ok_or("PATH is required")?,
        ));
        Ok(Command::new("bash")
            .args(["-e", "-o", "pipefail", "-c", &script])
            .current_dir(&self.root)
            .env("PATH", env::join_paths(paths)?)
            .env("RUNNER_TEMP", &self.root)
            .env("FAILURE_DIRECTORY", self.root.join("failure"))
            .env("GITHUB_PATH", self.root.join("github-path"))
            .env("CALL_LOG", self.root.join("calls.log"))
            .envs(overrides.iter().copied())
            .output()?)
    }

    fn calls(&self) -> std::io::Result<String> {
        fs::read_to_string(self.root.join("calls.log"))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("owned canonical fixture should clean up");
    }
}

#[test]
fn llvm_checksum_failure_prevents_installer_execution() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;

    // Act
    let output = fixture.run("Install canonical LLVM 22", &[("FAIL_CHECKSUM", "llvm.sh")])?;

    // Assert
    assert!(!output.status.success());
    assert!(!fixture.calls()?.contains("sudo "));
    assert!(!fixture.calls()?.contains("chmod "));
    assert!(fixture.root.join("failure/logs/install-llvm.log").is_file());
    Ok(())
}

#[test]
fn cmake_checksum_failure_prevents_archive_extraction() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;

    // Act
    let output = fixture.run(
        "Install exact CMake and Ninja",
        &[("FAIL_CHECKSUM", "cmake.tar.gz")],
    )?;

    // Assert
    assert!(!output.status.success());
    assert!(!fixture.calls()?.contains("tar "));
    assert!(
        fixture
            .root
            .join("failure/logs/install-build-tools.log")
            .is_file()
    );
    Ok(())
}

#[test]
fn ninja_checksum_failure_prevents_archive_extraction() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;

    // Act
    let output = fixture.run(
        "Install exact CMake and Ninja",
        &[("FAIL_CHECKSUM", "ninja.zip")],
    )?;

    // Assert
    assert!(!output.status.success());
    assert!(!fixture.calls()?.contains("unzip "));
    Ok(())
}

#[test]
fn rust_install_failure_prevents_override() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;

    // Act
    let output = fixture.run("Install pinned Rust", &[("FAIL_TOOL", "rustup")])?;

    // Assert
    assert!(!output.status.success());
    assert!(!fixture.calls()?.contains("rustup override"));
    Ok(())
}

#[test]
fn every_nonfinal_tool_identity_failure_is_terminal() -> TestResult {
    // Arrange
    let identities = [
        "TEST_OS",
        "TEST_ARCH",
        "TEST_RUST",
        "TEST_CMAKE",
        "TEST_NINJA",
    ];

    // Act and Assert
    for identity in identities {
        let fixture = Fixture::new()?;
        let output = fixture.run(
            "Assert the canonical tool identities",
            &[(identity, "wrong")],
        )?;
        assert!(!output.status.success(), "mismatched {identity} passed");
        assert!(!fixture.calls()?.contains("clang++-22 "));
        assert!(
            fixture
                .root
                .join("failure/logs/tool-identities.log")
                .is_file()
        );
    }
    Ok(())
}

#[test]
fn valid_preflight_steps_remain_successful() -> TestResult {
    // Arrange
    let steps = [
        "Install pinned Rust",
        "Install canonical LLVM 22",
        "Install exact CMake and Ninja",
        "Assert the canonical tool identities",
    ];

    // Act and Assert
    for step in steps {
        let fixture = Fixture::new()?;
        let output = fixture.run(step, &[])?;
        assert!(
            output.status.success(),
            "{step}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
