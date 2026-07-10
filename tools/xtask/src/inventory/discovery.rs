use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    DiscoveryEntry, DiscoveryKind, DiscoveryLedger, DiscoveryScope, InventoryError, SCHEMA_VERSION,
};

pub(super) const SORT_CONTRACT: &str = "kind rank public_header, source_area, test, example, build_option; then upstream_path and upstream_symbol";

pub(super) fn scan(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<DiscoveryLedger, InventoryError> {
    let upstream_root = repository_root.join("third_party/liquidfun");
    let box2d_root = upstream_root.join("liquidfun/Box2D");
    let engine_root = box2d_root.join("Box2D");
    let unittest_root = box2d_root.join("Unittests");
    let testbed_root = box2d_root.join("Testbed/Tests");
    let hello_world = box2d_root.join("HelloWorld/HelloWorld.cpp");

    let mut entries = Vec::new();
    for path in walk_files(&engine_root)? {
        let relative = upstream_path(&upstream_root, &path)?;
        let extension = path.extension().and_then(|value| value.to_str());
        if extension == Some("h") {
            entries.push(entry(DiscoveryKind::PublicHeader, relative.clone()));
        }
        if matches!(extension, Some("cpp" | "s")) {
            let Some(parent) = Path::new(&relative).parent() else {
                return Err(InventoryError::new(
                    "discovery",
                    "source file has no parent",
                ));
            };
            entries.push(entry(
                DiscoveryKind::SourceArea,
                path_text(parent, "source area")?,
            ));
        }
    }
    for path in walk_files(&unittest_root)? {
        let relative = upstream_path(&upstream_root, &path)?;
        if relative.ends_with("Tests.cpp") {
            entries.push(entry(DiscoveryKind::Test, relative));
        }
    }
    for path in walk_files(&testbed_root)? {
        let relative = upstream_path(&upstream_root, &path)?;
        if path.extension().and_then(|extension| extension.to_str()) == Some("h")
            && path.parent() == Some(testbed_root.as_path())
        {
            entries.push(entry(DiscoveryKind::Example, relative));
        }
    }
    if !hello_world.is_file() {
        return Err(InventoryError::new(
            "discovery",
            format!("missing allowlisted example {}", hello_world.display()),
        ));
    }
    entries.push(entry(
        DiscoveryKind::Example,
        upstream_path(&upstream_root, &hello_world)?,
    ));
    for path in walk_files(&box2d_root)? {
        if path.file_name().and_then(|name| name.to_str()) != Some("CMakeLists.txt") {
            continue;
        }
        let relative = upstream_path(&upstream_root, &path)?;
        let contents = fs::read_to_string(&path).map_err(|error| {
            InventoryError::new(
                "discovery",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        for symbol in cmake_options(&contents) {
            entries.push(DiscoveryEntry {
                kind: DiscoveryKind::BuildOption,
                upstream_path: relative.clone(),
                upstream_symbol: Some(symbol),
            });
        }
    }

    entries.sort_by(compare_entries);
    entries.dedup();
    Ok(DiscoveryLedger {
        schema_version: SCHEMA_VERSION,
        oracle_revision: oracle_revision.to_owned(),
        sort_contract: SORT_CONTRACT.to_owned(),
        scopes: scopes(),
        entries,
    })
}

pub(super) fn compare_entries(left: &DiscoveryEntry, right: &DiscoveryEntry) -> Ordering {
    left.kind
        .rank()
        .cmp(&right.kind.rank())
        .then_with(|| left.upstream_path.cmp(&right.upstream_path))
        .then_with(|| left.upstream_symbol.cmp(&right.upstream_symbol))
}

fn scopes() -> Vec<DiscoveryScope> {
    vec![
        scope(
            DiscoveryKind::PublicHeader,
            "liquidfun/Box2D/Box2D",
            "**/*.h",
        ),
        scope(
            DiscoveryKind::SourceArea,
            "liquidfun/Box2D/Box2D",
            "directories containing **/*.cpp or **/*.neon.s",
        ),
        scope(
            DiscoveryKind::Test,
            "liquidfun/Box2D/Unittests",
            "**/*Tests.cpp",
        ),
        scope(
            DiscoveryKind::Example,
            "liquidfun/Box2D/Testbed/Tests",
            "*.h",
        ),
        scope(
            DiscoveryKind::Example,
            "liquidfun/Box2D/HelloWorld",
            "HelloWorld.cpp",
        ),
        scope(
            DiscoveryKind::BuildOption,
            "liquidfun/Box2D",
            "option(NAME ...) in CMakeLists.txt",
        ),
    ]
}

fn scope(kind: DiscoveryKind, root: &str, matcher: &str) -> DiscoveryScope {
    DiscoveryScope {
        kind,
        root: root.to_owned(),
        matcher: matcher.to_owned(),
    }
}

fn entry(kind: DiscoveryKind, upstream_path: String) -> DiscoveryEntry {
    DiscoveryEntry {
        kind,
        upstream_path,
        upstream_symbol: None,
    }
}

fn cmake_options(contents: &str) -> BTreeSet<String> {
    contents
        .lines()
        .filter_map(|line| line.trim().strip_prefix("option("))
        .filter_map(|rest| {
            rest.split(|character: char| character.is_whitespace() || character == ')')
                .next()
        })
        .filter(|symbol| {
            !symbol.is_empty()
                && symbol
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        })
        .map(str::to_owned)
        .collect()
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, InventoryError> {
    if !root.is_dir() {
        return Err(InventoryError::new(
            "discovery",
            format!("missing allowlisted discovery root {}", root.display()),
        ));
    }
    let mut files = Vec::new();
    walk_directory(root, &mut files)?;
    Ok(files)
}

fn walk_directory(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), InventoryError> {
    let mut entries: Vec<_> = fs::read_dir(root)
        .map_err(|error| {
            InventoryError::new(
                "discovery",
                format!("failed to read {}: {error}", root.display()),
            )
        })?
        .collect::<Result<_, _>>()
        .map_err(|error| InventoryError::new("discovery", error.to_string()))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| {
            InventoryError::new(
                "discovery",
                format!("failed to inspect {}: {error}", entry.path().display()),
            )
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            walk_directory(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn upstream_path(upstream_root: &Path, path: &Path) -> Result<String, InventoryError> {
    let relative = path.strip_prefix(upstream_root).map_err(|error| {
        InventoryError::new(
            "path",
            format!(
                "{} escapes {}: {error}",
                path.display(),
                upstream_root.display()
            ),
        )
    })?;
    path_text(relative, "upstream path")
}

fn path_text(path: &Path, label: &str) -> Result<String, InventoryError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        InventoryError::new(
            "path",
            format!("{label} is not valid UTF-8: {}", path.display()),
        )
    })
}
