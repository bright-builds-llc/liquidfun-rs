//! Deterministic semantic discovery over the verified pinned upstream tree.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::{Map, Value, json};

use super::InventoryError;
use super::corpus::parse_manifest;

#[path = "discovery/source.rs"]
mod source;

use source::{contains_main, google_test_symbols, testbed_registrations, testbed_scenario_class};

const MAX_SOURCE_FILES: usize = 256;
const MAX_SOURCE_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_SOURCE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DISCOVERED_ITEMS: usize = 2_048;
const TESTBED_REGISTRY_PATH: &str = "liquidfun/Box2D/Testbed/Tests/TestEntries.cpp";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiscoveredKind {
    Example,
    TestbedEntry,
    UpstreamTest,
}

impl DiscoveredKind {
    const fn id_prefix(self) -> &'static str {
        match self {
            Self::Example => "example.",
            Self::TestbedEntry => "testbed.",
            Self::UpstreamTest => "upstream-test.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct DiscoveredSource {
    path: String,
    symbol: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DiscoveredItem {
    id: String,
    kind: DiscoveredKind,
    source: DiscoveredSource,
}

pub(crate) fn refresh(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<usize, InventoryError> {
    verify_pinned_checkout(repository_root, oracle_revision)?;
    let discovered = scan(repository_root)?;
    let snapshot_path = repository_root.join("reference/upstream-corpus.json");
    let merged = merge_reviewed_fields(&snapshot_path, oracle_revision, &discovered)?;
    atomic_write(&snapshot_path, &merged)?;
    Ok(discovered.len())
}

pub(crate) fn check_snapshot(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<usize, InventoryError> {
    let path = repository_root.join("reference/upstream-corpus.json");
    let bytes = fs::read(&path).map_err(|_| {
        InventoryError::new(
            "corpus-snapshot",
            "semantic snapshot is missing or unreadable",
        )
    })?;
    let manifest = parse_manifest(&bytes, oracle_revision)
        .map_err(|error| InventoryError::new("corpus-snapshot", error.to_string()))?;
    require_source_order(&bytes)?;
    let canonical = format_manifest(&manifest)?;
    if bytes != canonical.as_bytes() {
        return Err(InventoryError::new(
            "corpus-snapshot",
            "snapshot does not match canonical in-memory bytes; run `cargo xtask inventory corpus refresh` with the pinned checkout",
        ));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| InventoryError::new("corpus-snapshot", "snapshot schema is invalid"))?;
    value["items"]
        .as_array()
        .map(Vec::len)
        .ok_or_else(|| InventoryError::new("corpus-snapshot", "snapshot items are invalid"))
}

fn scan(repository_root: &Path) -> Result<Vec<DiscoveredItem>, InventoryError> {
    let upstream_root = repository_root.join("third_party/liquidfun");
    let unittest_root = upstream_root.join("liquidfun/Box2D/Unittests");
    let testbed_root = upstream_root.join("liquidfun/Box2D/Testbed/Tests");
    let hello_world = upstream_root.join("liquidfun/Box2D/HelloWorld/HelloWorld.cpp");
    let mut source_budget = SourceBudget::default();
    let mut items = Vec::new();

    for path in walk_files(&unittest_root, &mut source_budget)? {
        if path.extension().and_then(|value| value.to_str()) != Some("cpp")
            || !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with("Tests.cpp"))
        {
            continue;
        }
        let relative = upstream_path(&upstream_root, &path)?;
        let contents = read_source(&path, &mut source_budget)?;
        let symbols = google_test_symbols(&contents)?;
        if symbols.is_empty() {
            return Err(source_error(
                "allowlisted test source has no test declarations",
            ));
        }
        for symbol in symbols {
            items.push(discovered_item(
                DiscoveredKind::UpstreamTest,
                relative.clone(),
                symbol,
            )?);
        }
    }

    let registry_path = testbed_root.join("TestEntries.cpp");
    let registry_contents = read_source(&registry_path, &mut source_budget)?;
    for (title, factory) in testbed_registrations(&registry_contents)? {
        items.push(discovered_item(
            DiscoveredKind::TestbedEntry,
            TESTBED_REGISTRY_PATH.to_owned(),
            format!("{title}|{factory}::Create"),
        )?);
    }

    for path in walk_files(&testbed_root, &mut source_budget)? {
        if path.extension().and_then(|value| value.to_str()) != Some("h")
            || path.parent() != Some(testbed_root.as_path())
        {
            continue;
        }
        let relative = upstream_path(&upstream_root, &path)?;
        let contents = read_source(&path, &mut source_budget)?;
        let class = testbed_scenario_class(&contents)?;
        items.push(discovered_item(
            DiscoveredKind::Example,
            relative,
            format!("{class}::Create"),
        )?);
    }

    let hello_contents = read_source(&hello_world, &mut source_budget)?;
    if !contains_main(&hello_contents)? {
        return Err(source_error("HelloWorld source has no main declaration"));
    }
    items.push(discovered_item(
        DiscoveredKind::Example,
        upstream_path(&upstream_root, &hello_world)?,
        "main".to_owned(),
    )?);

    items.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    if items.len() > MAX_DISCOVERED_ITEMS {
        return Err(source_error("semantic declaration limit exceeded"));
    }
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for item in &items {
        if !ids.insert(item.id.clone()) || !sources.insert(item.source.clone()) {
            return Err(source_error("duplicate semantic registration"));
        }
    }
    Ok(items)
}

#[derive(Default)]
struct SourceBudget {
    files: usize,
    bytes: usize,
}

fn walk_files(root: &Path, budget: &mut SourceBudget) -> Result<Vec<PathBuf>, InventoryError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|_| source_error("allowlisted source root is missing"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(source_error(
            "allowlisted source root is not a real directory",
        ));
    }
    let mut files = Vec::new();
    walk_directory(root, &mut files, budget)?;
    Ok(files)
}

fn walk_directory(
    root: &Path,
    files: &mut Vec<PathBuf>,
    budget: &mut SourceBudget,
) -> Result<(), InventoryError> {
    let mut entries: Vec<_> = fs::read_dir(root)
        .map_err(|_| source_error("failed to read allowlisted source directory"))?
        .collect::<Result<_, _>>()
        .map_err(|_| source_error("failed to inspect allowlisted source directory"))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|_| source_error("failed to inspect source entry"))?;
        if file_type.is_symlink() {
            // The pinned tree contains a non-source asset-directory symlink.
            // Never traverse it; candidate source files are rechecked by
            // `read_source` and are rejected when they are symlinks.
            continue;
        }
        if file_type.is_dir() {
            walk_directory(&entry.path(), files, budget)?;
            continue;
        }
        if file_type.is_file() {
            budget.files = budget
                .files
                .checked_add(1)
                .ok_or_else(|| source_error("source file limit exceeded"))?;
            if budget.files > MAX_SOURCE_FILES {
                return Err(source_error("source file limit exceeded"));
            }
            files.push(entry.path());
            continue;
        }
        return Err(source_error("unknown source entry type"));
    }
    Ok(())
}

