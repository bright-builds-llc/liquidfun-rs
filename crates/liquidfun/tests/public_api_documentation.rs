//! Black-box contracts for the public documentation and safety boundary.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use liquidfun::math::Vec2;
use liquidfun::{
    AssociationMap, BodyDef, HandleError, NoDecisionHook, ParticleBufferBundle,
    ParticleBufferLanes, ParticleFlags, ParticleSystemDef, StepConfiguration, StepLimits, World,
};

type TestResult = Result<(), Box<dyn Error>>;

const CRATE_NAVIGATION_MARKERS: [&str; 10] = [
    "# API navigation",
    "Math and settings",
    "Collision and shapes",
    "World, bodies, and fixtures",
    "Joints and rope",
    "Particles and groups",
    "Callbacks and events",
    "Queries, observations, and profiles",
    "Handles and invalidation",
    "Errors and upstream naming",
];

const SAFETY_MARKERS: [&str; 10] = [
    "# Safety",
    "## Identity and invalidation",
    "## Contacts and callbacks",
    "## World locking and deferred commands",
    "## Application user data",
    "## Owned particle buffers",
    "## Owned events and observations",
    "## Renderer and oracle isolation",
    "## Panic policy",
    "unsafe_code = \"forbid\"",
];

#[test]
fn public_documentation_names_navigation_and_safety_contracts() -> TestResult {
    // Arrange
    let root = workspace_root();
    let crate_docs = fs::read_to_string(root.join("crates/liquidfun/src/lib.rs"))?;
    let safety = fs::read_to_string(root.join("SAFETY.md"))?;

    // Act, Assert
    for marker in CRATE_NAVIGATION_MARKERS {
        assert!(
            crate_docs.contains(marker),
            "crate documentation must contain `{marker}`"
        );
    }
    for marker in SAFETY_MARKERS {
        assert!(safety.contains(marker), "SAFETY.md must contain `{marker}`");
    }
    Ok(())
}

#[test]
fn handles_report_foreign_and_stale_identity_without_mutation() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let live = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let stale = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let mut foreign_world = World::new().expect("test world key should remain available");
    let foreign = foreign_world
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let stale_result = world.body_snapshot(stale);
    let foreign_result = world.body_snapshot(foreign);

    // Assert
    assert_eq!(stale_result, Err(HandleError::StaleOrDestroyed));
    assert_eq!(foreign_result, Err(HandleError::WrongWorld));
    assert!(world.contains_body(live));
}

#[test]
fn callbacks_return_owned_reports_after_world_unlocks() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let mut hook = NoDecisionHook;
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test configuration should be valid");

    // Act
    let report = world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("empty world step should succeed");
    let snapshot = world
        .body_snapshot(body)
        .expect("world should be unlocked after step");

    // Assert
    assert!(!world.is_locked());
    assert_eq!(report.events().len(), 0);
    assert_eq!(snapshot.position(), Vec2::ZERO);
}

#[test]
fn application_data_and_particle_buffers_remain_consumer_owned() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let mut names = AssociationMap::new();
    names.insert(body, String::from("player"));
    let lanes = ParticleBufferLanes::new(
        Vec::<Vec2>::with_capacity(2),
        Vec::<Vec2>::with_capacity(2),
        Vec::<ParticleFlags>::with_capacity(2),
        None,
    );
    let bundle = ParticleBufferBundle::fixed(2, lanes).expect("fixed lanes should be valid");
    let system = world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), bundle)
        .expect("particle system should fit");

    // Act
    let body_records = world.destroy_body(body).expect("body should be live");
    let removed_names = names.cleanup(&body_records);
    let teardown = world
        .destroy_particle_system_with_buffers(system)
        .expect("particle system should be live");
    let returned_lanes = teardown.into_lanes();

    // Assert
    assert_eq!(removed_names, vec![String::from("player")]);
    assert!(returned_lanes.positions().is_empty());
    assert!(returned_lanes.velocities().is_empty());
    assert!(returned_lanes.flags().is_empty());
}

#[test]
fn production_source_contains_no_constructible_unsafe() -> TestResult {
    // Arrange
    let source_root = workspace_root().join("crates/liquidfun/src");
    let source_files = rust_source_files(&source_root)?;

    // Act
    let violations = source_files
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            let stripped = strip_comments_and_literals(&source);
            contains_unsafe_construct(&stripped).then(|| path.display().to_string())
        })
        .collect::<Vec<_>>();

    // Assert
    assert!(
        violations.is_empty(),
        "production source contains constructible unsafe syntax: {violations:?}"
    );
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("liquidfun manifest must be nested beneath the workspace root")
        .to_path_buf()
}

