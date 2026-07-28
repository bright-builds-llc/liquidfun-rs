use super::{
    BuildEvidenceTier, BuildIdentity, BuildIdentityError, BuildIdentityFields,
    Phase4BuildIdentityFields, Sha256Hex,
};

fn valid_fields() -> BuildIdentityFields {
    BuildIdentityFields::new(
        "7f20402173fd143a3988c921bc384459c6a858f2",
        "adapter-v1",
        "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
        "oracle-debug",
        "Clang",
        "22.1.8",
        "x86_64-unknown-linux-gnu",
        "Debug",
        "-O0 -g",
        "-lc++",
        "none",
    )
}

fn canonical_phase4() -> Phase4BuildIdentityFields {
    Phase4BuildIdentityFields::new(
        "11".repeat(32),
        "Clang",
        "22.1.8",
        "x86_64-unknown-linux-gnu",
        "baseline",
        "<none>",
        "<none>",
        "O3",
        "precise",
        "off",
        "ieee",
        "scalar baseline",
        "linux",
        "glibc",
        "libm",
        "nearest_ties_even",
        true,
    )
}

#[test]
fn build_identity_validates_and_hashes_all_fields_stably() {
    // Arrange
    let fields = valid_fields();
    let changed_fields = valid_fields().with_compiler_id("AppleClang");

    // Act
    let first = BuildIdentity::new(fields.clone()).expect("valid identity should build");
    let second = BuildIdentity::new(fields).expect("same identity should build");
    let changed = BuildIdentity::new(changed_fields).expect("changed identity should build");

    // Assert
    assert_eq!(first.identity_sha256(), second.identity_sha256());
    assert_ne!(first.identity_sha256(), changed.identity_sha256());
    assert_eq!(
        first.oracle_revision(),
        "7f20402173fd143a3988c921bc384459c6a858f2"
    );
}

#[test]
fn build_identity_rejects_invalid_revision_and_empty_fields() {
    // Arrange
    let invalid_revision = valid_fields().with_oracle_revision("short");
    let empty_compiler = valid_fields().with_compiler_id("");

    // Act
    let revision_error = BuildIdentity::new(invalid_revision).expect_err("revision should fail");
    let compiler_error = BuildIdentity::new(empty_compiler).expect_err("field should fail");

    // Assert
    assert_eq!(revision_error, BuildIdentityError::InvalidOracleRevision);
    assert_eq!(
        compiler_error,
        BuildIdentityError::EmptyField("compiler_id")
    );
}

#[test]
fn build_identity_rejects_a_mismatched_reported_hash() {
    // Arrange
    let reported = Sha256Hex::new("00".repeat(32)).expect("zero hash is syntactically valid");

    // Act
    let error = BuildIdentity::from_reported(valid_fields(), &reported)
        .expect_err("mismatched identity hash should fail");

    // Assert
    assert_eq!(error, BuildIdentityError::IdentityHashMismatch);
}

#[test]
fn canonical_identity_rejects_forbidden_fp_flags() {
    // Arrange
    let fields = valid_fields()
        .with_phase4(canonical_phase4().with_feature_set("scalar baseline -ffast-math"));

    // Act
    let error = BuildIdentity::new(fields).expect_err("canonical fast math should fail");

    // Assert
    assert_eq!(error, BuildIdentityError::CanonicalForbiddenFlags);
}

#[test]
fn canonical_clang_identity_accepts_the_pinned_linux_target_triple() {
    // Arrange
    let mut phase4 = canonical_phase4();
    phase4.target_triple = "x86_64-pc-linux-gnu".to_owned();
    let fields = valid_fields()
        .with_target("x86_64-pc-linux-gnu")
        .with_phase4(phase4);

    // Act
    let identity = BuildIdentity::new(fields).expect("pinned Clang identity should validate");

    // Assert
    assert_eq!(identity.evidence_tier(), BuildEvidenceTier::D1Canonical);
}

#[test]
fn canonical_identity_decodes_and_rejects_hex_encoded_rustflags() {
    // Arrange: `-C` and `target-cpu=native` as a parsed rustflag vector.
    let encoded = "hexvec:2d43,7461726765742d6370753d6e6174697665";
    let mut phase4 = canonical_phase4();
    phase4.compiler_id = "rustc".to_owned();
    phase4.compiler_version = "1.97.0".to_owned();
    let fields = valid_fields()
        .with_compiler_id("rustc")
        .with_compiler_version("1.97.0")
        .with_effective_compile_flags(format!("encoded_rustflags={encoded}"))
        .with_phase4(phase4);

    // Act
    let error = BuildIdentity::new(fields).expect_err("native CPU tuning must fail closed");

    // Assert
    assert_eq!(error, BuildIdentityError::CanonicalForbiddenFlags);
}

