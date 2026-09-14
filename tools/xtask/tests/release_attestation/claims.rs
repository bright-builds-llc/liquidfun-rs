use super::*;

pub(super) fn claims_for(
    repository: &Path,
    evidence: &RequiredEvidence,
    archive_path: &Path,
    package_sha256: &str,
) -> Value {
    match evidence.kind.as_str() {
        "package" => json!({
            "package_name": "liquidfun",
            "package_sha256": package_sha256,
            "archive_path": repository_relative(repository, archive_path),
            "archive_sha256": package_sha256,
            "rust_version": "1.92",
            "scalar_mode": "strict_f32",
            "package_drift": false,
        }),
        "msrv" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "rust_version": "1.92",
        }),
        "platform" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "evidence_tier": "d2_supported",
        }),
        "conditional_platform" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "disposition": "unsupported",
            "recorded_at_unix": null,
            "expires_at_unix": null,
        }),
        "canonical_differential" => json!({
            "parity_tier": "d1_canonical",
            "coverage_authority": false,
            "performance_authority": false,
            "gap_count": 0,
        }),
        "rust_safety" => json!({
            "unsafe_waivers": 0,
            "advisory_waivers": 0,
            "unsafe_code": "forbid",
        }),
        "cpp_sanitizer" => json!({ "findings": 0 }),
        "fuzz" => json!({ "findings": 0, "target_count": 5 }),
        "regressions" => json!({
            "manifest_sha256": file_sha256(repository, "reference/regressions/manifest.toml"),
            "missing_results": 0,
            "unreviewed_results": 0,
        }),
        "rust_coverage" | "cpp_coverage" => json!({
            "contract_sha256": file_sha256(repository, "reference/coverage/contract.json"),
            "parity_authority": false,
            "missing_subsystems": 0,
        }),
        "performance" => {
            let manifest: toml::Value = toml::from_str(
                &fs::read_to_string(repository.join("reference/performance/manifest.toml"))
                    .expect("performance manifest"),
            )
            .expect("performance TOML");
            json!({
                "policy_sha256": manifest["policy_sha256"].as_str().expect("policy SHA"),
                "timing_authority": "unprofiled_wall_clock",
                "claim_scope": "workload_only",
                "claim_status": "no_generalized_performance_claim",
                "profile_authority": false,
                "reviewed_report_count": manifest["reviewed_reports"].as_array().expect("reports").len(),
            })
        }
        "docs" => json!({ "docs_complete": true, "rustdoc_warnings": 0 }),
        "notices" => json!({
            "notices_complete": true,
            "license": "MIT",
            "advisory_waivers": 0,
        }),
        "corpus_closure" => {
            let corpus_bytes =
                fs::read(repository.join("reference/upstream-corpus.json")).expect("corpus");
            let corpus: Value = serde_json::from_slice(&corpus_bytes).expect("corpus JSON");
            json!({
                "authority_sha256": sha256(&corpus_bytes),
                "item_count": corpus["items"].as_array().expect("corpus items").len(),
                "unresolved_count": 0,
                "nonterminal_count": 0,
            })
        }
        "compatibility_closure" => json!({
            "authority_sha256": file_sha256(repository, "reference/compatibility.json"),
            "gap_count": 0,
            "unexplained_count": 0,
            "mixed_commit_count": 0,
            "coverage_promoted_to_parity": false,
            "platform_promoted_to_parity": false,
        }),
        kind => panic!("unknown required evidence kind `{kind}`"),
    }
}
