//! Current-checkout oracle identity validation contracts.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    OracleCheckoutIdentityError, adapter_source_digest, effective_compile_command_sha256,
    validate_oracle_checkout_identity,
};
use liquidfun_test_protocol::{BuildIdentity, BuildIdentityFields, Phase4BuildIdentityFields};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
const PRESET: &str = "oracle-debug";
const UNITS: [&str; 4] = [
    "collision_probe.cpp",
    "math_probe.cpp",
    "protocol_bits.cpp",
    "rigid_world.cpp",
];

struct IdentityRepository {
    root: PathBuf,
}

impl IdentityRepository {
    fn new() -> io::Result<Self> {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/oracle-identity-tests/{}-{sequence}",
            std::process::id()
        ));
        let source = root.join("tools/reference/src");
        fs::create_dir_all(&source)?;
        fs::write(
            root.join("tools/reference/adapter-inputs.txt"),
            "tools/reference/src/adapter.cpp\ntools/reference/src/adapter.hpp\n",
        )?;
        fs::write(source.join("adapter.cpp"), b"adapter implementation v1")?;
        fs::write(source.join("adapter.hpp"), b"adapter interface v1")?;
        let fixture = Self { root };
        fixture.write_compile_database("-DREVIEWED=1")?;
        Ok(fixture)
    }

    fn write_compile_database(&self, common_flag: &str) -> io::Result<()> {
        let build = self.root.join("target/reference").join(PRESET);
        fs::create_dir_all(&build)?;
        let entries = UNITS
            .map(|unit| {
                let source = self.root.join("tools/reference/src").join(unit);
                serde_json::json!({
                    "directory": build,
                    "file": source,
                    "command": format!(
                        "clang++ -I{}/tools/reference/src {common_flag} -o {}/{unit}.o -c {}",
                        self.root.display(),
                        build.display(),
                        source.display()
                    ),
                })
            })
            .to_vec();
        fs::write(
            build.join("compile_commands.json"),
            serde_json::to_vec_pretty(&entries)?,
        )
    }

    fn write_compile_value(&self, value: &serde_json::Value) -> io::Result<()> {
        fs::write(
            self.root
                .join("target/reference")
                .join(PRESET)
                .join("compile_commands.json"),
            serde_json::to_vec_pretty(value)?,
        )
    }

    fn compile_value(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let bytes = fs::read(
            self.root
                .join("target/reference")
                .join(PRESET)
                .join("compile_commands.json"),
        )?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn identity(
        &self,
        adapter_digest: &str,
        compile_digest: &str,
    ) -> Result<BuildIdentity, liquidfun_test_protocol::BuildIdentityError> {
        let phase4 = Phase4BuildIdentityFields::new(
            compile_digest,
            "AppleClang",
            "21.0.0",
            "arm64-apple-darwin",
            "baseline",
            "<none>",
            "<none>",
            "O0",
            "precise",
            "off",
            "ieee",
            "scalar baseline",
            "macos",
            "libSystem",
            "libSystem",
            "nearest_ties_even",
            true,
        );
        BuildIdentity::new(
            BuildIdentityFields::new(
                "7f20402173fd143a3988c921bc384459c6a858f2",
                "fixture-adapter-v1",
                adapter_digest,
                PRESET,
                "AppleClang",
                "21.0.0",
                "arm64-apple-darwin",
                "Debug",
                "-O0 -g",
                "-lc++",
                "none",
            )
            .with_phase4(phase4),
        )
    }
}

impl Drop for IdentityRepository {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn adapter_digest_tracks_every_manifest_input() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = IdentityRepository::new()?;
    let baseline = adapter_source_digest(&repository.root)?;

    // Act
    fs::write(
        repository.root.join("tools/reference/src/adapter.hpp"),
        b"adapter interface v2",
    )?;
    let changed = adapter_source_digest(&repository.root)?;

    // Assert
    assert_ne!(baseline, changed);
    Ok(())
}

