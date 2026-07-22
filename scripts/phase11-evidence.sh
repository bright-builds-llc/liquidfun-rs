#!/usr/bin/env bash
set -euo pipefail

readonly upstream_revision="7f20402173fd143a3988c921bc384459c6a858f2"
readonly corpus_directory="crates/liquidfun-differential/tests/fixtures/catalog"
readonly max_failure_bytes=32768
readonly -a scenario_slugs=(
	"rigid-stack-stability"
	"joint-rope-behavior"
	"standalone-rope-evolution"
	"particle-forces-and-statistics"
	"particle-group-construction-append"
	"particle-aabb-query-controls"
	"particle-lifecycle-callbacks"
	"particle-mutations"
)
readonly -a comparison_slugs=(
	"standalone-rope-evolution"
	"particle-forces-and-statistics"
	"particle-aabb-query-controls"
)

usage() {
	echo "usage: $0 <canonical|sanitizer> <target/output-directory>" >&2
	exit 64
}

fail() {
	echo "phase11-evidence: $1" >&2
	exit 64
}

[[ $# -eq 2 ]] || usage
mode=$1
relative_output_dir=$2

case "$mode" in
canonical | sanitizer) ;;
*) usage ;;
esac
[[ -z "${LIQUIDFUN_PHASE11_ORACLE_MODE:-}" || "$LIQUIDFUN_PHASE11_ORACLE_MODE" == "$mode" ]] ||
	fail "environment mode differs from the fixed argument"
case "$relative_output_dir" in
target/*) ;;
*) fail "output must be beneath target/" ;;
esac
[[ "$relative_output_dir" != *".."* && "$relative_output_dir" != /* ]] ||
	fail "output path is unsafe"
[[ "${relative_output_dir##*/}" == "$mode" || "${relative_output_dir##*/}" == "phase11-$mode" ]] ||
	fail "output must end in $mode or phase11-$mode"

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
cd -- "$repository_root"

target_root="$repository_root/target"
[[ ! -L "$target_root" ]] || fail "target root must not be a symlink"
mkdir -p -- "$target_root"
target_root=$(cd -- "$target_root" && pwd -P)

current=$target_root
IFS='/' read -r -a output_components <<<"${relative_output_dir#target/}"
for component in "${output_components[@]}"; do
	[[ -n "$component" && "$component" != "." ]] || fail "output contains an empty component"
	candidate="$current/$component"
	[[ ! -L "$candidate" ]] || fail "output contains a symlink: $candidate"
	[[ ! -e "$candidate" || -d "$candidate" ]] || fail "output contains a non-directory: $candidate"
	mkdir -p -- "$candidate"
	current=$(cd -- "$candidate" && pwd -P)
	case "$current/" in
	"$target_root"/*) ;;
	*) fail "output escapes target/" ;;
	esac
done
output_dir=$current

for child in "$output_dir"/* "$output_dir"/.[!.]* "$output_dir"/..?*; do
	[[ -e "$child" || -L "$child" ]] || continue
	[[ ! -L "$child" ]] || fail "output contains a symlink: $child"
	case "${child##*/}" in
	cases | phase11-v1.json | debug.jsonl | release.jsonl | replay.jsonl | sanitizer.jsonl | identity.json) ;;
	*) fail "output contains an unexpected entry: $child" ;;
	esac
done
if [[ -e "$output_dir/cases" ]]; then
	[[ -d "$output_dir/cases" && ! -L "$output_dir/cases" ]] || fail "cases is not a regular directory"
	if find "$output_dir/cases" -type l -print -quit | grep -q .; then
		fail "cases contains a symlink"
	fi
	rm -rf -- "$output_dir/cases"
fi
rm -f -- "$output_dir/phase11-v1.json" "$output_dir/debug.jsonl" \
	"$output_dir/release.jsonl" "$output_dir/replay.jsonl" \
	"$output_dir/sanitizer.jsonl" "$output_dir/identity.json"
mkdir -p -- "$output_dir/cases"
chmod 0755 "$output_dir" "$output_dir/cases"

failure_dir="$target_root/phase11-evidence-failures/$mode"
mkdir -p -- "$failure_dir"
chmod 0755 "$target_root/phase11-evidence-failures" "$failure_dir"
command_log=$(mktemp "$failure_dir/.command.XXXXXX")
identity_tmp=$(mktemp "${output_dir%/*}/.$mode-phase11-identity.XXXXXX")
inventory_tmp=$(mktemp "${output_dir%/*}/.$mode-phase11-inventory.XXXXXX")
role_tmp=""
validator_log=""

cleanup_unfinished() {
	rm -f -- "$output_dir/identity.json" "$command_log" "$identity_tmp" "$inventory_tmp"
	[[ -z "$role_tmp" ]] || rm -f -- "$role_tmp"
	[[ -z "$validator_log" ]] || rm -f -- "$validator_log"
}
trap cleanup_unfinished EXIT

