#!/usr/bin/env bash
set -euo pipefail

usage() {
	echo "usage: $0 <canonical|sanitizer> <target/output-directory>" >&2
	exit 64
}

[[ $# -eq 2 ]] || usage
mode=$1
relative_output_dir=$2

case "$mode" in
canonical | sanitizer) ;;
*) usage ;;
esac

case "$relative_output_dir" in
target/*) ;;
*)
	echo "Phase 10 evidence output must be under target/" >&2
	exit 64
	;;
esac
[[ "$relative_output_dir" != *".."* && "$relative_output_dir" != /* ]] || {
	echo "unsafe Phase 10 evidence output path" >&2
	exit 64
}
[[ "${relative_output_dir##*/}" == "$mode" || "${relative_output_dir##*/}" == "phase10-$mode" ]] || {
	echo "Phase 10 evidence output must end in $mode or phase10-$mode" >&2
	exit 64
}

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
cd -- "$repository_root"

target_root="$repository_root/target"
if [[ -L "$target_root" ]]; then
	echo "Phase 10 evidence target root must not be a symlink" >&2
	exit 64
fi
mkdir -p -- "$target_root"
target_root=$(cd -- "$target_root" && pwd -P)

current=$target_root
IFS='/' read -r -a output_components <<<"${relative_output_dir#target/}"
for component in "${output_components[@]}"; do
	candidate="$current/$component"
	if [[ -L "$candidate" ]]; then
		echo "Phase 10 evidence output path contains a symlink: $candidate" >&2
		exit 64
	fi
	if [[ -e "$candidate" && ! -d "$candidate" ]]; then
		echo "Phase 10 evidence output path contains a non-directory: $candidate" >&2
		exit 64
	fi
	if [[ ! -e "$candidate" ]]; then
		mkdir -- "$candidate"
	fi
	current=$(cd -- "$candidate" && pwd -P)
	case "$current/" in
	"$target_root"/*) ;;
	*)
		echo "Phase 10 evidence output escapes target/" >&2
		exit 64
		;;
	esac
done
output_dir=$current

for child in cases identity.json phase10-manifest.json phase10-trace.log provenance.log inventory.log read-only.log; do
	if [[ -L "$output_dir/$child" ]]; then
		echo "Phase 10 evidence output contains a symlink: $output_dir/$child" >&2
		exit 64
	fi
done
rm -rf -- "$output_dir/cases"
rm -f -- \
	"$output_dir/identity.json" \
	"$output_dir/phase10-manifest.json" \
	"$output_dir/phase10-trace.log" \
	"$output_dir/provenance.log" \
	"$output_dir/inventory.log" \
	"$output_dir/read-only.log"

manifest="$output_dir/phase10-manifest.json"
relative_manifest="$relative_output_dir/phase10-manifest.json"
trace="$output_dir/phase10-trace.log"

remove_unfinished_identity() {
	rm -f -- "$output_dir/identity.json"
}
trap remove_unfinished_identity EXIT

LIQUIDFUN_PHASE10_ORACLE_MODE="$mode" \
	LIQUIDFUN_PHASE10_EVIDENCE_MANIFEST="$relative_manifest" \
	cargo test -p liquidfun-differential --all-features --test phase10_corpus \
	corpus_executes_d0_replay_and_two_engine_debug_release_comparison -- --exact --nocapture \
	2>&1 | tee "$trace"
printf '%s\n' 'phase10 trace status: ok' >>"$trace"

{
	cargo xtask provenance check
	printf '%s\n' 'phase10 provenance status: ok'
} 2>&1 | tee "$output_dir/provenance.log"
{
	cargo xtask inventory check
	printf '%s\n' 'phase10 inventory status: ok'
} 2>&1 | tee "$output_dir/inventory.log"
{
	git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md
	printf '%s\n' 'phase10 read-only status: ok'
} 2>&1 | tee "$output_dir/read-only.log"

test -s "$manifest"
cargo xtask phase10-evidence validate-content "$mode" "$relative_output_dir"

hash_file() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | awk '{print $1}'
	else
		shasum -a 256 "$1" | awk '{print $1}'
	fi
}

identity_files_tmp="${output_dir%/*}/.$mode-phase10-identity-files.$$.jsonl"
trap 'remove_unfinished_identity; rm -f -- "$identity_files_tmp"' EXIT
while IFS= read -r path; do
	relative=${path#"$output_dir/"}
	[[ "$relative" != "$path" && -f "$path" && ! -L "$path" ]]
	jq -cn --arg path "$relative" --arg sha256 "$(hash_file "$path")" \
		'{path: $path, sha256: $sha256}' >>"$identity_files_tmp"
done < <(find "$output_dir" -type f ! -name identity.json | LC_ALL=C sort)
identity_files=$(jq -s '.' "$identity_files_tmp")
semantic_manifest_sha256=$(jq -er '.semantic_manifest_sha256' "$manifest")

if [[ -n "${GITHUB_RUN_ID:-}" && -n "${GITHUB_SHA:-}" ]]; then
	identity_mode=exact-ref
	run_id=$GITHUB_RUN_ID
	head_sha=$GITHUB_SHA
	platform=linux-x86_64
	rust_version=1.97.0
	clang_version=22.1.8
	job_name="Phase 10 canonical Linux oracle"
	if [[ "$mode" == sanitizer ]]; then
		job_name="Phase 10 fail-fast sanitizer"
	fi
	artifact_name="phase10-$mode-$run_id-$head_sha"
else
	identity_mode=local
	run_id=0
	head_sha=local
	platform=local
	rust_version=local
	clang_version=local
	job_name="phase10-$mode-local"
	artifact_name=$job_name
fi

jq -n \
	--argjson schema_version 1 \
	--arg mode "$identity_mode" \
	--argjson run_id "$run_id" \
	--arg head_sha "$head_sha" \
	--arg job_name "$job_name" \
	--argjson artifact_id 0 \
	--arg artifact_name "$artifact_name" \
	--arg platform "$platform" \
	--arg rust_version "$rust_version" \
	--arg clang_version "$clang_version" \
	--arg semantic_manifest_sha256 "$semantic_manifest_sha256" \
	--argjson files "$identity_files" \
	'{schema_version: $schema_version, mode: $mode, run_id: $run_id,
    head_sha: $head_sha, job_name: $job_name, artifact_id: $artifact_id,
    artifact_name: $artifact_name, platform: $platform,
    rust_version: $rust_version, clang_version: $clang_version,
    upstream_revision: "7f20402173fd143a3988c921bc384459c6a858f2",
    protocol_version: "rigid-world-phase10-v1",
    generator_version: "phase10-corpus-v1",
    semantic_manifest_sha256: $semantic_manifest_sha256, files: $files}' \
	>"$output_dir/identity.json"

rm -f -- "$identity_files_tmp"
trap - EXIT
