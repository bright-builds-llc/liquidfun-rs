use sha2::{Digest, Sha256};

use super::{
    BuildEvidenceTier, BuildIdentityError, BuildIdentityFields, Phase4BuildIdentity,
    Phase4BuildIdentityFields, Sha256Hex,
};

pub(super) fn validate_oracle_revision(value: &str) -> Result<(), BuildIdentityError> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(BuildIdentityError::InvalidOracleRevision)
}

pub(super) fn validate_nonempty_fields(
    fields: &BuildIdentityFields,
) -> Result<(), BuildIdentityError> {
    let required = [
        ("adapter_revision", fields.adapter_revision.as_str()),
        ("cmake_preset", fields.cmake_preset.as_str()),
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target", fields.target.as_str()),
        ("build_type", fields.build_type.as_str()),
        (
            "effective_compile_flags",
            fields.effective_compile_flags.as_str(),
        ),
        ("effective_link_flags", fields.effective_link_flags.as_str()),
        ("sanitizer_mode", fields.sanitizer_mode.as_str()),
    ];
    if let Some((name, _)) = required.iter().find(|(_, value)| value.trim().is_empty()) {
        return Err(BuildIdentityError::EmptyField(name));
    }
    Ok(())
}

pub(super) fn validate_phase4_identity(
    fields: &Phase4BuildIdentityFields,
) -> Result<Phase4BuildIdentity, BuildIdentityError> {
    Sha256Hex::new(fields.compile_command_sha256.clone())
        .map_err(|_| BuildIdentityError::InvalidPhase4Field("compile_command_sha256"))?;
    let required = [
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target_triple", fields.target_triple.as_str()),
        ("target_cpu", fields.target_cpu.as_str()),
        ("target_features", fields.target_features.as_str()),
        ("sdk_or_sysroot", fields.sdk_or_sysroot.as_str()),
        ("optimization", fields.optimization.as_str()),
        ("fp_model", fields.fp_model.as_str()),
        ("fp_contract", fields.fp_contract.as_str()),
        ("denormal_mode", fields.denormal_mode.as_str()),
        ("feature_set", fields.feature_set.as_str()),
        ("os", fields.os.as_str()),
        ("libc", fields.libc.as_str()),
        ("libm", fields.libm.as_str()),
        ("rounding_mode", fields.rounding_mode.as_str()),
    ];
    if let Some((name, _)) = required.iter().find(|(_, value)| value.trim().is_empty()) {
        return Err(BuildIdentityError::InvalidPhase4Field(name));
    }
    Ok(Phase4BuildIdentity {
        fields: fields.clone(),
    })
}

pub(super) fn classify_evidence_tier(
    identity: &BuildIdentityFields,
    maybe_phase4: Option<&Phase4BuildIdentity>,
) -> Result<BuildEvidenceTier, BuildIdentityError> {
    let Some(phase4) = maybe_phase4 else {
        return Ok(BuildEvidenceTier::D3Exploratory);
    };
    let fields = &phase4.fields;
    let combined = format!(
        "{} {} {} {} {} {} {}",
        fields.optimization,
        fields.fp_model,
        fields.target_cpu,
        fields.target_features,
        fields.feature_set,
        identity.effective_compile_flags,
        identity.effective_link_flags,
    );
    let tokens = flag_tokens(&combined);
    let forbidden = tokens.iter().any(|word| has_unreviewed_codegen_flag(word));
    let canonical_compiler = (fields.compiler_id == "Clang" && fields.compiler_version == "22.1.8")
        || (fields.compiler_id == "rustc" && fields.compiler_version == "1.97.0");
    let canonical_target = match fields.compiler_id.as_str() {
        "Clang" => matches!(
            fields.target_triple.as_str(),
            "x86_64-pc-linux-gnu" | "x86_64-unknown-linux-gnu"
        ),
        "rustc" => fields.target_triple == "x86_64-unknown-linux-gnu",
        _ => false,
    };
    let canonical_candidate =
        canonical_compiler && canonical_target && fields.os.eq_ignore_ascii_case("linux");
    let canonical_features = match fields.compiler_id.as_str() {
        "Clang" => fields.target_features == "<none>",
        "rustc" => fields.target_features == "cfg=fxsr,sse,sse2;explicit=<none>",
        _ => false,
    };
    let canonical_codegen = fields.target_cpu == "baseline" && canonical_features && !forbidden;
    if canonical_candidate && !canonical_codegen {
        return Err(BuildIdentityError::CanonicalForbiddenFlags);
    }
    if canonical_candidate
        && (fields.fp_model != "precise"
            || fields.fp_contract != "off"
            || fields.denormal_mode != "ieee"
            || fields.rounding_mode != "nearest_ties_even"
            || !fields.gradual_underflow)
    {
        return Err(BuildIdentityError::CanonicalRuntimeWitness);
    }
    if canonical_candidate && canonical_codegen {
        return Ok(BuildEvidenceTier::D1Canonical);
    }
    if forbidden {
        return Ok(BuildEvidenceTier::D3Exploratory);
    }
    let supported_os = ["linux", "macos", "windows"]
        .iter()
        .any(|os| fields.os.eq_ignore_ascii_case(os));
    Ok(if supported_os {
        BuildEvidenceTier::D2Supported
    } else {
        BuildEvidenceTier::D3Exploratory
    })
}

