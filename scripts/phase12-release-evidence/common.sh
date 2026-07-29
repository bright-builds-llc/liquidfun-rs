#!/usr/bin/env bash
set -euo pipefail

hash_file() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | awk '{print $1}'
	else
		shasum -a 256 "$1" | awk '{print $1}'
	fi
}
validate_sha() {
	local candidate_sha=$1
	[[ "$candidate_sha" =~ ^[0-9a-f]{40}$ ]] ||
		fail "candidate SHA must be canonical lowercase full hex"
}
validate_run_id() {
	local run_id=$1
	[[ "$run_id" =~ ^[1-9][0-9]*$ ]] || fail "producer run ID is invalid"
}
validate_target_path() {
	local path=$1
	case "$path" in
	"$repository_root"/target/*) ;;
	*) fail "output must be confined beneath repository target/" ;;
	esac
	[[ ! -L "$repository_root/target" ]] || fail "target must not be a symbolic link"
	local current=$repository_root
	local relative=${path#"$repository_root/"}
	local component
	IFS='/' read -r -a components <<<"$relative"
	for component in "${components[@]}"; do
		current="$current/$component"
		[[ ! -L "$current" ]] || fail "output path contains a symbolic link"
	done
}
require_candidate_checkout() {
	local candidate_sha=$1
	validate_sha "$candidate_sha"
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] ||
		fail "checked-out commit differs from the frozen candidate"
}
emit_evidence() {
	local items_file=$1
	local output_directory=$2
	local candidate_sha=$3
	local kind=$4
	local target=$5
	local workflow=$6
	local job=$7
	local run_id=$8
	local toolchain=$9
	local claims=${10}
	local artifact_directory="$output_directory/artifacts"
	local artifact_path="$artifact_directory/$kind-${target//\//_}.json"
	local payload_sha256
	payload_sha256=$(jq -cjS '.' <<<"$claims" | hash_stream)
	jq -n \
		--arg kind "$kind" \
		--arg target "$target" \
		--arg candidate "$candidate_sha" \
		--arg payload_sha256 "$payload_sha256" \
		--argjson claims "$claims" \
		'{
		  schema_version: 1,
		  kind: $kind,
		  target: $target,
		  candidate_commit: $candidate,
		  status: "passed",
		  payload_sha256: $payload_sha256,
		  claims: $claims
		}' >"$artifact_path"
	local relative_artifact=${artifact_path#"$repository_root/"}
	jq -cn \
		--arg kind "$kind" \
		--arg target "$target" \
		--arg candidate "$candidate_sha" \
		--arg workflow "$workflow" \
		--arg job "$job" \
		--arg run_id "$run_id" \
		--arg artifact_path "$relative_artifact" \
		--arg artifact_sha256 "$(hash_file "$artifact_path")" \
		--arg payload_sha256 "$payload_sha256" \
		--arg toolchain "$toolchain" \
		'{
		  kind: $kind,
		  target: $target,
		  candidate_commit: $candidate,
		  producer: {workflow: $workflow, job: $job, run_id: $run_id},
		  artifact_path: $artifact_path,
		  artifact_sha256: $artifact_sha256,
		  payload_sha256: $payload_sha256,
		  toolchain: $toolchain,
		  review_status: "reviewed",
		  status: "passed"
		}' >>"$items_file"
}
hash_stream() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum | awk '{print $1}'
	else
		shasum -a 256 | awk '{print $1}'
	fi
}
prepare_output() {
	local candidate_sha=$1
	local output_directory=$2
	local release_run_id=$3
	require_candidate_checkout "$candidate_sha"
	validate_run_id "$release_run_id"
	validate_target_path "$output_directory"
	[[ ! -e "$output_directory" ]] || fail "release output already exists"
	mkdir -p -- "$output_directory/artifacts" "$output_directory/package"
	local items_file="$output_directory/cheap-items.jsonl"

	cargo xtask package create-artifact \
		--archive "${output_directory#"$repository_root/"}"/package/liquidfun.crate \
		--identity "${output_directory#"$repository_root/"}"/package/package-identity.json \
		--candidate-commit "$candidate_sha"
	cargo xtask package verify-artifact \
		--archive "${output_directory#"$repository_root/"}"/package/liquidfun.crate \
		--identity "${output_directory#"$repository_root/"}"/package/package-identity.json \
		--toolchain 1.97.0 \
		--target x86_64-unknown-linux-gnu
	cargo publish -p liquidfun --dry-run
	cargo deny check --locked
	RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps
	cargo test -p liquidfun --all-features --doc
	cargo xtask docs check
	cargo xtask inventory check
	cargo xtask inventory corpus check-snapshot
	cargo xtask inventory corpus check-closure
	cargo xtask provenance check
	git diff --exit-code -- \
		protocol scenarios reference COMPATIBILITY.md UPSTREAM-CORPUS.md

	local package_identity="$output_directory/package/package-identity.json"
	local package_sha256
	package_sha256=$(jq -er '.archive_sha256' "$package_identity")
	[[ "$(hash_file "$output_directory/package/liquidfun.crate")" == "$package_sha256" ]] ||
		fail "package archive differs from its identity"
	local package_claims
	package_claims=$(jq -cn \
		--arg package_sha256 "$package_sha256" \
		--arg archive_path "${output_directory#"$repository_root/"}/package/liquidfun.crate" \
		'{
		  package_name: "liquidfun",
		  package_sha256: $package_sha256,
		  archive_path: $archive_path,
		  archive_sha256: $package_sha256,
		  rust_version: "1.92",
		  scalar_mode: "strict_f32",
		  package_drift: false
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" package all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" "$package_claims"
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" docs all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		'{"docs_complete":true,"rustdoc_warnings":0}'
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" notices all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		'{"notices_complete":true,"license":"MIT","advisory_waivers":0}'

	local corpus_claims
	corpus_claims=$(jq -cn \
		--arg authority_sha256 "$(hash_file reference/upstream-corpus.json)" \
		--argjson item_count "$(jq '.items | length' reference/upstream-corpus.json)" \
		'{
		  authority_sha256: $authority_sha256,
		  item_count: $item_count,
		  unresolved_count: 0,
		  nonterminal_count: 0
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" corpus_closure all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" "$corpus_claims"
	local compatibility_claims
	compatibility_claims=$(jq -cn \
		--arg authority_sha256 "$(hash_file reference/compatibility.json)" \
		'{
		  authority_sha256: $authority_sha256,
		  gap_count: 0,
		  unexplained_count: 0,
		  mixed_commit_count: 0,
		  coverage_promoted_to_parity: false,
		  platform_promoted_to_parity: false
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" compatibility_closure all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		"$compatibility_claims"
	jq -s '.' "$items_file" >"$output_directory/cheap-items.json"
	local cheap_sha256
	cheap_sha256=$(hash_file "$output_directory/cheap-items.json")
	jq -n \
		--arg candidate "$candidate_sha" \
		--arg run_id "$release_run_id" \
		--arg cheap_sha256 "$cheap_sha256" \
		'{schema_version: 1, candidate_commit: $candidate, producer_run_id: $run_id,
		  item_count: 5, payload_sha256: $cheap_sha256}' \
		>"$output_directory/cheap-identity.json"
	printf 'phase12 release inexpensive evidence complete: %s\n' "$output_directory"
}