fn read_source(path: &Path, budget: &mut SourceBudget) -> Result<String, InventoryError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| source_error("allowlisted source is missing"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(source_error("allowlisted source is not a real file"));
    }
    let size = usize::try_from(metadata.len())
        .map_err(|_| source_error("source file byte limit exceeded"))?;
    if size > MAX_SOURCE_FILE_BYTES {
        return Err(source_error("source file byte limit exceeded"));
    }
    budget.bytes = budget
        .bytes
        .checked_add(size)
        .ok_or_else(|| source_error("total source byte limit exceeded"))?;
    if budget.bytes > MAX_TOTAL_SOURCE_BYTES {
        return Err(source_error("total source byte limit exceeded"));
    }
    fs::read_to_string(path).map_err(|_| source_error("source is not bounded UTF-8"))
}

fn discovered_item(
    kind: DiscoveredKind,
    path: String,
    symbol: String,
) -> Result<DiscoveredItem, InventoryError> {
    let slug = semantic_slug(&symbol)?;
    Ok(DiscoveredItem {
        id: format!("{}{slug}", kind.id_prefix()),
        kind,
        source: DiscoveredSource { path, symbol },
    })
}

fn semantic_slug(value: &str) -> Result<String, InventoryError> {
    let mut output = String::new();
    let mut separator = false;
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(char::from(byte.to_ascii_lowercase()));
            separator = false;
        } else {
            separator = true;
        }
    }
    if output.is_empty() || output.len() > 140 {
        return Err(source_error("semantic identity is invalid"));
    }
    Ok(output)
}