record_failure() {
	local label=$1
	local failure_path="$failure_dir/$label.log"
	local failure_tmp
	failure_tmp=$(mktemp "$failure_dir/.$label.XXXXXX")
	{
		printf 'phase11 evidence command failed: %s\n' "$label"
		printf '%s\n' 'first bounded bytes:'
		head -c "$max_failure_bytes" "$command_log"
		printf '\n%s\n' 'last bounded bytes:'
		tail -c "$max_failure_bytes" "$command_log"
	} >"$failure_tmp"
	chmod 0600 "$failure_tmp"
	mv -f -- "$failure_tmp" "$failure_path"
	cat -- "$failure_path" >&2
}

run_checked() {
	local label=$1
	shift
	printf 'phase11 evidence: %s\n' "$label"
	if "$@" >"$command_log" 2>&1; then
		rm -f -- "$failure_dir/$label.log"
		return 0
	fi
	record_failure "$label"
	exit 1
}

require_toolchain() {
	local rust_version
	rust_version=$(rustc --version)
	[[ "$rust_version" == rustc\ 1.97.0* ]] || fail "Rust 1.97.0 is required"
	command -v cmake >/dev/null || fail "CMake is required"
	command -v ninja >/dev/null || fail "Ninja is required"
	command -v jq >/dev/null || fail "jq is required"
	command -v "${CXX:-clang++}" >/dev/null || fail "the configured C++ compiler is required"
	if [[ "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]]; then
		cmake --version | grep -Fxq "cmake version 4.3.3" || fail "canonical CMake 4.3.3 is required"
		[[ "$(ninja --version)" == "1.13.2" ]] || fail "canonical Ninja 1.13.2 is required"
		"${CXX:-clang++}" --version | grep -Eq 'clang version 22\.1\.8' ||
			fail "canonical Clang 22.1.8 is required"
	fi
}

catalog_command() {
	local action=$1
	local slug=$2
	local preset=$3
	cargo xtask catalog "$action" \
		--scenario "$slug" \
		--timestep 0.016666668 \
		--velocity-iterations 8 \
		--position-iterations 3 \
		--particle-iterations 1 \
		--oracle-preset "$preset" \
		--session-profile one-shot \
		--output json \
		--commands auto
}

native_catalog_command() {
	local profile=$1
	local action=$2
	local slug=$3
	local preset=$4
	local -a cargo_arguments=(
		run --quiet --package liquidfun-differential --bin liquidfun-differential
	)
	if [[ "$profile" == "release" ]]; then
		cargo_arguments+=(--release)
	elif [[ "$profile" != "debug" ]]; then
		fail "unknown native profile"
	fi
	cargo "${cargo_arguments[@]}" -- catalog "$action" \
		--scenario "$slug" \
		--seed none \
		--timestep 0.016666668 \
		--velocity-iterations 8 \
		--position-iterations 3 \
		--particle-iterations 1 \
		--oracle-preset "$preset" \
		--session-profile one-shot \
		--output json \
		--commands auto
}

require_toolchain
run_checked upstream-verify cargo xtask upstream verify
[[ "$(git -C third_party/liquidfun rev-parse HEAD)" == "$upstream_revision" ]] ||
	fail "pinned upstream source revision differs"
run_checked corpus-validate cargo xtask phase11-evidence validate \
	--mode local --canonical-dir "$corpus_directory" --sanitizer-dir "$corpus_directory"

run_checked configure-debug cargo xtask upstream configure --preset oracle-debug
run_checked build-debug cargo xtask upstream build --preset oracle-debug
run_checked configure-release cargo xtask upstream configure --preset oracle-release
run_checked build-release cargo xtask upstream build --preset oracle-release
for slug in "${scenario_slugs[@]}"; do
	run_checked "native-debug-$slug" native_catalog_command debug run "$slug" oracle-debug
	run_checked "native-release-$slug" native_catalog_command release run "$slug" oracle-release
	run_checked "replay-debug-$slug" native_catalog_command debug replay "$slug" oracle-debug
	run_checked "replay-release-$slug" native_catalog_command release replay "$slug" oracle-release
done

if [[ "$mode" == "canonical" ]]; then
	for slug in "${comparison_slugs[@]}"; do
		run_checked "oracle-debug-$slug" catalog_command compare "$slug" oracle-debug
		run_checked "oracle-release-$slug" catalog_command compare "$slug" oracle-release
	done
else
	run_checked configure-sanitizer cargo xtask upstream configure --preset oracle-asan-ubsan
	run_checked build-sanitizer cargo xtask upstream build --preset oracle-asan-ubsan
	run_checked sanitizer-protocol-build cmake --build target/reference/oracle-asan-ubsan \
		--target liquidfun-reference-protocol-tests
	run_checked sanitizer-protocol-scope ctest --test-dir target/reference/oracle-asan-ubsan \
		--output-on-failure --no-tests=error -R '^liquidfun-reference-sanitizer-scope$'
	run_checked sanitizer-protocol env \
		UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 \
		ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 \
		ctest --test-dir target/reference/oracle-asan-ubsan \
		--output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'
	for slug in "${comparison_slugs[@]}"; do
		run_checked "sanitizer-$slug" env \
			UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 \
			ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 \
			cargo xtask catalog compare \
			--scenario "$slug" \
			--timestep 0.016666668 \
			--velocity-iterations 8 \
			--position-iterations 3 \
			--particle-iterations 1 \
			--oracle-preset oracle-asan-ubsan \
			--session-profile one-shot \
			--output json \
			--commands auto
	done
fi

install -m 0644 "$corpus_directory/phase11-v1.json" "$output_dir/phase11-v1.json"
for payload in "$corpus_directory"/cases/*.jsonl; do
	install -m 0644 "$payload" "$output_dir/cases/${payload##*/}"
done
for role in debug release replay sanitizer; do
	role_tmp=$(mktemp "${output_dir%/*}/.$mode-$role.XXXXXX")
	if cargo xtask phase11-evidence render-records "$role" >"$role_tmp" 2>"$command_log"; then
		chmod 0644 "$role_tmp"
		mv -f -- "$role_tmp" "$output_dir/$role.jsonl"
		role_tmp=""
	else
		rm -f -- "$role_tmp"
		record_failure "render-$role"
		exit 1
	fi
done

validator_log=$(mktemp "${output_dir%/*}/.$mode-phase11-validator.XXXXXX")
if cargo xtask phase11-evidence validate-content "$mode" "$relative_output_dir" \
	>"$validator_log" 2>&1; then
	rm -f -- "$failure_dir/validate-content.log"
else
	mv -f -- "$validator_log" "$command_log"
	record_failure validate-content
	exit 1
fi
semantic_sha256=$(sed -n 's/.*semantic-sha256=\([0-9a-f]\{64\}\).*/\1/p' "$validator_log")
rm -f -- "$validator_log"
validator_log=""
[[ ${#semantic_sha256} -eq 64 ]] || fail "validator omitted the semantic digest"

hash_file() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | awk '{print $1}'
	else
		shasum -a 256 "$1" | awk '{print $1}'
	fi
}

while IFS= read -r path; do
	relative=${path#"$output_dir/"}
	[[ "$relative" != "$path" && -f "$path" && ! -L "$path" ]] || fail "invalid identity input"
	jq -cn --arg path "$relative" --arg sha256 "$(hash_file "$path")" \
		'{path: $path, sha256: $sha256}' >>"$inventory_tmp"
done < <(find "$output_dir" -type f ! -name identity.json | LC_ALL=C sort)
inventory=$(jq -s '.' "$inventory_tmp")

if [[ "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]]; then
	[[ "${GITHUB_RUN_ID:-}" =~ ^[1-9][0-9]*$ ]] || fail "workflow dispatch run ID is invalid"
	[[ "${GITHUB_SHA:-}" =~ ^[0-9a-f]{40}$ ]] || fail "workflow dispatch SHA is invalid"
	identity_mode=exact-ref
	run_id=$GITHUB_RUN_ID
	head_sha=$GITHUB_SHA
	platform=linux-x86_64
	rust_version=1.97.0
	clang_version=22.1.8
	job_name="Phase 11 canonical Linux oracle"
	[[ "$mode" == "canonical" ]] || job_name="Phase 11 fail-fast sanitizer"
	artifact_name="phase11-$mode-$run_id-$head_sha"
else
	identity_mode=local
	run_id=0
	head_sha=local
	platform=local
	rust_version=local
	clang_version=local
	job_name="phase11-$mode-local"
	artifact_name=$job_name
fi

jq -n \
	--argjson schema_version 1 \
	--arg mode "$identity_mode" \
	--argjson run_id "$run_id" \
	--arg head_sha "$head_sha" \
	--arg job_name "$job_name" \
	--arg artifact_name "$artifact_name" \
	--arg platform "$platform" \
	--arg rust_version "$rust_version" \
	--arg clang_version "$clang_version" \
	--arg semantic_sha256 "$semantic_sha256" \
	--argjson files "$inventory" \
	'{schema_version: $schema_version, mode: $mode, run_id: $run_id,
    head_sha: $head_sha, job_name: $job_name, artifact_id: 0,
    artifact_name: $artifact_name, platform: $platform,
    rust_version: $rust_version, clang_version: $clang_version,
    upstream_revision: "7f20402173fd143a3988c921bc384459c6a858f2",
    protocol_version: "catalog-phase11-v1",
    generator_version: "phase11-evidence-v1",
    semantic_sha256: $semantic_sha256, files: $files}' >"$identity_tmp"
chmod 0644 "$identity_tmp"
mv -f -- "$identity_tmp" "$output_dir/identity.json"

rm -f -- "$command_log" "$inventory_tmp"
trap - EXIT
printf 'Phase 11 %s evidence complete: %s\n' "$mode" "$relative_output_dir"
