#!/usr/bin/env bash
set -euo pipefail
case "${0##*/}" in
git) printf '%s\n' "$TEST_CANDIDATE" ;;
uname) if [[ "$1" == -s ]]; then echo Linux; else echo x86_64; fi ;;
rustc) printf 'rustc 1.97.0\nrelease: 1.97.0\nhost: x86_64-unknown-linux-gnu\n' ;;
clang++-22) echo 'clang version 22.1.8' ;;
llvm-profdata-22)
	while [[ "$1" != -o ]]; do shift; done
	printf 'profile\n' >"$2"
	;;
llvm-cov-22) printf 'TN:\nSF:fixture.cpp\nDA:1,1\nend_of_record\n' ;;
cmake) exit 0 ;;
ctest)
	printf 'raw profile\n' >"$LLVM_PROFILE_FILE"
	case "$TEST_OUTCOME" in
	fail)
		echo 'preserved command failure'
		exit 42
		;;
	timeout)
		echo 'preserved timeout'
		exit 124
		;;
	killed)
		echo 'preserved premature kill'
		exit 137
		;;
	oversized) head -c 17000000 /dev/zero ;;
	*) echo 'one test passed' ;;
	esac
	;;
cargo)
	if [[ "$*" == *'llvm-cov --version'* ]]; then
		echo 'cargo-llvm-cov 0.8.7'
		exit 0
	fi
	if [[ "$*" == *'xtask safety-evidence validate-differential-leaves'* ]]; then
		while [[ "$1" != --output ]]; do shift; done
		printf '{"schema_version":1,"parity_authority":false,"exercised":["fixture.leaf"],"missed":[]}' >"$2"
		exit 0
	fi
	if [[ "$*" == *'xtask safety-evidence'* || "$*" == *'xtask upstream'* || "$*" == *'xtask inventory'* ]]; then exit 0; fi
	case "$TEST_OUTCOME" in
	fail)
		echo 'preserved command failure'
		exit 42
		;;
	timeout)
		echo 'preserved timeout'
		exit 124
		;;
	killed)
		echo 'preserved premature kill'
		exit 137
		;;
	oversized)
		head -c 17000000 /dev/zero
		exit 0
		;;
	esac
	if [[ "$*" == *llvm-cov* ]]; then
		while [[ "$1" != --output-path ]]; do shift; done
		printf 'TN:\nSF:fixture.rs\nDA:1,1\nend_of_record\n' >"$2"
		exit 0
	fi
	count=1
	if [[ -n "${LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY:-}" ]]; then touch "$LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY/fixture.leaf"; fi
	if [[ "$*" == *'math:: --'* ]]; then count=44; fi
	printf 'test result: ok. %s passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n' "$count"
	;;
*) exit 64 ;;
esac
