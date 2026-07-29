use super::{
    BTreeSet, BundleError, BundleErrorKind, Component, OpenOptions, Path, Serialize, Write, fs,
};

pub(super) fn reject_existing_or_linked_root(root: &Path) -> Result<(), BundleError> {
    if root.exists() {
        return Err(BundleError::new(
            BundleErrorKind::Write,
            "staging root must not already exist",
        ));
    }
    let Some(parent) = root.parent() else {
        return Err(BundleError::new(
            BundleErrorKind::Path,
            "staging root must have a parent",
        ));
    };
    reject_symlink(parent)
}

pub(super) fn write_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), BundleError> {
    let path = root.join(relative);
    let parent = path
        .parent()
        .ok_or_else(|| BundleError::new(BundleErrorKind::Path, "bundle file must have a parent"))?;
    create_directories(root, parent)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            BundleError::new(
                BundleErrorKind::Write,
                format!("failed to create staged file: {error}"),
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Write,
            format!("failed to write staged file: {error}"),
        )
    })
}

pub(super) fn create_directories(root: &Path, target: &Path) -> Result<(), BundleError> {
    let relative = target.strip_prefix(root).map_err(|_error| {
        BundleError::new(
            BundleErrorKind::Path,
            "bundle directory escaped staging root",
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(BundleError::new(
                BundleErrorKind::Path,
                "bundle directory has an unsafe component",
            ));
        };
        current.push(part);
        if current.exists() {
            reject_symlink(&current)?;
        } else {
            fs::create_dir(&current).map_err(|error| {
                BundleError::new(
                    BundleErrorKind::Write,
                    format!("failed to create bundle directory: {error}"),
                )
            })?;
        }
    }
    Ok(())
}

pub(super) fn collect_regular_files(root: &Path) -> Result<BTreeSet<String>, BundleError> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        reject_symlink(&directory)?;
        for entry in fs::read_dir(&directory).map_err(|error| {
            BundleError::new(
                BundleErrorKind::FileSet,
                format!("failed to enumerate bundle: {error}"),
            )
        })? {
            let entry = entry.map_err(|error| {
                BundleError::new(
                    BundleErrorKind::FileSet,
                    format!("failed to enumerate bundle: {error}"),
                )
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                BundleError::new(
                    BundleErrorKind::FileSet,
                    format!("failed to inspect bundle entry: {error}"),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(BundleError::new(
                    BundleErrorKind::Symlink,
                    "bundle contains a symbolic link",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_error| {
                    BundleError::new(BundleErrorKind::Path, "bundle entry escaped root")
                })?;
                paths.insert(path_text(relative)?);
            } else {
                return Err(BundleError::new(
                    BundleErrorKind::FileSet,
                    "bundle contains a non-regular entry",
                ));
            }
        }
    }
    Ok(paths)
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), BundleError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        BundleError::new(
            BundleErrorKind::FileSet,
            format!("failed to inspect bundle path: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(BundleError::new(
            BundleErrorKind::Symlink,
            "bundle path contains a symbolic link",
        ));
    }
    Ok(())
}

pub(super) fn path_text(path: &Path) -> Result<String, BundleError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| BundleError::new(BundleErrorKind::Path, "bundle path is not valid UTF-8"))
}

pub(super) fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, BundleError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Schema,
            format!("failed to encode bundle manifest: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}
