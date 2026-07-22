use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};

use sha2::{Digest, Sha256};

pub(super) fn exact_directory_files(
    directory: &Path,
    expected: &BTreeSet<String>,
) -> Result<(), String> {
    let actual = fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .map(|entry| {
            entry
                .map_err(|error| error.to_string())?
                .file_name()
                .into_string()
                .map_err(|_| "non-UTF-8 artifact path".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if &actual != expected {
        return Err("tracked artifact directory contains unknown or missing files".to_owned());
    }
    Ok(())
}

pub(super) fn verify_digest(root: &Path, path: &str, expected: &str) -> Result<(), String> {
    let actual = sha256(&read_regular(root, path)?);
    if actual != expected {
        return Err("tracked artifact digest is stale".to_owned());
    }
    Ok(())
}

pub(super) fn read_regular(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("artifact path is not confined".to_owned());
    }
    let path = root.join(path);
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("artifact path is not a regular file".to_owned());
    }
    fs::read(path).map_err(|error| error.to_string())
}

pub(super) fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = T::deserialize(&mut deserializer).map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())?;
    Ok(value)
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