fn has_unreviewed_codegen_flag(word: &str) -> bool {
    let lowered = word.to_ascii_lowercase();
    [
        "-ffast-math",
        "-ofast",
        "-fassociative-math",
        "-freciprocal-math",
        "-funsafe-math-optimizations",
        "target-cpu=",
        "target-feature=",
        "llvm-args=",
        "-march=",
        "-mcpu=",
        "-mtune=",
        "-mavx",
        "-mfma",
        "-msse",
        "unsafe-fp",
        "fp-contract=fast",
        "fp-contract=on",
        "no-nans-fp",
        "no-infs-fp",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn flag_tokens(value: &str) -> Vec<String> {
    value
        .split_ascii_whitespace()
        .flat_map(|word| {
            let Some((_, encoded)) = word.split_once("hexvec:") else {
                return vec![word.to_owned()];
            };
            encoded
                .split(',')
                .filter_map(decode_hex)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn decode_hex(value: &str) -> Option<String> {
    if value.is_empty() || !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16)?;
            let low = char::from(pair[1]).to_digit(16)?;
            u8::try_from((high << 4) | low).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

pub(super) fn hash_identity_fields(fields: &BuildIdentityFields) -> Sha256Hex {
    let values = [
        ("oracle_revision", fields.oracle_revision.as_str()),
        ("adapter_revision", fields.adapter_revision.as_str()),
        (
            "adapter_content_sha256",
            fields.adapter_content_sha256.as_str(),
        ),
        ("cmake_preset", fields.cmake_preset.as_str()),
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target", fields.target.as_str()),
        ("build_type", fields.build_type.as_str()),
        (
            "effective_compile_flags",
            fields.effective_compile_flags.as_str(),
        ),
        ("effective_link_flags", fields.effective_link_flags.as_str()),
        ("sanitizer_mode", fields.sanitizer_mode.as_str()),
    ];
    let mut hasher = Sha256::new();
    for (name, value) in values {
        hasher.update(name.len().to_be_bytes());
        hasher.update(name.as_bytes());
        hasher.update(value.len().to_be_bytes());
        hasher.update(value.as_bytes());
    }
    if let Some(phase4) = &fields.maybe_phase4 {
        let phase4_values = [
            (
                "compile_command_sha256",
                phase4.compile_command_sha256.as_str(),
            ),
            ("compiler_id", phase4.compiler_id.as_str()),
            ("compiler_version", phase4.compiler_version.as_str()),
            ("target_triple", phase4.target_triple.as_str()),
            ("target_cpu", phase4.target_cpu.as_str()),
            ("target_features", phase4.target_features.as_str()),
            ("sdk_or_sysroot", phase4.sdk_or_sysroot.as_str()),
            ("optimization", phase4.optimization.as_str()),
            ("fp_model", phase4.fp_model.as_str()),
            ("fp_contract", phase4.fp_contract.as_str()),
            ("denormal_mode", phase4.denormal_mode.as_str()),
            ("feature_set", phase4.feature_set.as_str()),
            ("os", phase4.os.as_str()),
            ("libc", phase4.libc.as_str()),
            ("libm", phase4.libm.as_str()),
            ("rounding_mode", phase4.rounding_mode.as_str()),
            (
                "gradual_underflow",
                if phase4.gradual_underflow {
                    "true"
                } else {
                    "false"
                },
            ),
        ];
        for (name, value) in phase4_values {
            hasher.update(name.len().to_be_bytes());
            hasher.update(name.as_bytes());
            hasher.update(value.len().to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    Sha256Hex::from_digest(hasher.finalize().into())
}
