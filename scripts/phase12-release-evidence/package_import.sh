#!/usr/bin/env bash
set -euo pipefail

validate_platform_run() {
	local candidate_sha=$1 run_id=$2 repository=$3 metadata=$4
	validate_sha "$candidate_sha"
	validate_run_id "$run_id"
	jq -e --arg candidate "$candidate_sha" --argjson run "$run_id" --arg repository "$repository" '
		.id == $run and .head_sha == $candidate and
		.repository.full_name == $repository and .path == ".github/workflows/platform.yml" and
		.status == "completed" and .conclusion == "success" and
		(.event == "workflow_dispatch" or .event == "schedule")
	' "$metadata" >/dev/null || fail "selected platform run is unrelated or incomplete"
}

import_platform_package() {
	local candidate_sha=$1 output_directory=$2 platform_run_id=$3 archive=$4 identity=$5
	validate_sha "$candidate_sha"
	validate_run_id "$platform_run_id"
	validate_target_path "$output_directory"
	validate_target_path "$output_directory/package"
	validate_target_path "$output_directory/package/liquidfun.crate"
	validate_target_path "$output_directory/package/package-identity.json"
	validate_target_path "$archive"
	validate_target_path "$identity"
	local directory=${archive%/*}
	[[ "${directory##*/}" == "phase12-package-$platform_run_id-$candidate_sha" &&
		"$archive" == "$directory/liquidfun.crate" &&
		"$identity" == "$directory/package-identity.json" ]] ||
		fail "package artifact does not belong to the selected platform run and candidate"
	[[ -f "$archive" && -f "$identity" ]] || fail "platform package or identity is missing"
	local archive_bytes identity_bytes
	archive_bytes=$(wc -c <"$archive")
	identity_bytes=$(wc -c <"$identity")
	((archive_bytes > 0 && archive_bytes <= 67108864 && identity_bytes > 0 && identity_bytes <= 1048576)) ||
		fail "platform package inputs exceed their bounds"
	jq -e --arg candidate "$candidate_sha" --arg hash "$(hash_file "$archive")" \
		--argjson bytes "$archive_bytes" '
		.schema_version == 1 and .candidate_commit == $candidate and
		.archive_sha256 == $hash and .archive_bytes == $bytes and
		.package == "liquidfun" and .created_with_toolchain == "1.97.0" and
		(.version | type == "string" and test("^[0-9]+[.][0-9]+[.][0-9]+([+-][A-Za-z0-9.-]+)?$"))
	' "$identity" >/dev/null || fail "platform package identity or bytes differ"
	local archive_sha256 identity_sha256
	archive_sha256=$(hash_file "$archive")
	identity_sha256=$(hash_file "$identity")
	mkdir -p -- "$output_directory/package"
	[[ ! -e "$output_directory/package/liquidfun.crate" &&
		! -e "$output_directory/package/package-identity.json" ]] || fail "package import already exists"
	cp -- "$archive" "$output_directory/package/liquidfun.crate"
	cp -- "$identity" "$output_directory/package/package-identity.json"
	[[ "$(hash_file "$output_directory/package/liquidfun.crate")" == "$archive_sha256" &&
	"$(hash_file "$output_directory/package/package-identity.json")" == "$identity_sha256" &&
	"$(jq -er '.archive_sha256' "$output_directory/package/package-identity.json")" == "$archive_sha256" ]] ||
		fail "platform package changed during import"
}

verify_publication_archive() {
	local output_directory=$1
	local identity="$output_directory/package/package-identity.json"
	local archive="$output_directory/package/liquidfun.crate"
	local version expected_hash identity_hash
	version=$(jq -er '.version' "$identity")
	expected_hash=$(jq -er '.archive_sha256' "$identity")
	identity_hash=$(hash_file "$identity")
	local dry_run_directory="$output_directory/publish-dry-run"
	[[ ! -e "$dry_run_directory" ]] || fail "publication dry-run output already exists"
	CARGO_TARGET_DIR="$dry_run_directory" cargo publish -p liquidfun --dry-run ||
		fail "publication dry run failed"
	local generated="$dry_run_directory/package/tmp-crate/liquidfun-$version.crate"
	validate_target_path "$generated"
	[[ -f "$generated" && ! -L "$generated" ]] || fail "publication dry run did not generate an archive"
	[[ "$(hash_file "$identity")" == "$identity_hash" &&
	"$(hash_file "$archive")" == "$expected_hash" &&
	"$(hash_file "$generated")" == "$expected_hash" ]] || fail "publication dry-run archive differs"
	cmp -s "$archive" "$generated" || fail "publication dry-run bytes differ"
}