fn rust_source_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn contains_unsafe_construct(source: &str) -> bool {
    let tokens = tokenize(source);
    tokens.windows(2).any(|pair| {
        pair[0] == "unsafe"
            && matches!(
                pair[1].as_str(),
                "{" | "(" | "fn" | "trait" | "impl" | "extern"
            )
    })
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in source.chars() {
        if character == '_' || character.is_ascii_alphanumeric() {
            current.push(character);
            continue;
        }
        if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
        if matches!(character, '{' | '(') {
            tokens.push(character.to_string());
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn strip_comments_and_literals(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment(usize),
        String { escaped: bool },
        Character { escaped: bool },
        RawString { hashes: usize },
    }

    let bytes = source.as_bytes();
    let mut stripped = String::with_capacity(source.len());
    let mut index = 0;
    let mut state = State::Code;
    while index < bytes.len() {
        match state {
            State::Code if bytes[index..].starts_with(b"//") => {
                stripped.push_str("  ");
                index += 2;
                state = State::LineComment;
            }
            State::Code if bytes[index..].starts_with(b"/*") => {
                stripped.push_str("  ");
                index += 2;
                state = State::BlockComment(1);
            }
            State::Code if bytes[index] == b'"' => {
                stripped.push(' ');
                index += 1;
                state = State::String { escaped: false };
            }
            State::Code if bytes[index] == b'\'' && starts_character_literal(bytes, index) => {
                stripped.push(' ');
                index += 1;
                state = State::Character { escaped: false };
            }
            State::Code => {
                let maybe_raw = raw_string_prefix(bytes, index);
                if let Some((prefix_len, hashes)) = maybe_raw {
                    stripped.extend(std::iter::repeat_n(' ', prefix_len));
                    index += prefix_len;
                    state = State::RawString { hashes };
                } else {
                    stripped.push(char::from(bytes[index]));
                    index += 1;
                }
            }
            State::LineComment => {
                let character = char::from(bytes[index]);
                stripped.push(if character == '\n' { '\n' } else { ' ' });
                index += 1;
                if character == '\n' {
                    state = State::Code;
                }
            }
            State::BlockComment(depth) if bytes[index..].starts_with(b"/*") => {
                stripped.push_str("  ");
                index += 2;
                state = State::BlockComment(depth + 1);
            }
            State::BlockComment(depth) if bytes[index..].starts_with(b"*/") => {
                stripped.push_str("  ");
                index += 2;
                state = if depth == 1 {
                    State::Code
                } else {
                    State::BlockComment(depth - 1)
                };
            }
            State::BlockComment(depth) => {
                let character = char::from(bytes[index]);
                stripped.push(if character == '\n' { '\n' } else { ' ' });
                index += 1;
                state = State::BlockComment(depth);
            }
            State::String { escaped } => {
                let byte = bytes[index];
                stripped.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
                state = if escaped {
                    State::String { escaped: false }
                } else if byte == b'\\' {
                    State::String { escaped: true }
                } else if byte == b'"' {
                    State::Code
                } else {
                    State::String { escaped: false }
                };
            }
            State::Character { escaped } => {
                let byte = bytes[index];
                stripped.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
                state = if escaped {
                    State::Character { escaped: false }
                } else if byte == b'\\' {
                    State::Character { escaped: true }
                } else if byte == b'\'' {
                    State::Code
                } else {
                    State::Character { escaped: false }
                };
            }
            State::RawString { hashes } => {
                let end = raw_string_end(bytes, index, hashes);
                if let Some(length) = end {
                    stripped.extend(std::iter::repeat_n(' ', length));
                    index += length;
                    state = State::Code;
                } else {
                    let character = char::from(bytes[index]);
                    stripped.push(if character == '\n' { '\n' } else { ' ' });
                    index += 1;
                    state = State::RawString { hashes };
                }
            }
        }
    }
    stripped
}

fn starts_character_literal(bytes: &[u8], index: usize) -> bool {
    let Some(next) = bytes.get(index + 1) else {
        return false;
    };
    *next == b'\\' || bytes.get(index + 2) == Some(&b'\'')
}

fn raw_string_prefix(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    if bytes.get(index) != Some(&b'r') {
        return None;
    }
    let mut cursor = index + 1;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then(|| (cursor - index + 1, cursor - index - 1))
}

fn raw_string_end(bytes: &[u8], index: usize, hashes: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'"') {
        return None;
    }
    let end = index.checked_add(hashes + 1)?;
    (end <= bytes.len() && bytes[index + 1..end].iter().all(|byte| *byte == b'#'))
        .then_some(hashes + 1)
}
