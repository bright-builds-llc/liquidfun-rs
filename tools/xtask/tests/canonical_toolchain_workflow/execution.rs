use super::{INSTALLER, TestResult, UPSTREAM_SHA256, root};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let directory = env::temp_dir().join(format!(
            "canonical-installer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let fixture = Self { directory };
        fs::create_dir(fixture.directory.join("bin"))?;
        // Substitute only the fixture digest and installation prefix. Production
        // has no environment override for either trust input; SHA validation is real.
        let payload = b"#!/bin/bash\nexit 0\n";
        fs::write(fixture.directory.join("payload.sh"), payload)?;
        let source = fs::read_to_string(root().join(INSTALLER))?
            .replace(UPSTREAM_SHA256, &format!("{:x}", Sha256::digest(payload)))
            .replace(
                "/usr/lib/llvm-22/bin",
                &fixture.directory.join("bin").to_string_lossy(),
            );
        fs::write(fixture.directory.join("installer.sh"), source)?;
        for tool in [
            "curl",
            "sudo",
            "git",
            "uname",
            "clang",
            "clang++",
            "clang-22",
            "clang++-22",
            "llvm-cov",
            "llvm-profdata",
            "llvm-cov-22",
            "llvm-profdata-22",
        ] {
            let destination = fixture.directory.join("bin").join(tool);
            fs::copy(
                root().join("tools/xtask/tests/canonical_toolchain_workflow/fake-tool.sh"),
                &destination,
            )?;
            fs::set_permissions(destination, fs::Permissions::from_mode(0o755))?;
        }
        Ok(fixture)
    }

    fn run(&self, overrides: &[(&str, &str)]) -> Result<Output, Box<dyn std::error::Error>> {
        let mut paths = vec![self.directory.join("bin")];
        paths.extend(env::split_paths(
            &env::var_os("PATH").ok_or("PATH required")?,
        ));
        let stdout = fs::File::create(self.directory.join("stdout"))?;
        let stderr = fs::File::create(self.directory.join("stderr"))?;
        let mut child = Command::new("bash")
            .arg(self.directory.join("installer.sh"))
            .current_dir(&self.directory)
            .env_clear()
            .env("PATH", env::join_paths(paths)?)
            .env("FIXTURE_ROOT", &self.directory)
            .env("GITHUB_PATH", self.directory.join("github-path"))
            .envs(overrides.iter().copied())
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?;
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() > Duration::from_secs(15) {
                child.kill()?;
                child.wait()?;
                return Err("installer test exceeded 15 seconds".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        Ok(Output {
            status,
            stdout: fs::read(self.directory.join("stdout"))?,
            stderr: fs::read(self.directory.join("stderr"))?,
        })
    }

    fn identities(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let attempts = self.directory.join("target/canonical-clang");
        if !attempts.exists() {
            return Ok(Vec::new());
        }
        Ok(fs::read_dir(attempts)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry| entry.path().join("identity.json"))
            .filter(|file| file.exists())
            .collect())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).expect("owned fixture cleanup");
    }
}

#[test]
fn valid_install_publishes_verified_identity_last() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    // Act
    let output = fixture.run(&[])?;
    // Assert
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let identities = fixture.identities()?;
    assert_eq!(identities.len(), 1);
    let identity: serde_json::Value = serde_json::from_slice(&fs::read(&identities[0])?)?;
    assert_eq!(identity["clang"], "22.1.8");
    assert_eq!(
        identity["candidate_sha"],
        "1111111111111111111111111111111111111111"
    );
    assert_eq!(identity["target"], "x86_64-pc-linux-gnu");
    assert!(fixture.directory.join("github-path").is_file());
    Ok(())
}

#[test]
fn substituted_download_never_executes_installer() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    // Act
    let output = fixture.run(&[("FAILURE", "checksum")])?;
    // Assert
    assert!(!output.status.success());
    assert!(!fs::read_to_string(fixture.directory.join("calls.log"))?.contains("sudo "));
    assert!(fixture.identities()?.is_empty());
    Ok(())
}

#[test]
fn failed_commands_and_wrong_tool_identities_never_publish_success() -> TestResult {
    for failure in [
        ("FAILURE", "download"),
        ("FAILURE", "install"),
        ("FAILURE", "compile"),
        ("FAKE_VERSION", "22.1.80"),
        ("FAKE_LLVM_VERSION", "22.1.9"),
        ("FAKE_TARGET", "aarch64-unknown-linux-gnu"),
    ] {
        // Arrange
        let fixture = Fixture::new()?;
        // Act
        let output = fixture.run(&[failure])?;
        // Assert
        assert!(!output.status.success(), "{failure:?} passed");
        assert!(fixture.identities()?.is_empty());
        assert!(!fixture.directory.join("github-path").exists());
    }
    Ok(())
}

#[test]
fn missing_matching_tool_is_terminal() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    fs::remove_file(fixture.directory.join("bin/llvm-profdata"))?;
    // Act
    let output = fixture.run(&[])?;
    // Assert
    assert!(!output.status.success());
    assert!(fixture.identities()?.is_empty());
    Ok(())
}

#[test]
fn symlinked_attempt_root_is_rejected() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    fs::create_dir(fixture.directory.join("outside"))?;
    std::os::unix::fs::symlink(
        fixture.directory.join("outside"),
        fixture.directory.join("target"),
    )?;
    // Act
    let output = fixture.run(&[])?;
    // Assert
    assert!(!output.status.success());
    assert_eq!(fs::read_dir(fixture.directory.join("outside"))?.count(), 0);
    Ok(())
}