fn merge_reviewed_fields(
    snapshot_path: &Path,
    oracle_revision: &str,
    discovered: &[DiscoveredItem],
) -> Result<String, InventoryError> {
    let mut reviewed = BTreeMap::<DiscoveredSource, Map<String, Value>>::new();
    if snapshot_path.exists() {
        let bytes = fs::read(snapshot_path)
            .map_err(|_| InventoryError::new("corpus-snapshot", "snapshot is unreadable"))?;
        parse_manifest(&bytes, oracle_revision)
            .map_err(|error| InventoryError::new("corpus-snapshot", error.to_string()))?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| InventoryError::new("corpus-snapshot", "snapshot schema is invalid"))?;
        let items = value["items"]
            .as_array()
            .ok_or_else(|| InventoryError::new("corpus-snapshot", "snapshot items are invalid"))?;
        for item in items {
            let source = source_from_value(item)?;
            let expected_id = item["id"]
                .as_str()
                .ok_or_else(|| InventoryError::new("corpus-snapshot", "snapshot ID is invalid"))?;
            let Some(current) = discovered
                .iter()
                .find(|candidate| candidate.source == source)
            else {
                return Err(InventoryError::new(
                    "corpus-snapshot",
                    "snapshot contains a stale semantic source identity",
                ));
            };
            if current.id != expected_id {
                return Err(InventoryError::new(
                    "corpus-snapshot",
                    "snapshot semantic ID does not match its source-derived identity",
                ));
            }
            let object = item.as_object().ok_or_else(|| {
                InventoryError::new("corpus-snapshot", "snapshot item is invalid")
            })?;
            let mut fields = Map::new();
            for name in [
                "applicability",
                "disposition",
                "compatibility_impact",
                "evidence",
                "review",
            ] {
                if let Some(value) = object.get(name) {
                    fields.insert(name.to_owned(), value.clone());
                }
            }
            reviewed.insert(source, fields);
        }
    }

    let mut items = Vec::with_capacity(discovered.len());
    for item in discovered {
        let mut value = serde_json::to_value(item)
            .map_err(|_| InventoryError::new("corpus-snapshot", "failed to encode item"))?;
        let object = value.as_object_mut().ok_or_else(|| {
            InventoryError::new("corpus-snapshot", "failed to encode item object")
        })?;
        if let Some(fields) = reviewed.remove(&item.source) {
            object.extend(fields);
        }
        items.push(value);
    }
    let raw = json!({
        "schema_version": 1,
        "oracle_revision": oracle_revision,
        "items": items,
    });
    let raw_bytes = serde_json::to_vec(&raw)
        .map_err(|_| InventoryError::new("corpus-snapshot", "failed to encode snapshot"))?;
    let manifest = parse_manifest(&raw_bytes, oracle_revision)
        .map_err(|error| InventoryError::new("corpus-snapshot", error.to_string()))?;
    format_manifest(&manifest)
}

fn format_manifest<T: Serialize>(manifest: &T) -> Result<String, InventoryError> {
    let mut output = serde_json::to_string_pretty(manifest)
        .map_err(|_| InventoryError::new("corpus-snapshot", "failed to encode snapshot"))?;
    output.push('\n');
    Ok(output)
}

