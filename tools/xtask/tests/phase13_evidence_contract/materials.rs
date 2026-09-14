//! Synthetic hashing contracts; these do not validate canonical native bytes.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use super::phase13_evidence::witness_materials_identity;

const MANIFEST: &str = "tools/reference/phase9-lifecycle-contact-witness.materials.json";
const FILES: [(&str, &str, &str); 4] = [
    ("build_rule", "inputs/build.txt", "build-v1\n"),
    ("generated_input", "inputs/generated.txt", "generated-v1\n"),
    ("header", "inputs/header.txt", "header-v1\n"),
    ("source", "inputs/source.txt", "source-v1\n"),
];

struct Fixture {
    root: PathBuf,
    materials: Vec<Value>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = super::temporary_directory(label);
        fs::create_dir_all(root.join("tools/reference")).expect("manifest directory");
        fs::create_dir_all(root.join("inputs")).expect("input directory");
        let mut materials = Vec::new();
        for (kind, identity, contents) in FILES {
            fs::write(root.join(identity), contents).expect("synthetic material writes");
            materials.push(json!({"kind": kind, "identity": identity}));
        }
        materials.push(json!({"kind": "compile_definition", "identity": "fixture:MODE=1"}));
        let fixture = Self { root, materials };
        fixture.write_manifest();
        fixture
    }

    fn write_manifest(&self) {
        fs::write(
            self.root.join(MANIFEST),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "target": "phase9-lifecycle-contact-witness",
                "preset": "oracle-debug",
                "materials": self.materials,
            }))
            .expect("synthetic manifest encodes"),
        )
        .expect("synthetic manifest writes");
    }

    fn identity(&self) -> (String, usize) {
        witness_materials_identity(&self.root).expect("complete synthetic materials hash")
    }

    fn cleanup(self) {
        fs::remove_dir_all(self.root).expect("synthetic fixture cleanup");
    }
}

#[test]
fn producer_hashes_complete_scoped_material_bytes() {
    // Arrange
    let fixture = Fixture::new("materials-known-digest");

    // Act
    let identity = fixture.identity();

    // Assert: independently calculated with Python hashlib and big-endian u64 lengths.
    assert_eq!(identity.1, 5);
    assert_eq!(
        identity.0,
        "9fca1c0db33ce9124311acd1cbff697c4e8f255e2dc5f78eae9435684cc112d4"
    );
    fixture.cleanup();
}

#[test]
fn producer_material_identity_ignores_unlisted_files() {
    // Arrange
    let fixture = Fixture::new("materials-unlisted");
    let baseline = fixture.identity();

    // Act
    fs::write(fixture.root.join("inputs/unlisted.txt"), "unrelated").expect("unlisted input");

    // Assert
    assert_eq!(fixture.identity(), baseline);
    fixture.cleanup();
}

#[test]
fn producer_material_identity_normalizes_order_and_duplicates() {
    // Arrange
    let mut fixture = Fixture::new("materials-set");
    let baseline = fixture.identity();

    // Act
    fixture.materials.reverse();
    fixture.materials.push(fixture.materials[0].clone());
    fixture.write_manifest();

    // Assert
    assert_eq!(fixture.identity(), baseline);
    fixture.cleanup();
}

#[test]
fn producer_material_identity_binds_each_file_kind_to_contents() {
    // Arrange, Act, Assert: every byte-bearing material kind must bind its contents.
    for (_, identity, _) in FILES {
        let fixture = Fixture::new("materials-changed");
        let baseline = fixture.identity();
        fs::write(fixture.root.join(identity), "changed").expect("scoped mutation");
        assert_ne!(fixture.identity().0, baseline.0, "{identity}");
        fixture.cleanup();
    }
}

#[test]
fn producer_material_identity_rejects_missing_required_file() {
    // Arrange
    let fixture = Fixture::new("materials-missing");
    fs::remove_file(fixture.root.join("inputs/source.txt")).expect("remove scoped input");

    // Act
    let error = witness_materials_identity(&fixture.root).expect_err("missing input must fail");

    // Assert
    let message = error.to_string();
    assert!(message.contains("Filesystem"));
    assert!(message.contains("inputs/source.txt"));
    fixture.cleanup();
}

#[test]
fn tracked_material_manifest_retains_complete_identity_inventory() {
    // Arrange
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join(MANIFEST)).expect("tracked materials manifest"))
            .expect("manifest JSON");

    // Act
    let materials = manifest["materials"]
        .as_array()
        .expect("material inventory");
    let identities = materials
        .iter()
        .map(|material| {
            (
                material["kind"].as_str().expect("material kind"),
                material["identity"].as_str().expect("material identity"),
            )
        })
        .collect::<BTreeSet<_>>();

    // Assert: inventory coverage is independent of native content acceptance.
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["target"], "phase9-lifecycle-contact-witness");
    assert_eq!(manifest["preset"], "oracle-debug");
    assert_eq!(materials.len(), 176);
    assert_eq!(identities.len(), 176);
    assert!(identities.iter().all(|(kind, identity)| {
        !identity.is_empty()
            && matches!(
                *kind,
                "build_rule"
                    | "compile_definition"
                    | "compile_fragment"
                    | "generated_input"
                    | "header"
                    | "include_path"
                    | "link_fragment"
                    | "link_input"
                    | "preset_value"
                    | "source"
            )
    }));
}
