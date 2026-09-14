//! A ready report projects the accepted source, never mutable later ledgers.

use std::{fs, path::Path, process::Command};

use super::InventoryError;

const INPUTS: [&str; 9] = [
    "reference/upstream-lock.toml",
    "reference/compatibility.json",
    "reference/discovery.json",
    "reference/upstream-corpus.json",
    "reference/artifacts/manifest.toml",
    "reference/platform/support.json",
    "reference/performance/manifest.toml",
    "reference/coverage/contract.json",
    "reference/regressions/manifest.toml",
];
const MAXIMUM_BYTES: u64 = 4 * 1024 * 1024;

pub(super) enum Projection {
    NotReady,
    Attested {
        candidate: String,
        attestation: String,
    },
}

impl Projection {
    pub(super) fn validate(
        root: &Path,
        maybe_attestation: Option<&str>,
    ) -> Result<Self, InventoryError> {
        let Some(attestation) = maybe_attestation else {
            return Ok(Self::NotReady);
        };
        let candidate = crate::release::attestation::validate_committed(root, attestation)
            .map_err(|error| InventoryError::new("attestation", error.to_string()))?;
        for relative in INPUTS {
            require_source_bytes(root, &candidate, relative)?;
        }
        Ok(Self::Attested {
            candidate,
            attestation: attestation.to_owned(),
        })
    }
}

fn require_source_bytes(
    root: &Path,
    candidate: &str,
    relative: &str,
) -> Result<(), InventoryError> {
    let object = format!("{candidate}:{relative}");
    let size = git(root, &["cat-file", "-s", &object])?;
    let size = std::str::from_utf8(&size)
        .ok()
        .and_then(|size| size.trim().parse::<u64>().ok())
        .filter(|size| *size <= MAXIMUM_BYTES)
        .ok_or_else(|| {
            InventoryError::new("source-input", "source input exceeds its byte bound")
        })?;
    let mut path = root.to_path_buf();
    for component in Path::new(relative).components() {
        path.push(component);
        if path.is_symlink() {
            return Err(InventoryError::new(
                "source-input",
                "source input contains a symbolic link",
            ));
        }
    }
    let metadata = fs::metadata(&path)
        .map_err(|error| InventoryError::new("source-input", error.to_string()))?;
    if !metadata.is_file() || metadata.len() != size {
        return Err(InventoryError::new(
            "source-input",
            format!("{relative} differs from accepted C"),
        ));
    }
    let current =
        fs::read(path).map_err(|error| InventoryError::new("source-input", error.to_string()))?;
    if current != git(root, &["cat-file", "blob", &object])? {
        return Err(InventoryError::new(
            "source-input",
            format!("{relative} differs from accepted C"),
        ));
    }
    Ok(())
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, InventoryError> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|error| InventoryError::new("source-input", error.to_string()))?;
    if !output.status.success() {
        return Err(InventoryError::new(
            "source-input",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(output.stdout)
}