fn source_from_value(item: &Value) -> Result<DiscoveredSource, InventoryError> {
    let path = item["source"]["path"]
        .as_str()
        .ok_or_else(|| InventoryError::new("corpus-snapshot", "source path is invalid"))?;
    let symbol = item["source"]["symbol"]
        .as_str()
        .ok_or_else(|| InventoryError::new("corpus-snapshot", "source symbol is invalid"))?;
    Ok(DiscoveredSource {
        path: path.to_owned(),
        symbol: symbol.to_owned(),
    })
}

fn require_source_order(bytes: &[u8]) -> Result<(), InventoryError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| InventoryError::new("corpus-snapshot", "snapshot schema is invalid"))?;
    let items = value["items"]
        .as_array()
        .ok_or_else(|| InventoryError::new("corpus-snapshot", "snapshot items are invalid"))?;
    let mut previous: Option<(DiscoveredSource, String)> = None;
    for item in items {
        let source = source_from_value(item)?;
        let id = item["id"]
            .as_str()
            .ok_or_else(|| InventoryError::new("corpus-snapshot", "snapshot ID is invalid"))?
            .to_owned();
        if previous
            .as_ref()
            .is_some_and(|previous| previous >= &(source.clone(), id.clone()))
        {
            return Err(InventoryError::new(
                "corpus-snapshot",
                "snapshot items are not in stable source order",
            ));
        }
        previous = Some((source, id));
    }
    Ok(())
}

fn verify_pinned_checkout(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    let upstream_root = repository_root.join("third_party/liquidfun");
    let gitlink = git_output(
        repository_root,
        &["rev-parse", "HEAD:third_party/liquidfun"],
    )?;
    let checkout = git_output(&upstream_root, &["rev-parse", "HEAD"])?;
    if gitlink != oracle_revision || checkout != oracle_revision {
        return Err(InventoryError::new(
            "revision",
            "initialized checkout and repository gitlink must match the pinned upstream revision",
        ));
    }
    let status = Command::new("git")
        .arg("-C")
        .arg(&upstream_root)
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .map_err(|_| InventoryError::new("revision", "failed to inspect pinned checkout"))?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Err(InventoryError::new(
            "revision",
            "pinned upstream checkout must be initialized and unmodified",
        ));
    }
    Ok(())
}

fn git_output(root: &Path, arguments: &[&str]) -> Result<String, InventoryError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|_| InventoryError::new("revision", "failed to inspect pinned checkout"))?;
    if !output.status.success() {
        return Err(InventoryError::new(
            "revision",
            "pinned upstream checkout is unavailable",
        ));
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|_| InventoryError::new("revision", "pinned revision is not UTF-8"))?;
    Ok(value.trim().to_owned())
}

fn atomic_write(path: &Path, contents: &str) -> Result<(), InventoryError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(InventoryError::new(
            "filesystem",
            "refusing to replace a symlink snapshot",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| InventoryError::new("filesystem", "snapshot path has no parent"))?;
    let temporary = parent.join(format!(".upstream-corpus-{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| InventoryError::new("filesystem", "failed to create atomic snapshot"))?;
    file.write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|_| InventoryError::new("filesystem", "failed to persist atomic snapshot"))?;
    fs::rename(&temporary, path)
        .map_err(|_| InventoryError::new("filesystem", "failed to publish atomic snapshot"))?;
    Ok(())
}

fn upstream_path(upstream_root: &Path, path: &Path) -> Result<String, InventoryError> {
    let relative = path
        .strip_prefix(upstream_root)
        .map_err(|_| source_error("source path escapes the pinned root"))?;
    let mut text = String::new();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(source_error("source path is not normalized"));
        };
        let value = value
            .to_str()
            .ok_or_else(|| source_error("source path is not UTF-8"))?;
        if !text.is_empty() {
            text.push('/');
        }
        text.push_str(value);
    }
    if text.is_empty() {
        return Err(source_error("source path is empty"));
    }
    Ok(text)
}

fn source_error(message: &'static str) -> InventoryError {
    InventoryError::new("corpus-source", message)
}
