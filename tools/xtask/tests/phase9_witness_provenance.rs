//! Focused regression coverage for target-scoped Phase 9 witness materials.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

#[derive(Debug, PartialEq, Eq)]
struct ProvenanceError {
    category: &'static str,
    message: String,
}

impl ProvenanceError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

impl Display for ProvenanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "provenance/{}: {}", self.category, self.message)
    }
}

impl Error for ProvenanceError {}

#[path = "../src/provenance/evidence_schema.rs"]
mod evidence_schema;
#[path = "../src/provenance/phase9_witness/materials.rs"]
mod materials;

use materials::{
    MATERIALS_PATH, MaterialsDerivation, resolve_declared_materials, validate_repository_binding,
    validate_target_scoped_materials,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const TARGET: &str = "phase9-lifecycle-contact-witness";
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct MaterialsFixture {
    root: PathBuf,
    build: PathBuf,
    reply: PathBuf,
    manifest: PathBuf,
    target_json: PathBuf,
    depfile: PathBuf,
}

impl MaterialsFixture {
    fn new() -> TestResult<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/phase9-materials-fixtures/{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let build = root.join("build");
        let reply = build.join(".cmake/api/v1/reply");
        fs::create_dir_all(root.join("tools/reference/src"))?;
        fs::create_dir_all(root.join("include"))?;
        fs::create_dir_all(build.join("generated"))?;
        fs::create_dir_all(&reply)?;
        fs::write(
            root.join("tools/reference/CMakeLists.txt"),
            "build-rule-v1\n",
        )?;
        fs::write(
            root.join("tools/reference/src/witness.cpp"),
            "#include \"used.hpp\"\n",
        )?;
        fs::write(root.join("include/used.hpp"), "used-v1\n")?;
        fs::write(build.join("generated/config.hpp"), "generated-v1\n")?;

        let index = reply.join("index-fixture.json");
        fs::write(
            &index,
            serde_json::to_vec_pretty(&json!({
                "objects": [{
                    "kind": "codemodel",
                    "jsonFile": "codemodel-v2.json",
                    "version": {"major": 2, "minor": 6}
                }]
            }))?,
        )?;
        fs::write(
            reply.join("codemodel-v2.json"),
            serde_json::to_vec_pretty(&json!({
                "kind": "codemodel",
                "version": {"major": 2, "minor": 6},
                "paths": {
                    "source": root.join("tools/reference"),
                    "build": build
                },
                "configurations": [{
                    "name": "Debug",
                    "directories": [],
                    "projects": [],
                    "targets": [{
                        "directoryIndex": 0,
                        "projectIndex": 0,
                        "id": format!("{TARGET}::fixture"),
                        "jsonFile": "target-phase9.json",
                        "name": TARGET
                    }]
                }]
            }))?,
        )?;
        let target_json = reply.join("target-phase9.json");
        write_target_json(&target_json, &root, &build, None)?;
        let presets = root.join("tools/reference/CMakePresets.json");
        fs::write(
            &presets,
            serde_json::to_vec_pretty(&json!({
                "version": 6,
                "configurePresets": [
                    {
                        "name": "oracle-base",
                        "hidden": true,
                        "cacheVariables": {"CMAKE_EXPORT_COMPILE_COMMANDS": "ON"}
                    },
                    {
                        "name": "oracle-debug",
                        "inherits": "oracle-base",
                        "cacheVariables": {"CMAKE_BUILD_TYPE": "Debug"}
                    }
                ]
            }))?,
        )?;
        let depfile = build.join("witness.d");
        fs::write(
            &depfile,
            format!(
                "witness.o: {} {} {}\n",
                root.join("tools/reference/src/witness.cpp").display(),
                root.join("include/used.hpp").display(),
                build.join("generated/config.hpp").display()
            ),
        )?;
        let manifest = root.join(MATERIALS_PATH);
        write_manifest(&manifest, &default_materials())?;
        Ok(Self {
            root,
            build,
            reply,
            manifest,
            target_json,
            depfile,
        })
    }

    fn derivation(&self) -> MaterialsDerivation {
        MaterialsDerivation {
            reply_index: self.reply.join("index-fixture.json"),
            reply_directory: self.reply.clone(),
            build_directory: self.build.clone(),
            presets_path: self.root.join("tools/reference/CMakePresets.json"),
            depfiles: vec![self.depfile.clone()],
        }
    }

    fn resolve(&self) -> Result<materials::ResolvedMaterials, ProvenanceError> {
        validate_target_scoped_materials(&self.root, &self.manifest, &self.derivation())
    }

    fn mutate_target(&self, mutation: TargetMutation) -> TestResult {
        write_target_json(&self.target_json, &self.root, &self.build, Some(mutation))
    }

    fn cleanup(self) -> TestResult {
        fs::remove_dir_all(self.root)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum TargetMutation {
    Source,
    Definition,
    CompileFragment,
}

#[test]
fn target_scoped_materials() -> TestResult {
    // Arrange
    let repository_materials = resolve_declared_materials(&workspace_root())?;
    let fixture = MaterialsFixture::new()?;
    let baseline = fixture.resolve()?;

    // Assert
    assert!(repository_materials.count > 100);
    assert_eq!(baseline.count, default_materials().len());

    // Act
    fs::write(fixture.root.join("unrelated-adapter.cpp"), "unrelated-v2\n")?;
    let unrelated = fixture.resolve()?;
    fs::write(
        fixture.root.join("tools/reference/src/witness.cpp"),
        "#include \"used.hpp\"\n// scoped change\n",
    )?;
    let scoped = fixture.resolve()?;

    // Assert
    assert_eq!(baseline.digest, unrelated.digest);
    assert_ne!(baseline.digest, scoped.digest);

    // Arrange
    fs::write(
        fixture.root.join("tools/reference/src/witness.cpp"),
        "#include \"used.hpp\"\n",
    )?;
    let repository_revision = commit_fixture(&fixture.root)?;
    fs::write(
        fixture.root.join("tools/reference/src/witness.cpp"),
        "#include \"used.hpp\"\n// edited bytes with a matching recomputed digest\n",
    )?;
    let edited = fixture.resolve()?;

    // Act
    let binding_error = validate_repository_binding(&fixture.root, &repository_revision)
        .expect_err("an edited digest cannot replace repository-bound materials");

    // Assert
    assert_ne!(baseline.digest, edited.digest);
    assert_eq!(binding_error.category, "repository");
    assert!(
        binding_error
            .message
            .contains("tools/reference/src/witness.cpp")
    );

    let cases = [
        (
            TargetMutation::Source,
            "tools/reference/src/extra.cpp",
            "unexpected source material",
        ),
        (
            TargetMutation::Definition,
            "EXTRA_MODE=1",
            "unexpected compile_definition material",
        ),
        (
            TargetMutation::CompileFragment,
            "-fno-builtin",
            "unexpected compile_fragment material",
        ),
    ];
    for (mutation, expected_identity, expected_kind) in cases {
        // Arrange
        let case_fixture = MaterialsFixture::new()?;
        case_fixture.mutate_target(mutation)?;

        // Act
        let error = case_fixture
            .resolve()
            .expect_err("an undeclared target material must fail");

        // Assert
        assert_eq!(error.category, "materials");
        assert!(error.message.contains(expected_kind), "{}", error.message);
        assert!(
            error.message.contains(expected_identity),
            "{}",
            error.message
        );
        case_fixture.cleanup()?;
    }

    // Arrange
    let header_fixture = MaterialsFixture::new()?;
    fs::write(header_fixture.root.join("include/extra.hpp"), "extra\n")?;
    let depfile = fs::read_to_string(&header_fixture.depfile)?;
    fs::write(
        &header_fixture.depfile,
        depfile.replace(
            '\n',
            &format!(
                " {}\n",
                header_fixture.root.join("include/extra.hpp").display()
            ),
        ),
    )?;

    // Act
    let header_error = header_fixture
        .resolve()
        .expect_err("an undeclared compiler dependency must fail");

    // Assert
    assert!(header_error.message.contains("include/extra.hpp"));
    header_fixture.cleanup()?;

    // Arrange
    let declared_fixture = MaterialsFixture::new()?;
    let mut declared = default_materials();
    declared.push(json!({
        "kind": "source",
        "identity": "tools/reference/src/z-declared-only.cpp"
    }));
    write_manifest(&declared_fixture.manifest, &declared)?;

    // Act
    let declared_error = declared_fixture
        .resolve()
        .expect_err("a declared-only material must fail");

    // Assert
    assert!(
        declared_error
            .message
            .contains("tools/reference/src/z-declared-only.cpp")
    );
    declared_fixture.cleanup()?;
    fixture.cleanup()
}

#[test]
fn phase13_evidence_classes_fail_closed() -> TestResult {
    // Arrange
    let manifest = fs::read_to_string(workspace_root().join("reference/artifacts/manifest.toml"))?;
    evidence_schema::parse_and_validate(&manifest, "7f20402173fd143a3988c921bc384459c6a858f2")?;
    let cases = [
        (
            manifest.replacen(
                "source_path = \"liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp\"\n",
                "",
                1,
            ),
            "source_path",
        ),
        (
            manifest.replacen(
                "alteration_summary = \"Repository-authored semantic observations generated from the pinned upstream oracle without copying source, raw object memory, or Rust-produced expectations.\"",
                "alteration_summary = \"\"",
                1,
            ),
            "alteration_summary",
        ),
        (
            manifest.replacen(
                "notice_refs = [\"THIRD_PARTY_NOTICES.md\"]",
                "notice_refs = []",
                1,
            ),
            "THIRD_PARTY_NOTICES.md",
        ),
        (
            manifest.replacen("digest_mode = \"exact_bytes_sha256\"\n", "", 1),
            "digest_mode",
        ),
        (
            manifest.replacen(
                "digest_mode = \"exact_bytes_sha256\"",
                "digest_mode = \"unknown\"",
                1,
            ),
            "incomplete or malformed",
        ),
        (
            manifest.replacen(
                "digest_mode = \"phase13_receipt_semantic_v2\"",
                "digest_mode = \"exact_bytes_sha256\"",
                1,
            ),
            "incomplete or malformed",
        ),
    ];

    for (candidate, expected) in cases {
        // Act
        let error = evidence_schema::parse_and_validate(
            &candidate,
            "7f20402173fd143a3988c921bc384459c6a858f2",
        )
        .expect_err("missing Phase 13 evidence provenance must fail");

        // Assert
        assert!(error.message.contains(expected), "{}", error.message);
    }
    Ok(())
}

fn default_materials() -> Vec<Value> {
    vec![
        json!({"kind": "build_rule", "identity": "tools/reference/CMakeLists.txt"}),
        json!({"kind": "compile_definition", "identity": format!("{TARGET}:MODE=1")}),
        json!({"kind": "compile_fragment", "identity": format!("{TARGET}:-O0")}),
        json!({"kind": "generated_input", "identity": "<build>/generated/config.hpp"}),
        json!({"kind": "header", "identity": "include/used.hpp"}),
        json!({"kind": "include_path", "identity": format!("{TARGET}:ordinary:<repo>/include")}),
        json!({"kind": "link_fragment", "identity": format!("{TARGET}:flags:-Wl,--as-needed")}),
        json!({"kind": "preset_value", "identity": "CMAKE_BUILD_TYPE=Debug"}),
        json!({"kind": "preset_value", "identity": "CMAKE_EXPORT_COMPILE_COMMANDS=ON"}),
        json!({"kind": "preset_value", "identity": "preset=oracle-debug"}),
        json!({"kind": "source", "identity": "tools/reference/src/witness.cpp"}),
    ]
}

fn write_manifest(path: &Path, materials: &[Value]) -> TestResult {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": 1,
            "target": TARGET,
            "preset": "oracle-debug",
            "materials": materials
        }))?,
    )?;
    Ok(())
}