#[test]
fn adapter_manifest_rejects_unsafe_duplicate_and_missing_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = IdentityRepository::new()?;
    let manifest = repository.root.join("tools/reference/adapter-inputs.txt");
    let invalid_manifests = [
        "../outside.cpp\n",
        "/absolute/outside.cpp\n",
        "tools/reference/src/adapter.cpp\ntools/reference/src/adapter.cpp\n",
    ];

    // Act / Assert
    for invalid in invalid_manifests {
        fs::write(&manifest, invalid)?;
        let error = adapter_source_digest(&repository.root)
            .expect_err("unsafe or duplicate manifest input must fail");
        assert!(matches!(
            error,
            OracleCheckoutIdentityError::InvalidAdapterPath { .. }
        ));
        assert!(error.to_string().len() < 256);
        assert!(
            !error
                .to_string()
                .contains(repository.root.to_string_lossy().as_ref())
        );
    }
    fs::write(&manifest, "tools/reference/src/missing.cpp\n")?;
    let missing = adapter_source_digest(&repository.root)
        .expect_err("missing reviewed adapter input must fail");
    assert!(matches!(
        missing,
        OracleCheckoutIdentityError::AdapterInputRead { .. }
    ));
    Ok(())
}

#[test]
fn effective_compile_digest_is_stable_across_repository_relocation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let first = IdentityRepository::new()?;
    let second = IdentityRepository::new()?;

    // Act
    let first_digest = effective_compile_command_sha256(&first.root, PRESET)?;
    let second_digest = effective_compile_command_sha256(&second.root, PRESET)?;

    // Assert
    assert_eq!(first_digest, second_digest);
    Ok(())
}

#[test]
fn effective_compile_digest_rejects_missing_duplicate_and_divergent_units()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = IdentityRepository::new()?;
    let baseline = repository.compile_value()?;
    let baseline_entries = baseline
        .as_array()
        .ok_or("compile database fixture must be an array")?;
    let missing = serde_json::Value::Array(baseline_entries[..3].to_vec());
    let mut duplicate_entries = baseline_entries.clone();
    duplicate_entries[3] = duplicate_entries[0].clone();
    let duplicate = serde_json::Value::Array(duplicate_entries);
    let mut divergent_entries = baseline_entries.clone();
    let command = divergent_entries[3]["command"]
        .as_str()
        .ok_or("fixture command must be a string")?
        .replace("-DREVIEWED=1", "-DUNREVIEWED=1");
    divergent_entries[3]["command"] = serde_json::Value::String(command);
    let divergent = serde_json::Value::Array(divergent_entries);
    let cases = [
        (
            missing,
            OracleCheckoutIdentityError::MissingCompileUnit {
                unit: "rigid_world.cpp",
            },
        ),
        (
            duplicate,
            OracleCheckoutIdentityError::DuplicateCompileUnit {
                unit: "collision_probe.cpp",
            },
        ),
        (
            divergent,
            OracleCheckoutIdentityError::DivergentCompileFlags,
        ),
    ];

    // Act / Assert
    for (value, expected) in cases {
        repository.write_compile_value(&value)?;
        let error = effective_compile_command_sha256(&repository.root, PRESET)
            .expect_err("invalid effective compile database must fail");
        assert_eq!(error, expected);
        assert!(error.to_string().len() < 256);
    }
    fs::write(
        repository
            .root
            .join("target/reference")
            .join(PRESET)
            .join("compile_commands.json"),
        b"not-json\n",
    )?;
    let malformed = effective_compile_command_sha256(&repository.root, PRESET)
        .expect_err("malformed effective compile database must fail");
    assert_eq!(
        malformed,
        OracleCheckoutIdentityError::CompileDatabaseMalformed
    );
    Ok(())
}

#[test]
fn checkout_validator_accepts_current_identity_and_rejects_each_stale_digest()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = IdentityRepository::new()?;
    let adapter = adapter_source_digest(&repository.root)?;
    let compile = effective_compile_command_sha256(&repository.root, PRESET)?;
    let current = repository.identity(&adapter, &compile)?;
    let stale_adapter = repository.identity(&"aa".repeat(32), &compile)?;
    let stale_compile = repository.identity(&adapter, &"bb".repeat(32))?;

    // Act
    let accepted = validate_oracle_checkout_identity(&repository.root, PRESET, &current);
    let adapter_error = validate_oracle_checkout_identity(&repository.root, PRESET, &stale_adapter);
    let compile_error = validate_oracle_checkout_identity(&repository.root, PRESET, &stale_compile);

    // Assert
    assert!(accepted.is_ok());
    assert_eq!(
        adapter_error,
        Err(OracleCheckoutIdentityError::AdapterDigestMismatch)
    );
    assert_eq!(
        compile_error,
        Err(OracleCheckoutIdentityError::CompileDigestMismatch)
    );
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root must be present")
        .to_path_buf()
}
