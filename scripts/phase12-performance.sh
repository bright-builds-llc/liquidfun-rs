#!/usr/bin/env bash
set -euo pipefail

usage() {
	echo "usage: $0 <calibrate|paired|validate>" >&2
	exit 64
}

[[ $# -eq 1 ]] || usage
mode=$1
case "$mode" in
calibrate | paired | validate) ;;
*) usage ;;
esac

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
cd -- "$repository_root"

target_root="$repository_root/target"
[[ ! -L "$target_root" ]] || {
	echo "phase12-performance: target must not be a symlink" >&2
	exit 1
}
mkdir -p -- "$target_root"

output_directory="$target_root/phase12-performance"
[[ ! -L "$output_directory" ]] || {
	echo "phase12-performance: output directory must not be a symlink" >&2
	exit 1
}
mkdir -p -- "$output_directory/logs"
[[ ! -L "$output_directory/logs" ]] || {
	echo "phase12-performance: log directory must not be a symlink" >&2
	exit 1
}

identity_path="$output_directory/identity.json"
identity_tmp=$(mktemp "$output_directory/.identity.XXXXXX")
summary_tmp=$(mktemp "$output_directory/.summary.XXXXXX")
mode_log="$output_directory/logs/$mode.log"

cleanup_unfinished() {
	rm -f -- "$identity_tmp" "$summary_tmp"
}
trap cleanup_unfinished EXIT
rm -f -- "$identity_path"

printf 'phase12-performance: mode=%s preset=oracle-release output=%s\n' \
	"$mode" "$output_directory"

case "$mode" in
paired)
	cargo xtask performance paired 2>&1 | tee "$mode_log"
	;;
calibrate)
	cargo xtask performance calibrate 2>&1 | tee "$mode_log"
	;;
validate)
	cargo xtask performance validate 2>&1 | tee "$mode_log"
	;;
esac

printf '{\n  "schema_version": 1,\n  "mode": "%s",\n  "claim_status": "no_generalized_performance_claim",\n  "oracle_preset": "oracle-release"\n}\n' \
	"$mode" >"$summary_tmp"
mv -f -- "$summary_tmp" "$output_directory/summary.json"

# The validator emits this bounded identity only after every prior artifact passes.
cargo xtask performance validate --emit-identity >"$identity_tmp"
mv -f -- "$identity_tmp" "$identity_path"
trap - EXIT

printf 'phase12-performance: complete mode=%s identity=%s\n' "$mode" "$identity_path"
