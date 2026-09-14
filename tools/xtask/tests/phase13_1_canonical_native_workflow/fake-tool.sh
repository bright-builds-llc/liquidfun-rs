#!/usr/bin/env bash
set -euo pipefail

tool=${0##*/}
printf '%s %s\n' "$tool" "$*" >>"$CALL_LOG"
if [[ "$tool" == "${FAIL_TOOL:-}" ]]; then
	exit 19
fi
case "$tool" in
sha256sum)
	input=$(cat)
	if [[ -n "${FAIL_CHECKSUM:-}" && "$input" == *"$FAIL_CHECKSUM"* ]]; then
		exit 19
	fi
	;;
uname)
	if [[ "$1" == -s ]]; then
		printf '%s\n' "${TEST_OS:-Linux}"
	else
		printf '%s\n' "${TEST_ARCH:-x86_64}"
	fi
	;;
rustc) printf '%s\n' "${TEST_RUST:-rustc 1.97.0}" ;;
cmake) printf '%s\n' "${TEST_CMAKE:-cmake version 4.3.3}" ;;
ninja) printf '%s\n' "${TEST_NINJA:-1.13.2}" ;;
clang++-22) printf '%s\n' "${TEST_CLANG:-clang version 22.1.8}" ;;
curl | chmod | sudo | tar | unzip | rustup | install-canonical-clang.sh) ;;
*) exit 64 ;;
esac
