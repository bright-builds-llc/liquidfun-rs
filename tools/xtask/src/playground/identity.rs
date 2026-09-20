use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use super::PlaygroundError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HostIdentity {
    pub(super) git_head: String,
    pub(super) os: String,
    pub(super) arch: String,
    pub(super) cpu_brand: String,
    pub(super) logical_cores: usize,
}

pub(super) fn host_identity(root: &Path) -> HostIdentity {
    HostIdentity {
        git_head: git_head(root),
        os: String::from(env::consts::OS),
        arch: String::from(env::consts::ARCH),
        cpu_brand: cpu_brand(),
        logical_cores: thread::available_parallelism().map_or(1, usize::from),
    }
}

pub(super) fn repository_root() -> Result<PathBuf, PlaygroundError> {
    let current_dir = env::current_dir().map_err(|error| {
        PlaygroundError::new(
            "repository",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/liquidfun/Cargo.toml").is_file()
    }) else {
        return Err(PlaygroundError::new(
            "repository",
            "could not find the liquidfun Cargo workspace",
        ));
    };
    Ok(root.to_path_buf())
}

fn git_head(root: &Path) -> String {
    let git_program = env::var_os("LIQUIDFUN_XTASK_GIT").unwrap_or_else(|| OsString::from("git"));
    let output = Command::new(git_program)
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output();
    let Ok(output) = output else {
        return String::from("unknown");
    };
    if !output.status.success() {
        return String::from("unknown");
    }
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|text| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
        .unwrap_or_else(|| String::from("unknown"))
}

fn cpu_brand() -> String {
    if cfg!(target_os = "macos") {
        return first_command_line("sysctl", &["-n", "machdep.cpu.brand_string"]);
    }
    if cfg!(target_os = "linux") {
        return linux_cpu_brand();
    }
    String::from("unknown")
}

fn first_command_line(program: &str, args: &[&str]) -> String {
    let output = Command::new(program).args(args).output();
    let Ok(output) = output else {
        return String::from("unknown");
    };
    if !output.status.success() {
        return String::from("unknown");
    }
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|text| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
        .unwrap_or_else(|| String::from("unknown"))
}

fn linux_cpu_brand() -> String {
    let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") else {
        return String::from("unknown");
    };
    for line in contents.lines() {
        let Some(value) = line.strip_prefix("model name") else {
            continue;
        };
        let Some(brand) = value.split(':').nth(1) else {
            continue;
        };
        let trimmed = brand.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    String::from("unknown")
}
