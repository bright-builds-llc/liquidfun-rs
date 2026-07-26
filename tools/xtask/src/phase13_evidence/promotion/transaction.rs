//! Bounded multi-file replacement with complete rollback on failure.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{PromotionError, PromotionErrorKind};

pub(super) fn replace_all(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
    maybe_fail_after: Option<usize>,
) -> Result<(), PromotionError> {
    let originals = paths
        .iter()
        .map(|path| {
            let target = repository_root.join(path);
            if target.is_file() {
                fs::read(&target).map(Some)
            } else {
                Ok(None)
            }
            .map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Transaction,
                    format!("failed to capture pre-promotion bytes for `{path}`: {error}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let result = replace_forward(repository_root, staging_root, paths, maybe_fail_after);
    if result.is_ok() {
        return Ok(());
    }
    rollback(repository_root, paths, &originals)?;
    result
}

fn replace_forward(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
    maybe_fail_after: Option<usize>,
) -> Result<(), PromotionError> {
    for (index, path) in paths.iter().enumerate() {
        if maybe_fail_after == Some(index) {
            return Err(PromotionError::new(
                PromotionErrorKind::Transaction,
                "injected partial replacement failure",
            ));
        }
        let bytes = fs::read(staging_root.join(path)).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("failed to read staged replacement `{path}`: {error}"),
            )
        })?;
        replace_one(repository_root, path, &bytes, index)?;
    }
    Ok(())
}

fn rollback(
    repository_root: &Path,
    paths: &[&str],
    originals: &[Option<Vec<u8>>],
) -> Result<(), PromotionError> {
    for (index, (path, maybe_bytes)) in paths.iter().zip(originals).enumerate() {
        if let Some(bytes) = maybe_bytes {
            replace_one(repository_root, path, bytes, paths.len() + index)?;
            continue;
        }
        let target = repository_root.join(path);
        if target.exists() {
            fs::remove_file(&target).map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Transaction,
                    format!("failed to remove newly promoted `{path}` during rollback: {error}"),
                )
            })?;
        }
    }
    Ok(())
}

fn replace_one(
    repository_root: &Path,
    relative: &str,
    bytes: &[u8],
    ordinal: usize,
) -> Result<(), PromotionError> {
    let target = repository_root.join(relative);
    let parent = target.parent().ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Path,
            format!("replacement `{relative}` has no parent"),
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Transaction,
            format!("failed to create parent for `{relative}`: {error}"),
        )
    })?;
    let temporary = temporary_sibling(&target, ordinal)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("failed to create temporary replacement for `{relative}`: {error}"),
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Transaction,
            format!("failed to write temporary replacement for `{relative}`: {error}"),
        )
    })?;
    file.sync_all().map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Transaction,
            format!("failed to sync temporary replacement for `{relative}`: {error}"),
        )
    })?;
    fs::rename(&temporary, &target).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Transaction,
            format!("failed to install replacement `{relative}`: {error}"),
        )
    })
}

fn temporary_sibling(target: &Path, ordinal: usize) -> Result<PathBuf, PromotionError> {
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Path,
                "replacement filename is not valid UTF-8",
            )
        })?;
    Ok(target.with_file_name(format!(
        ".{file_name}.phase13-{}-{ordinal}",
        std::process::id()
    )))
}
