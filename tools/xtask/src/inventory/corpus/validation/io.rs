//! Bounded authority-file reads for corpus closure validation.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::super::InventoryError;
use super::super::corpus::MAX_CORPUS_BYTES;
use super::closure_error;

pub(super) fn read_json_bounded<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &str,
) -> Result<T, InventoryError> {
    let bytes = read_bounded(path, label)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        closure_error(
            "schema",
            format!("invalid {label} in {}: {error}", path.display()),
        )
    })
}

pub(super) fn read_bounded(path: &Path, label: &str) -> Result<Vec<u8>, InventoryError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        closure_error(
            "filesystem",
            format!("failed to inspect {}: {error}", path.display()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(closure_error(
            "filesystem",
            format!("{label} must be a regular non-symlink file"),
        ));
    }
    if metadata.len() > MAX_CORPUS_BYTES as u64 {
        return Err(closure_error(
            "input-limit",
            format!("{label} exceeds the reviewed byte limit"),
        ));
    }
    fs::read(path).map_err(|error| {
        closure_error(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })
}
