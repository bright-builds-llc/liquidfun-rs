#!/usr/bin/env bash
set -euo pipefail

mode=$1
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
export PHASE12_COVERAGE_LIBRARY_ONLY=1
source scripts/phase12-coverage.sh

prepare_output() { printf '%s\n' "$fixture"; }
validate_contract() { return 0; }
rustc() {
	[[ "$1" == +1.97.0 && "$2" == -vV ]] || return 2
	if [[ "$mode" == nightly ]]; then
		printf 'rustc 1.99.0-nightly\nrelease: 1.99.0-nightly\n'
	else
		printf 'rustc 1.97.0\nrelease: 1.97.0\nhost: x86_64-unknown-linux-gnu\n'
	fi
}
cargo() {
	if [[ "$*" == 'xtask safety-evidence validate-coverage' ]]; then return 0; fi
	[[ "$1" == +1.97.0 ]] || return 2
	shift
	[[ "$1" == llvm-cov ]] || return 2
	shift
	if [[ "$1" == --version ]]; then
		printf 'cargo-llvm-cov 0.8.7\n'
		return 0
	fi
	[[ "$mode" != failed ]] || return 1
	printf '%s\n' "$@" >"$fixture/command.txt"
	printf 'TN:\nSF:src/lib.rs\nDA:1,1\nend_of_record\n' >"$fixture/rust.lcov"
}
timeout() {
	shift 3
	"$@"
}

if [[ "$mode" == override ]]; then
	export RUSTC=fixture-override
fi

status=0
(run_rust_coverage 1111111111111111111111111111111111111111) || status=$?
if [[ "$mode" != stable ]]; then
	[[ "$status" -ne 0 && ! -e "$fixture/identity.json" ]]
	exit
fi
[[ "$status" == 0 ]]
jq -e '.toolchain_identity == "rust-1.97.0"' "$fixture/identity.json" >/dev/null
jq -e '.toolchain_identity == "rust-1.97.0"' "$fixture/summary.json" >/dev/null
grep -Fxq -- --workspace "$fixture/command.txt"
grep -Fxq -- --all-features "$fixture/command.txt"
grep -Fxq -- --lcov "$fixture/command.txt"
