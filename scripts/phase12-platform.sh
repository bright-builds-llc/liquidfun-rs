#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s <target> <tier> <archive> <identity>\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase12-platform: %s\n' "$1" >&2
	exit 64
}

[[ $# -eq 4 ]] || usage
target=$1
tier=$2
archive_argument=$3
identity_argument=$4

case "$target" in
x86_64-unknown-linux-gnu | aarch64-unknown-linux-gnu | aarch64-apple-darwin | x86_64-pc-windows-msvc | x86_64-apple-darwin) ;;
*) fail "target is outside the reviewed platform policy" ;;
esac
[[ "$tier" == "d2_supported" ]] || fail "tier must be d2_supported"

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"

validate_input_file() {
	local argument=$1
	local label=$2
	case "$argument" in
	target/*) ;;
	*) fail "$label must be a repository-relative path under target/" ;;
	esac
	[[ "$argument" != *".."* && "$argument" != /* ]] ||
		fail "$label path must be normalized and traversal-free"

	local current=$repository_root
	local component
	IFS='/' read -r -a components <<<"$argument"
	for component in "${components[@]}"; do
		current="$current/$component"
		[[ ! -L "$current" ]] || fail "$label path contains a symbolic link"
	done
	[[ -f "$current" ]] || fail "$label must be a regular file"
	printf '%s\n' "$current"
}

archive_path=$(validate_input_file "$archive_argument" "archive")
identity_path=$(validate_input_file "$identity_argument" "identity")
support_path="$repository_root/reference/platform/support.json"
[[ -f "$support_path" && ! -L "$support_path" ]] || fail "platform support policy is unavailable"
command -v jq >/dev/null 2>&1 || fail "jq is required"

jq -e '
  .schema_version == 1 and
  (.archive_sha256 | type == "string")
' "$identity_path" >/dev/null || fail "artifact identity is malformed"

archive_sha256=$(jq -er '.archive_sha256' "$identity_path")
candidate_commit=$(jq -er '.candidate_commit' "$identity_path")
package_name=$(jq -er '.package' "$identity_path")
package_version=$(jq -er '.version' "$identity_path")
scalar_mode=$(jq -er '.scalar_mode' "$identity_path")
[[ "$archive_sha256" =~ ^[0-9a-f]{64}$ ]] || fail "archive identity hash is invalid"
[[ "$candidate_commit" =~ ^[0-9a-f]{40}$ ]] || fail "candidate commit is invalid"
[[ "$package_name" == "liquidfun" && -n "$package_version" ]] ||
	fail "artifact package identity is invalid"
[[ "$scalar_mode" == "strict_f32" ]] || fail "artifact scalar mode is invalid"
[[ "$(git rev-parse HEAD)" == "$candidate_commit" ]] ||
	fail "artifact candidate differs from the checked-out commit"

if command -v sha256sum >/dev/null 2>&1; then
	actual_archive_sha256=$(sha256sum "$archive_path" | awk '{print $1}')
else
	actual_archive_sha256=$(shasum -a 256 "$archive_path" | awk '{print $1}')
fi
[[ "$actual_archive_sha256" == "$archive_sha256" ]] ||
	fail "archive bytes differ from the reviewed identity"

rust_version=$(rustc --version | awk '{print $2}')
compiler_identity=$(rustc --version --verbose | tr '\n' ';')
case "$rust_version:$target" in
1.92.0:x86_64-unknown-linux-gnu) ;;
1.97.0:x86_64-unknown-linux-gnu | 1.97.0:aarch64-unknown-linux-gnu | 1.97.0:aarch64-apple-darwin | 1.97.0:x86_64-pc-windows-msvc | 1.97.0:x86_64-apple-darwin) ;;
*) fail "active Rust version is not valid for the requested target" ;;
esac

policy_tier=$(jq -er '.evidence_tier' "$support_path")
policy_scalar=$(jq -er '.scalar_mode' "$support_path")
policy_compiler=$(jq -er '.compiler_class' "$support_path")
policy_tolerance=$(jq -er '.tolerance_profile' "$support_path")
[[ "$policy_tier" == "$tier" && "$policy_scalar" == "$scalar_mode" ]] ||
	fail "platform policy differs from the requested evidence identity"
[[ "$policy_compiler" == "rustc-platform-native" && "$policy_tolerance" == "phase4-v1" ]] ||
	fail "platform compiler or tolerance policy is invalid"

native_evidence_recorded_at=null
runner_identity=${LIQUIDFUN_PLATFORM_RUNNER:-local}
[[ -n "$runner_identity" ]] || fail "runner identity is empty"
workflow_identity=${GITHUB_WORKFLOW:-local}
job_identity=${GITHUB_JOB:-local}
run_id=${GITHUB_RUN_ID:-0}
[[ -n "$workflow_identity" && -n "$job_identity" && "$run_id" =~ ^[0-9]+$ ]] ||
	fail "workflow execution identity is invalid"
if [[ "$target" == "x86_64-apple-darwin" ]]; then
	[[ "$runner_identity" == "macos-15-intel" ]] ||
		fail "conditional target requires the reviewed native runner"
	jq -e --arg target "$target" --arg runner "$runner_identity" '
	  .conditional_evidence_policy.max_age_days == 90 and
	  .conditional_evidence_policy.missing_or_expired_outcome == "unsupported" and
	  (.conditional_targets | length) == 1 and
	  .conditional_targets[0].target == $target and
	  .conditional_targets[0].tier == "conditional_supported" and
	  .conditional_targets[0].native_evidence.runner == $runner and
	  .conditional_targets[0].native_evidence.recorded_at_unix > 0 and
	  .conditional_targets[0].native_evidence.expires_at_unix ==
	    (.conditional_targets[0].native_evidence.recorded_at_unix + (90 * 86400))
	' "$support_path" >/dev/null || fail "conditional native evidence is invalid"
	native_evidence_recorded_at=$(jq -er '.conditional_targets[0].native_evidence.recorded_at_unix' "$support_path")
	native_evidence_expires_at=$(jq -er '.conditional_targets[0].native_evidence.expires_at_unix' "$support_path")
	now_unix=$(date +%s)
	((native_evidence_recorded_at <= now_unix && native_evidence_expires_at >= now_unix)) ||
		fail "conditional native evidence is unavailable or expired"
else
	jq -e --arg target "$target" '.durable_targets | index($target) != null' \
		"$support_path" >/dev/null || fail "durable target is absent from platform policy"
fi

cargo xtask package verify-artifact \
	--archive "$archive_argument" \
	--identity "$identity_argument" \
	--toolchain "$rust_version" \
	--target "$target"

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/liquidfun-platform.XXXXXX")
identity_temporary=
cleanup() {
	rm -rf -- "$temporary_directory"
	if [[ -n "$identity_temporary" ]]; then
		rm -f -- "$identity_temporary"
	fi
}
trap cleanup EXIT

tar -xzf "$archive_path" -C "$temporary_directory"
unpacked_crate="$temporary_directory/$package_name-$package_version"
[[ -f "$unpacked_crate/Cargo.toml" ]] || fail "verified archive did not extract to its package identity"
package_target="$temporary_directory/target"
(
	cd -- "$unpacked_crate"
	CARGO_TARGET_DIR="$package_target" cargo doc --all-features --no-deps --locked --target "$target"
	CARGO_TARGET_DIR="$package_target" cargo test --all-features --doc --locked --target "$target"
)
git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md

evidence_root="$repository_root/target/phase12-platform-evidence"
[[ ! -L "$repository_root/target" && ! -L "$evidence_root" ]] ||
	fail "platform evidence path contains a symbolic link"
mkdir -p -- "$evidence_root"
evidence_directory="$evidence_root/$target-$rust_version"
[[ ! -e "$evidence_directory" ]] || fail "platform evidence destination already exists"
mkdir -- "$evidence_directory"
printf '%s\n' '{"status":"verified","package_isolation":true,"rustdoc":true,"platform_smoke":true}' \
	>"$evidence_directory/verification.json"

recorded_at_unix=$(date +%s)
identity_temporary="$evidence_root/.$target-$rust_version-identity.$$.json"
jq -n \
	--arg archive_sha256 "$archive_sha256" \
	--arg target "$target" \
	--arg compiler "$compiler_identity" \
	--arg scalar_mode "$scalar_mode" \
	--arg tier "$tier" \
	--arg candidate_sha "$candidate_commit" \
	--arg runner "$runner_identity" \
	--arg workflow "$workflow_identity" \
	--arg job "$job_identity" \
	--argjson run_id "$run_id" \
	--argjson recorded_at_unix "$recorded_at_unix" \
	--argjson native_evidence_recorded_at_unix "$native_evidence_recorded_at" \
	'{
	  schema_version: 1,
	  archive_sha256: $archive_sha256,
	  target: $target,
	  compiler: $compiler,
	  scalar_mode: $scalar_mode,
	  tier: $tier,
	  candidate_sha: $candidate_sha,
	  runner: $runner,
	  workflow: $workflow,
	  job: $job,
	  run_id: $run_id,
	  recorded_at_unix: $recorded_at_unix,
	  native_evidence_recorded_at_unix: $native_evidence_recorded_at_unix
	}' >"$identity_temporary"
mv -- "$identity_temporary" "$evidence_directory/identity.json"
identity_temporary=
cleanup
trap - EXIT