#[test]
fn canonical_identity_rejects_fixed_cpu_and_simd_tuning() {
    for tuning in ["-march=haswell", "-mavx2", "-mfma"] {
        // Arrange
        let fields = valid_fields()
            .with_effective_compile_flags(tuning)
            .with_phase4(canonical_phase4());

        // Act
        let error = BuildIdentity::new(fields).expect_err("fixed tuning must fail closed");

        // Assert
        assert_eq!(error, BuildIdentityError::CanonicalForbiddenFlags);
    }
}

#[test]
fn canonical_identity_rejects_explicit_features_and_nested_llvm_fp_options() {
    for encoded in [
        "hexvec:2d43,7461726765742d666561747572653d2b617678322c2b666d61",
        "hexvec:2d43,6c6c766d2d617267733d2d656e61626c652d756e736166652d66702d6d617468",
    ] {
        // Arrange
        let fields = valid_fields()
            .with_compiler_id("rustc")
            .with_compiler_version("1.97.0")
            .with_effective_compile_flags(format!("encoded_rustflags={encoded}"))
            .with_phase4(Phase4BuildIdentityFields::new(
                "11".repeat(32),
                "rustc",
                "1.97.0",
                "x86_64-unknown-linux-gnu",
                "baseline",
                "cfg=fxsr,sse,sse2;explicit=<none>",
                "<none>",
                "O3",
                "precise",
                "off",
                "ieee",
                "scalar baseline",
                "linux",
                "glibc",
                "libm",
                "nearest_ties_even",
                true,
            ));

        // Act
        let error = BuildIdentity::new(fields).expect_err("LLVM tuning must fail closed");

        // Assert
        assert_eq!(error, BuildIdentityError::CanonicalForbiddenFlags);
    }
}

#[test]
fn phase4_compiler_must_match_base_identity_exactly() {
    // Arrange
    let mut phase4 = canonical_phase4();
    phase4.compiler_version = "22.1.7".to_owned();
    let fields = valid_fields().with_phase4(phase4);

    // Act
    let error = BuildIdentity::new(fields).expect_err("compiler mismatch must fail closed");

    // Assert
    assert_eq!(error, BuildIdentityError::Phase4CompilerMismatch);
}

#[test]
fn phase4_target_must_match_base_identity_exactly() {
    // Arrange
    let fields = valid_fields().with_phase4(canonical_phase4().with_sdk_or_sysroot("<none>"));
    let mismatched = BuildIdentityFields {
        target: "aarch64-unknown-linux-gnu".to_owned(),
        ..fields
    };

    // Act
    let error = BuildIdentity::new(mismatched).expect_err("target mismatch must fail closed");

    // Assert
    assert_eq!(error, BuildIdentityError::Phase4TargetMismatch);
}

#[test]
fn canonical_identity_requires_all_fields() {
    // Arrange
    let fields = valid_fields().with_phase4(canonical_phase4().with_sdk_or_sysroot(""));

    // Act
    let error = BuildIdentity::new(fields).expect_err("missing canonical field should fail");

    // Assert
    assert_eq!(
        error,
        BuildIdentityError::InvalidPhase4Field("sdk_or_sysroot")
    );
}

#[test]
fn supported_identity_cannot_promote_canonical_evidence() {
    // Arrange
    let supported = Phase4BuildIdentityFields::new(
        "22".repeat(32),
        "AppleClang",
        "21.0.0",
        "arm64-apple-darwin",
        "baseline",
        "<none>",
        "macos-sdk",
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

    // Act
    let identity = BuildIdentity::new(
        valid_fields()
            .with_compiler_id("AppleClang")
            .with_compiler_version("21.0.0")
            .with_target("arm64-apple-darwin")
            .with_phase4(supported),
    )
    .expect("supported identity should validate");

    // Assert
    assert_eq!(identity.evidence_tier(), BuildEvidenceTier::D2Supported);
    assert!(!identity.can_promote_canonical_evidence());
}