fn write_target_json(
    path: &Path,
    root: &Path,
    build: &Path,
    maybe_mutation: Option<TargetMutation>,
) -> TestResult {
    let mut sources = vec![json!({
        "path": "src/witness.cpp",
        "compileGroupIndex": 0,
        "sourceGroupIndex": 0,
        "backtrace": 0
    })];
    let mut definitions = vec![json!({"define": "MODE=1", "backtrace": 0})];
    let mut fragments = vec![json!({"fragment": "-O0", "backtrace": 0})];
    match maybe_mutation {
        Some(TargetMutation::Source) => {
            fs::write(root.join("tools/reference/src/extra.cpp"), "extra\n")?;
            sources.push(json!({
                "path": "src/extra.cpp",
                "compileGroupIndex": 0,
                "sourceGroupIndex": 0,
                "backtrace": 0
            }));
        }
        Some(TargetMutation::Definition) => {
            definitions.push(json!({"define": "EXTRA_MODE=1", "backtrace": 0}));
        }
        Some(TargetMutation::CompileFragment) => {
            fragments.push(json!({"fragment": "-fno-builtin", "backtrace": 0}));
        }
        None => {}
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "name": TARGET,
            "type": "EXECUTABLE",
            "paths": {
                "source": root.join("tools/reference"),
                "build": build
            },
            "dependencies": [],
            "sources": sources,
            "compileGroups": [{
                "compileCommandFragments": fragments,
                "defines": definitions,
                "includes": [{
                    "path": root.join("include"),
                    "isSystem": false,
                    "backtrace": 0
                }],
                "language": "CXX",
                "sourceIndexes": [0]
            }],
            "link": {
                "commandFragments": [{
                    "fragment": "-Wl,--as-needed",
                    "role": "flags"
                }],
                "language": "CXX"
            },
            "backtraceGraph": {
                "files": ["CMakeLists.txt"],
                "commands": ["add_executable"],
                "nodes": [{"file": 0}]
            }
        }))?,
    )?;
    Ok(())
}

fn commit_fixture(root: &Path) -> TestResult<String> {
    for arguments in [
        vec!["init", "--quiet"],
        vec!["config", "user.email", "phase13@example.invalid"],
        vec!["config", "user.name", "Phase 13 Test"],
        vec!["add", "."],
        vec!["commit", "--quiet", "-m", "fixture"],
    ] {
        let status = Command::new("git")
            .args(arguments)
            .current_dir(root)
            .status()?;
        if !status.success() {
            return Err(format!("fixture git command failed with {status}").into());
        }
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err("failed to read fixture revision".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("xtask must live beneath the workspace root")
}
